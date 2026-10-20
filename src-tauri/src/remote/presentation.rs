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
    Appearance,
    Session { #[serde(rename="agentId")] agent_id:String, #[serde(rename="threadId")] thread_id:String },
    Character { #[serde(rename="agentId")] agent_id:String },
}
impl Scope {
    fn key(&self)->ChatResult<String> {
        match self {
            Self::Home|Self::Appearance=>{}, Self::Session {agent_id,thread_id}=>{super::operations::segment(agent_id)?;super::operations::segment(thread_id)?;},
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
pub struct CharacterView {
    expanded:bool, model:Option<u8>, palette:String,
    #[serde(default)] cg:Option<String>,
    #[serde(default)] style:Option<CharacterStyle>,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct CharacterStyle {
    name:String,
    #[serde(default)] light_accent:Option<String>,
    #[serde(default)] dark_accent:Option<String>,
    #[serde(default)] secondary:Option<String>,
    bubble:String, decoration:String,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct AppearanceView {theme:String,palette:String,show_cg:bool,reduce_motion:bool,role_style:bool}
fn palette_valid(value:&str)->bool { ["standard","sky","night","sakura","mint","amber","crimson","silver","peach","violet","rose"].contains(&value) }
fn color_valid(value:&str)->bool {value.len()==7 && value.starts_with('#') && value.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)}
fn cg_valid(value:&str)->bool {
    if value.len()>2_800_000 {return false;}
    let Some((kind,data))=value.split_once(";base64,") else {return false;};
    let signature=match kind {"data:image/png"=>data.starts_with("iVBORw0KGgo"),"data:image/jpeg"=>data.starts_with("/9j/"),"data:image/webp"=>data.starts_with("UklGR"),_=>false};
    signature && data.len()%4==0 && data.bytes().all(|byte|byte.is_ascii_alphanumeric() || matches!(byte,b'+'|b'/'|b'='))
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct HomeView {agent_id:String,thread_id:String,title:String}

fn validate(scope:&Scope,value:Value)->ChatResult<Value> {
    if value.is_null() {return Ok(Value::Null);}
    let invalid=||ChatError::new("INVALID_INPUT","远程阅读偏好或草稿格式无效");
    match scope {
        Scope::Appearance=> {
            let value:AppearanceView=serde_json::from_value(value).map_err(|_|invalid())?;
            if !["light","dark","system"].contains(&value.theme.as_str()) || !palette_valid(&value.palette) {return Err(invalid());}
            Ok(json!(value))
        },
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
            if !palette_valid(&value.palette) || value.model.is_some_and(|model|![0,1,3].contains(&model)) || value.cg.as_ref().is_some_and(|cg|!cg_valid(cg)) {return Err(invalid());}
            if let Some(style)=&value.style {
                if style.name.is_empty() || style.name.len()>240 || !["standard","ribbon","soft"].contains(&style.bubble.as_str()) || !["standard","ribbon","constellation","none"].contains(&style.decoration.as_str())
                    || [&style.light_accent,&style.dark_accent,&style.secondary].iter().any(|color|color.as_ref().is_some_and(|color|!color_valid(color))) {return Err(invalid());}
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_character_preferences_remain_readable() {
        let scope=Scope::Character {agent_id:"pero".into()};
        let value=validate(&scope,json!({"expanded":true,"model":null,"palette":"violet"})).unwrap();
        assert_eq!(value["palette"],"violet");
        assert!(value["cg"].is_null() && value["style"].is_null());
    }
    #[test]
    fn appearance_requires_known_theme_and_palette() {
        let value=json!({"theme":"system","palette":"mint","showCg":true,"reduceMotion":false,"roleStyle":false});
        assert!(validate(&Scope::Appearance,value.clone()).is_ok());
        let mut invalid=value;invalid["theme"]=json!("javascript:alert(1)");
        assert!(validate(&Scope::Appearance,invalid).is_err());
    }
    #[test]
    fn character_style_rejects_executable_css_and_urls() {
        let scope=Scope::Character {agent_id:"nana".into()};
        let mut value=json!({"expanded":false,"model":null,"palette":"sky","style":{"name":"角色样式","lightAccent":"#467fc5","darkAccent":"#c2a7ef","secondary":"#d79bb8","bubble":"ribbon","decoration":"constellation"}});
        assert!(validate(&scope,value.clone()).is_ok());
        value["style"]["lightAccent"]=json!("url(https://example.com)");
        assert!(validate(&scope,value.clone()).is_err());
        value["style"]["lightAccent"]=json!("#467fc5");value["style"]["css"]=json!("body{display:none}");
        assert!(validate(&scope,value).is_err());
    }
    #[test]
    fn character_cg_rejects_active_and_remote_content() {
        assert!(!cg_valid("https://example.com/image.png"));
        assert!(!cg_valid("data:image/svg+xml;base64,PHN2Zz4="));
        assert!(!cg_valid("data:text/html;base64,PHNjcmlwdD4="));
        assert!(!cg_valid(&format!("data:image/png;base64,{}","A".repeat(2_800_001))));
    }
}
