use std::{collections::{HashMap,HashSet},sync::Arc,time::Duration};
use futures::{SinkExt,StreamExt};
use serde_json::{json,Value};
use tauri::{AppHandle,Emitter,Manager};
use tokio::sync::{mpsc,oneshot};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest,Message};
use crate::{mode::{Coordinator,Token},local::types::*};

struct Rpc {id:String,action:String,payload:Value,reply:oneshot::Sender<ChatResult<Value>>}
#[derive(Clone)]
pub struct Handle {pub token:Token,pub principal:String,pub window_id:String,pub session_id:String,queue:mpsc::Sender<Rpc>,task:Arc<tokio::task::JoinHandle<()>>}
impl Handle {
    pub fn stop(&self) {self.task.abort();}
    pub fn alive(&self)->bool {!self.task.is_finished()}
    pub async fn request(&self,action:&str,payload:Value)->ChatResult<Value> {
        let (reply,receive)=oneshot::channel();
        self.queue.try_send(Rpc {id:uuid::Uuid::new_v4().to_string(),action:action.into(),payload,reply})
            .map_err(|_|ChatError::new("REMOTE_GATEWAY_BUSY","远程交互连接未就绪或请求过多"))?;
        tokio::time::timeout(Duration::from_secs(15),receive).await.map_err(|_|super::transport::network())?
            .map_err(|_|super::transport::network())?
    }
}
fn envelope(node:&str,id:&str,kind:&str,payload:Value)->Value {
    json!({"protocolVersion":1,"id":id,"type":kind,"sourceId":node,"targetId":"server","timestamp":chrono::Utc::now().timestamp_millis(),"payload":payload})
}
fn emit(app:&AppHandle,token:&Token,kind:&str,data:Value) {
    if let Ok(_gate)=app.state::<Coordinator>().gate(Some(token),false) {let _=app.emit("remote-event",json!({"modeToken":token,"kind":kind,"data":data}));}
}

pub async fn connect(app:&AppHandle,session:&super::Session,principal:&str)->ChatResult<Handle> {
    if principal.is_empty() || principal.len()>64 {return Err(ChatError::new("INVALID_INPUT","远程角色身份无效"));}
    let mode=app.state::<Coordinator>();let token=session.token.clone();let mut changes=mode.subscribe();
    {let _gate=mode.gate(Some(&token),false)?;}
    let node=format!("ppc-{}-{}",token.installation_id,session.profile.id);
    let window_id=format!("{}:{}",node,uuid::Uuid::new_v4());
    let mut url=reqwest::Url::parse(&format!("{}/ws/gateway",session.profile.endpoint)).map_err(|_|super::transport::network())?;
    let scheme=if url.scheme()=="https" {"wss"} else {"ws"};url.set_scheme(scheme).map_err(|_|super::transport::network())?;
    let mut request=url.as_str().into_client_request().map_err(|_|super::transport::network())?;
    request.headers_mut().insert("Authorization",format!("Bearer {}",session.secret).parse().map_err(|_|super::transport::network())?);
    let config=tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default()
        .max_message_size(Some(super::transport::MAX_BODY)).max_frame_size(Some(super::transport::MAX_BODY));
    let (mut socket,_)=tokio::select! {biased;
        _=changes.changed()=>return Err(super::inactive()),
        result=tokio::time::timeout(Duration::from_secs(12),tokio_tungstenite::connect_async_with_config(request,Some(config),false))=>result.map_err(|_|super::transport::network())?.map_err(|error| {
            if let tokio_tungstenite::tungstenite::Error::Http(response)=&error {if let Err(error)=super::transport::status(response.status()) {return error;}}
            super::transport::network()
        })?,
    };
    socket.send(Message::Text(envelope(&node,&uuid::Uuid::new_v4().to_string(),"hello",json!({"nodeId":node,"principalId":principal,
        "token":session.secret,"deviceName":"PeroperoChat","platform":{"os":std::env::consts::OS,"runtime":"tauri"},
        "supportedVersions":[1],"features":["surface-v1","audience-v1","cursor-v1","input-seat-v1"]})).to_string().into())).await.map_err(|_|super::transport::network())?;
    let ack=tokio::select! {biased;
        _=changes.changed()=>return Err(super::inactive()),
        result=tokio::time::timeout(Duration::from_secs(8),socket.next())=>result.map_err(|_|super::transport::network())?.ok_or_else(super::transport::network)?.map_err(|_|super::transport::network())?,
    };
    let ack:Value=serde_json::from_str(ack.to_text().map_err(|_|super::transport::network())?).map_err(|_|super::transport::network())?;
    if ack["type"]!="hello_ack" || ack["payload"]["agreedVersion"]!=1 || !ack["payload"]["sessionId"].is_string()
        || !["surface-v1","input-seat-v1","cursor-v1","audience-v1"].iter().all(|feature|ack["payload"]["features"].as_array().is_some_and(|items|items.iter().any(|v|v==feature))) {
        return Err(ChatError::new("REMOTE_INCOMPATIBLE","目标 infOS 不支持所需 Gateway 与 Surface 协议"));
    }
    {let _gate=mode.gate(Some(&token),false)?;}
    let session_id=ack["payload"]["sessionId"].as_str().filter(|id|!id.is_empty()).ok_or_else(super::transport::network)?.to_owned();
    let handle_session_id=session_id.clone();
    let (queue,mut receiver)=mpsc::channel::<Rpc>(32);let app=app.clone();let run_token=token.clone();
    let task=tokio::spawn(async move {
        let mut pending:HashMap<String,(tokio::time::Instant,oneshot::Sender<ChatResult<Value>>)>=HashMap::new();
        let mut seen=HashSet::new();let mut heartbeat=tokio::time::interval(Duration::from_secs(15));let mut last_received=tokio::time::Instant::now();
        loop {
            tokio::select! {biased;
                _=changes.changed()=>break,
                _=heartbeat.tick()=> {
                    if last_received.elapsed()>Duration::from_secs(45) {break;}
                    pending.retain(|_,(started,_)|started.elapsed()<Duration::from_secs(15));
                    if socket.send(Message::Text(envelope(&node,&uuid::Uuid::new_v4().to_string(),"heartbeat",json!({})).to_string().into())).await.is_err() {break;}
                },
                request=receiver.recv()=> {
                    let Some(request)=request else {break};
                    if app.state::<Coordinator>().gate(Some(&run_token),false).is_err() {let _=request.reply.send(Err(super::inactive()));break;}
                    let mut payload=request.payload;payload["action"]=json!(request.action);
                    if socket.send(Message::Text(envelope(&node,&request.id,"request",payload).to_string().into())).await.is_err() {break;}
                    pending.insert(request.id,(tokio::time::Instant::now(),request.reply));
                },
                message=tokio::time::timeout(Duration::from_secs(50),socket.next())=> {
                    let Ok(Some(Ok(message)))=message else {break};
                    last_received=tokio::time::Instant::now();
                    if let Message::Ping(bytes)=message {if socket.send(Message::Pong(bytes)).await.is_err() {break;}continue;}
                    if message.is_close() {break;}
                    if !message.is_text() {continue;}
                    let Ok(text)=message.to_text() else {break};if text.len()>super::transport::MAX_BODY {break;}
                    let Ok(value)=serde_json::from_str::<Value>(text) else {break};
                    if value["protocolVersion"]!=1 {break;}
                    match value["type"].as_str() {
                        Some("response"|"error")=> {
                            if let Some((_,reply))=value["id"].as_str().and_then(|id|pending.remove(id)) {
                                let _=reply.send(if value["type"]=="error" {Err(ChatError::new("REMOTE_GATEWAY_REJECTED","交互席位或订阅已失效，请刷新后重试"))} else {Ok(value["payload"].clone())});
                            }
                        },
                        Some("push")=> {
                            let Some(id)=value["id"].as_str() else {break};
                            if !id.is_empty() && !seen.insert(id.to_owned()) {continue;}
                            if seen.len()>2048 {seen.clear();seen.insert(id.to_owned());}
                            emit(&app,&run_token,"gateway",value);
                        },
                        Some("heartbeat_ack")=>{},_=>break,
                    }
                },
            }
        }
        // 不把断开观察解释为取消远端任务；未决 RPC 直接失败，不自动重放写操作。
        for (_,(_,reply)) in pending {let _=reply.send(Err(super::transport::network()));}
        let _=tokio::time::timeout(Duration::from_secs(1),socket.close(None)).await;
        emit(&app,&run_token,"connection",json!({"state":"offline","sessionId":session_id}));
    });
    Ok(Handle {token,principal:principal.into(),window_id,session_id:handle_session_id,queue,task:Arc::new(task)})
}
