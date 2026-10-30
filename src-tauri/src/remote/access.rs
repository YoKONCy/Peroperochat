//! 原生设备配对。短期代码只用于兑换，设备凭据不返回前端、不写入 Web 存储。
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::local::types::*;
use super::transport;

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Method { Pair, Login, #[default] Bearer }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Device {pub id:String,pub expires_at:i64,pub admin:bool}
pub struct Credential {pub secret:String,pub device:Option<Device>}

pub async fn exchange(client:&reqwest::Client,endpoint:&str,method:Method,code:&str,name:&str,remember:bool)->ChatResult<Credential> {
    if code.trim().is_empty() || code.len()>512 || code.chars().any(char::is_control) {
        return Err(ChatError::new("INVALID_CREDENTIAL","请输入有效的配对码或凭据，以及不超过 80 字的设备名称"));
    }
    if matches!(method,Method::Bearer) {return Ok(Credential {secret:code.to_owned(),device:None});}
    if name.trim().is_empty() || name.chars().count()>80 || name.chars().any(char::is_control) {return Err(ChatError::new("INVALID_CREDENTIAL","设备名称不能为空、包含控制字符或超过 80 字"));}
    let route=if matches!(method,Method::Pair) {"pair"} else {"login"};
    let response=client.post(format!("{endpoint}/api/client-access/{route}"))
        .timeout(std::time::Duration::from_secs(15)).json(&json!({"code":code.trim(),"name":name.trim(),"remember":remember}))
        .send().await.map_err(|error| {let failure=transport::send_error(error);if failure.outcome.as_deref()==Some("not_sent") {failure} else {ChatError::new("REMOTE_PAIRING_UNKNOWN","配对响应中断，代码可能已兑换；请在 infOS 设备列表核对后重新生成配对码")}})?;
    if response.status()==reqwest::StatusCode::UNAUTHORIZED {return Err(ChatError::new("REMOTE_PAIRING_REJECTED","配对码或管理凭据无效，配对码也可能已过期；请重新生成后再试"));}
    transport::status(response.status())?;
    let secret=credential_cookie(response.headers())?;
    let result=async {
        let value=transport::envelope(&transport::body(response).await?)?;
        let id=value["deviceId"].as_str().filter(|id|uuid::Uuid::parse_str(id).is_ok()).ok_or_else(protocol)?;
        let expires_at=value["expiresAt"].as_i64().filter(|value|*value>chrono::Utc::now().timestamp_millis()).ok_or_else(protocol)?;
        let status=status(client,endpoint,&secret).await?;
        if status["authenticated"]!=true || status["deviceId"]!=id {return Err(protocol());}
        Ok(Device {id:id.into(),expires_at,admin:status["admin"].as_bool().unwrap_or(false)})
    }.await;
    match result {
        Ok(device)=>Ok(Credential {secret,device:Some(device)}),
        Err(error)=> {let _=revoke(client,endpoint,&secret).await;Err(error)},
    }
}
fn protocol()->ChatError {ChatError::new("REMOTE_PROTOCOL","服务端未返回有效的设备会话，请在 infOS 中核对设备授权")}
fn credential_cookie(headers:&reqwest::header::HeaderMap)->ChatResult<String> {
    let mut secret=None;
    for header in headers.get_all(reqwest::header::SET_COOKIE).iter() {
        let raw=header.to_str().map_err(|_|protocol())?;
        if raw.len()>4096 {return Err(protocol());}
        let first=raw.split(';').next().unwrap_or("").trim();
        if let Some(value)=first.strip_prefix("infos_device=") {
            if secret.is_some() || value.len()!=64 || !value.bytes().all(|byte|byte.is_ascii_hexdigit()) {return Err(protocol());}
            secret=Some(value.to_owned());
        }
    }
    secret.ok_or_else(protocol)
}
pub async fn status(client:&reqwest::Client,endpoint:&str,secret:&str)->ChatResult<serde_json::Value> {
    let response=client.get(format!("{endpoint}/api/client-access/status")).bearer_auth(secret)
        .timeout(std::time::Duration::from_secs(12)).send().await.map_err(transport::send_error)?;
    transport::status(response.status())?;
    let value=transport::envelope(&transport::body(response).await?)?;
    if !value["authenticated"].is_boolean() || !value["admin"].is_boolean() {return Err(protocol());}
    Ok(json!({"authenticated":value["authenticated"],"admin":value["admin"],"deviceId":value["deviceId"]}))
}
pub async fn revoke(client:&reqwest::Client,endpoint:&str,secret:&str)->ChatResult<()> {
    let response=client.post(format!("{endpoint}/api/client-access/logout")).bearer_auth(secret)
        .timeout(std::time::Duration::from_secs(12)).send().await.map_err(transport::send_error)?;
    transport::status(response.status())?;Ok(())
}
