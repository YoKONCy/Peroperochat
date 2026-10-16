use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use super::{conversation::ConversationService, db_lock, identity, repository, runtime::ChatRuntime, types::*};
use crate::db::DbState;

#[derive(Clone, Serialize, Deserialize)]
pub struct Member { pub id: String, pub name: String, pub enabled: bool }

pub fn initialize(conn: &Connection) -> ChatResult<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS local_group_batches(request_id TEXT PRIMARY KEY, content TEXT NOT NULL, plan TEXT NOT NULL, status TEXT NOT NULL, error TEXT);")?;
    Ok(())
}

pub fn normalize(agents: &Value, saved: &Value) -> ChatResult<Vec<Member>> {
    let items = saved.as_array().ok_or_else(|| ChatError::new("INVALID_CONFIG", "群聊回复设置必须是列表"))?;
    let mut result = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for item in items {
        let id = item.get("id").and_then(Value::as_str).ok_or_else(|| ChatError::new("INVALID_CONFIG", "群聊角色身份无效"))?;
        if let Some(agent) = agents.get(id) {
            if seen.insert(id.to_owned()) {
                result.push(Member { id:id.into(), name:agent["name"].as_str().unwrap_or(id).into(),
                    enabled:item.get("enabled").and_then(Value::as_bool).unwrap_or(true) });
            }
        }
    }
    for (id, agent) in agents.as_object().into_iter().flatten() {
        if seen.insert(id.clone()) { result.push(Member { id:id.clone(),name:agent["name"].as_str().unwrap_or(id).into(),enabled:true }); }
    }
    Ok(result)
}

fn settings_from(config: &Value, agents: &Value) -> ChatResult<Vec<Member>> {
    let raw = config.get("ppc.group.replySettings").and_then(Value::as_str).unwrap_or("");
    let saved = if raw.trim().is_empty() { json!([]) } else {
        serde_json::from_str(raw).map_err(|_| ChatError::new("INVALID_CONFIG", "群聊回复设置损坏，请重新保存"))?
    };
    normalize(agents, &saved)
}

#[tauri::command]
pub fn local_group_settings(app: AppHandle, members: Option<Value>) -> ChatResult<Vec<Member>> {
    let db = app.state::<DbState>();
    let mut conn = db_lock(&db)?;
    let tx = conn.transaction()?;
    let config = identity::snapshot_in(&tx)?;
    let agents = identity::agents_from(&config)?;
    let result = match members { Some(value) => normalize(&agents,&value)?, None => settings_from(&config,&agents)? };
    identity::set_value_in(&tx,"ppc.group.replySettings",&json!(serde_json::to_string(&result).map_err(|_| ChatError::new("INVALID_CONFIG","群聊回复设置无效"))?))?;
    tx.commit()?;
    Ok(result)
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct GroupRequest { pub request_id:String, pub thread_id:String, pub content:Value }

pub async fn execute(service: &ConversationService<'_>, request: GroupRequest,
    on_event: impl Fn(ChatEvent), state: impl Fn(&str,&str)->ChatResult<Value>) -> ChatResult<TurnResult> {
    if uuid::Uuid::parse_str(&request.request_id).is_err() || request.thread_id != repository::FAMILY_THREAD {
        return Err(ChatError::new("INVALID_INPUT","群聊请求身份无效"));
    }
    validate_content(&request.content)?;
    let mut lease = service.runtime.reserve(&request.thread_id,&request.request_id)?;
    if *lease.cancel.borrow() { return Err(ChatError::new("CANCELLED","已停止回复")); }
    let plan: Vec<(String,String)> = {
        let conn = db_lock(service.db)?;
        let existing: Option<(String,String,String,Option<String>)> = conn.query_row(
            "SELECT content,plan,status,error FROM local_group_batches WHERE request_id=?1",[&request.request_id],
            |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
        if let Some((content,plan,status,error)) = existing {
            if content != request.content.to_string() { return Err(ChatError::new("INVALID_INPUT","同一请求不能替换群聊内容")); }
            if status == "failed" { return Err(error.and_then(|e|serde_json::from_str(&e).ok()).unwrap_or_else(|| ChatError::new("INTERRUPTED","上次群聊已中断，请重新发送"))); }
            serde_json::from_str(&plan).map_err(|_| ChatError::new("STORAGE_ERROR","群聊执行计划无效"))?
        } else {
            let plan: Vec<_> = settings_from(service.config,service.agents)?.into_iter().filter(|m|m.enabled)
                .map(|m|(m.id,uuid::Uuid::new_v4().to_string())).collect();
            if plan.is_empty() { return Err(ChatError::new("INVALID_CONFIG","请至少启用一个回复角色")); }
            // 计划在首条消息前固定，重试不会重复插入用户消息或改变角色顺序。
            conn.execute("INSERT INTO local_group_batches(request_id,content,plan,status) VALUES(?1,?2,?3,'running')",
                params![request.request_id,request.content.to_string(),serde_json::to_string(&plan).unwrap()])?;
            plan
        }
    };
    let mut user_id = None;
    let mut last = None;
    for (agent_id, child_id) in plan {
        let turn = TurnRequest { request_id:child_id,thread_id:request.thread_id.clone(),agent_id,
            content:if user_id.is_none() { Some(request.content.clone()) } else { None },
            reply_to_message_id:user_id,regenerate_message_id:None,ephemeral:false };
        let outcome = service.execute_reserved(turn,&mut lease.cancel,|mut event| {
            event.request_id = request.request_id.clone(); on_event(event);
        },&state).await;
        match outcome {
            Ok(result) => {
                user_id = result.snapshot.messages.iter().find(|m|m.role=="user" && m.pair_id==result.message.pair_id).map(|m|m.id);
                if user_id.is_none() { return Err(ChatError::new("STORAGE_ERROR","群聊输入记录缺失")); }
                last = Some(result);
            }
            Err(error) => {
                db_lock(service.db)?.execute("UPDATE local_group_batches SET status='failed',error=?2 WHERE request_id=?1",
                    params![request.request_id,serde_json::to_string(&error).unwrap()])?;
                return Err(error);
            }
        }
    }
    db_lock(service.db)?.execute("UPDATE local_group_batches SET status='done' WHERE request_id=?1",[&request.request_id])?;
    let mut result = last.ok_or_else(|| ChatError::new("INVALID_CONFIG","群聊没有可回复的角色"))?;
    result.request_id = request.request_id;
    Ok(result)
}

#[tauri::command]
pub async fn local_group_chat(app: AppHandle, request: GroupRequest) -> ChatResult<TurnResult> {
    let runtime = app.state::<ChatRuntime>();
    let db = app.state::<DbState>();
    let mut config = identity::snapshot(&app)?;
    let agents = identity::agents_from(&config)?;
    let settings = identity::model_settings(&config)?;
    config["currentTime"] = json!(chrono::Local::now().format("%Y-%m-%d %H:%M:%S %:z").to_string());
    let service = ConversationService { db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings };
    let mut result=execute(&service,request,|event| { let _ = app.emit("local-chat-event",event); },|id,_|identity::state(&app,id)).await;
    let sync=super::reminder_platform::synchronize(&app);
    if let Err(error)=&sync {eprintln!("群聊提交后的系统提醒同步等待恢复：{}",error.message);}
    if let Ok(result)=&mut result {result.state["backgroundSyncPending"]=json!(sync.is_err());}
    result
}
