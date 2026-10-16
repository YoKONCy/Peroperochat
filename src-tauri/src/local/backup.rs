use std::collections::{BTreeMap, HashSet};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::AppHandle;
use crate::db::{DbState, Memory, Reminder, Topic};
use super::{db_lock, identity, repository, runtime::ChatRuntime, types::*};

#[derive(Serialize, Deserialize)]
struct Revision { message_id: i64, revision: i64, content: String,
    #[serde(default)] reasoning:String,#[serde(default)] usage:Value }

#[derive(Serialize, Deserialize)]
struct Backup {
    version: String,
    config: BTreeMap<String, Value>,
    agents: Value,
    threads: Vec<Thread>,
    messages: Vec<Message>,
    memories: Vec<Memory>,
    #[serde(default)]
    reminders: Vec<Reminder>,
    #[serde(default)]
    topics: Vec<Topic>,
    #[serde(default)]
    revisions: Vec<Revision>,
}

fn read_memories(conn: &Connection) -> ChatResult<Vec<Memory>> {
    let mut stmt = conn.prepare("SELECT id,content,tags,importance,timestamp,msg_timestamp,type,source,agent_id,source_message_id,diary_date,revision,source_message_ids FROM memories ORDER BY id")?;
    let rows = stmt.query_map([], |r| {
        let tags: String = r.get(2)?;
        Ok(Memory { id:r.get(0)?,content:r.get(1)?,tags:serde_json::from_str(&tags).unwrap_or_default(),importance:r.get(3)?,
            timestamp:r.get(4)?,msg_timestamp:r.get(5)?,type_:r.get(6)?,source:r.get(7)?,agent_id:r.get(8)?,source_message_id:r.get(9)?,diary_date:r.get(10)?,revision:r.get(11)?,source_message_ids:serde_json::from_str(&r.get::<_,String>(12)?).unwrap_or_default() })
    })?;
    Ok(rows.collect::<Result<Vec<_>,_>>()?)
}

#[tauri::command]
pub fn local_export_data(app: AppHandle, db: tauri::State<DbState>, runtime: tauri::State<ChatRuntime>) -> ChatResult<Value> {
    let _lease = runtime.maintenance()?;
    let agents = identity::agents(&app)?;
    let conn = db_lock(&db)?;
    let config: BTreeMap<String,Value> = serde_json::from_value(identity::snapshot_in(&conn)?).map_err(|_| invalid())?;
    let threads = {
        let mut stmt = conn.prepare("SELECT id,agent_id,channel FROM chat_threads")?;
        let rows = stmt.query_map([], |r| Ok(Thread { id:r.get(0)?,agent_id:r.get(1)?,channel:r.get(2)? }))?;
        rows.collect::<Result<Vec<_>,_>>()?
    };
    let messages = {
        let mut stmt = conn.prepare("SELECT id,thread_id,pair_id,role,agent_id,name,content,timestamp,revision,status,error,reasoning,usage FROM chat_messages ORDER BY id")?;
        let rows = stmt.query_map([], |r| {
            let content: String = r.get(6)?;
            Ok(Message { id:r.get(0)?,thread_id:r.get(1)?,pair_id:r.get(2)?,role:r.get(3)?,agent_id:r.get(4)?,name:r.get(5)?,
                content:serde_json::from_str(&content).unwrap_or(json!(content)),timestamp:r.get(7)?,revision:r.get(8)?,status:r.get(9)?,error:r.get(10)?,reasoning:r.get(11)?,usage:serde_json::from_str(&r.get::<_,String>(12)?).unwrap_or(json!({})) })
        })?;
        rows.collect::<Result<Vec<_>,_>>()?
    };
    let reminders = {
        let mut stmt = conn.prepare("SELECT id,task,time,timestamp,agent_id,delivered,recur,recur_until,revision,delivery_error FROM reminders ORDER BY id")?;
        let rows = stmt.query_map([], |r| Ok(Reminder { id:r.get(0)?,task:r.get(1)?,time:r.get(2)?,timestamp:r.get(3)?,
            agent_id:r.get(4)?,delivered:r.get(5)?,recur:r.get(6)?,recur_until:r.get(7)?,revision:r.get(8)?,delivery_error:r.get(9)? }))?;
        rows.collect::<Result<Vec<_>,_>>()?
    };
    let topics = {
        let mut stmt = conn.prepare("SELECT id,topic,revealed,timestamp,agent_id,time FROM topics ORDER BY id")?;
        let rows = stmt.query_map([], |r| Ok(Topic { id:r.get(0)?,topic:r.get(1)?,revealed:r.get(2)?,timestamp:r.get(3)?,agent_id:r.get(4)?,time:r.get(5)? }))?;
        rows.collect::<Result<Vec<_>,_>>()?
    };
    let revisions = {
        let mut stmt = conn.prepare("SELECT message_id,revision,content,reasoning,usage FROM chat_message_revisions")?;
        let rows = stmt.query_map([], |r| Ok(Revision { message_id:r.get(0)?,revision:r.get(1)?,content:r.get(2)?,reasoning:r.get(3)?,usage:serde_json::from_str(&r.get::<_,String>(4)?).unwrap_or(json!({})) }))?;
        rows.collect::<Result<Vec<_>,_>>()?
    };
    serde_json::to_value(Backup { version:"2.0".into(),config,agents,
        threads,messages,memories:read_memories(&conn)?,reminders,topics,revisions })
        .map_err(|_| ChatError::new("STORAGE_ERROR","无法生成备份"))
}

fn invalid() -> ChatError { ChatError::new("INVALID_BACKUP","备份格式无效，当前数据未被修改") }

fn parse_backup(data: Value) -> ChatResult<Backup> {
    let version = data.get("version").and_then(Value::as_str).ok_or_else(invalid)?;
    let mut backup: Backup = if version == "2.0" {
        serde_json::from_value(data).map_err(|_| invalid())?
    } else if version == "1.0" {
        let agents = identity::normalize_agents(data.get("agents").cloned().ok_or_else(invalid)?)?;
        let config: BTreeMap<String,Value> = serde_json::from_value(data.get("config").cloned().unwrap_or(json!({}))).map_err(|_| invalid())?;
        let mut backup = Backup { version:"2.0".into(),config,agents,threads:Vec::new(),messages:Vec::new(),memories:Vec::new(),
            reminders:Vec::new(),topics:Vec::new(),revisions:Vec::new() };
        let messages = data.get("messages").and_then(Value::as_object).ok_or_else(invalid)?;
        for (agent_id,items) in messages {
            let items = items.as_array().ok_or_else(invalid)?;
            let thread_id = repository::desktop_thread(agent_id);
            backup.threads.push(Thread { id:thread_id.clone(),agent_id:agent_id.clone(),channel:"desktop".into() });
            let mut pair_id = uuid::Uuid::new_v4().to_string();
            for item in items {
                let role = item.get("role").and_then(Value::as_str).ok_or_else(invalid)?;
                if role == "user" { pair_id = uuid::Uuid::new_v4().to_string(); }
                let content = item.get("content").cloned().ok_or_else(invalid)?;
                if content.as_str() == Some("__loading__") { continue; }
                backup.messages.push(Message { id:backup.messages.len() as i64+1,thread_id:thread_id.clone(),pair_id:pair_id.clone(),
                    role:role.into(),agent_id:if role == "assistant" { agent_id.clone() } else { String::new() },
                    name:if role == "assistant" { backup.agents.get(agent_id).and_then(|a| a.get("name")).and_then(Value::as_str).unwrap_or(agent_id).into() } else { "User".into() },
                    content,timestamp:item.get("timestamp").and_then(Value::as_i64).ok_or_else(invalid)?,revision:1,status:"active".into(),error:None,reasoning:String::new(),usage:json!({}) });
            }
        }
        if let Some(memories) = data.get("memories").and_then(Value::as_object) {
            for (agent_id,items) in memories {
                let items: Vec<Memory> = serde_json::from_value(items.clone()).map_err(|_| invalid())?;
                if items.iter().any(|m| &m.agent_id != agent_id) { return Err(invalid()); }
                backup.memories.extend(items);
            }
        }
        // 旧群聊保存在配置里，只在明确导入时转换一次。
        if let Some(raw) = backup.config.get("ppc.group.messages").and_then(Value::as_str) {
            let group: Vec<Value> = serde_json::from_str(raw).map_err(|_| invalid())?;
            backup.threads.push(Thread { id:repository::FAMILY_THREAD.into(),agent_id:String::new(),channel:"group".into() });
            let mut pair_id = uuid::Uuid::new_v4().to_string();
            for item in group {
                let role = item.get("role").and_then(Value::as_str).ok_or_else(invalid)?;
                let name = item.get("name").and_then(Value::as_str).unwrap_or("User");
                if role == "user" { pair_id = uuid::Uuid::new_v4().to_string(); }
                let agent_id = backup.agents.as_object().unwrap().iter()
                    .find(|(id,a)| id.eq_ignore_ascii_case(name) || a.get("name").and_then(Value::as_str) == Some(name))
                    .map(|(id,_)| id.clone()).unwrap_or_else(|| name.to_lowercase());
                backup.messages.push(Message { id:backup.messages.len() as i64+1,thread_id:repository::FAMILY_THREAD.into(),pair_id:pair_id.clone(),
                    role:role.into(),agent_id:if role == "assistant" { agent_id } else { String::new() },name:name.into(),
                    content:item.get("content").cloned().ok_or_else(invalid)?,timestamp:item.get("timestamp").and_then(Value::as_i64).ok_or_else(invalid)?,
                    revision:1,status:"active".into(),error:None,reasoning:String::new(),usage:json!({}) });
            }
        }
        backup
    } else { return Err(invalid()); };
    backup.agents = identity::normalize_agents(backup.agents)?;
    if backup.messages.len()>100_000 || backup.config.len()>10_000 || backup.memories.len()>100_000 || backup.revisions.len()>100_000 { return Err(invalid()); }
    let threads: BTreeMap<_,_> = backup.threads.iter().map(|t| (t.id.clone(),t)).collect();
    if threads.len() != backup.threads.len() { return Err(invalid()); }
    for thread in &backup.threads {
        if !((thread.channel == "desktop" && thread.id == repository::desktop_thread(&thread.agent_id)) ||
            (thread.channel == "group" && thread.id == repository::FAMILY_THREAD && thread.agent_id.is_empty())) { return Err(invalid()); }
    }
    let mut ids = HashSet::new();
    for message in &mut backup.messages {
        let thread = threads.get(&message.thread_id).ok_or_else(invalid)?;
        if message.id <= 0 || !ids.insert(message.id) || message.revision < 1 || message.pair_id.is_empty() ||
            !matches!(message.role.as_str(),"user"|"assistant"|"system") ||
            !matches!(message.status.as_str(),"active"|"pending"|"failed"|"cancelled"|"interrupted"|"deleted") ||
            (message.role == "assistant" && thread.channel == "desktop" && message.agent_id != thread.agent_id) ||
            !(message.content.is_string() || message.content.is_array()) || message.reasoning.len()>2*1024*1024 || message.usage.to_string().len()>64_000 { return Err(invalid()); }
        if message.status == "pending" { message.status = "interrupted".into(); message.error = Some("备份中的回复未完成，请重新生成".into()); }
        if message.status=="deleted" {message.content=json!("");message.reasoning.clear();message.usage=json!({});message.error=None;}
    }
    if backup.revisions.iter().any(|r| !ids.contains(&r.message_id) || r.revision < 1) { return Err(invalid()); }
    let mut reminder_ids=HashSet::new();
    for reminder in &mut backup.reminders {
        if reminder.id.is_some_and(|id|id<=0 || id>9_007_199_254_740_991 || !reminder_ids.insert(id)) || reminder.revision<1
            || reminder.agent_id.is_empty() || reminder.agent_id.len()>128 || reminder.task.trim().is_empty() || reminder.task.len()>2000 {
            return Err(invalid());
        }
        let time=chrono::NaiveDateTime::parse_from_str(&reminder.time,"%Y-%m-%d %H:%M:%S").map_err(|_|invalid())?;
        if time.format("%Y-%m-%d %H:%M:%S").to_string()!=reminder.time {return Err(invalid());}
        let recur=reminder.recur.get_or_insert_with(||"none".into());
        if !["none","daily","weekly"].contains(&recur.as_str()) {return Err(invalid());}
        if reminder.recur_until.as_ref().is_some_and(|value|value.trim().is_empty()) {reminder.recur_until=None;}
        if let Some(until)=&reminder.recur_until {
            let until=chrono::NaiveDateTime::parse_from_str(until,"%Y-%m-%d %H:%M:%S").map_err(|_|invalid())?;
            if recur=="none" || until<time {return Err(invalid());}
        }
        if reminder.delivery_error.as_ref().is_some_and(|value|value.len()>2000) {return Err(invalid());}
    }
    if backup.memories.iter().any(|m|m.id.is_some_and(|id|id<=0) || m.revision<1 || m.agent_id.len()>128 || m.content.len()>16_000 || !(0..=10).contains(&m.importance)
        || m.tags.len()>16 || m.tags.iter().any(|s|s.len()>100) || m.source_message_ids.len()>100_000
        || m.diary_date.as_ref().is_some_and(|date|chrono::NaiveDate::parse_from_str(date,"%Y-%m-%d").is_err())
        || m.source_message_id.is_some_and(|id|!ids.contains(&id)) || m.source_message_ids.iter().any(|id|!ids.contains(id))) { return Err(invalid()); }
    let deleted:HashSet<_>=backup.messages.iter().filter(|m|m.status=="deleted").map(|m|m.id).collect();
    backup.memories.retain(|m|!m.source_message_id.is_some_and(|id|deleted.contains(&id)) && !m.source_message_ids.iter().any(|id|deleted.contains(id)));
    backup.revisions.retain(|r|!deleted.contains(&r.message_id));
    if backup.config.values().any(|value| !value.is_string()) { return Err(invalid()); }
    // 旧备份的 API Key 使用另一键名；统一迁移后不再双读写。
    if let Some(key) = backup.config.remove("ppc.apiKey") { backup.config.entry("api_key".into()).or_insert(key); }
    backup.config.remove("ppc.group.messages");
    backup.config.insert("ppc.agents".into(),json!(backup.agents.to_string()));
    let config=serde_json::to_value(&backup.config).map_err(|_|invalid())?;
    super::model_config::from_config(&config).map_err(|_|invalid())?;
    super::prompts::load(&config).map_err(|_|invalid())?;
    Ok(backup)
}

fn clear_database(conn: &Connection) -> ChatResult<()> {
    super::group::initialize(conn)?;
    super::blackjack::initialize(conn)?;
    conn.execute_batch("DELETE FROM local_group_batches; DELETE FROM local_blackjack_games;")?;
    conn.execute("INSERT OR IGNORE INTO memory_tdb_pending(agent_id) SELECT DISTINCT agent_id FROM memories",[])?;
    conn.execute_batch("DELETE FROM chat_tool_calls;DELETE FROM chat_turns;DELETE FROM chat_message_revisions;DELETE FROM chat_messages;DELETE FROM chat_threads;
        DELETE FROM memories;DELETE FROM reminders;DELETE FROM topics;DELETE FROM messages;")?;
    Ok(())
}

fn write_backup(conn: &Connection, backup: &Backup) -> ChatResult<()> {
    clear_database(conn)?;
    for thread in &backup.threads { repository::ensure_thread(conn,&thread.id,&thread.agent_id,&thread.channel)?; }
    for message in &backup.messages {
        conn.execute("INSERT INTO chat_messages(id,thread_id,pair_id,role,agent_id,name,content,timestamp,revision,status,error,reasoning,usage)
            VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",params![message.id,message.thread_id,message.pair_id,message.role,message.agent_id,
                message.name,message.content.to_string(),message.timestamp,message.revision,message.status,message.error,message.reasoning,message.usage.to_string()])?;
    }
    for memory in &backup.memories {
        conn.execute("INSERT INTO memories(id,content,tags,importance,timestamp,msg_timestamp,type,source,agent_id,source_message_id,diary_date,revision,source_message_ids) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
            params![memory.id,memory.content,json!(memory.tags).to_string(),memory.importance,memory.timestamp,memory.msg_timestamp,memory.type_,memory.source,memory.agent_id,memory.source_message_id,memory.diary_date,memory.revision,json!(memory.source_message_ids).to_string()])?;
        super::diary::dirty(conn,&memory.agent_id)?;
    }
    for reminder in &backup.reminders {
        conn.execute("INSERT INTO reminders(id,task,time,timestamp,agent_id,delivered,recur,recur_until,revision,delivery_error) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![reminder.id,reminder.task,reminder.time,reminder.timestamp,reminder.agent_id,reminder.delivered.unwrap_or(false),reminder.recur,reminder.recur_until,reminder.revision.max(1),reminder.delivery_error])?;
    }
    for topic in &backup.topics {
        conn.execute("INSERT INTO topics(id,topic,revealed,timestamp,agent_id,time) VALUES(?1,?2,?3,?4,?5,?6)",params![topic.id,topic.topic,topic.revealed,topic.timestamp,topic.agent_id,topic.time])?;
    }
    for revision in &backup.revisions {
        conn.execute("INSERT INTO chat_message_revisions(message_id,revision,content,reasoning,usage) VALUES(?1,?2,?3,?4,?5)",params![revision.message_id,revision.revision,revision.content,revision.reasoning,revision.usage.to_string()])?;
    }
    conn.execute("DELETE FROM local_configs",[])?;
    for (key,value) in &backup.config { identity::set_value_in(conn,key,value)?; }
    Ok(())
}

#[tauri::command]
pub async fn local_import_data(app:AppHandle, db: tauri::State<'_,DbState>, runtime: tauri::State<'_,ChatRuntime>, data: Value) -> ChatResult<()> {
    let backup = parse_backup(data)?;
    let _lease = runtime.maintenance()?;
    let mut conn = db_lock(&db)?;
    let tx = conn.transaction()?;
    write_backup(&tx,&backup)?;
    tx.commit()?;
    let result=db.diary.synchronize(&conn);drop(conn);
    if let Err(error)=super::reminder_platform::synchronize(&app) {eprintln!("备份恢复后的系统提醒同步等待恢复：{}",error.message);}
    result
}

#[tauri::command]
pub async fn local_reset_data(app:AppHandle, db: tauri::State<'_,DbState>, runtime: tauri::State<'_,ChatRuntime>, clear_config: bool) -> ChatResult<()> {
    let _lease = runtime.maintenance()?;
    let mut conn = db_lock(&db)?;
    let tx = conn.transaction()?;
    clear_database(&tx)?;
    if clear_config { tx.execute("DELETE FROM local_configs",[])?; }
    else { identity::set_value_in(&tx,"ppc.conversation_count",&json!("0"))?; }
    tx.commit()?;
    let result=db.diary.synchronize(&conn);drop(conn);
    if let Err(error)=super::reminder_platform::synchronize(&app) {eprintln!("数据重置后的系统提醒取消等待恢复：{}",error.message);}
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reminder_backup_keeps_revision_and_accepts_records_before_revision_migration() {
        let db=super::super::tests::state();let mut conn=db.conn.lock().unwrap();
        let original=json!({"version":"2.0","config":{},"agents":{"pero":{"name":"Pero"}},"threads":[],"messages":[],"memories":[],
            "reminders":[{"id":10,"task":"浇花","time":"2030-01-02 12:00:00","timestamp":1,"agent_id":"pero","delivered":false,"recur":"daily","recur_until":null,"revision":7,"delivery_error":"等待通知权限"}]});
        let backup=parse_backup(original.clone()).unwrap();
        {let tx=conn.transaction().unwrap();write_backup(&tx,&backup).unwrap();tx.commit().unwrap();}
        let restored=super::super::reminders::entry(&conn,"pero",10).unwrap();assert_eq!(restored.revision,7);assert_eq!(restored.delivery_error.as_deref(),Some("等待通知权限"));
        let mut legacy=original.clone();legacy["reminders"][0].as_object_mut().unwrap().remove("revision");legacy["reminders"][0].as_object_mut().unwrap().remove("delivery_error");
        assert_eq!(parse_backup(legacy).unwrap().reminders[0].revision,1);
        let mut invalid=original;invalid["reminders"][0]["time"]=json!("错误时间");assert!(parse_backup(invalid).is_err());
        assert_eq!(super::super::reminders::entry(&conn,"pero",10).unwrap().revision,7);
    }
    #[test]
    fn old_backup_is_validated_before_replacing_data() {
        let data = json!({"version":"1.0","config":{"ppc.apiKey":"密钥"},"agents":{"pero":{"name":"Pero"}},
            "messages":{"pero":[{"role":"user","content":"你好","timestamp":100},{"role":"assistant","content":"主人你好","timestamp":101}]},"memories":{}});
        let backup = parse_backup(data).unwrap();
        assert_eq!(backup.messages[0].pair_id,backup.messages[1].pair_id);
        assert_eq!(backup.config["api_key"],"密钥");
        assert!(!backup.config.contains_key("ppc.apiKey"));
        assert!(parse_backup(json!({"version":"1.0","agents":{},"messages":{}})).is_err());
        let mut invalid = serde_json::to_value(backup).unwrap();
        invalid["messages"][1]["threadId"] = json!("desktop:nana");
        assert!(parse_backup(invalid).is_err());
    }
    #[test]
    fn import_failure_rolls_back_messages_and_configuration_together() {
        let mut conn = super::super::tests::database();
        repository::migrate(&mut conn,&[],&json!({})).unwrap();
        identity::set_value_in(&conn,"原配置",&json!("保留")).unwrap();
        repository::ensure_thread(&conn,"desktop:pero","pero","desktop").unwrap();
        repository::insert_message(&conn,"desktop:pero","原轮次","user","","User",&json!("原消息"),1,"active").unwrap();
        let raw = json!({"version":"1.0","config":{},"agents":{"pero":{"name":"Pero"}},
            "messages":{"pero":[]},"memories":{"pero":[
                {"id":1,"content":"第一条","tags":[],"importance":5,"timestamp":1,"type_":"event","source":"mobile","agent_id":"pero"},
                {"id":1,"content":"冲突记录","tags":[],"importance":5,"timestamp":2,"type_":"event","source":"mobile","agent_id":"pero"}
            ]}});
        let backup = parse_backup(raw).unwrap();
        {
            let tx = conn.transaction().unwrap();
            assert!(write_backup(&tx,&backup).is_err());
        }
        assert_eq!(repository::snapshot(&conn,"desktop:pero").unwrap().messages[0].content,"原消息");
        assert_eq!(identity::snapshot_in(&conn).unwrap()["原配置"],"保留");
    }

    #[test]
    fn reset_and_import_clear_cached_tdb_and_validate_new_configuration() {
        let db=super::super::tests::state();let mut conn=db.conn.lock().unwrap();
        let draft=super::super::diary::Draft {content:"被重置的日记".into(),date:None,tags:vec![],id:None,expected_revision:None};
        super::super::diary::apply(&conn,"pero",&draft,&[],1).unwrap();
        assert_eq!(db.diary.query(&conn,"pero",&Default::default()).unwrap().len(),1);
        {let tx=conn.transaction().unwrap();clear_database(&tx).unwrap();tx.commit().unwrap();}
        assert!(db.diary.query(&conn,"pero",&Default::default()).unwrap().is_empty());
        let raw=json!({"version":"1.0","config":{},"agents":{"pero":{"name":"佩洛"}},"messages":{"pero":[]},"memories":{"pero":[
            {"id":2,"content":"导入的日记","tags":[],"importance":5,"timestamp":2,"type_":"event","source":"diary","agent_id":"pero"}]}});
        let backup=parse_backup(raw.clone()).unwrap();
        {let tx=conn.transaction().unwrap();write_backup(&tx,&backup).unwrap();tx.commit().unwrap();}
        let entries=db.diary.query(&conn,"pero",&Default::default()).unwrap();assert_eq!(entries.len(),1);assert_eq!(entries[0].content,"导入的日记");
        let mut invalid=raw.clone();invalid["config"]["ppc.modelSettings"]=json!(json!({"contextWindow":1}).to_string());assert!(parse_backup(invalid).is_err());
        let mut invalid=raw;invalid["config"]["ppc.promptWorkspace"]=json!("{损坏}");assert!(parse_backup(invalid).is_err());
        assert_eq!(db.diary.query(&conn,"pero",&Default::default()).unwrap()[0].content,"导入的日记");
    }

}
