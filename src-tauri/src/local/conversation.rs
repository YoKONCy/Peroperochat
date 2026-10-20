use std::time::{Duration,Instant};
use serde_json::{json,Value};
use crate::db::DbState;
use super::{chat_tools::ToolSession,context,diary,model_config,react,repository,runtime::ChatRuntime,types::*,db_lock};

pub struct ConversationService<'a> {
    pub db:&'a DbState,pub runtime:&'a ChatRuntime,pub agents:&'a Value,pub config:&'a Value,pub settings:&'a super::provider::ModelSettings,
}
impl ConversationService<'_> {
    pub async fn execute(&self,request:TurnRequest,on_event:impl Fn(ChatEvent),state:impl Fn(&str,&str)->ChatResult<Value>)->ChatResult<TurnResult> {
        let mut lease=self.runtime.reserve(&request.thread_id,&request.request_id)?;
        self.execute_reserved(request,&mut lease.cancel,on_event,state).await
    }

    pub(super) async fn execute_reserved(&self,request:TurnRequest,cancel:&mut tokio::sync::watch::Receiver<bool>,
        on_event:impl Fn(ChatEvent),state:impl Fn(&str,&str)->ChatResult<Value>)->ChatResult<TurnResult> {
        if uuid::Uuid::parse_str(&request.request_id).is_err() { return Err(ChatError::new("INVALID_INPUT","请求身份无效")); }
        if *cancel.borrow() { return Err(ChatError::new("CANCELLED","已停止回复")); }
        let agent=self.agents.get(&request.agent_id).ok_or_else(||ChatError::new("AGENT_NOT_FOUND","角色不存在"))?;
        let cached={ let conn=db_lock(self.db)?;
            repository::existing_turn(&conn,&request)?.map(|message|repository::snapshot(&conn,&request.thread_id).map(|snapshot|(message,snapshot))).transpose()? };
        if let Some((message,snapshot))=cached { return Ok(TurnResult { request_id:request.request_id,message,snapshot,state:state(&request.agent_id,"")? }); }
        model_config::validate(self.settings)?;
        if self.settings.model.trim().is_empty() || self.settings.model.contains("请先") { return Err(ChatError::new("INVALID_CONFIG","请先在设置中选择模型")); }
        let (prepared,model,mut manifest)={
            let mut conn=db_lock(self.db)?;
            if request.thread_id==repository::FAMILY_THREAD { repository::ensure_thread(&conn,&request.thread_id,"","group")?; }
            else if request.thread_id==repository::desktop_thread(&request.agent_id) { repository::ensure_thread(&conn,&request.thread_id,&request.agent_id,"desktop")?; }
            else { return Err(ChatError::new("INVALID_THREAD","角色与会话不匹配")); }
            let (diary,diary_unavailable)=match self.db.diary.query(&conn,&request.agent_id,&diary::Query { limit:8,..Default::default() }) {
                Ok(entries)=>(entries.into_iter().filter(|entry|!request.regenerate_message_id.is_some_and(|id|entry.source_message_ids.contains(&id))).collect::<Vec<_>>(),false),
                Err(error)=> {eprintln!("本轮暂不加载日记：{}",error.message);(Vec::new(),true)}
            };
            let thread=repository::thread(&conn,&request.thread_id)?;
            let prepared=repository::prepare_turn(&mut conn,&request,agent["name"].as_str().unwrap_or(&request.agent_id))?;
            let tool_tokens=if self.settings.tools_enabled { context::estimated_tokens(&json!(super::chat_tools::definitions()).to_string()) } else { 0 };
            let budget=self.settings.context_window.saturating_sub(self.settings.max_tokens+tool_tokens+512);
            let compiled=context::compile(&conn,context::ContextInput { thread:&thread,agent_id:&request.agent_id,identity:self.agents,config:self.config,
                current:&prepared.current_input,before_id:prepared.before_id,rounds:self.settings.memory_rounds,budget,diary:&diary });
            match compiled { Ok((model,mut manifest))=> {manifest["diaryUnavailable"]=json!(diary_unavailable);(prepared,model,manifest)},Err(error)=> { repository::finish_turn(&mut conn,&request,&prepared,"",Some(&error),&json!({}))?;return Err(error); } }
        };
        let emit=|kind:&str,message:Option<Message>,content:Option<String>|on_event(ChatEvent { request_id:request.request_id.clone(),thread_id:request.thread_id.clone(),agent_id:request.agent_id.clone(),
            kind:kind.into(),message,user:if kind=="started" {prepared.user.clone()} else {None},content });
        emit("started",Some(prepared.assistant.clone()),None);
        let mut output=String::new();let mut preview=String::new();let mut saved=Instant::now();let mut tools=ToolSession::default();let mut live_tools:Vec<ToolActivity>=Vec::new();
        if prepared.regenerate {tools.excluded_sources.push(prepared.assistant.id);}
        let outcome=tokio::select! {
            biased;
            _=cancel.changed()=>Err(ChatError::new("CANCELLED","已停止回复")),
            result=tokio::time::timeout(Duration::from_secs(self.settings.turn_timeout),react::run(self.db,self.settings,&request.agent_id,model,&mut tools,&mut output,|kind,delta| {
                if kind=="step" { preview.clear();if !prepared.regenerate { repository::update_partial(&*db_lock(self.db)?,prepared.assistant.id,"")?; } }
                if kind=="delta" {
                    preview.push_str(delta);
                    if !prepared.regenerate && saved.elapsed()>=Duration::from_millis(250) {
                        repository::update_partial(&*db_lock(self.db)?,prepared.assistant.id,&preview)?;saved=Instant::now();
                    }
                }
                if kind=="tool" {
                    if let Ok(item)=serde_json::from_str::<ToolActivity>(delta) {
                        if super::tool_timeline::valid(std::slice::from_ref(&item)) {
                            if let Some(previous)=live_tools.iter_mut().find(|previous|previous.id==item.id) {*previous=item;}
                            else if live_tools.len()<24 {live_tools.push(item);}
                            if !prepared.regenerate {repository::update_tool_timeline(&*db_lock(self.db)?,prepared.assistant.id,&live_tools)?;}
                        }
                    }
                }
                emit(kind,None,if delta.is_empty() {None} else {Some(delta.into())});Ok(())
            }))=>result.unwrap_or_else(|_|Err(ChatError::new("TIMEOUT","本轮回复超时，请重试"))),
        };
        let outcome=if *cancel.borrow() { Err(ChatError::new("CANCELLED","已停止回复")) } else { outcome };
        let (completion,mut error)=match outcome { Ok(result)=>(result,None),Err(mut error)=> {
            error.message=model_config::redact(self.settings,&error.message);
            (Completion { content:context::visible_text(&output),..Default::default() },Some(error))
        } };
        manifest["toolCalls"]=json!(tools.traces.len());
        let message={
            let mut conn=db_lock(self.db)?;
            match repository::finish_with_tools(&mut conn,&request,&prepared,&completion,error.as_ref(),&manifest,&tools) {
                Ok(message)=>message,Err(mut commit_error)=> {
                    commit_error.message=model_config::redact(self.settings,&commit_error.message);error=Some(commit_error);
                    repository::finish_with_tools(&mut conn,&request,&prepared,&completion,error.as_ref(),&manifest,&tools)?
                }
            }
        };
        if let Some(error)=error { emit(if error.code=="CANCELLED" {"cancelled"} else {"error"},Some(message),None);return Err(error); }
        let mut projection=state(&request.agent_id,&completion.content).unwrap_or_else(|_|json!({"agentId":request.agent_id}));
        projection["diaryWrites"]=json!(tools.diary_write_count());
        projection["reminderChanges"]=json!(tools.reminder_change_count());
        #[cfg(target_os="android")]
        if tools.reminder_change_count()>0 {
            // 对话事务已成功提交，由上层拿到 AppHandle 后登记系统调度。
            projection["backgroundSyncPending"]=json!(true);
        }
        if let Err(error)=self.db.diary.synchronize(&*db_lock(self.db)?) {
            eprintln!("回复已提交，日记索引等待恢复：{}",error.message);projection["diarySyncPending"]=json!(true);
        }
        emit("done",Some(message.clone()),None);
        let snapshot=repository::snapshot(&*db_lock(self.db)?,&request.thread_id)?;
        Ok(TurnResult { request_id:request.request_id,message,snapshot,state:projection })
    }
}
