use rusqlite::OptionalExtension;
use serde::{Deserialize,Serialize};
use tauri::{AppHandle,Manager};
use crate::{mode::{Coordinator,Target},local::types::*};

#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Profile {pub id:String,pub server_id:String,pub endpoint:String,pub display_name:String,pub app_version:String}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ProfileInput {pub endpoint:String,pub token:String,#[serde(default)] pub allow_insecure:bool}

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
pub async fn remote_save_profile(app:AppHandle,input:ProfileInput)->ChatResult<Profile> {
    if input.token.len()>4096 || input.token.contains(['\r','\n']) {return Err(ChatError::new("INVALID_CREDENTIAL","远程凭据格式无效"));}
    let endpoint=normalize_endpoint(&input.endpoint,input.allow_insecure)?;
    let identity=super::transport::probe(&endpoint,&input.token).await?;
    let profile=Profile {id:uuid::Uuid::new_v4().to_string(),server_id:identity.server_id,endpoint,display_name:identity.display_name,app_version:identity.app_version};
    let mode=app.state::<Coordinator>();let _switch=mode.switch_lock.lock().await;
    let conn=crate::mode::open_existing(mode.path())?;
    let count:u32=conn.query_row("SELECT COUNT(*) FROM remote_profiles",[],|r|r.get(0))?;
    if count>=20 {return Err(ChatError::new("PROFILE_LIMIT","最多保存 20 个连接，请先删除不用的配置"));}
    super::vault::save(&app,&profile.id,&input.token)?;
    if let Err(error)=conn.execute("INSERT INTO remote_profiles(id,value) VALUES(?1,?2)",rusqlite::params![profile.id,serde_json::to_string(&profile).unwrap()]) {
        let _=super::vault::delete(&app,&profile.id);return Err(error.into());
    }
    Ok(profile)
}
#[tauri::command]
pub async fn remote_delete_profile(app:AppHandle,id:String)->ChatResult<()> {
    let mode=app.state::<Coordinator>();let _switch=mode.switch_lock.lock().await;
    let profile=get(&mode,&id)?;let target=Target::Remote {profile_id:id.clone()};
    if mode.snapshot()?.selected_target==target {return Err(ChatError::new("PROFILE_ACTIVE","请先切换到其他运行目标，再删除连接"));}
    super::vault::delete(&app,&profile.id)?;
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
}
