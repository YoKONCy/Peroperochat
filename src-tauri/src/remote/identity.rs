//! 容器重建可能改变由主机名派生的实例编号；原设备授权才是连接连续性的依据。
use std::path::Path;
use crate::{local::types::*, mode::{self, Token}};
use super::{access, profiles::{self, Profile}, transport::{self, Identity}};

pub(super) struct VerifiedIdentity { previous:Profile, identity:Identity }
impl VerifiedIdentity {
    pub(super) fn recovered(&self)->bool {self.previous.server_id!=self.identity.server_id}
    pub(super) fn server_id(&self)->&str {&self.identity.server_id}
}
fn changed()->ChatError {
    ChatError::new("REMOTE_IDENTITY_CHANGED","无法确认这个 infOS 与原连接属于同一实例；原记录保留，请核对服务端的设备授权与数据目录")
}
fn conflict()->ChatError {ChatError::new("REMOTE_CONFLICT","连接信息刚刚更新，请重新连接")}

pub(super) async fn verify(client:&reqwest::Client,profile:&Profile,secret:&str)->ChatResult<VerifiedIdentity> {
    let mut identity=transport::probe_with(client,&profile.endpoint,secret).await?;
    if identity.server_id!=profile.server_id {
        // 只有旧令牌仍对应同一持久化设备时才迁移；新配对、共享管理令牌不能作为证明。
        let device=profile.device.as_ref().filter(|device|uuid::Uuid::parse_str(&device.id).is_ok()).ok_or_else(changed)?;
        if device.expires_at<=chrono::Utc::now().timestamp_millis() {
            return Err(ChatError::new("REMOTE_AUTH_REQUIRED","原设备授权已过期，请重新配对"));
        }
        let status=access::status(client,&profile.endpoint,secret).await?;
        if status["authenticated"]!=true {return Err(ChatError::new("REMOTE_AUTH_REQUIRED","原设备授权已失效，请重新配对"));}
        if status["deviceId"]!=device.id {return Err(changed());}
        // 两次实例探测夹住授权检查，避免重启或反向代理切换期间保存混合结果。
        let confirmed=transport::probe_with(client,&profile.endpoint,secret).await?;
        if confirmed.server_id!=identity.server_id {return Err(ChatError::new("REMOTE_NETWORK","infOS 正在重启，请稍后重试连接"));}
        identity=confirmed;
    }
    Ok(VerifiedIdentity {previous:profile.clone(),identity})
}

pub(super) fn persist(control:&Path,token:&Token,verified:&VerifiedIdentity)->ChatResult<Profile> {
    let gate=mode::lock_existing(control,Some(token),false)?;
    let old=&verified.previous;
    let mut current=profiles::read_in(&gate.connection,&old.id)?;
    // 前后台可同时探测；只允许相同授权的幂等更新，不能覆盖换证、重新配对或别的实例。
    let mut authority=current.clone();
    authority.server_id=old.server_id.clone();authority.display_name=old.display_name.clone();authority.app_version=old.app_version.clone();
    if authority!=*old || current.server_id!=old.server_id && current.server_id!=verified.identity.server_id {return Err(conflict());}
    if old.session_owner.as_ref().is_some_and(|owner|owner!=&gate.snapshot.runtime_owner_id) {return Err(super::inactive());}
    if old.device.as_ref().is_some_and(|device|device.expires_at<=chrono::Utc::now().timestamp_millis()) {
        return Err(ChatError::new("REMOTE_AUTH_REQUIRED","设备授权已过期，请重新配对"));
    }
    let identity=&verified.identity;
    if current.server_id!=identity.server_id || current.display_name!=identity.display_name || current.app_version!=identity.app_version {
        current.server_id=identity.server_id.clone();current.display_name=identity.display_name.clone();current.app_version=identity.app_version.clone();
        gate.connection.execute("UPDATE remote_profiles SET value=?1 WHERE id=?2",rusqlite::params![serde_json::to_string(&current).map_err(|_|super::inactive())?,current.id])?;
        gate.connection.execute_batch("COMMIT")?;
    }
    Ok(current)
}

#[cfg(test)]
mod tests;
