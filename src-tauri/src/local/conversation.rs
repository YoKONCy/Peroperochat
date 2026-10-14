use std::time::{Duration, Instant};
use serde_json::{json, Value};
use crate::db::DbState;
use super::{context, provider, repository, runtime::ChatRuntime, types::*, db_lock};

// 业务服务不依赖窗口、IPC 或 UI 生命周期，Tauri 命令仅注入本地资源。
pub struct ConversationService<'a> {
    pub db: &'a DbState,
    pub runtime: &'a ChatRuntime,
    pub agents: &'a Value,
    pub config: &'a Value,
    pub settings: &'a provider::ModelSettings,
}

impl ConversationService<'_> {
    pub async fn execute(&self, request: TurnRequest,
        on_event: impl Fn(ChatEvent),
        persist_state: impl Fn(&str, &str) -> ChatResult<Value>) -> ChatResult<TurnResult> {
    if uuid::Uuid::parse_str(&request.request_id).is_err() { return Err(ChatError::new("INVALID_INPUT", "请求身份无效")); }
    let mut lease = self.runtime.reserve(&request.thread_id,&request.request_id)?;
    self.execute_reserved(request, &mut lease.cancel, on_event, persist_state).await
    }

    // 整轮群聊持有外层租约，各角色共用取消信号，角色之间也不允许插入其他写操作。
    pub(super) async fn execute_reserved(&self, request: TurnRequest,
        cancel: &mut tokio::sync::watch::Receiver<bool>, on_event: impl Fn(ChatEvent),
        persist_state: impl Fn(&str, &str) -> ChatResult<Value>) -> ChatResult<TurnResult> {
    if *cancel.borrow() { return Err(ChatError::new("CANCELLED","已停止回复")); }
    let agents = self.agents;
    let agent = agents.get(&request.agent_id).ok_or_else(|| ChatError::new("AGENT_NOT_FOUND","角色不存在"))?;
    let name = agent.get("name").and_then(Value::as_str).unwrap_or(&request.agent_id);
    let settings = self.settings;
    provider::endpoint(&settings.base,"chat/completions")?;
    if settings.model.trim().is_empty() || settings.model.contains("请先") { return Err(ChatError::new("INVALID_CONFIG","请先在设置中选择模型")); }
    let db = self.db;
    let (prepared, model, manifest) = {
        let mut conn = db_lock(&db)?;
        if let Some(message) = repository::existing_turn(&conn,&request)? {
            return Ok(TurnResult { request_id:request.request_id.clone(),message,snapshot:repository::snapshot(&conn,&request.thread_id)?,state:json!({}) });
        }
        if request.thread_id == repository::FAMILY_THREAD {
            repository::ensure_thread(&conn,&request.thread_id,"","group")?;
        } else if request.thread_id == repository::desktop_thread(&request.agent_id) {
            repository::ensure_thread(&conn,&request.thread_id,&request.agent_id,"desktop")?;
        } else { return Err(ChatError::new("INVALID_THREAD","角色与会话不匹配")); }
        let thread = repository::thread(&conn,&request.thread_id)?;
        let prepared = repository::prepare_turn(&mut conn,&request,name)?;
        let compiled = context::compile(&conn,context::ContextInput { thread:&thread,agent_id:&request.agent_id,
            identity:agents,config:self.config,current:&prepared.current_input,before_id:prepared.before_id,rounds:settings.memory_rounds });
        match compiled {
            Ok((model,manifest)) => (prepared,model,manifest),
            Err(error) => {
                repository::finish_turn(&mut conn,&request,&prepared,"",Some(&error),&json!({}))?;
                return Err(error);
            }
        }
    };
    let emit = |kind: &str, message: Option<Message>, content: Option<String>| {
        on_event(ChatEvent { request_id:request.request_id.clone(),thread_id:request.thread_id.clone(),
            agent_id:request.agent_id.clone(),kind:kind.into(),message,
            user:if kind == "started" { prepared.user.clone() } else { None },content });
    };
    emit("started",Some(prepared.assistant.clone()),None);
    let mut content = String::new();
    let mut partial = String::new();
    let mut saved_at = Instant::now();
    let outcome = tokio::select! {
        biased;
        _ = cancel.changed() => Err(ChatError::new("CANCELLED","已停止回复")),
        result = tokio::time::timeout(Duration::from_secs(300),provider::complete(&settings,&model,&mut content,|delta| {
            partial.push_str(delta);
            if !prepared.regenerate && saved_at.elapsed() >= Duration::from_millis(250) {
                repository::update_partial(&*db_lock(&db)?,prepared.assistant.id,&partial)?;
                saved_at = Instant::now();
            }
            emit("delta",None,Some(delta.into()));
            Ok(())
        })) => result.unwrap_or_else(|_| Err(ChatError::new("TIMEOUT","本轮回复超时，请重试"))),
    };
    let outcome = if *cancel.borrow() { Err(ChatError::new("CANCELLED","已停止回复")) } else { outcome };
    let mut outcome = outcome.and_then(|_| if context::visible_text(&content).is_empty() {
        Err(ChatError::new("EMPTY_RESPONSE","模型没有返回可显示的回复，请重新生成"))
    } else { Ok(()) });
    if let Err(error) = &mut outcome {
        if !settings.api_key.is_empty() { error.message = error.message.replace(&settings.api_key,"[已隐藏]"); }
    }
    let message = {
        let mut conn = db_lock(&db)?;
        repository::finish_turn(&mut conn,&request,&prepared,&content,outcome.as_ref().err(),&manifest)?
    };
    if let Err(error) = outcome {
        emit(if error.code == "CANCELLED" { "cancelled" } else { "error" },Some(message),None);
        return Err(error);
    }
    let state = persist_state(&request.agent_id,&content).unwrap_or_else(|error| {
        eprintln!("回复已保存，但角色状态更新失败：{}",error.message);
        json!({"agentId":request.agent_id})
    });
    emit("done",Some(message.clone()),None);
    let snapshot = repository::snapshot(&*db_lock(&db)?,&request.thread_id)?;
    Ok(TurnResult { request_id:request.request_id.clone(),message,snapshot,state })
    }
}
