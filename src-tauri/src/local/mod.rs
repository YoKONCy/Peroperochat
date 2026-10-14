pub mod context;
pub mod conversation;
pub mod identity;
pub mod metadata;
pub mod provider;
pub mod repository;
pub mod runtime;
pub mod types;
pub mod backup;
pub mod group;
pub mod blackjack;
pub mod environment;
pub mod settings;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use crate::db::DbState;
use runtime::ChatRuntime;
use types::*;

fn db_lock(state: &DbState) -> ChatResult<std::sync::MutexGuard<'_,rusqlite::Connection>> {
    state.conn.lock().map_err(|_| ChatError::new("STORAGE_ERROR", "本地数据库不可用"))
}

pub fn initialize(app: &AppHandle) -> ChatResult<()> {
    identity::initialize(app)?;
    let agents = identity::agents(app)?;
    let raw_group = identity::config(app,"ppc.group.messages")?;
    let group: Vec<Value> = if raw_group.trim().is_empty() { Vec::new() } else {
        serde_json::from_str(&raw_group).map_err(|_| ChatError::new("INVALID_CONFIG","旧群聊记录损坏，原数据已保留，请先恢复备份"))?
    };
    let db = app.state::<DbState>();
    repository::migrate(&mut *db_lock(&db)?,&group,&agents)?;
    identity::set_config(app,"ppc.agents",&agents.to_string())?;
    group::initialize(&*db_lock(&db)?)?;
    blackjack::initialize(&*db_lock(&db)?)?;
    let conn=db_lock(&db)?;
    conn.execute("INSERT OR IGNORE INTO local_configs(key,value) VALUES('ppc.conversation_count','\"0\"')",[])?;
    Ok(())
}

#[tauri::command]
pub fn local_agents(app: AppHandle) -> ChatResult<Value> { identity::agents(&app) }

#[tauri::command]
pub fn local_delete_agent(app: AppHandle, runtime: tauri::State<ChatRuntime>, id: String) -> ChatResult<Value> {
    let _lease = runtime.maintenance()?;
    let mut agents = identity::agents(&app)?;
    agents.as_object_mut().unwrap().remove(&id);
    let agents = identity::normalize_agents(agents)?;
    identity::set_config(&app,"ppc.agents",&agents.to_string())?;
    // 角色定义删除不隐式擦除历史和记忆，重新导入同一身份时仍可恢复。
    Ok(agents)
}

#[tauri::command]
pub fn local_snapshot(app: AppHandle, db: tauri::State<DbState>, agent_id: String, channel: String) -> ChatResult<Snapshot> {
    if !matches!(channel.as_str(), "desktop" | "group") { return Err(ChatError::new("INVALID_THREAD","会话类型无效")); }
    if channel == "desktop" && identity::agents(&app)?.get(&agent_id).is_none() {
        return Err(ChatError::new("AGENT_NOT_FOUND", "角色不存在"));
    }
    let thread_id = if channel == "group" { repository::FAMILY_THREAD.into() } else { repository::desktop_thread(&agent_id) };
    let conn = db_lock(&db)?;
    repository::ensure_thread(&conn,&thread_id,if channel == "group" { "" } else { &agent_id },&channel)?;
    repository::snapshot(&conn,&thread_id)
}

#[tauri::command]
pub fn local_delete_message(db: tauri::State<DbState>, runtime: tauri::State<ChatRuntime>, thread_id: String, message_id: i64, pair: bool) -> ChatResult<Snapshot> {
    // 先取得同一会话的运行租约，消除“检查空闲后又开始生成”的竞态。
    let _lease = runtime.reserve(&thread_id,&uuid::Uuid::new_v4().to_string())?;
    let mut conn = db_lock(&db)?;
    repository::delete_message(&mut conn,&thread_id,message_id,pair)?;
    repository::snapshot(&conn,&thread_id)
}

#[tauri::command]
pub fn local_cancel_chat(runtime: tauri::State<ChatRuntime>, request_id: String) -> ChatResult<()> { runtime.cancel(&request_id) }

#[tauri::command]
pub async fn local_chat(app: AppHandle, request: TurnRequest) -> ChatResult<TurnResult> {
    let runtime = app.state::<ChatRuntime>();
    let db = app.state::<DbState>();
    let mut config = identity::snapshot(&app)?;
    let agents = identity::agents_from(&config)?;
    let settings = identity::model_settings(&config)?;
    config["currentTime"] = json!(chrono::Local::now().format("%Y-%m-%d %H:%M:%S %:z").to_string());
    let service = conversation::ConversationService { db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings };
    service.execute(request,|event| { let _ = app.emit("local-chat-event",event); },
        |agent_id,_| identity::state(&app,agent_id)).await
}

#[cfg(test)]
mod tests;
