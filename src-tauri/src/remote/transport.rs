use std::time::Duration;
use serde::{Deserialize,Serialize};
use serde_json::Value;
use crate::local::types::*;

pub const MAX_BODY:usize=8*1024*1024;
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Identity {pub server_id:String,pub display_name:String,pub app_version:String}

pub fn client()->ChatResult<reqwest::Client> {
    reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).connect_timeout(Duration::from_secs(10))
        .build().map_err(|_|ChatError::new("REMOTE_NETWORK","远程网络客户端初始化失败"))
}
pub async fn body(mut response:reqwest::Response)->ChatResult<Vec<u8>> {
    if response.content_length().is_some_and(|n|n>MAX_BODY as u64) {return Err(ChatError::new("REMOTE_TOO_LARGE","远程响应过大，请缩小查询范围"));}
    let mut bytes=Vec::new();
    while let Some(chunk)=response.chunk().await.map_err(|_|network())? {
        if bytes.len()+chunk.len()>MAX_BODY {return Err(ChatError::new("REMOTE_TOO_LARGE","远程响应过大"));}
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
pub fn network()->ChatError {ChatError::new("REMOTE_NETWORK","远程连接中断；已发送的操作可能仍在服务端执行，请刷新核对")}
pub fn send_error(error:reqwest::Error)->ChatError {
    if error.is_connect() {network().with_outcome("not_sent")} else {network()}
}
pub fn before_send(error:ChatError)->ChatError {
    if error.outcome.is_some() {error} else {error.with_outcome("not_sent")}
}
pub fn status(status:reqwest::StatusCode)->ChatResult<()> {
    if status.is_success() {return Ok(());}
    let error=match status.as_u16() {
        401=>ChatError::new("REMOTE_AUTH_REQUIRED","远程设备授权已失效，请重新配对或登录"),
        403=>ChatError::new("REMOTE_FORBIDDEN","当前设备没有执行此操作的权限"),
        404=>ChatError::new("REMOTE_NOT_FOUND","远程对象不存在或目标版本不支持此接口"),
        409=>ChatError::new("REMOTE_CONFLICT","远程状态已改变，请刷新后重新操作"),
        400|422=>ChatError::new("REMOTE_INVALID_INPUT","服务端拒绝了操作参数，请刷新后核对"),
        429=>ChatError::new("REMOTE_RATE_LIMIT","远程请求过于频繁，请稍后再试"),
        _=>ChatError::new("REMOTE_HTTP",format!("远程服务响应异常（HTTP {}）",status.as_u16())),
    };
    Err(if matches!(status.as_u16(),400|401|403|404|405|409|413|415|422|429) {error.with_outcome("rejected")} else {error})
}
pub async fn checked(response:reqwest::Response)->ChatResult<reqwest::Response> {
    let code=response.status();
    if code.is_success() {return Ok(response);}
    if code==reqwest::StatusCode::CONFLICT {
        // 仅识别服务端明确在副作用之前拒绝的席位错误，不透传错误正文或秘密。
        if let Ok(bytes)=body(response).await {
            if let Ok(value)=serde_json::from_slice::<Value>(&bytes) {
                if value["data"]["reason"].as_str().is_some_and(|reason|["INPUT_SEAT_REQUIRED","INPUT_SEAT_EXPIRED","INPUT_SEAT_IDENTITY_MISMATCH"].contains(&reason)) {
                    return Err(ChatError::new("REMOTE_INPUT_SEAT_RETRY","输入席位已失效，需要重新获取").with_outcome("rejected"));
                }
            }
        }
    }
    status(code)?;Err(network())
}
pub fn envelope(bytes:&[u8])->ChatResult<Value> {
    let value:Value=serde_json::from_slice(bytes).map_err(|_|ChatError::new("REMOTE_PROTOCOL","远程响应不是有效 JSON"))?;
    if !value["code"].as_str().is_some_and(|code|["OK","CREATED","ACCEPTED"].contains(&code)) {return Err(ChatError::new("REMOTE_PROTOCOL","远程响应缺少标准成功结果信封"));}
    Ok(value["data"].clone())
}
pub async fn probe(endpoint:&str,token:&str)->ChatResult<Identity> {
    let response=client()?.get(format!("{endpoint}/api/distributed/identity")).bearer_auth(token).timeout(Duration::from_secs(12)).send().await.map_err(|_|network())?;
    status(response.status())?;
    let value=envelope(&body(response).await?)?;
    let identity:Identity=serde_json::from_value(value).map_err(|_|ChatError::new("REMOTE_PROTOCOL","目标服务未提供 infOS 实例身份"))?;
    if identity.server_id.trim().is_empty() || identity.server_id.len()>256 || identity.app_version.len()>100 || identity.display_name.len()>256 {
        return Err(ChatError::new("REMOTE_PROTOCOL","目标服务身份无效"));
    }
    Ok(identity)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn server(status:u16,body:String,headers:&str)->(String,std::sync::mpsc::Receiver<String>) {
        use std::io::{BufRead,Write};
        let listener=std::net::TcpListener::bind("127.0.0.1:0").unwrap();let address=listener.local_addr().unwrap();
        let (send,receive)=std::sync::mpsc::channel();let headers=headers.to_owned();
        std::thread::spawn(move || {
            let (mut socket,_)=listener.accept().unwrap();socket.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
            let mut reader=std::io::BufReader::new(socket.try_clone().unwrap());let mut request=String::new();
            loop {let mut line=String::new();reader.read_line(&mut line).unwrap();if line=="\r\n" {break;}request.push_str(&line);}
            let _=send.send(request);write!(socket,"HTTP/1.1 {status} OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\n{headers}Connection: close\r\n\r\n{body}",body.len()).unwrap();
        });
        (format!("http://{address}/infos"),receive)
    }
    #[tokio::test]
    async fn identity_uses_deployment_prefix_and_auth_without_following_redirects() {
        let (url,requests)=server(200,serde_json::json!({"code":"OK","data":{"serverId":"server-a","displayName":"测试实例","appVersion":"0.9"}}).to_string(),"");
        assert_eq!(probe(&url,"test-secret").await.unwrap().server_id,"server-a");
        let request=requests.recv().unwrap().to_lowercase();assert!(request.starts_with("get /infos/api/distributed/identity "));assert!(request.contains("authorization: bearer test-secret"));
        let (url,_)=server(302,"{}".into(),"Location: http://127.0.0.1:1/steal\r\n");
        assert_eq!(probe(&url,"test-secret").await.unwrap_err().code,"REMOTE_HTTP");
    }
    #[tokio::test]
    async fn auth_failure_does_not_forward_remote_body_or_secrets() {
        let (url,_)=server(401,"test-secret 是错误令牌".into(),"");let error=probe(&url,"test-secret").await.unwrap_err();
        assert_eq!(error.code,"REMOTE_AUTH_REQUIRED");assert!(!error.message.contains("test-secret"));
        assert!(envelope(br#"{"code":"ERROR","data":{}}"#).is_err());
        assert_eq!(envelope(br#"{"code":"OK"}"#).unwrap(),Value::Null);
    }
}
