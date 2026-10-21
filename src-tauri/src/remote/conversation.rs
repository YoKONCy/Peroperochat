//! 消息修订、检查点回滚与重新生成由原生层串接，页面只提交意图与确认。
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use tauri::{AppHandle,Manager};
use crate::{mode::{Coordinator,Token},local::types::*};
use super::{cache,operations::{Action,Query}};

#[derive(Default)]
pub struct Workflows(pub tokio::sync::Mutex<()>);
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
struct Plan {id:String,token:Token,thread_id:String,agent_id:String,message_id:u64,input:Value,preview:Value,phase:String,#[serde(default)] rewind_request_id:Option<String>,#[serde(default)] send_request_id:Option<String>}
fn connection(app:&AppHandle,token:&Token)->ChatResult<rusqlite::Connection> {
    let conn=cache::connection(app,token)?;
    conn.execute_batch("CREATE TABLE IF NOT EXISTS regeneration_plans(id TEXT PRIMARY KEY,value TEXT NOT NULL,created INTEGER NOT NULL)")?;Ok(conn)
}
fn save(app:&AppHandle,plan:&Plan)->ChatResult<()> {
    let raw=serde_json::to_string(plan).map_err(|_|super::inactive())?;
    if raw.len()>2*1024*1024 {return Err(ChatError::new("REMOTE_TOO_LARGE","重新生成清单过大，请在原会话处理"));}
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(&plan.token),false)?;
    let conn=connection(app,&plan.token)?;
    conn.execute("INSERT INTO regeneration_plans VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET value=excluded.value",rusqlite::params![plan.id,raw,chrono::Utc::now().timestamp_millis()])?;
    conn.execute("DELETE FROM regeneration_plans WHERE id IN (SELECT id FROM regeneration_plans WHERE json_extract(value,'$.phase')='completed' ORDER BY created DESC LIMIT -1 OFFSET 10)",[])?;Ok(())
}
fn load(app:&AppHandle,token:&Token,id:&str)->ChatResult<Plan> {
    super::validate_token(app,token)?;
    let conn=connection(app,token)?;let raw:String=conn.query_row("SELECT value FROM regeneration_plans WHERE id=?1",[id],|r|r.get(0))?;
    let plan:Plan=serde_json::from_str(&raw).map_err(|_|super::inactive())?;
    if plan.token!=*token {return Err(ChatError::new("REGENERATION_EXPIRED","运行模式已改变，请恢复输入后重新确认，不继续旧回滚"));}Ok(plan)
}
fn input_from_projection(data:&Value,message:u64)->ChatResult<(u64,Value)> {
    let messages=data["messages"].as_array().ok_or_else(||ChatError::new("REMOTE_PROTOCOL","消息已保存，但服务端未返回对话快照"))?;
    let index=messages.iter().position(|item|item["messageId"].as_str().and_then(|id|id.parse::<u64>().ok()).or_else(||item["messageId"].as_u64())==Some(message))
        .ok_or_else(||ChatError::new("REMOTE_PROTOCOL","消息已保存，但快照缺少目标消息"))?;
    let target=&messages[index];
    let input=messages[..=index].iter().rev().find(|item|item["role"]=="user" && (target["pairId"].is_null() || item["pairId"]==target["pairId"]))
        .ok_or_else(||ChatError::new("REMOTE_PROTOCOL","消息已保存，但未找到本轮输入，请刷新原会话"))?;
    let id=input["messageId"].as_str().and_then(|id|id.parse().ok()).or_else(||input["messageId"].as_u64()).ok_or_else(super::inactive)?;
    let content=input["content"].as_str().filter(|s|!s.trim().is_empty()).ok_or_else(super::inactive)?;
    let ids:Vec<_>=input["attachments"].as_array().into_iter().flatten().filter_map(|file|file["id"].as_str()).collect();
    Ok((id,json!({"content":content,"attachmentIds":ids,"attachments":input["attachments"],"revision":input["revision"],"pairId":input["pairId"]})))
}
fn scope(preview:&Value)->Value {json!({"threadId":preview["threadId"],"agentId":preview["agentId"],"wholeThread":preview["wholeThread"],"targetMessageId":preview["targetMessageId"],"pairIds":preview["pairIds"],"pairCount":preview["pairCount"],"files":preview["files"],"forceWarning":preview["forceWarning"]})}
fn recovered_phase(phase:&str,receipt:Option<&str>)->String {
    match (phase,receipt) {
        ("rewinding",Some("completed"))=>"rewound",
        ("submitting",Some("completed"))=>"completed",
        ("rewinding",Some("rejected"|"failed"|"not_sent"))=>"ready",
        ("submitting",Some("rejected"|"failed"|"not_sent"))=>"rewound",
        _=>phase,
    }.into()
}
fn effort_options(model:&str)->Value {
    let lower=model.to_ascii_lowercase();
    let levels=lower.find("gemini").is_some_and(|index| {
        let rest=lower[index+6..].trim_start_matches(['-','_',' ']);
        rest.strip_prefix('4').is_some_and(|tail|tail.is_empty() || tail.starts_with(['.','_',' ','/','-']))
    });
    let options:Vec<_>=[("","跟随模型"),("off","关闭思考"),("low","低"),("medium","中"),("high","高"),("xhigh","超高"),("max","最高")].into_iter()
        .filter(|(value,_)|!levels || !["off","xhigh","max"].contains(value)).map(|(value,label)|json!({"value":value,"label":label})).collect();json!(options)
}
#[tauri::command]
pub async fn remote_model_controls(app:AppHandle,token:Token,thread_id:String)->ChatResult<Value> {
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();
    let data=tokio::select! {biased;_ =changes.changed()=>return Err(super::inactive()),result=async {
        let session=super::session(&app,&token).await?;
        let mut value=super::perform(&session,super::operations::Route {method:reqwest::Method::GET,path:format!("/chat/threads/{}/model-settings",super::operations::segment(&thread_id)?),params:vec![],body:None}).await?;
        value["effortOptions"]=effort_options(value["effective"]["modelId"].as_str().unwrap_or_default());
        let relay=super::perform(&session,super::operations::Route {method:reqwest::Method::POST,path:"/configs/batch".into(),params:vec![],body:Some(json!({"keys":["multimodalRelay.enabled","multimodalRelay.modelConfigId"]}))}).await;
        value["relayAvailable"]=relay.map(|v|json!(v["multimodalRelay.enabled"]=="true" && v["multimodalRelay.modelConfigId"].as_str().is_some_and(|s|!s.is_empty()))).unwrap_or(Value::Null);
        Ok::<_,ChatError>(value)
    }=>result?};
    super::validate_token(&app,&token)?;Ok(json!({"modeToken":token,"data":data}))
}
async fn preview(app:&AppHandle,token:&Token,thread:&str,message:u64)->ChatResult<Value> {
    let session=super::session(app,token).await?;
    super::perform(&session,Action::RewindPreview {id:thread.into(),message_id:Some(message),whole_thread:false}.route()?).await
}
#[tauri::command]
pub async fn remote_prepare_regeneration(app:AppHandle,token:Token,request_id:String,thread_id:String,agent_id:String,message_id:u64,content:String,revision:u64)->ChatResult<Value> {
    let state=app.state::<Workflows>();let _lock=state.0.lock().await;super::validate_token(&app,&token)?;
    let count:i64=connection(&app,&token)?.query_row("SELECT COUNT(*) FROM regeneration_plans WHERE json_extract(value,'$.phase') NOT IN ('completed','discarded')",[],|r|r.get(0))?;
    if count>=20 {return Err(ChatError::new("REQUEST_UNRESOLVED","有较多未完成的重新生成记录，请先恢复输入或核对后处理"));}
    if message_id==0 || revision==0 || content.trim().is_empty() || content.len()>128*1024 {return Err(ChatError::new("INVALID_INPUT","消息修订或内容无效"));}
    let session=super::session(&app,&token).await?;
    let thread=super::perform(&session,Query::Thread {id:thread_id.clone()}.route()?).await?;
    let actual_agent=thread["thread"]["agentId"].as_str().ok_or_else(super::inactive)?;
    if thread["thread"]["channel"]!="desktop" {return Err(ChatError::new("INVALID_INPUT","此重新生成流程仅用于普通会话，群聊请使用房间操作"));}
    if !agent_id.is_empty() && actual_agent!=agent_id {return Err(ChatError::new("REMOTE_CONFLICT","角色与目标会话不一致"));}
    let agent_id=actual_agent.to_string();
    let result=super::remote_action(app.clone(),token.clone(),request_id,Action::EditMessage {thread_id:thread_id.clone(),id:message_id.to_string(),content,expected_revision:Some(revision)}).await?;
    let (message_id,input)=input_from_projection(&result["data"],message_id)?;
    let mut plan=Plan {id:uuid::Uuid::new_v4().to_string(),token:token.clone(),thread_id,agent_id,message_id,input,preview:Value::Null,phase:"saved".into(),rewind_request_id:None,send_request_id:None};
    save(&app,&plan)?;
    match preview(&app,&token,&plan.thread_id,plan.message_id).await {
        Ok(value)=> {plan.preview=value;plan.phase="ready".into();save(&app,&plan)?;},
        Err(_)=>return Ok(json!({"modeToken":token,"data":{"planId":plan.id,"state":"saved","message":"正文已保存，回滚预览读取失败；可取消或刷新后重试。"}})),
    }
    Ok(json!({"modeToken":token,"data":{"planId":plan.id,"state":"review","preview":plan.preview}}))
}
#[tauri::command]
pub async fn remote_confirm_rewind(app:AppHandle,token:Token,request_id:String,thread_id:String,message_id:Option<u64>,whole_thread:bool,expected:Value)->ChatResult<Value> {
    if expected.to_string().len()>1024*1024 {return Err(ChatError::new("INVALID_INPUT","回滚清单过大，请缩小范围"));}
    let session=super::session(&app,&token).await?;
    let fresh=super::perform(&session,Action::RewindPreview {id:thread_id.clone(),message_id,whole_thread}.route()?).await?;
    super::validate_token(&app,&token)?;
    if scope(&fresh)!=scope(&expected) {return Ok(json!({"modeToken":token,"data":{"state":"review","preview":fresh}}));}
    super::remote_action(app.clone(),token.clone(),request_id,Action::Rewind {id:thread_id,message_id,whole_thread,preserve_input_attachments:false}).await?;
    Ok(json!({"modeToken":token,"data":{"state":"completed"}}))
}
#[tauri::command]
pub async fn remote_commit_regeneration(app:AppHandle,token:Token,id:String)->ChatResult<Value> {
    let state=app.state::<Workflows>();let _lock=state.0.lock().await;let mut plan=load(&app,&token,&id)?;
    if !["ready","saved","rewound"].contains(&plan.phase.as_str()) {return Err(ChatError::new("REQUEST_UNRESOLVED","重新生成已执行或结果待核对，不会重复提交"));}
    if plan.phase!="rewound" {
        let fresh=preview(&app,&token,&plan.thread_id,plan.message_id).await?;
        if plan.phase=="saved" || scope(&fresh)!=scope(&plan.preview) {plan.preview=fresh;plan.phase="ready".into();save(&app,&plan)?;return Ok(json!({"modeToken":token,"data":{"state":"review","preview":plan.preview,"planId":id}}));}
        // 再次读取本轮输入，避免确认期间其他客户端修订后仍发送旧内容。
        let session=super::session(&app,&token).await?;
        let cursor=plan.message_id.checked_add(1).ok_or_else(||ChatError::new("REMOTE_PROTOCOL","消息身份超出范围"))?;
        let snapshot=super::perform(&session,Query::Conversation {id:plan.thread_id.clone(),before_cursor:Some(cursor.to_string()),page_size:1}.route()?).await?;
        let (_,input)=input_from_projection(&snapshot,plan.message_id)?;
        if input!=plan.input {return Err(ChatError::new("REMOTE_CONFLICT","本轮输入已由其他客户端修改，请关闭后重新编辑并确认").with_outcome("not_sent"));}
        let rewind_id=uuid::Uuid::new_v4().to_string();plan.rewind_request_id=Some(rewind_id.clone());plan.phase="rewinding".into();save(&app,&plan)?;
        let result=super::remote_action(app.clone(),token.clone(),rewind_id,Action::Rewind {id:plan.thread_id.clone(),message_id:Some(plan.message_id),whole_thread:false,preserve_input_attachments:true}).await;
        if let Err(error)=result {
            plan.phase=if matches!(error.outcome.as_deref(),Some("not_sent"|"rejected"|"failed")) {"ready"} else {"unknown"}.into();let _=save(&app,&plan);return Err(error);
        }
        plan.phase="rewound".into();save(&app,&plan)?;
    }
    let send_id=uuid::Uuid::new_v4().to_string();plan.send_request_id=Some(send_id.clone());plan.phase="submitting".into();save(&app,&plan)?;
    let request=super::ChatRequest {request_id:send_id,thread_id:plan.thread_id.clone(),agent_id:plan.agent_id.clone(),content:plan.input["content"].as_str().unwrap_or_default().into(),attachment_ids:serde_json::from_value(plan.input["attachmentIds"].clone()).map_err(|_|super::inactive())?,image_mode:Some("auto".into()),workspace_context:None,generation_type:Some("regenerate".into())};
    match super::remote_chat(app.clone(),token.clone(),request).await {
        Ok(_)=> {plan.phase="completed".into();save(&app,&plan)?;Ok(json!({"modeToken":token,"data":{"state":"accepted","threadId":plan.thread_id}}))},
        Err(error)=> {
            plan.phase=if matches!(error.outcome.as_deref(),Some("not_sent"|"rejected"|"failed")) {"rewound"} else {"unknown"}.into();let _=save(&app,&plan);
            if plan.phase=="rewound" {Ok(json!({"modeToken":token,"data":{"state":"rewound","planId":id,"message":"回滚已完成，但重新生成未入队。可重试提交或恢复输入；不会再次回滚。"}}))} else {Err(error)}
        },
    }
}
#[tauri::command]
pub async fn remote_regeneration_recovery(app:AppHandle,token:Token,thread_id:String)->ChatResult<Value> {
    let workflows=app.state::<Workflows>();let _lock=workflows.0.lock().await;
    super::validate_token(&app,&token)?;let conn=connection(&app,&token)?;
    let mut stmt=conn.prepare("SELECT value FROM regeneration_plans ORDER BY created DESC LIMIT 100")?;
    let rows=stmt.query_map([],|row|row.get::<_,String>(0))?;
    let rows=rows.collect::<Result<Vec<_>,_>>()?;drop(stmt);
    let mut recovery=Value::Null;
    for row in rows {let mut plan:Plan=serde_json::from_str(&row).map_err(|_|super::inactive())?;
        let step=if plan.phase=="unknown" {if plan.send_request_id.is_some() {"submitting"} else {"rewinding"}} else {plan.phase.as_str()};
        let receipt=if step=="rewinding" {plan.rewind_request_id.as_ref()} else if step=="submitting" {plan.send_request_id.as_ref()} else {None};
        let state=receipt.and_then(|id|conn.query_row("SELECT state FROM requests WHERE id=?1",[id],|r|r.get::<_,String>(0)).ok());
        let next=recovered_phase(step,state.as_deref());
        if next!=step && next!=plan.phase {plan.phase=next;let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(&token),false)?;conn.execute("UPDATE regeneration_plans SET value=?2 WHERE id=?1",rusqlite::params![plan.id,serde_json::to_string(&plan).map_err(|_|super::inactive())?])?;}
        if plan.thread_id==thread_id && plan.phase!="completed" && plan.phase!="discarded" {
            recovery=json!({"planId":plan.id,"state":plan.phase,"input":plan.input,"threadId":plan.thread_id,"agentId":plan.agent_id,"canRestore":plan.phase=="rewound","canDismiss":state.as_deref()==Some("reviewed")});break;
        }
    }
    super::validate_token(&app,&token)?;Ok(json!({"modeToken":token,"data":recovery}))
}
#[tauri::command]
pub async fn remote_discard_regeneration(app:AppHandle,token:Token,id:String)->ChatResult<Value> {
    let state=app.state::<Workflows>();let _lock=state.0.lock().await;
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(&token),false)?;let conn=connection(&app,&token)?;
    // 显式放弃只移除本机恢复草稿，不宣称撤销服务端操作。
    conn.execute("DELETE FROM regeneration_plans WHERE id=?1",[id])?;Ok(json!({"modeToken":token,"data":null}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn regeneration_uses_same_pair_input_and_original_attachments() {
        let snapshot=json!({"messages":[{"messageId":"1","role":"user","pairId":"a","content":"旧轮"},{"messageId":"2","role":"user","pairId":"b","content":"修订输入","attachments":[{"id":"image"}]},{"messageId":"3","role":"assistant","pairId":"b","content":"回答"}]});
        let (id,input)=input_from_projection(&snapshot,3).unwrap();assert_eq!(id,2);assert_eq!(input["content"],"修订输入");assert_eq!(input["attachmentIds"],json!(["image"]));
        assert!(input_from_projection(&snapshot,99).is_err());
    }
    #[test]
    fn follows_infos_reasoning_level_constraints() {
        assert_eq!(effort_options("gemini-4-pro").as_array().unwrap().len(),4);
        assert_eq!(effort_options("gemini-4.0-pro").as_array().unwrap().len(),4);
        assert_eq!(effort_options("gemini-40").as_array().unwrap().len(),7);
        assert_eq!(effort_options("other-model").as_array().unwrap().len(),7);
    }
    #[test]
    fn recovery_never_assumes_an_uncertain_mutation_succeeded() {
        assert_eq!(recovered_phase("rewinding",Some("completed")),"rewound");
        assert_eq!(recovered_phase("submitting",Some("completed")),"completed");
        assert_eq!(recovered_phase("rewinding",Some("rejected")),"ready");
        assert_eq!(recovered_phase("submitting",Some("not_sent")),"rewound");
        assert_eq!(recovered_phase("submitting",Some("reviewed")),"submitting");
        assert_eq!(recovered_phase("unknown",Some("completed")),"unknown");
        assert_eq!(recovered_phase("rewinding",None),"rewinding");
    }
    #[test]
    fn rewind_confirmation_includes_scope_and_file_preservation_decisions() {
        let before=json!({"threadId":"a","wholeThread":false,"targetMessageId":1,"pairIds":["p"],"pairCount":1,"files":[{"path":"a.txt","action":"restore_edited"}]});
        let mut changed=before.clone();changed["files"][0]["action"]=json!("preserve_changed");assert_ne!(scope(&before),scope(&changed));
        changed=before.clone();changed["targetMessageId"]=json!(2);assert_ne!(scope(&before),scope(&changed));
        changed=before.clone();changed["generatedAt"]=json!("later");assert_eq!(scope(&before),scope(&changed));
    }
}
