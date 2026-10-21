use serde_json::{json, Value};
use tauri::{AppHandle, Manager};
use rusqlite::{params, Connection, OptionalExtension};
use crate::db::DbState;
use super::db_lock;
use super::provider::ModelSettings;
use super::types::*;

pub fn initialize(app: &AppHandle) -> ChatResult<()> {
    let db = app.state::<DbState>();
    let mut conn = db_lock(&db)?;
    conn.execute_batch("CREATE TABLE IF NOT EXISTS local_configs(key TEXT PRIMARY KEY,value TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS local_config_migrations(version INTEGER PRIMARY KEY);")?;
    let migrated: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM local_config_migrations WHERE version=1)",[],|r|r.get(0))?;
    if migrated { return Ok(()); }
    let path = app.path().app_data_dir().map_err(|_| ChatError::new("STORAGE_ERROR","无法获取本地数据目录"))?.join("data/settings.json");
    let config: Value = if path.exists() {
        let bytes = std::fs::read(path).map_err(|_| ChatError::new("STORAGE_ERROR","旧配置读取失败，原文件已保留"))?;
        serde_json::from_slice(&bytes).map_err(|_| ChatError::new("INVALID_CONFIG","旧配置文件损坏，原文件已保留"))?
    } else { json!({}) };
    let items = config.as_object().ok_or_else(|| ChatError::new("INVALID_CONFIG","旧配置格式无效"))?;
    if let Some(raw) = items.get("ppc.agents").and_then(Value::as_str).filter(|raw| !raw.trim().is_empty()) {
        let agents = serde_json::from_str(raw).map_err(|_| ChatError::new("INVALID_CONFIG","旧角色配置损坏，原文件已保留"))?;
        normalize_agents(agents)?;
    }
    if let Some(raw) = items.get("ppc.group.messages").and_then(Value::as_str).filter(|raw| !raw.trim().is_empty()) {
        serde_json::from_str::<Vec<Value>>(raw).map_err(|_| ChatError::new("INVALID_CONFIG","旧群聊记录损坏，原文件已保留"))?;
    }
    let tx = conn.transaction()?;
    for (key,value) in items { set_value_in(&tx,key,value)?; }
    if let Some(key) = items.get("ppc.apiKey").filter(|_| !items.get("api_key").and_then(Value::as_str).is_some_and(|key| !key.is_empty())) {
        set_value_in(&tx,"api_key",key)?;
    }
    tx.execute("DELETE FROM local_configs WHERE key='ppc.apiKey'",[])?;
    tx.execute("INSERT INTO local_config_migrations(version) VALUES(1)",[])?;
    tx.commit()?;
    Ok(())
}

pub fn snapshot_in(conn: &Connection) -> ChatResult<Value> {
    let mut stmt = conn.prepare("SELECT key,value FROM local_configs")?;
    let rows = stmt.query_map([],|r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?)))?;
    let mut values = serde_json::Map::new();
    for item in rows {
        let (key,value) = item?;
        values.insert(key,serde_json::from_str(&value).map_err(|_| ChatError::new("INVALID_CONFIG","本地配置格式无效"))?);
    }
    Ok(Value::Object(values))
}

pub fn update_builtin_personas(conn:&mut Connection)->ChatResult<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS local_config_migrations(version INTEGER PRIMARY KEY);")?;
    let tx=conn.transaction()?;
    let updated:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM local_config_migrations WHERE version=3)",[],|r|r.get(0))?;
    if updated {return Ok(());}
    let config=snapshot_in(&tx)?;
    let defaults:Value=serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../shared/defaultAgents.json")))
        .map_err(|_|ChatError::new("INVALID_CONFIG","默认角色配置无效"))?;
    if config["ppc.agents"].as_str().is_some_and(|raw|!raw.trim().is_empty()) {
        let mut agents=agents_from(&config)?;
        // 按本次人设更新要求，仅替换这两个已有角色的正文，后续用户编辑不再被启动流程覆盖。
        for id in ["pero","nana"] {
            if let Some(agent)=agents.get_mut(id) {agent["persona_prompt"]=defaults[id]["persona_prompt"].clone();}
        }
        set_value_in(&tx,"ppc.agents",&json!(agents.to_string()))?;
    }
    tx.execute("INSERT INTO local_config_migrations(version) VALUES(3)",[])?;
    tx.commit()?;Ok(())
}

pub fn snapshot(app: &AppHandle) -> ChatResult<Value> {
    let db = app.state::<DbState>();
    let conn = db_lock(&db)?;
    snapshot_in(&conn)
}

pub fn config(app: &AppHandle, key: &str) -> ChatResult<String> {
    let db = app.state::<DbState>();
    let conn = db_lock(&db)?;
    let raw: Option<String> = conn.query_row("SELECT value FROM local_configs WHERE key=?1",[key],|r|r.get(0)).optional()?;
    match raw {
        Some(raw) => Ok(serde_json::from_str::<Value>(&raw).map_err(|_| ChatError::new("INVALID_CONFIG","本地配置格式无效"))?
            .as_str().unwrap_or("").to_owned()),
        None => Ok(String::new()),
    }
}

pub fn set_value_in(conn: &Connection, key: &str, value: &Value) -> ChatResult<()> {
    conn.execute("INSERT INTO local_configs(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![key,value.to_string()])?;
    Ok(())
}

pub fn set_config(app: &AppHandle, key: &str, value: &str) -> ChatResult<()> {
    let db = app.state::<DbState>();
    let conn = db_lock(&db)?;
    set_value_in(&conn,key,&json!(value))
}

pub fn agents(app: &AppHandle) -> ChatResult<Value> {
    agents_from(&snapshot(app)?)
}

pub fn agents_from(config: &Value) -> ChatResult<Value> {
    let saved = config.get("ppc.agents").and_then(Value::as_str).unwrap_or("");
    let value = if saved.trim().is_empty() {
        serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../shared/defaultAgents.json")))
            .map_err(|_| ChatError::new("INVALID_CONFIG", "默认角色配置无效"))?
    } else {
        serde_json::from_str(&saved).map_err(|_| ChatError::new("INVALID_CONFIG", "角色配置损坏，请从备份恢复"))?
    };
    normalize_agents(value)
}

pub fn normalize_agents(mut value: Value) -> ChatResult<Value> {
    let all = value.as_object_mut().ok_or_else(|| ChatError::new("INVALID_CONFIG", "角色配置必须是对象"))?;
    if all.is_empty() || all.len() > 200 { return Err(ChatError::new("INVALID_CONFIG", "至少需要保留一个角色，最多支持 200 个角色")); }
    for (id, agent) in all {
        if id.trim().is_empty() || id.len() > 128 || id.contains(['/', '\\', '\0']) {
            return Err(ChatError::new("INVALID_CONFIG", "角色 ID 为空或包含无效字符"));
        }
        let fields = agent.as_object_mut().ok_or_else(|| ChatError::new("INVALID_CONFIG", "角色配置格式无效"))?;
        if !fields.get("name").and_then(Value::as_str).is_some_and(|name| !name.trim().is_empty() && name.len() <= 200) {
            return Err(ChatError::new("INVALID_CONFIG", "角色名称不能为空或过长"));
        }
        fields.insert("id".into(), json!(id));
        super::appearance::normalize_assets(fields, id)?;
        // 旧角色记录可能未保存头像，只补缺失字段，保留用户指定的头像及显式空值。
        if !fields.contains_key("avatar") {
            if let Some(path)=match id.as_str() {"pero"=>Some("/avatars/pero.jpg"),"nana"=>Some("/avatars/nana.jpg"),_=>None} {
                fields.insert("avatar".into(),json!(path));
            }
        }
        for key in ["system_prompt", "persona_prompt", "post_system_prompt"] {
            if fields.get(key).is_some_and(|v| !v.is_string()) {
                return Err(ChatError::new("INVALID_CONFIG", "角色提示词必须是文本"));
            }
        }
    }
    Ok(value)
}

// 角色定义与 Live2D 派生台词共用一个事务，前端仅提交编辑表单。
pub fn save_agent_in(conn: &mut Connection, id: &str, mut agent: Value, create: bool) -> ChatResult<Value> {
    let tx = conn.transaction()?;
    let config = snapshot_in(&tx)?;
    let mut agents = agents_from(&config)?;
    if create && agents.get(id).is_some() { return Err(ChatError::new("INVALID_CONFIG","该角色 ID 已存在")); }
    let fields = agent.as_object_mut().ok_or_else(|| ChatError::new("INVALID_CONFIG","角色配置必须是对象"))?;
    if let Some(text) = fields.remove("idle_messages_text") {
        let text = text.as_str().ok_or_else(|| ChatError::new("INVALID_CONFIG","闲置台词必须是文本"))?;
        fields.insert("idle_messages".into(), json!(text.lines().map(str::trim).filter(|s| !s.is_empty()).collect::<Vec<_>>()));
    }
    let idle = fields.get("idle_messages").cloned().unwrap_or(json!([]));
    let idle = idle.as_array().filter(|items|items.iter().all(Value::is_string))
        .ok_or_else(|| ChatError::new("INVALID_CONFIG","闲置台词必须是文本列表"))?;
    let now = chrono::Utc::now().timestamp_millis();
    fields.insert("id".into(),json!(id));
    fields.insert("updated_at".into(),json!(now));
    fields.insert("created_at".into(),agents.get(id).and_then(|a|a.get("created_at")).cloned().unwrap_or(json!(now)));
    let key = format!("ppc.{id}.waifu.texts");
    let mut texts = config.get(&key).and_then(Value::as_str).and_then(|s|serde_json::from_str::<Value>(s).ok())
        .filter(Value::is_object).unwrap_or(json!({}));
    texts.as_object_mut().unwrap().retain(|key,_| !key.starts_with("idleMessages_"));
    texts["welcome"] = fields.get("welcome_message").cloned().unwrap_or(json!(""));
    if !texts["welcome"].is_string() { return Err(ChatError::new("INVALID_CONFIG","欢迎台词必须是文本")); }
    for (index,line) in idle.iter().take(50).enumerate() { texts[format!("idleMessages_{:02}",index+1)] = line.clone(); }
    agents[id] = agent;
    let agents = normalize_agents(agents)?;
    set_value_in(&tx,"ppc.agents",&json!(agents.to_string()))?;
    set_value_in(&tx,&key,&json!(texts.to_string()))?;
    tx.commit()?;
    Ok(agents)
}

#[tauri::command]
pub async fn local_role_action(app: AppHandle, runtime: tauri::State<'_,super::runtime::ChatRuntime>,
    action: String, agent: Option<Value>, id: Option<String>, data: Option<String>, mode_token:crate::mode::Token) -> ChatResult<Value> {
    crate::mode::LOCAL_TOKEN.scope(mode_token, async {
    let _lease = runtime.maintenance()?;
    let db = app.state::<DbState>();
    let mut conn = db_lock(&db)?;
    let agents = agents_from(&snapshot_in(&conn)?)?;
    let mut agent = match action.as_str() {
        "create" | "save" => agent.ok_or_else(|| ChatError::new("INVALID_CONFIG","角色配置缺失"))?,
        "import" => serde_json::from_str(&data.unwrap_or_default()).map_err(|_| ChatError::new("INVALID_CONFIG","角色文件格式无效"))?,
        "duplicate" => agents.get(&id.unwrap_or_default()).cloned().ok_or_else(|| ChatError::new("AGENT_NOT_FOUND","角色不存在"))?,
        _ => return Err(ChatError::new("INVALID_INPUT","角色操作无效")),
    };
    if !agent.is_object() { return Err(ChatError::new("INVALID_CONFIG","角色配置必须是对象")); }
    let mut target = agent.get("id").and_then(Value::as_str).unwrap_or("").to_owned();
    if action == "import" && target.is_empty() { return Err(ChatError::new("INVALID_CONFIG","角色文件缺少 ID")); }
    if target.is_empty() || action == "duplicate" || (action == "import" && agents.get(&target).is_some()) {
        target = format!("custom_{}",uuid::Uuid::new_v4());
    }
    if action == "duplicate" { agent["name"] = json!(format!("{} (副本)",agent["name"].as_str().unwrap_or("角色"))); }
    let agents = save_agent_in(&mut conn,&target,agent,action != "save")?;
    drop(conn);
    if let Err(error)=super::reminder_platform::synchronize(&app) {eprintln!("角色变更后的系统提醒同步等待恢复：{}",error.message);}
    Ok(json!({"agents":agents,"agentId":target}))

    }).await
}

#[tauri::command]
pub fn local_save_model_preferences(app: AppHandle, agent_id: String, model_id: u32, texture_id: u32, mode_token:crate::mode::Token) -> ChatResult<()> {
    crate::mode::LOCAL_TOKEN.sync_scope(mode_token, || {
    if model_id > 10000 || texture_id > 10000 { return Err(ChatError::new("INVALID_INPUT","模型或服装编号无效")); }
    let db = app.state::<DbState>();
    let mut conn = db_lock(&db)?;
    let tx = conn.transaction()?;
    if agents_from(&snapshot_in(&tx)?)?.get(&agent_id).is_none() { return Err(ChatError::new("AGENT_NOT_FOUND","角色不存在")); }
    for (key,value) in [(format!("ppc.{agent_id}.modelId"),model_id),(format!("ppc.{agent_id}.modelTexturesId"),texture_id),
        ("ppc.modelId".into(),model_id),("ppc.modelTexturesId".into(),texture_id)] {
        set_value_in(&tx,&key,&json!(value.to_string()))?;
    }
    tx.commit()?;
    Ok(())

    })
}

pub fn state(app: &AppHandle, agent_id: &str) -> ChatResult<Value> {
    state_from(&snapshot(app)?,agent_id)
}

fn state_from(config:&Value, agent_id:&str)->ChatResult<Value> {
    let agent=agents_from(config)?.get(agent_id).cloned().ok_or_else(||ChatError::new("AGENT_NOT_FOUND","角色不存在"))?;
    let get=|suffix:&str|config[format!("ppc.{agent_id}.{suffix}")].as_str().unwrap_or("").to_owned();
    let mut texts=serde_json::from_str::<Value>(&get("waifu.texts")).ok().filter(Value::is_object).unwrap_or(json!({}));
    if texts.get("welcome").is_none() { texts["welcome"]=agent.get("welcome_message").cloned().unwrap_or(json!("")); }
    if let Some(lines)=agent["idle_messages"].as_array().filter(|_|texts.get("idleMessages").is_none()) {
        for (index,line) in lines.iter().take(50).enumerate() {
            let key=format!("idleMessages_{:02}",index+1);
            if texts.get(&key).is_none() { texts[key]=line.clone(); }
        }
    }
    let state_value=|key:&str| {
        let saved=get(key);
        if saved.trim().is_empty() { agent["default_state"][key].as_str().unwrap_or("").to_owned() } else { saved }
    };
    Ok(json!({"agentId":agent_id,"mood":state_value("mood"),"vibe":state_value("vibe"),"mind":state_value("mind"),"texts":texts}))
}

#[tauri::command]
pub fn local_character_projection(app:AppHandle, agent_id:String, mode_token:crate::mode::Token)->ChatResult<Value> {
    crate::mode::LOCAL_TOKEN.sync_scope(mode_token, || {
    let db=app.state::<DbState>();
    let conn=db_lock(&db)?;
    let config=snapshot_in(&conn)?;
    let mut result=state_from(&config,&agent_id)?;
    for suffix in ["modelId","modelTexturesId"] {
        result[suffix]=config.get(format!("ppc.{agent_id}.{suffix}")).and_then(Value::as_str)
            .filter(|s| !s.is_empty()).or_else(||config[format!("ppc.{suffix}")].as_str()).unwrap_or("").into();
    }
    let memories=match db.diary.query(&conn,&agent_id,&super::diary::Query {limit:10,..Default::default()}) {
        Ok(entries)=>entries,Err(error)=> {eprintln!("角色投影暂不加载日记：{}",error.message);result["diaryUnavailable"]=json!(true);Vec::new()}
    };
    let mut tags=Vec::<String>::new();
    for memory in memories { for tag in memory.tags { if tags.len()<8 && !tag.trim().is_empty() && !tags.contains(&tag) { tags.push(tag); } } }
    result["memoryTags"]=json!(tags);
    result["environment"]=super::environment::projection(&config);
    result["presentation"]=super::appearance::presentation(&config,&agent_id)?;
    Ok(result)

    })
}

pub fn model_settings(config:&Value)->ChatResult<ModelSettings> { super::model_config::from_config(config) }
