use serde_json::{json,Value};
use tauri::{AppHandle,Manager};
use crate::db::DbState;
use super::{context,db_lock,diary,identity,prompts,repository,runtime::ChatRuntime,types::*};

#[tauri::command]
pub fn local_model_configuration(app:AppHandle)->ChatResult<Value> {
    let settings=identity::model_settings(&identity::snapshot(&app)?)?;
    let mut value=serde_json::to_value(&settings).map_err(|_|ChatError::new("INVALID_CONFIG","模型配置无法读取"))?;
    value.as_object_mut().unwrap().remove("apiKey");
    value["hasApiKey"]=json!(!settings.api_key.is_empty());
    for value in value["headers"].as_object_mut().unwrap().values_mut() { *value=json!("[已隐藏]"); }
    Ok(value)
}

#[tauri::command]
pub fn local_diary_query(db:tauri::State<DbState>,agent_id:String,query:diary::Query)->ChatResult<Vec<diary::Entry>> {
    let conn=db_lock(&db)?;db.diary.query(&conn,&agent_id,&query)
}

#[tauri::command]
pub fn local_manage_diary(db:tauri::State<DbState>,runtime:tauri::State<ChatRuntime>,agent_id:String,action:String,draft:Option<diary::Draft>,id:Option<i64>)->ChatResult<()> {
    let _lease=runtime.maintenance()?;let mut conn=db_lock(&db)?;
    if identity::agents_from(&identity::snapshot_in(&conn)?)?.get(&agent_id).is_none() { return Err(ChatError::new("AGENT_NOT_FOUND","角色不存在")); }
    let tx=conn.transaction()?;
    match action.as_str() {
        "write"=> { diary::apply(&tx,&agent_id,&draft.ok_or_else(||ChatError::new("INVALID_INPUT","日记内容缺失"))?,&[],chrono::Utc::now().timestamp_millis())?; }
        "delete"=>diary::delete(&tx,&agent_id,id.ok_or_else(||ChatError::new("INVALID_INPUT","日记身份缺失"))?)?,
        _=>return Err(ChatError::new("INVALID_INPUT","日记操作无效")),
    }
    tx.commit()?;db.diary.synchronize(&conn)
}

#[tauri::command]
pub fn local_diary_day(db:tauri::State<DbState>,agent_id:String,date:String)->ChatResult<String> {
    let conn=db_lock(&db)?;
    let entries=db.diary.query(&conn,&agent_id,&diary::Query { from:Some(date.clone()),to:Some(date.clone()),limit:10_000,..Default::default() })?;
    let mut markdown=format!("# {date}\n\n");
    for entry in entries.into_iter().rev() { markdown.push_str(&format!("{}\n\n",entry.content)); }
    if markdown.len()>2*1024*1024 { return Err(ChatError::new("OUTPUT_LIMIT","当日日记过大，请按条目读取")); }
    Ok(markdown)
}

#[tauri::command]
pub fn local_prompt_preview(app:AppHandle,agent_id:String,channel:String,content:Value,draft:bool)->ChatResult<Value> {
    validate_content(&content)?;
    if !["desktop","group"].contains(&channel.as_str()) { return Err(ChatError::new("INVALID_THREAD","预览通道无效")); }
    let db=app.state::<DbState>();let conn=db_lock(&db)?;
    let mut config=identity::snapshot_in(&conn)?;
    config["currentTime"]=json!(chrono::Local::now().format("%Y-%m-%d %H:%M:%S %:z").to_string());
    let agents=identity::agents_from(&config)?;let settings=identity::model_settings(&config)?;let workspace=prompts::load(&config)?;
    let thread_id=if channel=="group" {repository::FAMILY_THREAD.into()} else {repository::desktop_thread(&agent_id)};
    let thread=match repository::thread(&conn,&thread_id) { Ok(thread)=>thread,Err(error) if error.code=="NOT_FOUND"=>Thread {id:thread_id,agent_id:agent_id.clone(),channel},Err(error)=>return Err(error) };
    let diary=db.diary.query(&conn,&agent_id,&diary::Query {limit:8,..Default::default()})?;
    let tool_tokens=if settings.tools_enabled {context::estimated_tokens(&json!(super::chat_tools::definitions()).to_string())} else {0};
    let (messages,manifest)=context::compile_bundle(&conn,context::ContextInput {thread:&thread,agent_id:&agent_id,identity:&agents,config:&config,
        current:&content,before_id:Some(i64::MAX),rounds:settings.memory_rounds,diary:&diary,budget:settings.context_window.saturating_sub(settings.max_tokens+tool_tokens+512)},
        if draft {&workspace.draft} else {&workspace.active.bundle},if draft {"draft"} else {&workspace.active.id})?;
    Ok(json!({"messages":messages,"manifest":manifest,"tools":if settings.tools_enabled {super::chat_tools::definitions()} else {Vec::new()}}))
}
