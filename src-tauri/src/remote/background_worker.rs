//! 通知专用 Gateway 客户端，不订阅会话渲染、不申请输入席位、不执行远程操作。
use std::{path::Path,time::Duration};
use futures::{SinkExt,StreamExt};
use serde_json::{json,Value};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest,Message};
use crate::{local::types::*,mode::Token};
use super::{background_store as store,transport};

fn envelope(node:&str,kind:&str)->Value {
    json!({"protocolVersion":1,"id":uuid::Uuid::new_v4().to_string(),"type":kind,"sourceId":node,"targetId":"server","timestamp":chrono::Utc::now().timestamp_millis(),"payload":{}})
}
pub(super) async fn run(root:&Path,token:&Token,secret:&str,stopped:&mut impl FnMut()->bool,show:&mut impl FnMut(&Value)->ChatResult<bool>)->ChatResult<()> {
    tokio::select! {biased;
        result=async {
            loop {
                if stopped() {return Err(super::inactive());}
                store::runnable(root,token)?;
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }=>result,
        result=session(root,token,secret,show)=>result,
    }
}
async fn session(root:&Path,token:&Token,secret:&str,show:&mut impl FnMut(&Value)->ChatResult<bool>)->ChatResult<()> {
    if secret.is_empty() || secret.len()>4096 || secret.contains(['\r','\n']) {return Err(ChatError::new("REMOTE_AUTH_REQUIRED","后台凭据不可用，请重新授权"));}
    let profile=store::runnable(root,token)?;
    store::state(root,token,"connecting","")?;
    let identity=transport::probe(&profile.endpoint,secret).await?;
    if identity.server_id!=profile.server_id {return Err(ChatError::new("REMOTE_IDENTITY_CHANGED","服务地址对应的 infOS 实例已改变"));}
    let conn=store::connection(root,token)?;
    let mut principal=store::principal(&conn)?;drop(conn);
    if principal.is_none() {
        let response=transport::client()?.get(format!("{}/api/agents",profile.endpoint)).bearer_auth(secret).timeout(Duration::from_secs(15)).send().await.map_err(|_|transport::network())?;
        transport::status(response.status())?;
        let agents=transport::envelope(&transport::body(response).await?)?;
        let items=agents.as_array().ok_or_else(||ChatError::new("REMOTE_PROTOCOL","后台角色列表格式无效"))?;
        principal=items.iter().find(|agent|agent["isEnabled"]==true).or_else(||items.first()).and_then(|agent|agent["id"].as_str()).map(str::to_owned);
    }
    let principal=principal.ok_or_else(||ChatError::new("REMOTE_NOT_FOUND","尚无可接收通知的角色"))?;
    if principal.is_empty() || principal.len()>64 || principal.chars().any(char::is_control) {return Err(ChatError::new("REMOTE_PROTOCOL","后台角色身份无效"));}
    let node=format!("ppc-notifier-{}-{}",token.installation_id,profile.id);
    let mut url=reqwest::Url::parse(&format!("{}/ws/gateway",profile.endpoint)).map_err(|_|transport::network())?;
    let scheme=if url.scheme()=="https" {"wss"} else {"ws"};url.set_scheme(scheme).map_err(|_|transport::network())?;
    let mut request=url.as_str().into_client_request().map_err(|_|transport::network())?;
    request.headers_mut().insert("Authorization",format!("Bearer {secret}").parse().map_err(|_|transport::network())?);
    let config=tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default().max_message_size(Some(256*1024)).max_frame_size(Some(256*1024));
    let (mut socket,_)=tokio::time::timeout(Duration::from_secs(12),tokio_tungstenite::connect_async_with_config(request,Some(config),false)).await
        .map_err(|_|transport::network())?.map_err(|error| {
            if let tokio_tungstenite::tungstenite::Error::Http(response)=&error {if let Err(error)=transport::status(response.status()) {return error;}}
            transport::network()
        })?;
    let mut hello=envelope(&node,"hello");hello["payload"]=json!({"nodeId":node,"principalId":principal,"deviceName":"PeroperoChat 通知接收",
        "platform":{"os":"android","runtime":"native"},"supportedVersions":[1],"features":["audience-v1","cursor-v1"]});
    tokio::time::timeout(Duration::from_secs(8),socket.send(Message::Text(hello.to_string().into()))).await.map_err(|_|transport::network())?.map_err(|_|transport::network())?;
    let ack=tokio::time::timeout(Duration::from_secs(8),socket.next()).await.map_err(|_|transport::network())?.ok_or_else(transport::network)?.map_err(|_|transport::network())?;
    let ack:Value=serde_json::from_str(ack.to_text().map_err(|_|transport::network())?).map_err(|_|ChatError::new("REMOTE_PROTOCOL","后台握手不是有效 JSON"))?;
    if ack["type"]=="error" {return Err(protocol_error(&ack));}
    if ack["protocolVersion"]!=1 || ack["type"]!="hello_ack" || ack["payload"]["agreedVersion"]!=1
        || !ack["payload"]["features"].as_array().is_some_and(|features|features.iter().any(|f|f=="audience-v1")) {
        return Err(ChatError::new("REMOTE_INCOMPATIBLE","infOS 不支持后台通知所需的 Gateway 协议"));
    }
    // 握手后的未读重放就是持久恢复来源，不把易失流序号当作跨服务端重启的通知游标。
    store::state(root,token,"online","")?;
    let mut tick=tokio::time::interval(Duration::from_secs(5));let mut last=tokio::time::Instant::now();let mut heartbeat=tokio::time::Instant::now();
    loop {
        tokio::select! {biased;
            _=tick.tick()=> {
                store::runnable(root,token)?;
                let current=store::principal(&store::connection(root,token)?)?;
                if current.as_ref().is_some_and(|value|value!=&principal) {return Err(transport::network());}
                if last.elapsed()>Duration::from_secs(45) {return Err(transport::network());}
                if heartbeat.elapsed()>=Duration::from_secs(15) {
                    tokio::time::timeout(Duration::from_secs(8),socket.send(Message::Text(envelope(&node,"heartbeat").to_string().into()))).await.map_err(|_|transport::network())?.map_err(|_|transport::network())?;
                    heartbeat=tokio::time::Instant::now();store::state(root,token,"online","")?;
                }
                store::deliver(root,token,show)?;
            },
            message=socket.next()=> {
                let message=message.ok_or_else(transport::network)?.map_err(|_|transport::network())?;last=tokio::time::Instant::now();
                match message {
                    Message::Ping(bytes)=> {tokio::time::timeout(Duration::from_secs(8),socket.send(Message::Pong(bytes))).await.map_err(|_|transport::network())?.map_err(|_|transport::network())?;},
                    Message::Close(frame)=> {
                        if frame.is_some_and(|f|f.code==tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Policy) {return Err(ChatError::new("REMOTE_AUTH_REQUIRED","后台设备连接已被拒绝，请打开应用核对授权"));}
                        return Err(transport::network());
                    },
                    Message::Text(text)=> {
                        let value:Value=serde_json::from_str(&text).map_err(|_|ChatError::new("REMOTE_PROTOCOL","后台事件不是有效 JSON"))?;
                        if value["protocolVersion"]!=1 {return Err(ChatError::new("REMOTE_INCOMPATIBLE","后台事件协议版本不兼容"));}
                        if value["type"]=="error" {return Err(protocol_error(&value));}
                        if value["type"]=="push" {store::capture(root,token,&principal,&value)?;}
                    },_=>{},
                }
            },
        }
    }
}
fn protocol_error(value:&Value)->ChatError {
    if value["payload"]["code"]=="GATEWAY_UNAUTHORIZED" {ChatError::new("REMOTE_AUTH_REQUIRED","后台设备授权已失效，请重新配对")}
    else {ChatError::new("REMOTE_INCOMPATIBLE","infOS 拒绝了后台通知连接，请打开应用核对")}
}

#[cfg(test)]
#[path="background_worker_tests.rs"]
mod tests;
