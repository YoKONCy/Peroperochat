pub mod context;
pub mod conversation;
pub mod identity;
pub mod appearance;
pub mod provider;
pub mod repository;
pub mod runtime;
pub mod types;
pub mod backup;
pub mod group;
pub mod blackjack;
pub mod environment;
pub mod settings;
pub mod diary;
pub mod chat_tools;
pub mod tool_timeline;
pub mod prompts;
pub mod model_config;
pub mod wire;
pub mod provider_stream;
pub mod react;
pub mod management;
pub mod reminders;
pub mod reminder_dispatch;
pub mod reminder_platform;
#[cfg(any(target_os="android",test))]
mod reminder_jni;

#[cfg(test)]
mod standalone_tests;
#[cfg(test)]
mod reminder_tests;
#[cfg(test)]
mod reminder_dispatch_tests;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use crate::db::DbState;
use runtime::ChatRuntime;
use types::*;

pub(crate) struct LocalConnection<'a> {
    connection:std::sync::MutexGuard<'a,rusqlite::Connection>,
    _gate:Option<crate::mode::Gate>,
}
impl std::ops::Deref for LocalConnection<'_> {type Target=rusqlite::Connection;fn deref(&self)->&Self::Target {&self.connection}}
impl std::ops::DerefMut for LocalConnection<'_> {fn deref_mut(&mut self)->&mut Self::Target {&mut self.connection}}
pub(crate) fn db_lock(state: &DbState) -> ChatResult<LocalConnection<'_>> {
    // 统一锁顺序：控制库 → 独立数据库；控制锁仅覆盖同步存储或通知提交。
    let gate=state.mode.get().map(|mode| {
        if crate::mode::LOCAL_INITIALIZING.try_with(|allowed|*allowed).unwrap_or(false) {mode.initialization_gate()}
        else {mode.local_gate()}
    }).transpose()?;
    let connection=state.conn.lock().map_err(|_| ChatError::new("STORAGE_ERROR", "本地数据库不可用"))?;
    Ok(LocalConnection {connection,_gate:gate})
}

pub fn initialize(app: &AppHandle) -> ChatResult<()> {
    identity::initialize(app)?;
    {
        let db=app.state::<DbState>();
        identity::update_builtin_personas(&mut *db_lock(&db)?)?;
    }
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
pub fn local_agents(app: AppHandle, mode_token:crate::mode::Token) -> ChatResult<Value> {
    crate::mode::LOCAL_TOKEN.sync_scope(mode_token, || { identity::agents(&app)
    })
}

#[tauri::command]
pub async fn local_delete_agent(app: AppHandle, runtime: tauri::State<'_,ChatRuntime>, id: String, mode_token:crate::mode::Token) -> ChatResult<Value> {
    crate::mode::LOCAL_TOKEN.scope(mode_token, async {
    let _lease = runtime.maintenance()?;
    let mut agents = identity::agents(&app)?;
    agents.as_object_mut().unwrap().remove(&id);
    let agents = identity::normalize_agents(agents)?;
    identity::set_config(&app,"ppc.agents",&agents.to_string())?;
    if let Err(error)=reminder_platform::synchronize(&app) {eprintln!("角色删除后的系统提醒取消等待恢复：{}",error.message);}
    // 角色定义删除不隐式擦除历史和记忆，重新导入同一身份时仍可恢复。
    Ok(agents)

    }).await
}

#[tauri::command]
pub fn local_snapshot(app: AppHandle, db: tauri::State<DbState>, agent_id: String, channel: String, mode_token:crate::mode::Token) -> ChatResult<Snapshot> {
    crate::mode::LOCAL_TOKEN.sync_scope(mode_token, || {
    if !matches!(channel.as_str(), "desktop" | "group") { return Err(ChatError::new("INVALID_THREAD","会话类型无效")); }
    if channel == "desktop" && identity::agents(&app)?.get(&agent_id).is_none() {
        return Err(ChatError::new("AGENT_NOT_FOUND", "角色不存在"));
    }
    let thread_id = if channel == "group" { repository::FAMILY_THREAD.into() } else { repository::desktop_thread(&agent_id) };
    let conn = db_lock(&db)?;
    repository::ensure_thread(&conn,&thread_id,if channel == "group" { "" } else { &agent_id },&channel)?;
    repository::snapshot(&conn,&thread_id)

    })
}

#[tauri::command]
pub fn local_delete_message(db: tauri::State<DbState>, runtime: tauri::State<ChatRuntime>, thread_id: String, message_id: i64, pair: bool, mode_token:crate::mode::Token) -> ChatResult<Snapshot> {
    crate::mode::LOCAL_TOKEN.sync_scope(mode_token, || {
    // 先取得同一会话的运行租约，消除“检查空闲后又开始生成”的竞态。
    let _lease = runtime.reserve(&thread_id,&uuid::Uuid::new_v4().to_string())?;
    let mut conn = db_lock(&db)?;
    repository::delete_message(&mut conn,&thread_id,message_id,pair)?;
    if let Err(error)=db.diary.synchronize(&conn) { eprintln!("消息已删除，日记索引等待恢复：{}",error.message); }
    repository::snapshot(&conn,&thread_id)

    })
}

#[tauri::command]
pub fn local_cancel_chat(runtime: tauri::State<ChatRuntime>, request_id: String, mode_token:crate::mode::Token) -> ChatResult<()> {
    crate::mode::LOCAL_TOKEN.sync_scope(mode_token, || { runtime.cancel(&request_id)
    })
}

#[tauri::command]
pub async fn local_chat(app: AppHandle, request: TurnRequest, mode_token:crate::mode::Token) -> ChatResult<TurnResult> {
    crate::mode::LOCAL_TOKEN.scope(mode_token, async {
    let mode=app.state::<crate::mode::Coordinator>();let mut changes=mode.subscribe();
    let token=mode.token()?;
    let runtime = app.state::<ChatRuntime>();
    let db = app.state::<DbState>();
    let mut config = identity::snapshot(&app)?;
    let agents = identity::agents_from(&config)?;
    let settings = identity::model_settings(&config)?;
    config["currentTime"] = json!(chrono::Local::now().format("%Y-%m-%d %H:%M:%S %:z").to_string());
    let service = conversation::ConversationService { db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings };
    let execute=service.execute(request,|event| {
        if let Ok(_gate)=mode.gate(Some(&token),true) {let _=app.emit("local-chat-event",event);}
    },|agent_id,_| identity::state(&app,agent_id));
    let mut result=tokio::select! {biased;
        _=changes.changed()=>return Err(ChatError::new("MODE_INACTIVE","运行模式已切换")),
        result=execute=>result?,
    };
    if result.state["reminderChanges"].as_u64().unwrap_or(0)>0 {
        result.state["backgroundSyncPending"]=json!(reminder_platform::synchronize(&app).is_err());
    }
    Ok(result)

    }).await
}

#[cfg(test)]
mod tests;
