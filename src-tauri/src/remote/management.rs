//! 远程管理的有限接口目录。配置与密钥仅经过内存，不进入投影缓存或操作回执。
use serde::Deserialize;
use serde_json::{json,Value,Map};
use sha2::{Digest,Sha256};
use tauri::{AppHandle,Manager};
use crate::{mode::{Coordinator,Token},local::types::*};
use super::{operations::{Route,segment},cache,transport};

#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Request {
    pub operation:String,
    pub id:Option<String>,
    #[serde(default)] pub params:Map<String,Value>,
    pub body:Option<Value>,
}

// 手动修订与 infOS 保持相同字段边界；正文不进入本机回执，只保存请求指纹。
fn validate_archive_edit(input:&Request)->ChatResult<()> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Affect { tones:Vec<String>, valence:u8, arousal:u8 }
    #[derive(Deserialize)]
    #[serde(rename_all="camelCase",deny_unknown_fields)]
    struct MemoryEdit { narrative:String, importance:u8, affect:Affect, participants:Vec<String>, places:Vec<String>, objects:Vec<String>, topics:Vec<String>, expected_status:String, #[serde(default)] regenerate_vector:bool, operation_id:String }
    #[derive(Deserialize)]
    #[serde(rename_all="camelCase",deny_unknown_fields)]
    struct FactEdit { statement:String, source:Option<String>, confidence:Option<f64>, operation_id:String }
    let invalid=||ChatError::new("INVALID_INPUT","请检查正文、数值与标签后再保存");
    let text=|value:&str,max:usize,required:bool|!value.contains('\0') && value.trim().chars().count()<=max && (!required || !value.trim().is_empty());
    let tags=|values:&Vec<String>|values.len()<=100 && values.iter().all(|value|text(value,200,true));
    match input.operation.as_str() {
        "memories.edit"=>{
            let edit:MemoryEdit=serde_json::from_value(input.body.clone().ok_or_else(invalid)?).map_err(|_|invalid())?;
            let _=edit.regenerate_vector;
            if !text(&edit.narrative,20_000,true) || edit.importance>10 || edit.affect.valence>10 || edit.affect.arousal>10
                || ![&edit.affect.tones,&edit.participants,&edit.places,&edit.objects,&edit.topics].into_iter().all(tags)
                || !["active","archived"].contains(&edit.expected_status.as_str()) || uuid::Uuid::parse_str(&edit.operation_id).is_err() {return Err(invalid());}
        },
        "facts.edit"=>{
            let edit:FactEdit=serde_json::from_value(input.body.clone().ok_or_else(invalid)?).map_err(|_|invalid())?;
            if !text(&edit.statement,20_000,true) || edit.source.as_ref().is_some_and(|value|!text(value,2000,false))
                || edit.confidence.is_some_and(|value|!value.is_finite() || !(0.0..=1.0).contains(&value)) || uuid::Uuid::parse_str(&edit.operation_id).is_err() {return Err(invalid());}
        },
        _=>{},
    }
    Ok(())
}

fn endpoint(input:&Request)->ChatResult<(Route,bool)> {
    validate_archive_edit(input)?;
    validate_answer_preferences(input)?;
    let id=match &input.id {Some(id)=>segment(id)?,None=>String::new()};
    // 路径与方法只能从此目录产生，不接受任意 URL、HTTP 方法或子应用入口。
    let (method,path,keys,write):(&str,String,&[&str],bool)=match input.operation.as_str() {
        "voice.tts_config"=>("GET","/voice/tts/config".into(),&[],false),
        "voice.save_tts_config"=>("PUT","/voice/tts/config".into(),&[],true),
        "voice.tts_adapters"=>("GET","/voice/tts/adapters".into(),&[],false),
        "owner_avatar.get"|"owner_avatar.reset"=> {
            if input.id.is_some() || input.body.is_some() {return Err(ChatError::new("INVALID_INPUT","用户头像不需要额外参数"));}
            let reset=input.operation=="owner_avatar.reset";
            (if reset {"DELETE"} else {"GET"},"/system/user-avatar".into(),&[],reset)
        },
        "agents.list"=>("GET","/agents".into(),&[],false),
        "agents.get"=>("GET",format!("/agents/{id}"),&[],false),
        "agents.create"=>("POST","/agents".into(),&[],true),
        "agents.save"=>("PUT",format!("/agents/{id}"),&[],true),
        "agents.delete"=>("DELETE",format!("/agents/{id}"),&[],true),
        "agents.export"=>("GET",format!("/agents/{id}/export"),&[],false),
        "agents.enable"=>("POST",format!("/agents/{id}/enable"),&[],true),
        "agents.disable"=>("POST",format!("/agents/{id}/disable"),&[],true),
        "agents.reload"=>("POST","/agents/reload".into(),&[],true),
        "agents.capabilities"=>("GET",format!("/agents/{id}/capabilities"),&[],false),
        "agents.save_capabilities"=>("PUT",format!("/agents/{id}/capabilities"),&[],true),
        "agents.companion"=>("GET",format!("/agents/{id}/companion"),&[],false),
        "agents.save_companion"=>("PUT",format!("/agents/{id}/companion"),&[],true),
        "tools.list"=>("GET","/agents/tools".into(),&["includeAdvanced"],false),
        "models.list"=>("GET","/models".into(),&[],false),
        "models.get"=>("GET",format!("/models/{id}"),&[],false),
        "models.create"=>("POST","/models".into(),&[],true),
        "models.save"=>("PUT",format!("/models/{id}"),&[],true),
        "models.delete"=>("DELETE",format!("/models/{id}"),&[],true),
        "models.test"=>("POST",format!("/models/{id}/test"),&[],true),
        "models.remote"=>("POST","/models/list-remote".into(),&[],false),
        "models.fallbacks"=>("GET",format!("/models/{id}/fallbacks"),&[],false),
        "models.save_fallbacks"=>("PUT",format!("/models/{id}/fallbacks"),&[],true),
        "providers.list"=>("GET","/models/providers".into(),&[],false),
        "providers.get"=>("GET",format!("/models/providers/{id}"),&[],false),
        "providers.create"=>("POST","/models/providers".into(),&[],true),
        "providers.save"=>("PUT",format!("/models/providers/{id}"),&[],true),
        "providers.delete"=>("DELETE",format!("/models/providers/{id}"),&["replacementId"],true),
        "providers.probe"=>("POST",format!("/models/providers/{id}/probe"),&[],true),
        "providers.models"=>("POST",format!("/models/providers/{id}/models"),&[],false),
        "configs.get"=>("GET",format!("/configs/{id}"),&[],false),
        "configs.list"=>("GET","/configs".into(),&["prefix"],false),
        "configs.save"=>("PUT","/configs".into(),&[],true),
        "configs.delete"=>("DELETE",format!("/configs/{id}"),&[],true),
        "configs.batch"=>("POST","/configs/batch".into(),&[],false),
        "configs.save_batch"=>("PUT","/configs/batch".into(),&[],true),
        "configs.export"=>("POST","/configs/export".into(),&[],false),
        "configs.import"=>("POST","/configs/import".into(),&[],true),
        "configs.embedding"=>("POST","/configs/embedding/activate".into(),&[],true),
        "prompts.get"=>("GET","/prompts".into(),&[],false),
        "prompts.save"=>("PUT","/prompts/draft".into(),&[],true),
        "prompts.validate"=>("POST","/prompts/validate".into(),&[],false),
        "prompts.publish"=>("POST","/prompts/publish".into(),&[],true),
        "prompts.restore"=>("POST","/prompts/restore".into(),&[],true),
        "prompts.release"=>("GET",format!("/prompts/versions/{id}"),&[],false),
        "prompts.preview"=>("POST","/prompts/preview".into(),&[],false),
        "prompts.variables"=>("GET","/prompts/variables".into(),&["scope"],false),
        "prompts.reset_variables"=>("POST","/prompts/variables/reset".into(),&[],true),
        "prompts.import"=>("POST","/prompts/import/sillytavern".into(),&[],false),
        "prompts.export"=>("POST","/prompts/export/sillytavern".into(),&[],false),
        "memories.list"=>("GET","/memories".into(),&["agentId","agentIds","query","channels","statuses","modes","importanceMin","importanceMax","tones","participants","places","objects","topics","eventAtFrom","eventAtTo","createdAtFrom","createdAtTo","sort","order","page","pageSize"],false),
        "memories.get"=>("GET",format!("/memories/{id}"),&[],false),
        "memories.edit"=>("PATCH",format!("/memories/{id}"),&[],true),
        "memories.source"=>("GET",format!("/memories/{id}/source"),&[],false),
        "memories.archive"=>("POST",format!("/memories/{id}/archive"),&[],true),
        "memories.restore"=>("POST",format!("/memories/{id}/restore"),&[],true),
        "memories.delete"=>("DELETE",format!("/memories/{id}"),&["deleteSource"],true),
        "memories.batch"=>("POST","/memories/batch".into(),&[],true),
        "memories.graph"=>("GET","/memories/graph".into(),&["agentId","includeArchived","status","limit"],false),
        "memories.deletions"=>("GET","/memories/deletions/status".into(),&[],false),
        "memories.retry_deletion"=>("POST",format!("/memories/deletions/{id}/retry"),&[],true),
        "facts.list"=>("GET","/knowledge/facts".into(),&["query"],false),
        "facts.edit"=>("PATCH",format!("/knowledge/facts/{id}"),&[],true),
        "facts.retract"=>("POST",format!("/knowledge/facts/{id}/retract"),&[],true),
        "facts.delete"=>("DELETE",format!("/knowledge/facts/{id}"),&[],true),
        "documents.list"=>("GET","/knowledge/documents".into(),&["query","page","pageSize"],false),
        "documents.get"=>("GET",format!("/knowledge/documents/{id}"),&[],false),
        "documents.create"=>("POST","/knowledge/documents".into(),&[],true),
        "documents.retry"=>("POST",format!("/knowledge/documents/{id}/retry"),&[],true),
        "documents.delete"=>("DELETE",format!("/knowledge/documents/{id}"),&[],true),
        "documents.reindex"=>("POST","/knowledge/reindex".into(),&[],true),
        "documents.search"=>("POST","/knowledge/search".into(),&[],false),
        "mcp.list"=>("GET","/mcp/configs".into(),&[],false),
        "mcp.create"=>("POST","/mcp/configs".into(),&[],true),
        "mcp.save"=>("PUT",format!("/mcp/configs/{id}"),&[],true),
        "mcp.delete"=>("DELETE",format!("/mcp/configs/{id}"),&[],true),
        "mcp.toggle"=>("POST",format!("/mcp/configs/{id}/toggle"),&[],true),
        "mcp.connect"=>("POST","/mcp/connect".into(),&[],true),
        "mcp.reconnect"=>("POST",format!("/mcp/{id}/reconnect"),&[],true),
        "mcp.status"=>("GET","/mcp/status".into(),&[],false),
        "mcp.tools"=>("GET","/mcp/tools".into(),&[],false),
        "skills.list"=>("GET","/mcp/skills".into(),&[],false),
        "skills.get"=>("GET",format!("/mcp/skills/{id}/content"),&[],false),
        "skills.import"=>("POST","/mcp/skills/import".into(),&[],true),
        "skills.reload"=>("POST","/mcp/skills/reload".into(),&[],true),
        "skills.delete"=>("DELETE",format!("/mcp/skills/{id}"),&[],true),
        "distributed.identity"=>("GET","/distributed/identity".into(),&[],false),
        "distributed.list"=>("GET","/distributed/servers".into(),&[],false),
        "distributed.probe"=>("POST","/distributed/servers/probe".into(),&[],false),
        "distributed.create"=>("POST","/distributed/servers".into(),&[],true),
        "distributed.delete"=>("DELETE",format!("/distributed/servers/{id}"),&[],true),
        "distributed.sync"=>("POST",format!("/distributed/sync/{id}"),&[],true),
        "distributed.pending"=>("GET","/distributed/pending".into(),&[],false),
        "distributed.rollback"=>("POST","/distributed/rollback".into(),&[],true),
        "distributed.nodes"=>("GET","/distributed/capability-nodes".into(),&[],false),
        "distributed.invite"=>("POST","/distributed/capability-invites".into(),&[],true),
        "system.info"=>("GET","/system/info".into(),&[],false),
        "system.usage"=>("GET","/system/token-usage".into(),&["period","timeZone","agentId","threadId","provider","modelId"],false),
        "system.usage_filters"=>("GET","/system/token-usage/filters".into(),&["agentId"],false),
        "system.tokens"=>("POST","/system/token-count".into(),&[],false),
        "runtime.tasks"=>("GET","/runtime/tasks".into(),&[],false),
        "runtime.windows"=>("GET","/runtime/window-agent".into(),&[],false),
        "access.get"=>("GET","/client-access".into(),&[],false),
        "access.status"=>("GET","/client-access/status".into(),&[],false),
        "access.save"=>("PUT","/client-access/settings".into(),&[],true),
        "access.pair"=>("POST","/client-access/pairings".into(),&[],true),
        "access.revoke"=>("DELETE",format!("/client-access/devices/{id}"),&[],true),
        "maintenance.get"=>("GET","/scheduler/status".into(),&[],false),
        "maintenance.trigger"=>("POST","/maintenance/trigger".into(),&[],true),
        "maintenance.indexes"=>("GET","/maintenance/indexes".into(),&[],false),
        "maintenance.reindex"=>("POST","/maintenance/reindex".into(),&[],true),
        "maintenance.memory"=>("GET","/maintenance/memory-config".into(),&[],false),
        "maintenance.save_memory"=>("PUT","/maintenance/memory-config".into(),&[],true),
        "scheduler.proactive"=>("GET","/scheduler/proactive".into(),&[],false),
        "scheduler.save_proactive"=>("PUT","/scheduler/proactive".into(),&[],true),
        "scheduler.check"=>("POST","/scheduler/proactive/check".into(),&[],true),
        "scheduler.activity"=>("GET","/scheduler/activity".into(),&["agentId"],false),
        "scheduler.tasks"=>("GET","/scheduler/tasks".into(),&[],false),
        "scheduler.reminders"=>("GET","/scheduler/reminders".into(),&["agentId"],false),
        "scheduler.cancel"|"scheduler.retry"|"scheduler.pause"|"scheduler.resume"=>("POST",format!("/scheduler/reminders/{id}/{}",input.operation.split('.').next_back().unwrap()),&[],true),
        "scheduler.accept"|"scheduler.dismiss"=>("POST",format!("/scheduler/suggestions/{id}/{}",input.operation.split('.').next_back().unwrap()),&[],true),
        "scheduler.create"=>("POST","/scheduler/agent-tasks".into(),&[],true),
        "scheduler.trigger"=>("POST",format!("/scheduler/trigger/{id}"),&[],true),
        "facilities.list"=>("GET","/stronghold/facilities".into(),&[],false),
        "facilities.create"=>("POST","/stronghold/facilities".into(),&[],true),
        "rooms.list"=>("GET","/stronghold/rooms".into(),&["facilityId"],false),
        "rooms.create"=>("POST","/stronghold/rooms".into(),&[],true),
        "rooms.save"=>("PUT",format!("/stronghold/rooms/{id}"),&[],true),
        "rooms.delete"=>("DELETE",format!("/stronghold/rooms/{id}"),&[],true),
        "rooms.environment"=>("PUT",format!("/stronghold/rooms/{id}/env"),&[],true),
        "rooms.projection"=>("GET",format!("/stronghold/rooms/{id}/projection"),&["limit"],false),
        "rooms.members"=>("GET",format!("/stronghold/rooms/{id}/members"),&[],false),
        "rooms.add_member"=>("POST",format!("/stronghold/rooms/{id}/members"),&[],true),
        "rooms.move"=>("POST","/stronghold/locations/move".into(),&[],true),
        "rooms.location"=>("GET",format!("/stronghold/locations/{id}"),&[],false),
        "rooms.message"=>("POST",format!("/stronghold/rooms/{id}/messages"),&[],true),
        "rooms.delete_message"=> {
            let message=segment(input.params.get("messageId").and_then(Value::as_str).unwrap_or(""))?;
            ("DELETE",format!("/stronghold/rooms/{id}/messages/{message}"),&["messageId"],true)
        },
        "rooms.butler"=>("POST",format!("/stronghold/rooms/{id}/butler-command"),&[],true),
        "butler.get"=>("GET","/stronghold/butler".into(),&[],false),
        "butler.save"=>("PUT","/stronghold/butler/toggle".into(),&[],true),
        "workspace.tree"=>("GET","/workspace/tree".into(),&["agentId","threadId","path"],false),
        "workspace.read"=>("POST","/workspace/read".into(),&[],false),
        "workspace.write"=>("POST","/workspace/write".into(),&[],true),
        "workspace.rename"=>("POST","/workspace/rename".into(),&[],true),
        "workspace.delete"=>("POST","/workspace/delete".into(),&[],true),
        "workspace.search"=>("POST","/workspace/search".into(),&[],false),
        "terminals.list"=>("GET","/terminals".into(),&["agentId","threadId"],false),
        "terminals.create"=>("POST","/terminals".into(),&[],true),
        "terminals.read"=>("GET",format!("/terminals/{id}/read"),&["agentId","threadId","cursor","limit"],false),
        "terminals.write"|"terminals.resize"|"terminals.interrupt"|"terminals.kill"|"terminals.close"|"terminals.archive"|"terminals.pin"=>("POST",format!("/terminals/{id}/{}",input.operation.split('.').next_back().unwrap()),&[],true),
        "terminals.cleanup"=>("POST","/terminals/cleanup".into(),&[],true),
        "thread.models"=>("GET",format!("/chat/threads/{id}/model-settings"),&[],false),
        "thread.save_models"=>("PUT",format!("/chat/threads/{id}/model-settings"),&[],true),
        "thread.flow"=>("GET",format!("/chat/threads/{id}/flow-state"),&["agentId"],false),
        "thread.clear_flow"=>("DELETE",format!("/chat/threads/{id}/flow-state"),&["agentId"],true),
        "thread.tokens"=>("POST",format!("/chat/threads/{id}/token-budget-preview"),&[],false),
        "thread.rewind_preview"=>("POST",format!("/chat/threads/{id}/rewind-preview"),&[],false),
        "thread.pause"|"thread.resume"|"thread.inject"=>("POST",format!("/chat/tasks/{}",input.operation.split('.').next_back().unwrap()),&[],true),
        "voice.status"=>("GET","/voice/status".into(),&[],false),
        "logs.list"=>("GET","/chat/threads".into(),&["agentId","channel","purpose","page","pageSize"],false),
        "logs.get"=>("GET",format!("/chat/threads/{id}"),&["page","pageSize"],false),
        "tasks.list"=>("GET","/background-tasks".into(),&["category","agentId","status","keyword","from","to","page","pageSize"],false),
        "tasks.get"=>("GET",format!("/background-tasks/{id}"),&[],false),
        "tasks.create"=>("POST","/background-tasks".into(),&[],true),
        "tasks.pause"|"tasks.resume"|"tasks.retry"|"tasks.cancel"|"tasks.resume-interrupted"|"tasks.read"=>("POST",format!("/background-tasks/{id}/{}",input.operation.split('.').next_back().unwrap()),&[],true),
        "tasks.delete"=>("DELETE",format!("/background-tasks/{id}"),&[],true),
        "tasks.read_all"=>("POST","/background-tasks/mark-all-read".into(),&[],true),
        "assets.list"=>("GET","/assets".into(),&[],false),
        "assets.rescan"=>("POST","/assets/rescan".into(),&[],true),
        "assets.export"=>("POST","/assets/export-to-custom".into(),&[],true),
        "assets.restore"=>("POST","/assets/restore".into(),&[],true),
        "inbound.list"=>("GET","/inbound-routes".into(),&[],false),
        "inbound.create"=>("POST","/inbound-routes".into(),&[],true),
        "inbound.save"=>("PUT",format!("/inbound-routes/{id}"),&[],true),
        "inbound.delete"=>("DELETE",format!("/inbound-routes/{id}"),&[],true),
        "inbound.resolve"=>("GET","/inbound-routes/resolve".into(),&["source","identifier"],false),
        "integrations.get"=>("GET","/integrations".into(),&[],false),
        "integrations.extension"=> {
            if !["deepwiki","sciverse"].contains(&id.as_str()) {return Err(ChatError::new("INVALID_INPUT","扩展类型无效"));}
            ("PUT",format!("/integrations/extensions/{id}"),&[],true)
        },
        "integrations.feature"|"integrations.source"|"integrations.delete_source"=> {
            if !["search","pdf"].contains(&id.as_str()) {return Err(ChatError::new("INVALID_INPUT","集成功能类型无效"));}
            if input.operation=="integrations.delete_source" {
                let source=segment(input.params.get("sourceId").and_then(Value::as_str).unwrap_or(""))?;
                ("DELETE",format!("/integrations/features/{id}/sources/{source}"),&["sourceId"],true)
            } else if input.operation=="integrations.source" {("POST",format!("/integrations/features/{id}/sources"),&[],true)}
            else {("PUT",format!("/integrations/features/{id}"),&[],true)}
        },
        "tarot.today"=>("GET","/tarot/today".into(),&["agentId","timeZone"],false),
        "tarot.history"=>("GET","/tarot/history".into(),&["agentId"],false),
        "tarot.draw"=>("POST","/tarot/draw".into(),&[],true),
        "tarot.interpret"=>("POST",format!("/tarot/{id}/interpret"),&[],true),
        "reset.logs"=>("POST","/reset/clear-logs".into(),&[],true),
        "reset.memories"=>("POST","/reset/memories".into(),&[],true),
        "reset.factory"=>("POST","/reset/factory".into(),&[],true),
        _=>return Err(ChatError::new("INVALID_INPUT","不支持的远程管理操作")),
    };
    if path.contains("//") || path.ends_with('/') {return Err(ChatError::new("INVALID_INPUT","请选择远程对象"));}
    if input.body.as_ref().is_some_and(|v|v.to_string().len()>1024*1024) {return Err(ChatError::new("INVALID_INPUT","编辑内容超过 1 MiB，请缩小范围"));}
    let mut params=Vec::new();
    for (key,value) in &input.params {
        if input.operation=="rooms.delete_message" && key=="messageId" {continue;}
        if input.operation=="integrations.delete_source" && key=="sourceId" {continue;}
        if !keys.contains(&key.as_str()) {return Err(ChatError::new("INVALID_INPUT","不支持的远程查询条件"));}
        let text=match value {Value::String(v)=>v.clone(),Value::Bool(_)|Value::Number(_)=>value.to_string(),Value::Null=>continue,_=>return Err(ChatError::new("INVALID_INPUT","查询条件必须是文本、数字或开关"))};
        if text.len()>8192 || text.contains('\0') {return Err(ChatError::new("INVALID_INPUT","远程查询条件过长"));}
        params.push((key.clone(),text));
    }
    let body=if matches!(method,"POST"|"PUT"|"PATCH") {Some(input.body.clone().filter(|value|!value.is_null()).unwrap_or_else(||json!({})))} else {input.body.clone()};
    Ok((Route {method:reqwest::Method::from_bytes(method.as_bytes()).unwrap(),path,params,body},write))
}
fn validate_answer_preferences(input:&Request)->ChatResult<()> {
    if input.operation!="configs.save_batch" {return Ok(());}
    let Some(items)=input.body.as_ref().and_then(|body|body["items"].as_array()) else {return Ok(());};
    for item in items {
        match item["key"].as_str() {
            Some("system.wholesomeMode")=> {
                if !matches!(item["value"].as_str(),Some("true"|"false")) {return Err(ChatError::new("INVALID_INPUT","健全模式设置无效，请重新选择"));}
            },
            Some("system.assistantPrefill")=> {
                let config=item["value"].as_str().and_then(|raw|serde_json::from_str::<Value>(raw).ok());
                let valid=config.as_ref().is_some_and(|config|config.is_object() && config["enabled"].is_boolean()
                    && config["content"].as_str().is_some_and(|text|text.encode_utf16().count()<=8000)
                    && (config.get("mode").is_none() || matches!(config["mode"].as_str(),Some("once"|"continuous"))));
                if !valid {return Err(ChatError::new("INVALID_INPUT","请检查预填充设置，正文最多 8000 个字符"));}
            },
            _=>{},
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn remote_manage(app:AppHandle,token:Token,request:Request,request_id:String)->ChatResult<Value> {
    manage(&app,&token,&request,&request_id).await.map_err(transport::before_send)
}
async fn manage(app:&AppHandle,token:&Token,request:&Request,request_id:&str)->ChatResult<Value> {
    let (route,write)=endpoint(request)?;
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();super::validate_token(app,token)?;
    let session=tokio::select! {biased;_=changes.changed()=>return Err(super::inactive()),result=super::session(app,token)=>result?,};
    if write {
        cache::prepare(app,token,request_id,&audit_context(request,&route))?;
    }
    let result=tokio::select! {biased;
        _=changes.changed()=>return Err(super::inactive().with_outcome(if write {"unknown"} else {"not_sent"})),
        result=super::perform(&session,route)=>result,
    };
    if write {
        // 返回值可能包含 API 密钥或配对码，持久记录只保留成功与失败状态。
        let receipt=result.as_ref().map(|_|Value::Null).map_err(Clone::clone);
        cache::settle(app,token,request_id,receipt)?;
    }
    let data=result?;super::validate_token(app,token)?;
    if request.operation=="owner_avatar.reset" {keep_avatar_snapshot(app,token,None,&data,None);}
    Ok(json!({"modeToken":token,"requestId":request_id,"data":data}))
}
fn audit_context(request:&Request,route:&Route)->Value {
    let fingerprint=json!({"path":route.path,"params":route.params,"body":route.body});
    json!({"kind":"management","operation":request.operation,"id":request.id,"sha256":format!("{:x}",Sha256::digest(fingerprint.to_string().as_bytes()))})
}

#[tauri::command]
pub async fn remote_manage_upload(app:AppHandle,token:Token,request_id:String,kind:String,id:Option<String>,file:super::resources::Upload,fields:std::collections::HashMap<String,String>)->ChatResult<Value> {
    upload(&app,&token,&request_id,&kind,id.as_deref(),file,fields).await.map_err(transport::before_send)
}
async fn upload(app:&AppHandle,token:&Token,request_id:&str,kind:&str,id:Option<&str>,file:super::resources::Upload,fields:std::collections::HashMap<String,String>)->ChatResult<Value> {
    let (path,_part,allowed)=match kind {
        "avatar"=>(format!("/agents/{}/avatar",segment(id.unwrap_or(""))?),"avatar",Vec::<&str>::new()),
        "owner_avatar"=> {
            if id.is_some() || file.mime_type!="image/png" {return Err(ChatError::new("INVALID_FILE","请选择裁切后的 PNG 头像"));}
            validate_owner_avatar(&file.bytes)?;
            ("/system/user-avatar".into(),"avatar",Vec::<&str>::new())
        },
        "tts_reference"=> {
            super::tts::validate_reference(&file.bytes)?;
            ("/voice/tts/references".into(),"audio",Vec::<&str>::new())
        },
        "knowledge"=>("/knowledge/documents/upload".into(),"file",vec!["title","replaceId"]),
        "asr"=>("/voice/asr".into(),"audio",vec!["language"]),
        _=>return Err(ChatError::new("INVALID_INPUT","不支持的远程上传类型")),
    };
    if file.bytes.is_empty() || file.bytes.len()>transport::MAX_BODY || file.name.is_empty() || file.name.len()>240 || file.name.contains(['/', '\\']) || file.name.chars().any(char::is_control) || file.mime_type.len()>128 || file.mime_type.contains(['\r','\n'])
        || fields.iter().any(|(key,value)|!allowed.contains(&key.as_str()) || value.len()>2048) {return Err(ChatError::new("INVALID_FILE","文件或上传字段无效，文件上限 8 MiB"));}
    let avatar_bytes=if matches!(kind,"avatar"|"owner_avatar") {Some(file.bytes.clone())} else {None};
    let sha=format!("{:x}",Sha256::digest(&file.bytes));
    let part=reqwest::multipart::Part::bytes(file.bytes).file_name(file.name).mime_str(&file.mime_type).map_err(|_|ChatError::new("INVALID_FILE","文件类型无效"))?;
    let mut form=reqwest::multipart::Form::new().part(part_name(kind),part);
    for (key,value) in &fields {form=form.text(key.clone(),value.clone());}
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();super::validate_token(app,token)?;
    let session=tokio::select! {biased;_=changes.changed()=>return Err(super::inactive()),result=super::session(app,token)=>result?,};
    cache::prepare(app,token,request_id,&json!({"kind":"management_upload","operation":kind,"id":id,"sha256":sha,"fields":fields}))?;
    let result=tokio::select! {biased;_=changes.changed()=>return Err(super::inactive().with_outcome("unknown")),result=async {
        let response=session.client.request(upload_method(kind),format!("{}/api{path}",session.profile.endpoint)).bearer_auth(&session.secret).timeout(std::time::Duration::from_secs(90)).multipart(form).send().await.map_err(transport::send_error)?;
        let response=super::checked_response(&session,response).await?;transport::envelope(&transport::body(response).await?)
    }=>result,};
    cache::settle(app,token,request_id,result.as_ref().map(|_|Value::Null).map_err(Clone::clone))?;
    let data=result?;super::validate_token(app,token)?;
    if matches!(kind,"avatar"|"owner_avatar") {keep_avatar_snapshot(app,token,id,&data,avatar_bytes);}
    Ok(json!({"modeToken":token,"data":data}))
}
fn keep_avatar_snapshot(app:&AppHandle,token:&Token,agent:Option<&str>,profile:&Value,bytes:Option<Vec<u8>>) {
    // 写成功的回执和已提交图片可立即供离线显示；缓存失败不把成功上传伪装成失败。
    let _=(||->ChatResult<()> {
        let resource=if let Some(id)=agent {json!({"kind":"agent_avatar","id":id})} else {
            if !profile["name"].is_string() || !(profile["version"].is_null() || profile["version"].as_str().is_some_and(|version|uuid::Uuid::parse_str(version).is_ok())) {return Ok(());}
            let key=serde_json::to_string(&super::operations::Query::OwnerAvatar).map_err(|_|super::inactive())?;
            let ticket=cache::begin_read(app,token,&key)?;cache::save(app,token,&key,ticket,profile)?;
            json!({"kind":"owner_avatar","version":profile["version"]})
        };
        if let Some(bytes)=bytes {
            let key=json!({"resource":resource,"range":null}).to_string();let ticket=cache::begin_read(app,token,&format!("blob:{key}"))?;
            cache::save_blob(app,token,&key,ticket,&cache::Blob {mime:"image/png".into(),content_range:None,bytes})?;
        }
        Ok(())
    })();
}
fn part_name(kind:&str)->&'static str {match kind {"avatar"|"owner_avatar"=>"avatar","asr"|"tts_reference"=>"audio",_=>"file"}}
fn upload_method(kind:&str)->reqwest::Method {if kind=="owner_avatar" {reqwest::Method::PUT} else {reqwest::Method::POST}}
fn validate_owner_avatar(bytes:&[u8])->ChatResult<()> {
    // 与 infOS 的裁切接口一致，只发送 512px 方形 PNG；头像内容不写入操作回执。
    if bytes.len()<45 || bytes.len()>5*1024*1024 || !bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        || bytes[8..12]!=13u32.to_be_bytes() || &bytes[12..16]!=b"IHDR"
        || bytes[16..20]!=512u32.to_be_bytes() || bytes[20..24]!=512u32.to_be_bytes()
        || &bytes[bytes.len()-8..bytes.len()-4]!=b"IEND" {
        return Err(ChatError::new("INVALID_FILE","请重新裁切头像后再保存"));
    }
    Ok(())
}

#[tauri::command]
pub async fn remote_voice_tts(app:AppHandle,token:Token,input:super::tts::Input,request_id:String)->ChatResult<tauri::ipc::Response> {
    super::tts::synthesize(app,token,request_id,input).await.map_err(transport::before_send)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn input(operation:&str,id:Option<&str>,body:Option<Value>)->Request {
        Request {operation:operation.into(),id:id.map(str::to_owned),params:Map::new(),body}
    }
    #[test]
    fn finite_catalog_rejects_arbitrary_routes_and_missing_identity() {
        for name in ["/api/models","applications.list","browser.open","http.request"] {assert!(endpoint(&input(name,None,None)).is_err());}
        for id in [None,Some("."),Some("..")] {assert!(endpoint(&input("models.save",id,Some(json!({})))).is_err());}
        let mut query=input("workspace.tree",None,None);query.params.insert("url".into(),json!("https://example.invalid"));
        assert!(endpoint(&query).is_err());
    }
    #[test]
    fn reads_and_writes_use_actual_server_routes() {
        let (config,write)=endpoint(&input("voice.tts_config",None,None)).unwrap();assert_eq!(config.path,"/voice/tts/config");assert!(!write);
        let (saved,write)=endpoint(&input("voice.save_tts_config",None,Some(json!({"version":1})))).unwrap();assert_eq!(saved.method,reqwest::Method::PUT);assert!(write);
        let (adapters,write)=endpoint(&input("voice.tts_adapters",None,None)).unwrap();assert_eq!(adapters.path,"/voice/tts/adapters");assert!(!write);
        assert_eq!(part_name("tts_reference"),"audio");assert_eq!(upload_method("tts_reference"),reqwest::Method::POST);
        let (route,write)=endpoint(&input("maintenance.get",None,None)).unwrap();assert_eq!(route.path,"/scheduler/status");assert!(!write);
        let (route,write)=endpoint(&input("terminals.read",Some("terminal-a"),None)).unwrap();assert_eq!(route.path,"/terminals/terminal-a/read");assert_eq!(route.method,reqwest::Method::GET);assert!(!write);
        let (route,write)=endpoint(&input("tasks.resume-interrupted",Some("task-a"),None)).unwrap();assert!(write);assert_eq!(route.body,Some(json!({})));
        let (route,write)=endpoint(&input("prompts.preview",None,Some(json!({"bundle":{}})))).unwrap();assert_eq!(route.method,reqwest::Method::POST);assert!(!write);
        let (route,write)=endpoint(&input("thread.rewind_preview",Some("thread-a"),Some(json!({"wholeThread":true})))).unwrap();assert_eq!(route.path,"/chat/threads/thread-a/rewind-preview");assert!(!write);
    }
    #[test]
    fn persisted_context_contains_hash_instead_of_credentials() {
        let first=input("providers.save",Some("provider-a"),Some(json!({"apiKey":"绝不落盘的测试密钥"})));
        let second=input("providers.save",Some("provider-a"),Some(json!({"apiKey":"另外一个测试密钥"})));
        let context=audit_context(&first,&endpoint(&first).unwrap().0);
        assert!(!context.to_string().contains("测试密钥"));assert!(context.get("body").is_none());
        assert_ne!(context["sha256"],audit_context(&second,&endpoint(&second).unwrap().0)["sha256"]);
    }
    #[test]
    fn answer_preferences_use_server_config_keys_and_preserve_prefill_exactly() {
        let raw=json!({"enabled":true,"content":"  <reply>\n先想一想\n","mode":"continuous"}).to_string();
        let body=json!({"items":[{"key":"system.wholesomeMode","value":"false"},{"key":"system.assistantPrefill","value":raw}]});
        let request=input("configs.save_batch",None,Some(body.clone()));
        let (route,write)=endpoint(&request).unwrap();assert!(write);assert_eq!(route.path,"/configs/batch");assert_eq!(route.method,reqwest::Method::PUT);assert_eq!(route.body,Some(body));
        let audit=audit_context(&request,&route);assert!(!audit.to_string().contains("先想一想"));
        let (route,write)=endpoint(&input("configs.batch",None,Some(json!({"keys":["system.wholesomeMode","system.assistantPrefill"]})))).unwrap();
        assert!(!write);assert_eq!(route.method,reqwest::Method::POST);assert_eq!(route.path,"/configs/batch");
    }
    #[test]
    fn answer_preferences_reject_invalid_flags_modes_and_utf16_overflow() {
        let setting=|key:&str,value:Value|input("configs.save_batch",None,Some(json!({"items":[{"key":key,"value":value}]})));
        for value in [json!(true),json!("on"),Value::Null] {assert!(endpoint(&setting("system.wholesomeMode",value)).is_err());}
        for config in [Value::Null,json!({"enabled":"true","content":"正文"}),json!({"enabled":true}),json!({"enabled":true,"content":"正文","mode":"next"}),json!({"enabled":false,"content":"字".repeat(8001)})] {
            assert!(endpoint(&setting("system.assistantPrefill",Value::String(config.to_string()))).is_err());
        }
        for config in [json!({"enabled":false,"content":""}),json!({"enabled":true,"content":"字".repeat(8000),"mode":"once"}),json!({"enabled":true,"content":"\u{1F600}".repeat(4000),"mode":"continuous"})] {
            assert!(endpoint(&setting("system.assistantPrefill",Value::String(config.to_string()))).is_ok());
        }
        let config=json!({"enabled":true,"content":"\u{1F600}".repeat(4001)});
        assert!(endpoint(&setting("system.assistantPrefill",Value::String(config.to_string()))).is_err());
    }
    #[test]
    fn archive_edits_follow_revision_routes_and_reject_invalid_payloads() {
        let memory=json!({"narrative":"我记得这次散步。","importance":5,"affect":{"tones":["轻松"],"valence":7,"arousal":2},"participants":[],"places":[],"objects":[],"topics":[],"expectedStatus":"archived","regenerateVector":false,"operationId":"00000000-0000-4000-8000-000000000001"});
        let request=input("memories.edit",Some("memory-a"),Some(memory.clone()));
        let (route,write)=endpoint(&request).unwrap();assert!(write);assert_eq!(route.method,reqwest::Method::PATCH);assert_eq!(route.path,"/memories/memory-a");assert_eq!(route.body,Some(memory.clone()));
        for (key,value) in [("importance",json!(11)),("narrative",json!("  ")),("expectedStatus",json!("deleted")),("operationId",json!("not-uuid")),("topics",json!(["字".repeat(201)])),("unexpected",json!(true))] {
            let mut body=memory.clone();body[key]=value;assert!(endpoint(&input("memories.edit",Some("memory-a"),Some(body))).is_err());
        }
        let fact=json!({"statement":"周二进行复盘。","source":"用户补充","confidence":0.9,"operationId":"00000000-0000-4000-8000-000000000002"});
        let (route,write)=endpoint(&input("facts.edit",Some("fact-a"),Some(fact.clone()))).unwrap();assert!(write);assert_eq!(route.method,reqwest::Method::PATCH);assert_eq!(route.path,"/knowledge/facts/fact-a");
        let context=audit_context(&request,&endpoint(&request).unwrap().0);assert!(!context.to_string().contains("散步"));
        let mut invalid=fact;invalid["confidence"]=json!(1.1);assert!(endpoint(&input("facts.edit",Some("fact-a"),Some(invalid))).is_err());
        assert!(endpoint(&input("facts.edit",None,None)).is_err());
    }
    #[test]
    fn owner_avatar_routes_keep_method_and_identity_separate_from_agent_uploads() {
        let (read,write)=endpoint(&input("owner_avatar.get",None,None)).unwrap();
        assert_eq!(read.path,"/system/user-avatar");assert_eq!(read.method,reqwest::Method::GET);assert!(!write);
        let (reset,write)=endpoint(&input("owner_avatar.reset",None,None)).unwrap();
        assert_eq!(reset.path,read.path);assert_eq!(reset.method,reqwest::Method::DELETE);assert!(reset.body.is_none());assert!(write);
        assert!(endpoint(&input("owner_avatar.reset",Some("other-user"),None)).is_err());
        assert!(endpoint(&input("owner_avatar.get",None,Some(json!({"url":"https://example.invalid"})))).is_err());
        assert_eq!(upload_method("owner_avatar"),reqwest::Method::PUT);assert_eq!(part_name("owner_avatar"),"avatar");
        assert_eq!(upload_method("avatar"),reqwest::Method::POST);assert_eq!(upload_method("knowledge"),reqwest::Method::POST);
    }
    #[test]
    fn owner_avatar_rejects_uncropped_wrong_format_and_oversized_images() {
        let mut png=vec![0u8;64];png[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");png[8..12].copy_from_slice(&13u32.to_be_bytes());
        png[12..16].copy_from_slice(b"IHDR");png[16..20].copy_from_slice(&512u32.to_be_bytes());png[20..24].copy_from_slice(&512u32.to_be_bytes());png[56..60].copy_from_slice(b"IEND");
        assert!(validate_owner_avatar(&png).is_ok());
        let mut wide=png.clone();wide[16..20].copy_from_slice(&1024u32.to_be_bytes());assert!(validate_owner_avatar(&wide).is_err());
        let mut wrong=png.clone();wrong[0]=0;assert!(validate_owner_avatar(&wrong).is_err());
        assert!(validate_owner_avatar(&[]).is_err());assert!(validate_owner_avatar(&vec![0;5*1024*1024+1]).is_err());
        let mut unfinished=png;unfinished[56..60].fill(0);assert!(validate_owner_avatar(&unfinished).is_err());
    }
    #[test]
    fn mutation_payload_and_query_values_are_bounded() {
        let body=json!({"content":"字".repeat(400_000)});assert!(endpoint(&input("workspace.write",None,Some(body))).is_err());
        let mut query=input("logs.list",None,None);query.params.insert("page".into(),json!({"nested":true}));assert!(endpoint(&query).is_err());
    }
}
