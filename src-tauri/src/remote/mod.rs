//! infOS 主体客户端。只执行传输、身份隔离和展示投影，不运行本地 Agent 或工具。
pub mod profiles;
pub mod certificates;
mod access;
pub mod vault;
mod transport;
mod identity;
mod cache;
mod operations;
pub mod management;
pub mod tts;
pub mod files;
pub mod downloads;
pub mod conversation;
mod context;
mod gateway;
pub mod resources;
pub mod preview;
pub mod presentation;
pub mod notifications;
pub mod background;
mod background_store;
mod background_worker;
#[cfg(any(target_os="android",test))]
mod background_jni;

use std::sync::Mutex;
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use tauri::{AppHandle,Emitter,Manager};
use crate::{mode::{Coordinator,Token,Target},local::types::*};

#[derive(Default)]
pub struct Runtime {
    gateway:Mutex<Option<gateway::Handle>>,
    desired:Mutex<Option<(Token,Option<String>)>>,
    retry_blocked:Mutex<bool>,
    blocked_error:Mutex<Option<ChatError>>,
    connect_lock:tokio::sync::Mutex<()>,
    subscriptions:Mutex<StreamSet>,
    confirmed_subscriptions:Mutex<std::collections::HashSet<String>>,
    cursors:Mutex<std::collections::HashMap<String,u64>>,
    seen_messages:Mutex<std::collections::HashSet<String>>,
    input_lock:tokio::sync::Mutex<()>,
    verified_connection: tokio::sync::Mutex<Option<VerifiedConnection>>,
}
struct VerifiedConnection {
    token: Token,
    profile: profiles::Profile,
    client: reqwest::Client,
    checked_at: std::time::Instant,
}
/// 同时保持的远程订阅上限（与服务端 Gateway 连接的订阅数无关，仅为本机保护）。
const SUBSCRIPTION_LIMIT: usize = 100;
/// 网关入站帧的新鲜度阈值：心跳每 15 秒一次，超过 20 秒无任何入站帧视为陈旧。
const GATEWAY_FRESH_WINDOW: std::time::Duration = std::time::Duration::from_secs(20);
/// 远程订阅集合，记录每个流最近一次使用的序号，达到上限时淘汰最久未使用的会话或执行流。
/// 通知流长期有效，不参与淘汰；当前请求的流受保护。淘汰不会通知服务端（协议没有取消订阅），
/// 因此被淘汰流的事件在接收端直接丢弃，游标保持不变，重新订阅时由服务端补发或要求快照。
/// 只丢弃确实被本机淘汰过的流；服务端按受众主动推送、但本机从未显式订阅的流照常转发。
#[derive(Default)]
pub(super) struct StreamSet {tick:u64,used:std::collections::HashMap<String,u64>,evicted:std::collections::HashSet<String>}
impl StreamSet {
    pub(super) fn contains(&self,stream:&str)->bool {self.used.contains_key(stream)}
    pub(super) fn len(&self)->usize {self.used.len()}
    pub(super) fn touch(&mut self,stream:&str) {self.tick+=1;self.used.insert(stream.to_owned(),self.tick);self.evicted.remove(stream);}
    pub(super) fn clear(&mut self) {self.used.clear();self.evicted.clear();}
    pub(super) fn is_evicted(&self,stream:&str)->bool {self.evicted.contains(stream)}
    pub(super) fn streams(&self)->std::collections::HashSet<String> {self.used.keys().cloned().collect()}
    /// 淘汰最久未使用、且可以淘汰的流；返回被淘汰的流名。没有可淘汰项时返回 None。
    pub(super) fn evict_lru(&mut self,protect:&str)->Option<String> {
        let victim=self.used.iter()
            .filter(|(stream,_)|stream.as_str()!=protect && (stream.starts_with("thread:") || stream.starts_with("execution:")))
            .min_by_key(|(_,tick)|**tick).map(|(stream,_)|stream.clone())?;
        self.used.remove(&victim);self.evicted.insert(victim.clone());Some(victim)
    }
}
impl VerifiedConnection {
    fn reusable(&self, token: &Token, profile: &profiles::Profile, now: std::time::Instant) -> bool {
        self.token == *token && self.profile == *profile
            && now.saturating_duration_since(self.checked_at) < std::time::Duration::from_secs(10)
    }
}
pub(super) struct Session {app:AppHandle,token:Token,profile:profiles::Profile,secret:String,client:reqwest::Client}
pub(super) fn inactive()->ChatError {ChatError::new("MODE_INACTIVE","远程运行代次已失效，请刷新当前模式")}
pub fn stop_all(app:&AppHandle) {
    if let Ok(mut verified)=app.state::<Runtime>().verified_connection.try_lock() {*verified=None;}
    preview::clear(app);
    if let Ok(mut error)=app.state::<Runtime>().blocked_error.lock() {*error=None;}
    if let Ok(mut blocked)=app.state::<Runtime>().retry_blocked.lock() {*blocked=false;}
    if let Ok(mut handle)=app.state::<Runtime>().gateway.lock() {if let Some(handle)=handle.take() {handle.stop();}}
    if let Ok(mut desired)=app.state::<Runtime>().desired.lock() {*desired=None;}
    if let Ok(mut subscriptions)=app.state::<Runtime>().subscriptions.lock() {subscriptions.clear();}
    if let Ok(mut subscriptions)=app.state::<Runtime>().confirmed_subscriptions.lock() {subscriptions.clear();}
    if let Ok(mut cursors)=app.state::<Runtime>().cursors.lock() {cursors.clear();}
    if let Ok(mut seen)=app.state::<Runtime>().seen_messages.lock() {seen.clear();}
}
pub(super) fn notify_error(app:&AppHandle,token:&Token,error:&ChatError) {
    if error.code!="REMOTE_AUTH_REQUIRED" && !certificates::is_certificate_error(&error.code) {return;}
    if let Ok(_gate)=app.state::<Coordinator>().gate(Some(token),false) {
        background::connection_failed(app,token,&error.code);
        let runtime=app.state::<Runtime>();
        if let Ok(mut verified)=runtime.verified_connection.try_lock() {*verified=None;}
        if let Ok(mut blocked)=runtime.retry_blocked.lock() {*blocked=true;}
        if let Ok(mut value)=runtime.blocked_error.lock() {*value=Some(error.clone());}
        if let Ok(mut handle)=runtime.gateway.lock() {if let Some(handle)=handle.take() {handle.stop();}}
        let _=app.emit("remote-event",json!({"modeToken":token,"kind":"connection","data":{"state":if certificates::is_certificate_error(&error.code) {"incompatible"} else {"auth_required"},"code":error.code,"message":error.message}}));
    }
}
async fn checked_response(session:&Session,response:reqwest::Response)->ChatResult<reqwest::Response> {
    transport::checked(response).await.map_err(|error| {notify_error(&session.app,&session.token,&error);error})
}

pub fn start(app:AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut delay=2;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
            let desired=app.state::<Runtime>().desired.lock().ok().and_then(|value|value.clone());
            let blocked=app.state::<Runtime>().retry_blocked.lock().map(|value|*value).unwrap_or(true);
            let Some((token,_))=desired.filter(|_|!blocked) else {delay=2;continue};
            if validate_token(&app,&token).is_err() {delay=2;continue;}
            match ensure_gateway(&app,token.clone()).await {
                Ok(_)=>delay=2,
                Err(error)=> {
                    let blocked=["REMOTE_AUTH_REQUIRED","REMOTE_IDENTITY_CHANGED","REMOTE_INCOMPATIBLE","REMOTE_PROTOCOL","REMOTE_CERTIFICATE_CHANGED","REMOTE_CERTIFICATE_UNTRUSTED","REMOTE_CERTIFICATE_EXPIRED","REMOTE_CERTIFICATE_INVALID","CREDENTIAL_UNAVAILABLE","PROFILE_NOT_FOUND","INVALID_PROFILE"].contains(&error.code.as_str());
                    if let Ok(mut value)=app.state::<Runtime>().retry_blocked.lock() {*value=blocked;}
                    if let Ok(mut value)=app.state::<Runtime>().blocked_error.lock() {*value=if blocked {Some(error.clone())} else {None};}
                    if let Ok(_gate)=app.state::<Coordinator>().gate(Some(&token),false) {
                        let state=if error.code=="REMOTE_AUTH_REQUIRED" {"auth_required"} else if blocked {"incompatible"} else {"reconnecting"};
                        let _=app.emit("remote-event",json!({"modeToken":token,"kind":"connection","data":{"state":state,"code":error.code,"message":error.message}}));
                    }
                    delay=(delay*2).min(30);
                }
            }
        }
    });
}
pub fn validate_token(app:&AppHandle,token:&Token)->ChatResult<()> {
    if !matches!(token.target,Target::Remote {..}) {return Err(inactive());}
    // 只读校验：此处不持有写事务；需要持久写入的路径会在自己的门控事务中再次确认令牌。
    app.state::<Coordinator>().verify(token)
}
async fn session(app:&AppHandle,token:&Token)->ChatResult<Session> {
    validate_token(app,token)?;
    let Target::Remote {profile_id}= &token.target else {return Err(inactive());};
    let profile=profiles::get(&app.state::<Coordinator>(),profile_id)?;
    if let Some(owner)=&profile.session_owner {
        let mode=app.state::<Coordinator>();
        if mode.snapshot()?.runtime_owner_id!=*owner {
            {let _gate=mode.gate(Some(token),false)?;let _=vault::delete(app,profile.credential_key());}
            let error=ChatError::new("REMOTE_AUTH_REQUIRED","上次未保持登录的设备会话已结束，请重新配对");notify_error(app,token,&error);return Err(error);
        }
    }
    if profile.device.as_ref().is_some_and(|device|device.expires_at<=chrono::Utc::now().timestamp_millis()) {
        let error=ChatError::new("REMOTE_AUTH_REQUIRED","设备会话已过期，请重新配对；原连接的草稿与记录仍保留");notify_error(app,token,&error);return Err(error);
    }
    let secret=vault::load(app,profile.credential_key()).map_err(|error| {
        if error.code=="CREDENTIAL_UNAVAILABLE" {
            let error=ChatError::new("REMOTE_AUTH_REQUIRED","本机设备凭据暂不可用，请为原连接重新授权；草稿与记录仍保留");
            notify_error(app,token,&error);error
        } else {error}
    })?;
    let runtime = app.state::<Runtime>();
    // 同一批请求共用一次身份探测与连接池，配置或运行代次改变后立即失效。
    let mut saved = runtime.verified_connection.lock().await;
    let (client, profile, recovered) = if let Some(verified) = saved.as_ref().filter(|entry| {
        entry.reusable(token, &profile, std::time::Instant::now())
    }) {
        (verified.client.clone(), profile, false)
    } else {
        *saved = None;
        let client = transport::client_for(&profile.endpoint, profile.certificate.as_ref())?;
        let verified = identity::verify(&client, &profile, &secret).await.map_err(|error| {
            notify_error(app, token, &error);
            error
        })?;
        let profile = identity::persist(app.state::<Coordinator>().path(), token, &verified)?;
        *saved = Some(VerifiedConnection {
            token: token.clone(),
            profile: profile.clone(),
            client: client.clone(),
            checked_at: std::time::Instant::now(),
        });
        (client, profile, verified.recovered())
    };
    drop(saved);
    if recovered { background::identity_recovered(app, token).await; }
    validate_token(app, token)?;
    Ok(Session {app:app.clone(),token:token.clone(),profile,secret,client})
}
async fn perform(session:&Session,route:operations::Route)->ChatResult<Value> {
    let limit=if route.path.ends_with("/projection") || route.path.starts_with("/chat/threads/") || route.path.starts_with("/chat/queue/") {transport::MAX_PROJECTION_BODY} else {transport::MAX_BODY};
    perform_limited(session,route,limit).await
}
async fn perform_limited(session:&Session,route:operations::Route,limit:usize)->ChatResult<Value> {
    let mut url=reqwest::Url::parse(&format!("{}/api{}",session.profile.endpoint,route.path)).map_err(|_|transport::network())?;
    if !route.params.is_empty() {url.query_pairs_mut().extend_pairs(route.params);}
    let mut request=session.client.request(route.method,url).bearer_auth(&session.secret).timeout(std::time::Duration::from_secs(45));
    if let Some(body)=route.body {request=request.json(&body);}
    let response=request.send().await.map_err(transport::send_error)?;
    let response=checked_response(session,response).await?;transport::envelope(&transport::body_with_limit(response,limit).await?)
}
#[tauri::command]
pub async fn remote_session_status(app:AppHandle,token:Token)->ChatResult<Value> {
    let session=session(&app,&token).await?;
    let data=access::status(&session.client,&session.profile.endpoint,&session.secret).await.map_err(|error| {notify_error(&app,&token,&error);error})?;
    validate_token(&app,&token)?;
    if data["authenticated"]!=true {
        let error=ChatError::new("REMOTE_AUTH_REQUIRED","设备授权已失效，请重新配对");notify_error(&app,&token,&error);return Err(error);
    }
    if session.profile.device.as_ref().is_some_and(|device|data["deviceId"]!=device.id) {return Err(ChatError::new("REMOTE_IDENTITY_CHANGED","设备会话身份已改变，请重新授权"));}
    Ok(json!({"modeToken":token,"data":data}))
}
#[tauri::command]
pub async fn remote_logout_device(app:AppHandle,token:Token)->ChatResult<Value> {
    let session=session(&app,&token).await?;
    if session.profile.device.is_none() {return Err(ChatError::new("REMOTE_FORBIDDEN","兼容凭据不支持撤销设备，请在 infOS 管理端处理"));}
    access::revoke(&session.client,&session.profile.endpoint,&session.secret).await?;
    validate_token(&app,&token)?;
    notify_error(&app,&token,&ChatError::new("REMOTE_AUTH_REQUIRED","已撤销此设备授权，原连接记录保留，可重新配对"));
    Ok(json!({"modeToken":token,"data":{"authenticated":false}}))
}
fn cache_key(query:&operations::Query)->String {serde_json::to_string(query).unwrap()}

#[tauri::command]
pub async fn remote_query(app:AppHandle,token:Token,query:operations::Query,#[allow(non_snake_case)] cached_only:Option<bool>)->ChatResult<Value> {
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();validate_token(&app,&token)?;
    let key=cache_key(&query);query.route()?;
    if cached_only==Some(true) {return Ok(json!({"modeToken":token,"cached":true,"data":cache::load(&app,&token,&key)?}));}
    let ticket=cache::begin_read(&app,&token,&key)?;
    let mut data=tokio::select! {biased;
        _=changes.changed()=>return Err(inactive()),
        result=async {let session=session(&app,&token).await?;operations::fetch_query(&query,|route,limit|perform_limited(&session,route,limit)).await}=>match result {
            Ok(data)=>data,
            Err(error)=> {
                if matches!(error.code.as_str(),"REMOTE_NOT_FOUND"|"REMOTE_FORBIDDEN") {
                    let thread=if error.code=="REMOTE_NOT_FOUND" {if let operations::Query::Thread {id}=&query {Some(id.as_str())} else {None}} else {None};
                    let _=cache::forget_query(&app,&token,&key,ticket,thread);
                }
                return Err(error);
            },
        },
    };
    if matches!(query,operations::Query::Agent {..}) {
        data=json!({"id":data["id"],"name":data["name"],"description":data["description"],"publicProfile":data["publicProfile"],"ownerAppellation":data["ownerAppellation"]});
    }
    if matches!(query,operations::Query::Conversation {..}|operations::Query::TaskProjection {..}) && (data["protocolVersion"]!=1 || !data["surfaces"].is_array()) {
        return Err(ChatError::new("REMOTE_INCOMPATIBLE","服务端未返回支持的 Surface 投影版本"));
    }
    if let operations::Query::Conversation {id,..}=&query {
        if data["threadId"]!=*id {return Err(ChatError::new("REMOTE_PROTOCOL","服务端投影会话身份不匹配"));}
    }
    if let operations::Query::AgentState {id,thread_id}=&query {
        if data["agentId"]!=*id || data["threadId"]!=json!(thread_id) {return Err(ChatError::new("REMOTE_PROTOCOL","角色状态不属于当前会话"));}
    }
    if let operations::Query::Queue {id}=&query {validate_queue(&data,id)?;}
    if let operations::Query::QueueLive {id}=&query {
        if !data.is_null() && (data["threadId"]!=*id || !matches!(data["state"].as_str(),Some("open"|"committed"|"failed"|"disposed")) || !data["nodes"].is_array()) {return Err(ChatError::new("REMOTE_PROTOCOL","队列流式投影身份无效"));}
    }
    if !cache::save(&app,&token,&key,ticket,&data)? {return Err(ChatError::new("REMOTE_STALE_READ","正在同步最新内容"));}
    Ok(json!({"modeToken":token,"cached":false,"data":data}))
}

#[tauri::command]
pub async fn remote_action(app:AppHandle,token:Token,request_id:String,action:operations::Action)->ChatResult<Value> {
    action_inner(app,token,request_id,action).await.map_err(transport::before_send)
}
async fn action_inner(app:AppHandle,token:Token,request_id:String,action:operations::Action)->ChatResult<Value> {
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();validate_token(&app,&token)?;
    let route=action.route()?;
    // 探测失败发生在登记和写请求之前，调用者可以明确重试；发送后绝不自动重放。
    let session=tokio::select! {biased;_ =changes.changed()=>return Err(inactive()),result=session(&app,&token)=>result?,};
    // 获取主会话由服务端幂等创建，读取失败后可直接重试。
    let tracked = !matches!(&action, operations::Action::LatestThread {..});
    if tracked { cache::prepare(&app,&token,&request_id,&json!({"kind":"action","input":action}))?; }
    let result=tokio::select! {biased;_ =changes.changed()=>return Err(inactive().with_outcome("unknown")),result=perform(&session,route)=>result,}
        .and_then(|data| {
            let thread = &data["thread"];
            let valid = match &action {
                operations::Action::LatestThread {agent_id} => thread["agentId"] == *agent_id && thread["id"].as_str().is_some_and(|id| !id.is_empty()),
                operations::Action::ResetThread {id} => thread["id"] == *id && thread["generation"].as_u64().is_some_and(|generation| generation > 0),
                operations::Action::ForkThread {id,..} => thread["id"].as_str().is_some_and(|created| !created.is_empty() && created != id),
                _ => true,
            };
            if !valid { return Err(ChatError::new("REMOTE_PROTOCOL", "服务端未返回匹配的会话身份").with_outcome("unknown")); }
            Ok(data)
        });
    let data = if tracked { cache::settle(&app,&token,&request_id,result)? } else { result? };
    Ok(json!({"modeToken":token,"requestId":request_id,"data":data}))
}

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ChatRequest {#[serde(default,skip_serializing_if="Option::is_none")] pub expected_generation:Option<u64>,pub request_id:String,pub thread_id:String,pub agent_id:String,pub content:String,#[serde(default)] pub attachment_ids:Vec<String>,pub image_mode:Option<String>,#[serde(default)] pub workspace_context:Option<context::WorkspaceContext>,#[serde(default)] pub generation_type:Option<String>}

#[tauri::command]
pub async fn remote_chat(app:AppHandle,token:Token,request:ChatRequest)->ChatResult<Value> {
    chat_inner(app,token,request).await.map_err(transport::before_send)
}
async fn chat_inner(app:AppHandle,token:Token,request:ChatRequest)->ChatResult<Value> {
    if request.content.trim().is_empty() || request.content.len()>128*1024 || request.attachment_ids.len()>5 || request.attachment_ids.iter().any(|id|uuid::Uuid::parse_str(id).is_err())
        || request.image_mode.as_ref().is_some_and(|mode|!["auto","native","relay"].contains(&mode.as_str()))
        || request.generation_type.as_ref().is_some_and(|kind|!["normal","regenerate"].contains(&kind.as_str()))
        || request.expected_generation.is_some_and(|generation|generation==0 || generation>9_007_199_254_740_991) {return Err(ChatError::new("INVALID_INPUT","远程聊天内容或附件无效"));}
    operations::segment(&request.thread_id)?;operations::segment(&request.agent_id)?;
    if let Some(context)=&request.workspace_context {context.validate()?;}
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();validate_token(&app,&token)?;
    let session=tokio::select! {biased;_ =changes.changed()=>return Err(inactive()),result=session(&app,&token)=>result?,};
    cache::prepare(&app,&token,&request.request_id,&json!({"kind":"chat","transport":"queue","input":request}))?;
    let mut body=json!({"clientMessageId":request.request_id,"threadId":request.thread_id,"agentId":request.agent_id,"content":request.content,"attachmentIds":request.attachment_ids});
    if let Some(image_mode)=&request.image_mode {body["imageMode"]=json!(image_mode);}
    if let Some(context)=&request.workspace_context {body["workspaceContext"]=json!(context);}
    if let Some(kind)=&request.generation_type {body["generationType"]=json!(kind);}
    if let Some(generation)=request.expected_generation {body["expectedGeneration"]=json!(generation);}
    let result=tokio::select! {biased;
        _=changes.changed()=>return Err(inactive().with_outcome("unknown")),
        result=perform(&session,operations::Route {method:reqwest::Method::POST,path:format!("/chat/queue/{}",operations::segment(&request.thread_id)?),params:Vec::new(),body:Some(body)})=>result,
    }.and_then(|data| {
        // 成功提交后返回当前队列，已完成的消息可能已经移出 items。
        validate_queue(&data,&request.thread_id)?;
        Ok(data)
    });
    let data=cache::settle(&app,&token,&request.request_id,result)?;
    // 入队回执只表示服务端已接收，后续执行状态由队列及 Gateway 投影继续同步。
    Ok(json!({"modeToken":token,"requestId":request.request_id,"threadId":request.thread_id,"state":"accepted","data":data}))
}
fn validate_queue(data:&Value,thread:&str)->ChatResult<()> {
    if data["threadId"]!=thread || !data["paused"].is_boolean() || !data["items"].is_array() {
        return Err(ChatError::new("REMOTE_PROTOCOL","服务端队列身份或格式无效"));
    }
    Ok(())
}

fn validate_frame(value:&Value)->ChatResult<()> {
    if value["protocolVersion"]!=1 || !value["surfaceId"].is_string() || !value["generation"].is_string() || !value["operationId"].is_string()
        || value["sequence"].as_u64().is_none() || value["revision"].as_u64().is_none() || !value["operation"]["type"].is_string() {
        return Err(ChatError::new("REMOTE_INCOMPATIBLE","Surface 流帧缺少必要身份或版本"));
    }
    Ok(())
}

#[tauri::command]
pub async fn remote_connect(app:AppHandle,token:Token,principal_id:Option<String>,force:Option<bool>)->ChatResult<Value> {
    if let Some(principal)=&principal_id {
        operations::segment(principal)?;
        if principal.len()>64 {return Err(ChatError::new("INVALID_INPUT","远程角色身份过长"));}
    }
    validate_token(&app,&token)?;
    // 强制重连只在存活连接陈旧时才重建；健康连接直接复用并恢复订阅，避免先停后连造成无网关窗口。
    let fresh;
    {
        let runtime=app.state::<Runtime>();let _connect=runtime.connect_lock.lock().await;
        let _gate=app.state::<Coordinator>().gate(Some(&token),false)?;
        fresh=force==Some(true) && !gateway_healthy(&runtime,&token);
        *app.state::<Runtime>().desired.lock().map_err(|_|inactive())?=Some((token.clone(),principal_id.clone()));
        *app.state::<Runtime>().retry_blocked.lock().map_err(|_|inactive())?=false;
        *app.state::<Runtime>().blocked_error.lock().map_err(|_|inactive())?=None;
    }
    // 不在此处停止旧网关：新连接就绪后由 ensure_gateway_with 原子替换并停止旧连接，确认订阅也在替换时清空。
    ensure_gateway_with(&app,token,fresh).await
}
/// 存活且最近收到过入站帧的同一令牌网关；可直接复用，无需重建。
fn gateway_healthy(runtime:&Runtime,token:&Token)->bool {
    runtime.gateway.lock().ok().and_then(|handle|handle.as_ref().map(|h|h.token==*token && h.alive() && h.inbound_within(GATEWAY_FRESH_WINDOW))).unwrap_or(false)
}
fn blocked_error(runtime:&Runtime)->ChatError {runtime.blocked_error.lock().ok().and_then(|value|value.clone()).unwrap_or_else(||ChatError::new("REMOTE_AUTH_REQUIRED","设备授权需要重新确认，请重新配对或登录"))}
fn validate_selection(runtime:&Runtime,token:&Token,principal:&Option<String>)->ChatResult<()> {
    let selected=runtime.desired.lock().map_err(|_|inactive())?;
    if selected.as_ref()!=Some(&(token.clone(),principal.clone())) {return Err(ChatError::new("REMOTE_SELECTION_CHANGED","远程角色选择已经改变，请重新读取当前连接"));}Ok(())
}
async fn ensure_gateway(app:&AppHandle,token:Token)->ChatResult<Value> {
    ensure_gateway_with(app,token,false).await
}
/// `fresh=true` 时跳过现有连接的复用分支，直接建立新连接；新连接就绪后才替换并停止旧连接。
async fn ensure_gateway_with(app:&AppHandle,token:Token,fresh:bool)->ChatResult<Value> {
    let runtime=app.state::<Runtime>();let _connect=runtime.connect_lock.lock().await;
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();validate_token(&app,&token)?;
    let requested=runtime.desired.lock().map_err(|_|inactive())?.clone().filter(|(target,_)|*target==token).ok_or_else(inactive)?.1;
    if *runtime.retry_blocked.lock().map_err(|_|inactive())? {return Err(blocked_error(&runtime));}
    // 恢复意图已经登记；角色发现失败也会由原生监督循环继续恢复。
    let principal_id=match requested.clone().or(cache::principal(app,&token,None)?) {
        Some(principal)=>principal,
        None=> {
            let key=cache_key(&operations::Query::Agents);let ticket=cache::begin_read(app,&token,&key)?;
            let agents=tokio::select! {biased;
                _=changes.changed()=>return Err(inactive()),
                result=async {let session=session(app,&token).await?;perform(&session,operations::Query::Agents.route()?).await}=>result?,
            };
            let items=agents.as_array().ok_or_else(||ChatError::new("REMOTE_PROTOCOL","远程角色列表格式无效"))?;
            let principal=items.iter().find(|agent|agent["isEnabled"]==true).or_else(||items.first())
                .and_then(|agent|agent["id"].as_str()).ok_or_else(||ChatError::new("REMOTE_NOT_FOUND","服务端尚无可用角色"))?.to_owned();
            cache::save(app,&token,&key,ticket,&agents)?;principal
        },
    };
    operations::segment(&principal_id)?;
    if principal_id.len()>64 {return Err(ChatError::new("REMOTE_PROTOCOL","远程角色身份过长"));}
    let existing=if fresh {None} else {runtime.gateway.lock().map_err(|_|inactive())?.as_ref().filter(|h|h.token==token && h.principal==principal_id && h.alive()).cloned()};
    if let Some(handle)=existing {
        restore_subscriptions(app,&token,&handle).await?;
        validate_selection(&runtime,&token,&requested)?;
        if *runtime.retry_blocked.lock().map_err(|_|inactive())? {return Err(blocked_error(&runtime));}
        return Ok(json!({"modeToken":token,"state":"online","principalId":principal_id,"sessionId":handle.session_id}));
    }
    let new=tokio::select! {biased;
        _=changes.changed()=>return Err(inactive()),
        result=async {let session=session(app,&token).await?;gateway::connect(app,&session,&principal_id).await}=>result?,
    };
    {
        let _gate=match mode.gate(Some(&token),false) {Ok(gate)=>gate,Err(error)=> {new.stop();return Err(error);}};
        if let Err(error)=validate_selection(&runtime,&token,&requested) {new.stop();return Err(error);}
        let runtime=app.state::<Runtime>();let mut handle=runtime.gateway.lock().map_err(|_|inactive())?;
        if let Some(old)=handle.replace(new.clone()) {old.stop();}
        runtime.confirmed_subscriptions.lock().map_err(|_|inactive())?.clear();
    }
    restore_subscriptions(app,&token,&new).await?;
    if let Err(error)=validate_selection(&runtime,&token,&requested) {new.stop();return Err(error);}
    cache::principal(app,&token,Some(&principal_id))?;
    let _gate=mode.gate(Some(&token),false)?;
    if *runtime.retry_blocked.lock().map_err(|_|inactive())? {new.stop();return Err(blocked_error(&runtime));}
    let data=json!({"state":"online","principalId":principal_id,"sessionId":new.session_id});
    app.emit("remote-event",json!({"modeToken":token,"kind":"connection","data":data})).map_err(|_|inactive())?;
    Ok(json!({"modeToken":token,"state":"online","principalId":principal_id,"sessionId":new.session_id}))
}
async fn restore_subscriptions(app:&AppHandle,token:&Token,handle:&gateway::Handle)->ChatResult<()> {
    let runtime=app.state::<Runtime>();
    let desired=runtime.subscriptions.lock().map_err(|_|inactive())?.streams();
    let confirmed=runtime.confirmed_subscriptions.lock().map_err(|_|inactive())?.clone();
    for stream in desired.difference(&confirmed) {
        validate_token(app,token)?;
        let cursor=runtime.cursors.lock().map_err(|_|inactive())?.get(stream).copied().unwrap_or(0);
        // 收到订阅确认才算恢复；失败撤销此连接，下轮重新握手并补齐全部订阅。
        let result=match handle.request("gateway.subscribe",json!({"streamId":stream,"lastSequence":cursor})).await {
            Ok(value)=>value,
            Err(error)=> {handle.stop();runtime.confirmed_subscriptions.lock().map_err(|_|inactive())?.clear();return Err(error);},
        };
        let latest=result["latestSequence"].as_u64().ok_or_else(||ChatError::new("REMOTE_PROTOCOL","Gateway 未返回有效的恢复游标"))?;
        if result["snapshotRequired"]==true || latest<cursor {
            let _gate=app.state::<Coordinator>().gate(Some(token),false)?;
            // 服务端重启或事件窗口已过期时，先通知前端重新读权威快照，不能沿用旧序号。
            runtime.cursors.lock().map_err(|_|inactive())?.insert(stream.clone(),latest);
            app.emit("remote-event",json!({"modeToken":token,"kind":"snapshot_required","data":{"streamId":stream}})).map_err(|_|inactive())?;
        } else {
            let events=result["events"].as_array().ok_or_else(||ChatError::new("REMOTE_PROTOCOL","Gateway 补发列表无效"))?;
            for event in events {
                if event["streamId"]!=*stream {return Err(ChatError::new("REMOTE_PROTOCOL","Gateway 补发流身份不匹配"));}
                gateway::accept_push(app,token,event.clone())?;
            }
            let _gate=app.state::<Coordinator>().gate(Some(token),false)?;
            let mut cursors=runtime.cursors.lock().map_err(|_|inactive())?;
            let current=cursors.entry(stream.clone()).or_default();*current=(*current).max(latest);
        }
        let _gate=app.state::<Coordinator>().gate(Some(token),false)?;
        runtime.confirmed_subscriptions.lock().map_err(|_|inactive())?.insert(stream.clone());
    }
    Ok(())
}
fn gateway_handle(app:&AppHandle,token:&Token)->ChatResult<gateway::Handle> {
    validate_token(app,token)?;
    app.state::<Runtime>().gateway.lock().map_err(|_|inactive())?.as_ref().filter(|handle|handle.token==*token && handle.alive()).cloned()
        .ok_or_else(||ChatError::new("REMOTE_GATEWAY_OFFLINE","远程交互连接已断开，请重新连接"))
}

#[derive(Deserialize)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum Subscription {Thread {id:String},Execution {id:String},Notifications {#[serde(rename="principalId")] principal_id:String}}
#[tauri::command]
pub async fn remote_subscribe(app:AppHandle,token:Token,subscription:Subscription)->ChatResult<Value> {
    let stream=match subscription {Subscription::Thread {id}=> {operations::segment(&id)?;format!("thread:{id}")},
        Subscription::Execution {id}=> {operations::segment(&id)?;format!("execution:{id}")},
        Subscription::Notifications {principal_id}=> {operations::segment(&principal_id)?;format!("notification:{principal_id}")}};
    let _handle=gateway_handle(&app,&token)?;
    {
        // 订阅集合只是本机记录，只读校验即可；达到上限时淘汰最久未使用的会话流，而不是拒绝新会话。
        app.state::<Coordinator>().verify(&token)?;
        let runtime=app.state::<Runtime>();let mut subscriptions=runtime.subscriptions.lock().map_err(|_|inactive())?;
        if !subscriptions.contains(&stream) && subscriptions.len()>=SUBSCRIPTION_LIMIT {
            let victim=subscriptions.evict_lru(&stream).ok_or_else(||ChatError::new("REMOTE_SUBSCRIPTION_LIMIT","远程订阅过多，请关闭不用的会话"))?;
            runtime.confirmed_subscriptions.lock().map_err(|_|inactive())?.remove(&victim);
            eprintln!("远程订阅已达上限，淘汰最久未使用的流：{victim}");
        }
        subscriptions.touch(&stream);
    }

    // 与重连串行恢复全部期望订阅，存活连接不等于订阅已经确认。
    ensure_gateway(&app,token.clone()).await?;
    validate_token(&app,&token)?;Ok(json!({"modeToken":token,"data":{"streamId":stream,"subscribed":true}}))
}

#[derive(Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct SurfaceInput {pub surface_id:String,pub node_id:String,pub generation:String,pub action:String,pub payload:Value}
#[tauri::command]
pub async fn remote_surface_input(app:AppHandle,token:Token,request_id:String,input:SurfaceInput)->ChatResult<Value> {
    surface_input_inner(app,token,request_id,input).await.map_err(transport::before_send)
}
async fn surface_input_inner(app:AppHandle,token:Token,request_id:String,input:SurfaceInput)->ChatResult<Value> {
    if uuid::Uuid::parse_str(&request_id).is_err() || !input.payload.is_object()
        || [&input.surface_id,&input.node_id,&input.generation].iter().any(|id|id.is_empty() || id.len()>1024 || id.chars().any(char::is_control))
        || !["approval.resolve","agent-input.resolve","background-task.submit-input"].contains(&input.action.as_str()) || input.payload.to_string().len()>64*1024 {return Err(ChatError::new("INVALID_INPUT","远程交互输入无效"));}
    let runtime=app.state::<Runtime>();let _input=runtime.input_lock.lock().await;
    validate_token(&app,&token)?;
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();
    let session=session(&app,&token).await?;
    // 角色归属由服务端请求查询决定，不能借当前界面角色代替审批/问题的所有者。
    let principal=resolve_input_principal(&session,&input).await?;
    let mut prepared=false;
    let mut result=Err(ChatError::new("REMOTE_GATEWAY_OFFLINE","交互连接尚未就绪，请稍后重试"));
    for attempt in 0..2 {
        // 获取席位没有业务副作用，可以在握手丢失后恢复一次；提交超时不能重放。
        if let Err(error)=ensure_gateway(&app,token.clone()).await {result=Err(error);break;}
        let handle=gateway_handle(&app,&token)?;
        let seat=match handle.request("input_seat.acquire",json!({"principalId":principal,"windowId":handle.window_id,"leaseMs":60000})).await {
            Ok(value)=>value["seat"].clone(),
            Err(error) if attempt==0 && ["REMOTE_INPUT_SESSION_EXPIRED","REMOTE_NETWORK"].contains(&error.code.as_str())=> {handle.stop();continue;},
            Err(error)=> {result=Err(error);break;},
        };
        if seat["principalId"]!=principal || seat["windowId"]!=handle.window_id || seat["sessionId"]!=handle.session_id || !seat["seatId"].as_str().is_some_and(|id|!id.is_empty()) || !seat["epoch"].as_u64().is_some_and(|epoch|epoch>0) {
            result=Err(ChatError::new("REMOTE_CONFLICT","远程输入席位身份无效，请重新连接"));break;
        }
        if !prepared {
            if let Err(error)=cache::prepare(&app,&token,&request_id,&json!({"kind":"surface_input","input":input,"principalId":principal})) {
                let _=handle.request("input_seat.release",json!({"seatId":seat["seatId"]})).await;return Err(error);
            }
            prepared=true;
        }
        let mut body=json!(input);body["seat"]=json!({"seatId":seat["seatId"],"sessionId":seat["sessionId"],"windowId":seat["windowId"],"epoch":seat["epoch"]});
        body["payload"]["principalId"]=json!(principal);
        result=tokio::select! {biased;
            _=changes.changed()=>return Err(inactive().with_outcome("unknown")),
            result=perform(&session,operations::Route {method:reqwest::Method::POST,path:"/surfaces/input".into(),params:Vec::new(),body:Some(body)})=>result,
        };
        let _=handle.request("input_seat.release",json!({"seatId":seat["seatId"]})).await;
        if attempt==0 && result.as_ref().is_err_and(|error|error.code=="REMOTE_INPUT_SEAT_RETRY" && error.outcome.as_deref()==Some("rejected")) {continue;}
        break;
    }
    let data=if prepared {cache::settle(&app,&token,&request_id,result)?} else {result?};
    validate_token(&app,&token).map_err(|error|error.with_outcome("unknown"))?;
    Ok(json!({"modeToken":token,"requestId":request_id,"data":data}))
}
async fn resolve_input_principal(session:&Session,input:&SurfaceInput)->ChatResult<String> {
    let request=if input.action=="approval.resolve" {
        let id=input.payload["approvalId"].as_str().ok_or_else(inactive)?;
        perform(session,operations::Route {method:reqwest::Method::GET,path:format!("/approvals/{}",operations::segment(id)?),params:Vec::new(),body:None}).await?
    } else if input.action=="agent-input.resolve" {
        let id=input.payload["inputId"].as_str().ok_or_else(inactive)?;
        let value=perform(session,operations::Query::Questions.route()?).await?;
        value["requests"].as_array().and_then(|items|items.iter().find(|request|request["id"]==id)).cloned().ok_or_else(||ChatError::new("REMOTE_CONFLICT","问题已处理或失效，请刷新"))?
    } else {
        let id=input.surface_id.strip_prefix("background-task:").ok_or_else(inactive)?;
        perform(session,operations::Query::Task {id:id.into()}.route()?).await?
    };
    if request.get("status").is_some_and(|status|status!="pending" && status!="waiting_input") && input.action!="background-task.submit-input" {
        return Err(ChatError::new("REMOTE_CONFLICT","此交互已经处理，请刷新"));
    }
    request["agentId"].as_str().filter(|id|!id.is_empty()).map(str::to_owned).ok_or_else(||ChatError::new("REMOTE_PROTOCOL","服务端未提供交互归属"))
}

#[tauri::command(async)]
pub fn remote_pending_requests(app:AppHandle,token:Token)->ChatResult<Value> {cache::pending(&app,&token)}

#[tauri::command(async)]
pub fn remote_review_request(app:AppHandle,token:Token,request_id:String,fingerprint:String,note:String)->ChatResult<Value> {
    cache::review(&app,&token,&request_id,&fingerprint,&note)?;
    Ok(json!({"modeToken":token,"requestId":request_id,"state":"reviewed"}))
}

#[tauri::command]
pub async fn remote_reconcile_request(app:AppHandle,token:Token,request_id:String)->ChatResult<Value> {
    let record=cache::request(&app,&token,&request_id)?;
    if record["state"]!="unknown" {return Ok(json!({"modeToken":token,"requestId":request_id,"state":record["state"]}));}
    let input=&record["context"]["input"];
    if record["context"]["transport"]=="queue" {
        let thread=input["threadId"].as_str().ok_or_else(inactive)?;
        let agent=input["agentId"].as_str().ok_or_else(inactive)?;
        let session=session(&app,&token).await?;
        let snapshot=perform(&session,operations::Query::Queue {id:thread.into()}.route()?).await?;
        validate_token(&app,&token)?;validate_queue(&snapshot,thread)?;
        // 只用服务端保存的客户端消息 UUID 确认入队，不能按正文猜测或自动重发。
        let mut accepted=snapshot["items"].as_array().is_some_and(|items|items.iter().any(|item|item["id"]==request_id && item["input"]["agentId"]==agent));
        if !accepted {
            let projection=perform(&session,operations::Query::Conversation {id:thread.into(),before_cursor:None,page_size:60}.route()?).await?;
            validate_token(&app,&token)?;
            if projection["protocolVersion"]!=1 || projection["threadId"]!=thread {return Err(ChatError::new("REMOTE_PROTOCOL","队列核对返回了其他会话"));}
            let pair=format!("queue_{request_id}");
            accepted=projection["messages"].as_array().is_some_and(|messages|messages.iter().any(|message|message["role"]=="user" && message["threadId"]==thread && message["pairId"]==pair));
        }
        if accepted {
            match cache::confirm(&app,&token,&request_id,Ok(json!({"threadId":thread,"queueItemId":request_id,"accepted":true}))) {
                Ok(_)=>{},Err(error) if error.code=="REQUEST_STATE_CHANGED"=>{},Err(error)=>return Err(error),
            }
            return Ok(json!({"modeToken":token,"requestId":request_id,"state":cache::request(&app,&token,&request_id)?["state"]}));
        }
        return Ok(json!({"modeToken":token,"requestId":request_id,"state":"unknown"}));
    }
    // 只凭已经观察到的持久消息身份确认；不按相同文本、时间或列表中缺少消息来猜测结果。
    if record["context"]["kind"]=="chat" {
        if let (Some(thread),Some(agent),Some(message))=(input["threadId"].as_str(),input["agentId"].as_str(),record["result"]["messageId"].as_str()) {
            let session=session(&app,&token).await?;
            let projection=perform(&session,operations::Query::Conversation {id:thread.into(),before_cursor:None,page_size:60}.route()?).await?;
            validate_token(&app,&token)?;
            if projection["protocolVersion"]!=1 || projection["threadId"]!=thread || !projection["surfaces"].is_array() {return Err(ChatError::new("REMOTE_INCOMPATIBLE","核对请求未返回兼容的会话投影"));}
            if let Some(surface)=projection["surfaces"].as_array().and_then(|items|items.iter().find(|surface|
                surface["messageId"]==message && surface["threadId"]==thread && surface["principalId"]==agent && matches!(surface["state"].as_str(),Some("committed"|"failed")))) {
                let state=if surface["state"]=="failed" {"failed"} else {"completed"};
                let receipt=if state=="completed" {Ok(json!({"threadId":thread,"messageId":message,"confirmedFromProjection":true}))}
                    else {Err(ChatError::new("REMOTE_EXECUTION_FAILED","服务端持久消息已确认执行失败").with_outcome("failed"))};
                // 状态检查与回执落盘共用事务，不能覆盖并发的显式核对。
                match cache::confirm(&app,&token,&request_id,receipt) {
                    Ok(_)=>{},Err(error) if error.outcome.as_deref()==Some("failed")=>{},
                    Err(error) if error.code=="REQUEST_STATE_CHANGED"=>return Ok(json!({"modeToken":token,"requestId":request_id,"state":cache::request(&app,&token,&request_id)?["state"]})),
                    Err(error)=>return Err(error),
                }
                return Ok(json!({"modeToken":token,"requestId":request_id,"state":state}));
            }
        }
    }
    Ok(json!({"modeToken":token,"requestId":request_id,"state":"unknown"}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn queued_chat_keeps_generation_and_legacy_retry_fingerprints() {
        let old = json!({"requestId":"request-a","threadId":"main","agentId":"pero","content":"原消息","attachmentIds":[],"imageMode":null,"workspaceContext":null,"generationType":null});
        let request: ChatRequest = serde_json::from_value(old.clone()).unwrap();
        assert_eq!(serde_json::to_value(request).unwrap(), old);
        let mut current = old;
        current["expectedGeneration"] = json!(7);
        let request: ChatRequest = serde_json::from_value(current).unwrap();
        assert_eq!(request.expected_generation, Some(7));
    }
    #[test]
    fn surface_version_is_checked_before_forwarding() {
        assert!(validate_frame(&json!({"protocolVersion":2,"operation":{"type":"surface.open"}})).is_err());
        assert!(validate_frame(&json!({"protocolVersion":1,"surfaceId":"a","generation":"g","operationId":"o","sequence":1,"revision":1,"operation":{"type":"surface.open"}})).is_ok());
    }
    #[test]
    fn lru_evicts_oldest_conversation_stream_and_keeps_notifications_and_requested_stream() {
        let mut set=StreamSet::default();
        set.touch("notification:p1");
        set.touch("thread:a");
        set.touch("thread:b");
        // 重新使用 a 之后，最久未使用的是 b。
        set.touch("thread:a");
        assert_eq!(set.evict_lru("thread:new"),Some("thread:b".to_owned()));
        assert!(!set.contains("thread:b") && set.contains("thread:a") && set.contains("notification:p1"));
        // 被请求的流受保护；只剩通知流时无可淘汰项。
        assert_eq!(set.evict_lru("thread:a"),None);
        assert!(set.contains("thread:a"));
    }
    #[test]
    fn lru_never_evicts_notifications_even_when_they_are_oldest() {
        let mut set=StreamSet::default();
        set.touch("notification:p1");
        set.touch("execution:x");
        assert_eq!(set.evict_lru("thread:new"),Some("execution:x".to_owned()));
        assert_eq!(set.evict_lru("thread:new"),None);
        assert!(set.contains("notification:p1"));
    }
}
