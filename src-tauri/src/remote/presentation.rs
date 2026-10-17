//! 手机端展示偏好与草稿，只保存在当前远程连接分区，不参与服务端业务状态推导。
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use tauri::{AppHandle, Manager};
use crate::{mode::{Coordinator, Token}, local::types::*};

#[derive(Deserialize, Serialize)]
#[serde(tag="kind", rename_all="snake_case", deny_unknown_fields)]
pub enum Scope {
    Home,
    Session { #[serde(rename="agentId")] agent_id:String, #[serde(rename="threadId")] thread_id:String },
    Character { #[serde(rename="agentId")] agent_id:String },
}
impl Scope {
    fn key(&self)->ChatResult<String> {
        match self {
            Self::Home=>{}, Self::Session {agent_id,thread_id}=>{super::operations::segment(agent_id)?;super::operations::segment(thread_id)?;},
            Self::Character {agent_id}=>{super::operations::segment(agent_id)?;},
        }
        Ok(format!("ui:{}",serde_json::to_string(self).map_err(|_|super::inactive())?))
    }
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct DraftAttachment {id:String,name:String}
#[derive(Deserialize, Serialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct SessionView {
    draft:String, attachments:Vec<DraftAttachment>, anchor:Option<String>, offset:f64,
    at_end:bool, cursor:Option<String>,
    #[serde(default)] edit_id:String,
    #[serde(default)] send_request_id:Option<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct CharacterView { expanded:bool, model:Option<u8>, palette:String }
#[derive(Deserialize, Serialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct HomeView {agent_id:String,thread_id:String,title:String}

fn validate(scope:&Scope,value:Value)->ChatResult<Value> {
    if value.is_null() {return Ok(Value::Null);}
    let invalid=||ChatError::new("INVALID_INPUT","远程阅读偏好或草稿格式无效");
    match scope {
        Scope::Home=> {
            let value:HomeView=serde_json::from_value(value).map_err(|_|invalid())?;
            super::operations::segment(&value.agent_id)?;super::operations::segment(&value.thread_id)?;
            if value.title.len()>1024 {return Err(invalid());}Ok(json!(value))
        },
        Scope::Session {..}=> {
            let value:SessionView=serde_json::from_value(value).map_err(|_|invalid())?;
            if value.draft.len()>128*1024 || value.attachments.len()>5 || !value.offset.is_finite() || value.offset.abs()>100000.0
                || value.anchor.as_ref().is_some_and(|id|id.len()>1024) || value.cursor.as_ref().is_some_and(|id|id.len()>256)
                || value.edit_id.len()>64 || value.send_request_id.as_ref().is_some_and(|id|uuid::Uuid::parse_str(id).is_err())
                || value.attachments.iter().any(|file|uuid::Uuid::parse_str(&file.id).is_err() || file.name.len()>240) {return Err(invalid());}
            Ok(json!(value))
        },
        Scope::Character {..}=> {
            let value:CharacterView=serde_json::from_value(value).map_err(|_|invalid())?;
            if !["sky","violet","rose"].contains(&value.palette.as_str()) || value.model.is_some_and(|model|![0,1,3].contains(&model)) {return Err(invalid());}
            Ok(json!(value))
        },
    }
}

fn settle_sent_draft(tx:&rusqlite::Transaction<'_>,scope:&Scope,value:&mut Value)->ChatResult<bool> {
    if !matches!(scope,Scope::Session {..}) {return Ok(false);}
    let Some(id)=value["sendRequestId"].as_str() else {return Ok(false);};
    let state:Option<String>=tx.query_row("SELECT state FROM requests WHERE id=?1",[id],|row|row.get(0)).optional()?;
    if state.as_deref()!=Some("completed") {return Ok(false);}
    // 读写都核对持久回执，防止延迟保存把已发送草稿重新写回；新编辑已解除请求关联。
    value["draft"]=json!("");value["attachments"]=json!([]);value["sendRequestId"]=Value::Null;value["editId"]=json!(uuid::Uuid::new_v4().to_string());
    Ok(true)
}

#[tauri::command]
pub fn remote_ui_read(app:AppHandle,token:Token,scope:Scope)->ChatResult<Value> {
    super::validate_token(&app,&token)?;let key=scope.key()?;
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(&token),false)?;
    let mut conn=super::cache::connection(&app,&token)?;
    let tx=conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let raw:Option<String>=tx.query_row("SELECT value FROM preferences WHERE key=?1",[&key],|row|row.get(0)).optional()?;
    let mut value=validate(&scope,raw.map(|raw|serde_json::from_str(&raw).map_err(|_|ChatError::new("STORAGE_ERROR","远程阅读偏好损坏，请重置该项偏好"))).transpose()?.unwrap_or(Value::Null))?;
    if settle_sent_draft(&tx,&scope,&mut value)? {
        tx.execute("UPDATE preferences SET value=?2 WHERE key=?1",params![key,value.to_string()])?;
    }
    tx.commit()?;Ok(json!({"modeToken":token,"data":value}))
}

#[tauri::command]
pub fn remote_ui_write(app:AppHandle,token:Token,scope:Scope,value:Value)->ChatResult<Value> {
    super::validate_token(&app,&token)?;let key=scope.key()?;let mut value=validate(&scope,value)?;
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(&token),false)?;
    let mut conn=super::cache::connection(&app,&token)?;
    let tx=conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if value.is_null() {
        tx.execute("DELETE FROM preferences WHERE key=?1",[key])?;tx.commit()?;
        return Ok(json!({"modeToken":token,"data":Value::Null}));
    }
    settle_sent_draft(&tx,&scope,&mut value)?;
    let count:i64=tx.query_row("SELECT COUNT(*) FROM preferences WHERE key LIKE 'ui:%' AND key<>?1",[&key],|row|row.get(0))?;
    let bytes:i64=tx.query_row("SELECT COALESCE(SUM(length(CAST(value AS BLOB))),0) FROM preferences WHERE key LIKE 'ui:%' AND key<>?1",[&key],|row|row.get(0))?;
    let raw=value.to_string();
    if count>=256 || bytes+raw.len() as i64>8*1024*1024 {return Err(ChatError::new("REMOTE_DRAFT_LIMIT","本连接保存的草稿与阅读记录过多，请先清理不用的草稿"));}
    tx.execute("INSERT INTO preferences VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![key,raw])?;
    tx.commit()?;Ok(json!({"modeToken":token,"data":value}))
}
