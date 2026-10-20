use rusqlite::OptionalExtension;
use serde::{Deserialize,Serialize};
use tauri::{AppHandle,Emitter,Manager};
use crate::{mode::{Coordinator,Target,Token},local::types::*};

#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Profile {
    pub id:String,pub server_id:String,pub endpoint:String,pub display_name:String,pub app_version:String,
    #[serde(default)] pub credential_id:Option<String>,
    #[serde(default)] pub device:Option<super::access::Device>,
    #[serde(default)] pub session_owner:Option<String>,
}
impl Profile {pub fn credential_key(&self)->&str {self.credential_id.as_deref().unwrap_or(&self.id)}}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ProfileInput {
    pub endpoint:String,#[serde(default)] pub token:String,#[serde(default)] pub code:String,
    #[serde(default)] pub method:super::access::Method,
    #[serde(default="device_name")] pub name:String,#[serde(default="remember")] pub remember:bool,
    #[serde(default)] pub profile_id:Option<String>,#[serde(default)] pub allow_insecure:bool,
}
fn device_name()->String {"PeroperoChat".into()}
fn remember()->bool {true}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct PairingLink {endpoint:String,code:String,requires_insecure:bool}
fn decode_pairing(raw:&str)->ChatResult<PairingLink> {
    let invalid=||ChatError::new("INVALID_PAIRING_QR","这不是有效的 infOS 配对二维码，请在 infOS「局域网与设备」重新生成");
    if raw.len()>4096 || raw.chars().any(char::is_control) {return Err(invalid());}
    let mut url=reqwest::Url::parse(raw.trim()).map_err(|_|invalid())?;
    if !matches!(url.scheme(),"http"|"https") || !url.username().is_empty() || url.password().is_some() || url.query().is_some() {return Err(invalid());}
    let fragment=url.fragment().ok_or_else(invalid)?;
    let (page,query)=fragment.split_once('?').ok_or_else(invalid)?;
    if page!="/app" {return Err(invalid());}
    let pairs=reqwest::Url::parse(&format!("https://pair.invalid/?{query}")).map_err(|_|invalid())?.query_pairs().map(|(key,value)|(key.into_owned(),value.into_owned())).collect::<Vec<_>>();
    if pairs.len()!=1 || pairs[0].0!="pair" || pairs[0].1.len()!=10 || !pairs[0].1.bytes().all(|value|value.is_ascii_hexdigit()) {return Err(invalid());}
    let code=pairs[0].1.to_ascii_uppercase();
    if matches!(url.host_str(),Some("localhost"|"127.0.0.1"|"::1"|"[::1]"|"0.0.0.0")) {return Err(ChatError::new("INVALID_PAIRING_QR","二维码地址指向本机，请在 infOS 中选择手机可访问的电脑地址"));}
    let requires_insecure=url.scheme()=="http";
    url.set_fragment(None);
    Ok(PairingLink {endpoint:normalize_endpoint(url.as_str(),true)?,code,requires_insecure})
}
#[tauri::command]
pub fn remote_decode_pairing(app:AppHandle,raw:String,mode_token:Token)->ChatResult<PairingLink> {
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(&mode_token),false)?;
    // 扫码只校验并填表，不自动兑换配对码，也不替用户允许明文 HTTP。
    decode_pairing(&raw)
}

pub fn normalize_endpoint(raw:&str,allow_insecure:bool)->ChatResult<String> {
    let mut url=reqwest::Url::parse(raw.trim()).map_err(|_|ChatError::new("INVALID_ENDPOINT","请输入完整的 infOS 服务地址"))?;
    if raw.len()>2048 || !["http","https"].contains(&url.scheme()) || url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() || url.query().is_some() || url.fragment().is_some() {
        return Err(ChatError::new("INVALID_ENDPOINT","服务地址不能包含凭据、查询参数或片段"));
    }
    let loopback=matches!(url.host_str(),Some("localhost"|"127.0.0.1"|"[::1]"|"::1"));
    if url.scheme()=="http" && !loopback && !allow_insecure {return Err(ChatError::new("INSECURE_ENDPOINT","此连接未使用 HTTPS，须明确允许后才可保存"));}
    let path=url.path().trim_end_matches('/');let path=path.strip_suffix("/api").unwrap_or(path).to_owned();url.set_path(&path);
    Ok(url.to_string().trim_end_matches('/').to_owned())
}
pub fn get(mode:&Coordinator,id:&str)->ChatResult<Profile> {
    if uuid::Uuid::parse_str(id).is_err() {return Err(ChatError::new("PROFILE_NOT_FOUND","远程连接配置不存在"));}
    let conn=crate::mode::open_existing(mode.path())?;
    let raw:Option<String>=conn.query_row("SELECT value FROM remote_profiles WHERE id=?1",[id],|r|r.get(0)).optional()?;
    serde_json::from_str(&raw.ok_or_else(||ChatError::new("PROFILE_NOT_FOUND","远程连接配置不存在"))?)
        .map_err(|_|ChatError::new("INVALID_PROFILE","远程连接配置损坏"))
}
#[tauri::command]
pub fn remote_profiles(mode:tauri::State<Coordinator>)->ChatResult<Vec<Profile>> {
    let conn=crate::mode::open_existing(mode.path())?;let mut stmt=conn.prepare("SELECT value FROM remote_profiles ORDER BY rowid")?;
    let rows=stmt.query_map([],|r|r.get::<_,String>(0))?;
    rows.map(|raw|serde_json::from_str(&raw?).map_err(|_|ChatError::new("INVALID_PROFILE","远程连接配置损坏"))).collect()
}
#[tauri::command]
pub async fn remote_save_profile(app:AppHandle,input:ProfileInput,mode_token:Token)->ChatResult<Profile> {
    let mode=app.state::<Coordinator>();
    {let _gate=mode.gate(Some(&mode_token),false)?;}
    let endpoint=normalize_endpoint(&input.endpoint,input.allow_insecure)?;
    let previous=input.profile_id.as_deref().map(|id|get(&mode,id)).transpose()?;
    if previous.as_ref().is_some_and(|profile|profile.endpoint!=endpoint) {return Err(ChatError::new("REMOTE_IDENTITY_CHANGED","重新授权不能更改连接地址，请另存连接"));}
    // 兑换完成后核对稳定实例身份；同一地址换了实例也不能复用旧数据分区。
    let code=if matches!(input.method,super::access::Method::Bearer) {&input.token} else {&input.code};
    let credential=super::access::exchange(&endpoint,input.method,code,&input.name,input.remember).await?;
    let result=async {
        let identity=super::transport::probe(&endpoint,&credential.secret).await?;
        if previous.as_ref().is_some_and(|profile|profile.server_id!=identity.server_id) {return Err(ChatError::new("REMOTE_IDENTITY_CHANGED","地址对应的 infOS 实例已改变，不能复用原连接的数据分区"));}
        let profile=Profile {id:previous.as_ref().map(|profile|profile.id.clone()).unwrap_or_else(||uuid::Uuid::new_v4().to_string()),
            server_id:identity.server_id,endpoint:endpoint.clone(),display_name:identity.display_name,app_version:identity.app_version,
            credential_id:Some(uuid::Uuid::new_v4().to_string()),device:credential.device.clone(),
            session_owner:if credential.device.is_some() && !input.remember {Some(mode.snapshot()?.runtime_owner_id)} else {None}};
        let _switch=mode.switch_lock.lock().await;
        {let _gate=mode.gate(Some(&mode_token),false)?;}
        let mut conn=crate::mode::open_existing(mode.path())?;
        if let Some(old)=&previous {
            let current=get(&mode,&old.id)?;
            if current.credential_id!=old.credential_id {return Err(ChatError::new("REMOTE_CREDENTIAL_CHANGED","连接授权已被另一操作更新，请重新打开连接面板"));}
        } else {
            let count:u32=conn.query_row("SELECT COUNT(*) FROM remote_profiles",[],|r|r.get(0))?;
            if count>=20 {return Err(ChatError::new("PROFILE_LIMIT","最多保存 20 个连接，请先删除不用的配置"));}
        }
        // 新凭据先以独立安全存储键落盘，再原子切换指针；失败时旧凭据仍完整可用。
        super::vault::save(&app,profile.credential_key(),&credential.secret)?;
        let active=mode_token.target==(Target::Remote {profile_id:profile.id.clone()});
        if active {
            match mode.begin_switch(mode_token.target.clone()) {
                Ok(state)=> {let _=app.emit("mode-event",&state);super::stop_all(&app);},
                Err(error)=> {let _=super::vault::delete(&app,profile.credential_key());return Err(error);},
            }
        }
        let saved=(||->ChatResult<()> {
            let tx=conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            tx.execute("INSERT INTO remote_profiles(id,value) VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET value=excluded.value",rusqlite::params![profile.id,serde_json::to_string(&profile).map_err(|_|super::inactive())?])?;
            tx.commit()?;Ok(())
        })();
        if let Err(error)=saved {
            let _=super::vault::delete(&app,profile.credential_key());
            if active {if let Ok(state)=mode.fail() {let _=app.emit("mode-event",state);}}
            return Err(error);
        }
        if active {
            if let Err(error)=super::background::suspend_async(app.clone()).await {
                if let Ok(state)=mode.fail() {let _=app.emit("mode-event",state);}
                return Err(error);
            }
            match mode.activate() {
                Ok(state)=> {let _=app.emit("mode-event",state);},
                Err(error)=> {if let Ok(state)=mode.fail() {let _=app.emit("mode-event",state);}return Err(error);},
            }
        }
        if let Some(old)=&previous {let _=super::vault::delete(&app,old.credential_key());}
        if active {
            // 授权已经保存成功，后台适配失败不能伪装成配对失败而诱导重复兑换。
            if let Err(error)=super::background::reauthorized(&app) {eprintln!("后台接收授权状态等待恢复：{}",error.message);}
            super::background::resume_async(app.clone()).await;
        }
        Ok(profile)
    }.await;
    // 未保存的新设备会话尽力撤销；不撤销用户原来的管理或兼容凭据。
    if result.is_err() && credential.device.is_some() {
        let retained=input.profile_id.as_deref().and_then(|id|get(&mode,id).ok()).is_some_and(|profile|profile.device.as_ref().map(|device|&device.id)==credential.device.as_ref().map(|device|&device.id));
        if !retained {let _=super::access::revoke(&endpoint,&credential.secret).await;}
    }
    result
}
#[tauri::command]
pub async fn remote_delete_profile(app:AppHandle,id:String)->ChatResult<()> {
    let mode=app.state::<Coordinator>();let _switch=mode.switch_lock.lock().await;
    let profile=get(&mode,&id)?;let target=Target::Remote {profile_id:id.clone()};
    if mode.snapshot()?.selected_target==target {return Err(ChatError::new("PROFILE_ACTIVE","请先切换到其他运行目标，再删除连接"));}
    super::vault::delete(&app,profile.credential_key())?;
    crate::mode::open_existing(mode.path())?.execute("DELETE FROM remote_profiles WHERE id=?1",[id])?;Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn endpoint_cannot_smuggle_credentials_or_change_scheme() {
        assert_eq!(normalize_endpoint("https://example.com/infos/api/",false).unwrap(),"https://example.com/infos");
        for url in ["https://u:p@example.com","https://example.com/?token=abc","file:///tmp","https://example.com/#x","http://192.168.1.2"] {assert!(normalize_endpoint(url,false).is_err());}
        assert!(normalize_endpoint("http://192.168.1.2:9120",true).is_ok());
    }
    #[test]
    fn pairing_link_is_strict_and_does_not_grant_http_consent() {
        let link=decode_pairing("http://192.168.1.2:9122/#/app?pair=aabbccddee").unwrap();
        assert_eq!(link.endpoint,"http://192.168.1.2:9122");assert_eq!(link.code,"AABBCCDDEE");assert!(link.requires_insecure);
        assert_eq!(decode_pairing("https://example.com/infos/#/app?pair=AABBCCDDEE").unwrap().endpoint,"https://example.com/infos");
        for invalid in ["https://localhost/#/app?pair=AABBCCDDEE","javascript:alert(1)","https://u:p@example.com/#/app?pair=AABBCCDDEE","https://example.com/#/app?pair=AABBCCDDEE&pair=1122334455","https://example.com/#/app?token=secret"] {assert!(decode_pairing(invalid).is_err());}
    }
}
