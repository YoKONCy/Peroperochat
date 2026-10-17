//! infOS 主体客户端。只执行传输、身份隔离和展示投影，不运行本地 Agent 或工具。
pub mod profiles;
pub mod vault;
mod transport;
mod cache;
mod operations;
mod gateway;
pub mod resources;
pub mod presentation;

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
    connect_lock:tokio::sync::Mutex<()>,
    subscriptions:Mutex<std::collections::HashSet<String>>,
    confirmed_subscriptions:Mutex<std::collections::HashSet<String>>,
}
pub(super) struct Session {token:Token,profile:profiles::Profile,secret:String,client:reqwest::Client}
pub(super) fn inactive()->ChatError {ChatError::new("MODE_INACTIVE","远程运行代次已失效，请刷新当前模式")}
pub fn stop_all(app:&AppHandle) {
    if let Ok(mut handle)=app.state::<Runtime>().gateway.lock() {if let Some(handle)=handle.take() {handle.stop();}}
    if let Ok(mut desired)=app.state::<Runtime>().desired.lock() {*desired=None;}
    if let Ok(mut subscriptions)=app.state::<Runtime>().subscriptions.lock() {subscriptions.clear();}
    if let Ok(mut subscriptions)=app.state::<Runtime>().confirmed_subscriptions.lock() {subscriptions.clear();}
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
                    let blocked=["REMOTE_AUTH_REQUIRED","REMOTE_IDENTITY_CHANGED","REMOTE_INCOMPATIBLE","CREDENTIAL_UNAVAILABLE"].contains(&error.code.as_str());
                    if let Ok(mut value)=app.state::<Runtime>().retry_blocked.lock() {*value=blocked;}
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
    let _gate=app.state::<Coordinator>().gate(Some(token),false)?;Ok(())
}
async fn session(app:&AppHandle,token:&Token)->ChatResult<Session> {
    validate_token(app,token)?;
    let Target::Remote {profile_id}= &token.target else {return Err(inactive());};
    let profile=profiles::get(&app.state::<Coordinator>(),profile_id)?;
    let secret=vault::load(app,profile_id)?;
    let identity=transport::probe(&profile.endpoint,&secret).await?;
    if identity.server_id!=profile.server_id {return Err(ChatError::new("REMOTE_IDENTITY_CHANGED","此地址对应的 infOS 实例已改变，请另存连接；旧缓存不会复用"));}
    validate_token(app,token)?;
    Ok(Session {token:token.clone(),profile,secret,client:transport::client()?})
}
async fn perform(session:&Session,route:operations::Route)->ChatResult<Value> {
    let mut url=reqwest::Url::parse(&format!("{}/api{}",session.profile.endpoint,route.path)).map_err(|_|transport::network())?;
    if !route.params.is_empty() {url.query_pairs_mut().extend_pairs(route.params);}
    let mut request=session.client.request(route.method,url).bearer_auth(&session.secret).timeout(std::time::Duration::from_secs(45));
    if let Some(body)=route.body {request=request.json(&body);}
    let response=request.send().await.map_err(transport::send_error)?;
    transport::status(response.status())?;transport::envelope(&transport::body(response).await?)
}
fn cache_key(query:&operations::Query)->String {serde_json::to_string(query).unwrap()}

#[tauri::command]
pub async fn remote_query(app:AppHandle,token:Token,query:operations::Query,#[allow(non_snake_case)] cached_only:Option<bool>)->ChatResult<Value> {
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();validate_token(&app,&token)?;
    let key=cache_key(&query);let route=query.route()?;
    if cached_only==Some(true) {return Ok(json!({"modeToken":token,"cached":true,"data":cache::load(&app,&token,&key)?}));}
    let ticket=cache::begin_read(&app,&token,&key)?;
    let mut data=tokio::select! {biased;
        _=changes.changed()=>return Err(inactive()),
        result=async {let session=session(&app,&token).await?;perform(&session,route).await}=>result?,
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
    if !cache::save(&app,&token,&key,ticket,&data)? {return Err(ChatError::new("REMOTE_STALE_READ","读取期间远程状态已刷新或发生操作，请重新读取"));}
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
    cache::prepare(&app,&token,&request_id,&json!({"kind":"action","input":action}))?;
    let result=tokio::select! {biased;_ =changes.changed()=>return Err(inactive().with_outcome("unknown")),result=perform(&session,route)=>result,};
    let data=cache::settle(&app,&token,&request_id,result)?;
    Ok(json!({"modeToken":token,"requestId":request_id,"data":data}))
}

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ChatRequest {pub request_id:String,pub thread_id:String,pub agent_id:String,pub content:String,#[serde(default)] pub attachment_ids:Vec<String>,pub image_mode:Option<String>}

#[tauri::command]
pub async fn remote_chat(app:AppHandle,token:Token,request:ChatRequest)->ChatResult<Value> {
    chat_inner(app,token,request).await.map_err(transport::before_send)
}
async fn chat_inner(app:AppHandle,token:Token,request:ChatRequest)->ChatResult<Value> {
    if request.content.trim().is_empty() || request.content.len()>128*1024 || request.attachment_ids.len()>5 || request.attachment_ids.iter().any(|id|uuid::Uuid::parse_str(id).is_err())
        || request.image_mode.as_ref().is_some_and(|mode|!["auto","native","relay"].contains(&mode.as_str())) {return Err(ChatError::new("INVALID_INPUT","远程聊天内容或附件无效"));}
    operations::segment(&request.thread_id)?;operations::segment(&request.agent_id)?;
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();validate_token(&app,&token)?;
    let session=tokio::select! {biased;_ =changes.changed()=>return Err(inactive()),result=session(&app,&token)=>result?,};
    cache::prepare(&app,&token,&request.request_id,&json!({"kind":"chat","input":request}))?;
    let mut body=json!({"threadId":request.thread_id,"agentId":request.agent_id,"content":request.content,"attachmentIds":request.attachment_ids});
    if let Some(image_mode)=&request.image_mode {body["imageMode"]=json!(image_mode);}
    let result=tokio::select! {biased;
        _=changes.changed()=>return Err(inactive().with_outcome("unknown")),
        result=async {
            let response=tokio::time::timeout(std::time::Duration::from_secs(45),session.client.post(format!("{}/api/chat/stream",session.profile.endpoint)).bearer_auth(&session.secret).json(&body).send())
                .await.map_err(|_|transport::network())?.map_err(transport::send_error)?;
            let mut observed=json!({});
            transport::stream(response,|kind,data| {
                if kind=="surface" {
                    validate_frame(&data)?;
                    let operation=&data["operation"];
                    let frame_thread=match operation["type"].as_str() {
                        Some("surface.open")=>operation.get("threadId"),
                        Some("surface.commit")=>operation["surface"].get("threadId"),_=>None,
                    };
                    if frame_thread.is_some_and(|thread|thread!=&Value::String(request.thread_id.clone())) {return Err(ChatError::new("REMOTE_PROTOCOL","Surface 帧会话身份不匹配"));}
                    let principal=match operation["type"].as_str() {
                        Some("surface.open")=>operation.get("principalId"),
                        Some("surface.commit")=>operation["surface"].get("principalId"),_=>None,
                    };
                    if principal.is_some_and(|principal|principal!=&Value::String(request.agent_id.clone())) {return Err(ChatError::new("REMOTE_PROTOCOL","Surface 帧角色身份不匹配"));}
                }
                if kind=="done" && data["threadId"]!=request.thread_id {return Err(ChatError::new("REMOTE_PROTOCOL","远程执行终态会话身份不匹配"));}
                let mut facts=json!({});
                for name in ["executionId","messageId"] {
                    let value=if kind=="surface" && name=="messageId" {&data["operation"]["surface"][name]} else {&data[name]};
                    if value.as_str().is_some_and(|value|!value.is_empty() && value.len()<=1024) && observed[name]!=*value {facts[name]=value.clone();observed[name]=value.clone();}
                }
                if facts.as_object().is_some_and(|value|!value.is_empty()) {cache::observe(&app,&token,&request.request_id,&facts)?;}
                let _gate=mode.gate(Some(&token),false)?;
                app.emit("remote-event",json!({"modeToken":token,"kind":kind,"requestId":request.request_id,"threadId":request.thread_id,"data":data}))
                    .map_err(|_|ChatError::new("REMOTE_EVENT_ERROR","远程显示事件投递失败"))?;Ok(())
            }).await?;
            Ok(json!({"threadId":request.thread_id,"state":"completed"}))
        }=>result,
    };
    cache::settle(&app,&token,&request.request_id,result)?;
    Ok(json!({"modeToken":token,"requestId":request.request_id,"threadId":request.thread_id,"state":"completed"}))
}
fn validate_frame(value:&Value)->ChatResult<()> {
    if value["protocolVersion"]!=1 || !value["surfaceId"].is_string() || !value["generation"].is_string() || !value["operationId"].is_string()
        || value["sequence"].as_u64().is_none() || value["revision"].as_u64().is_none() || !value["operation"]["type"].is_string() {
        return Err(ChatError::new("REMOTE_INCOMPATIBLE","Surface 流帧缺少必要身份或版本"));
    }
    Ok(())
}

#[tauri::command]
pub async fn remote_connect(app:AppHandle,token:Token,principal_id:Option<String>)->ChatResult<Value> {
    if let Some(principal)=&principal_id {
        operations::segment(principal)?;
        if principal.len()>64 {return Err(ChatError::new("INVALID_INPUT","远程角色身份过长"));}
    }
    validate_token(&app,&token)?;
    {
        let _gate=app.state::<Coordinator>().gate(Some(&token),false)?;
        *app.state::<Runtime>().desired.lock().map_err(|_|inactive())?=Some((token.clone(),principal_id.clone()));
        *app.state::<Runtime>().retry_blocked.lock().map_err(|_|inactive())?=false;
    }
    ensure_gateway(&app,token).await
}
fn validate_selection(runtime:&Runtime,token:&Token,principal:&Option<String>)->ChatResult<()> {
    let selected=runtime.desired.lock().map_err(|_|inactive())?;
    if selected.as_ref()!=Some(&(token.clone(),principal.clone())) {return Err(ChatError::new("REMOTE_SELECTION_CHANGED","远程角色选择已经改变，请重新读取当前连接"));}Ok(())
}
async fn ensure_gateway(app:&AppHandle,token:Token)->ChatResult<Value> {
    let runtime=app.state::<Runtime>();let _connect=runtime.connect_lock.lock().await;
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();validate_token(&app,&token)?;
    let requested=runtime.desired.lock().map_err(|_|inactive())?.clone().filter(|(target,_)|*target==token).ok_or_else(inactive)?.1;
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
    let existing=runtime.gateway.lock().map_err(|_|inactive())?.as_ref().filter(|h|h.token==token && h.principal==principal_id && h.alive()).cloned();
    if let Some(handle)=existing {
        restore_subscriptions(app,&token,&handle).await?;
        validate_selection(&runtime,&token,&requested)?;
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
    let data=json!({"state":"online","principalId":principal_id,"sessionId":new.session_id});
    app.emit("remote-event",json!({"modeToken":token,"kind":"connection","data":data})).map_err(|_|inactive())?;
    Ok(json!({"modeToken":token,"state":"online","principalId":principal_id,"sessionId":new.session_id}))
}
async fn restore_subscriptions(app:&AppHandle,token:&Token,handle:&gateway::Handle)->ChatResult<()> {
    let runtime=app.state::<Runtime>();
    let desired=runtime.subscriptions.lock().map_err(|_|inactive())?.clone();
    let confirmed=runtime.confirmed_subscriptions.lock().map_err(|_|inactive())?.clone();
    for stream in desired.difference(&confirmed) {
        validate_token(app,token)?;
        // 收到订阅确认才算恢复；失败撤销此连接，下轮重新握手并补齐全部订阅。
        if let Err(error)=handle.request("gateway.subscribe",json!({"streamId":stream,"lastSequence":0})).await {
            handle.stop();
            runtime.confirmed_subscriptions.lock().map_err(|_|inactive())?.clear();
            return Err(error);
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
        let _gate=app.state::<Coordinator>().gate(Some(&token),false)?;
        let runtime=app.state::<Runtime>();let mut subscriptions=runtime.subscriptions.lock().map_err(|_|inactive())?;
        if subscriptions.len()>=100 && !subscriptions.contains(&stream) {return Err(ChatError::new("REMOTE_SUBSCRIPTION_LIMIT","远程订阅过多，请关闭不用的会话"));}
        subscriptions.insert(stream.clone());
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
    let handle=gateway_handle(&app,&token)?;let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();
    let session=session(&app,&token).await?;
    // 角色归属由服务端请求查询决定，不能借当前界面角色代替审批/问题的所有者。
    let principal=resolve_input_principal(&session,&input).await?;
    let seat=handle.request("input_seat.acquire",json!({"principalId":principal,"windowId":handle.window_id,"leaseMs":60000})).await?["seat"].clone();
    if seat["principalId"]!=principal || seat["windowId"]!=handle.window_id || seat["sessionId"]!=handle.session_id || !seat["seatId"].as_str().is_some_and(|id|!id.is_empty()) || !seat["epoch"].as_u64().is_some_and(|epoch|epoch>0) {return Err(ChatError::new("REMOTE_CONFLICT","远程输入席位身份无效，请重新连接"));}
    let mut body=json!(input);body["seat"]=json!({"seatId":seat["seatId"],"sessionId":seat["sessionId"],"windowId":seat["windowId"],"epoch":seat["epoch"]});
    body["payload"]["principalId"]=json!(principal);
    if let Err(error)=cache::prepare(&app,&token,&request_id,&json!({"kind":"surface_input","input":input,"principalId":principal})) {
        let _=handle.request("input_seat.release",json!({"seatId":seat["seatId"]})).await;return Err(error);
    }
    let result=tokio::select! {biased;
        _=changes.changed()=>return Err(inactive().with_outcome("unknown")),
        result=perform(&session,operations::Route {method:reqwest::Method::POST,path:"/surfaces/input".into(),params:Vec::new(),body:Some(body)})=>result,
    };
    let result=cache::settle(&app,&token,&request_id,result);
    let _=handle.request("input_seat.release",json!({"seatId":seat["seatId"]})).await;
    let data=result?;
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

#[tauri::command]
pub fn remote_pending_requests(app:AppHandle,token:Token)->ChatResult<Value> {cache::pending(&app,&token)}

#[tauri::command]
pub fn remote_review_request(app:AppHandle,token:Token,request_id:String,fingerprint:String,note:String)->ChatResult<Value> {
    cache::review(&app,&token,&request_id,&fingerprint,&note)?;
    Ok(json!({"modeToken":token,"requestId":request_id,"state":"reviewed"}))
}

#[tauri::command]
pub async fn remote_reconcile_request(app:AppHandle,token:Token,request_id:String)->ChatResult<Value> {
    let record=cache::request(&app,&token,&request_id)?;
    if record["state"]!="unknown" {return Ok(json!({"modeToken":token,"requestId":request_id,"state":record["state"]}));}
    let input=&record["context"]["input"];
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
    fn surface_version_is_checked_before_forwarding() {
        assert!(validate_frame(&json!({"protocolVersion":2,"operation":{"type":"surface.open"}})).is_err());
        assert!(validate_frame(&json!({"protocolVersion":1,"surfaceId":"a","generation":"g","operationId":"o","sequence":1,"revision":1,"operation":{"type":"surface.open"}})).is_ok());
    }
}
