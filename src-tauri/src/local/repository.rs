use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json::{json, Value};
use uuid::Uuid;
use super::types::*;

pub const FAMILY_THREAD: &str = "group:family";

pub fn desktop_thread(agent_id: &str) -> String { format!("desktop:{agent_id}") }

pub fn migrate(conn: &mut Connection, group: &[Value], agents: &Value) -> ChatResult<()> {
    conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;
        CREATE TABLE IF NOT EXISTS local_configs(key TEXT PRIMARY KEY,value TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS chat_migrations(version INTEGER PRIMARY KEY);
        CREATE TABLE IF NOT EXISTS chat_threads(id TEXT PRIMARY KEY,agent_id TEXT NOT NULL,channel TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS chat_messages(
            id INTEGER PRIMARY KEY AUTOINCREMENT,thread_id TEXT NOT NULL REFERENCES chat_threads(id),
            pair_id TEXT NOT NULL,role TEXT NOT NULL,agent_id TEXT NOT NULL,name TEXT NOT NULL,
            content TEXT NOT NULL,timestamp INTEGER NOT NULL,revision INTEGER NOT NULL DEFAULT 1,
            status TEXT NOT NULL DEFAULT 'active',error TEXT);
        CREATE INDEX IF NOT EXISTS chat_messages_thread ON chat_messages(thread_id,id);
        CREATE INDEX IF NOT EXISTS chat_messages_pair ON chat_messages(thread_id,pair_id);
        CREATE TABLE IF NOT EXISTS chat_message_revisions(
            message_id INTEGER NOT NULL,revision INTEGER NOT NULL,content TEXT NOT NULL,
            PRIMARY KEY(message_id,revision));
        CREATE TABLE IF NOT EXISTS chat_turns(
            request_id TEXT PRIMARY KEY,thread_id TEXT NOT NULL,agent_id TEXT NOT NULL,
            user_message_id INTEGER,assistant_message_id INTEGER NOT NULL,regenerate INTEGER NOT NULL,
            status TEXT NOT NULL,manifest TEXT NOT NULL DEFAULT '{}',error TEXT);")?;
    for (table,column,kind) in [("chat_turns","fingerprint","TEXT"),("chat_turns","assistant_revision","INTEGER"),("chat_messages","reasoning","TEXT NOT NULL DEFAULT ''"),("chat_messages","usage","TEXT NOT NULL DEFAULT '{}'"),
        ("chat_message_revisions","reasoning","TEXT NOT NULL DEFAULT ''"),("chat_message_revisions","usage","TEXT NOT NULL DEFAULT '{}'" )] {
        let columns={ let mut stmt=conn.prepare(&format!("PRAGMA table_info({table})"))?;
            let rows=stmt.query_map([],|r|r.get::<_,String>(1))?;rows.collect::<Result<Vec<_>,_>>()? };
        if !columns.iter().any(|name|name==column) { conn.execute(&format!("ALTER TABLE {table} ADD COLUMN {column} {kind}"),[])?; }
    }
    conn.execute_batch("CREATE TABLE IF NOT EXISTS chat_tool_calls(request_id TEXT NOT NULL,message_id INTEGER NOT NULL,call_id TEXT NOT NULL,name TEXT NOT NULL,arguments TEXT NOT NULL,result TEXT NOT NULL,PRIMARY KEY(request_id,call_id));")?;
    let has_source = {
        let mut stmt = conn.prepare("PRAGMA table_info(memories)")?;
        let mut columns = stmt.query_map([], |row| row.get::<_,String>(1))?;
        columns.any(|column| column.as_deref() == Ok("source_message_id"))
    };
    if !has_source { conn.execute("ALTER TABLE memories ADD COLUMN source_message_id INTEGER",[])?; }
    super::diary::initialize(conn)?;
    super::reminders::initialize(conn)?;
    conn.execute("CREATE INDEX IF NOT EXISTS memory_source ON memories(agent_id,source_message_id)",[])?;
    let migrated: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM chat_migrations WHERE version=1)", [], |r| r.get(0))?;
    if !migrated {
        let tx = conn.transaction()?;
        let legacy = {
            let mut stmt = tx.prepare("SELECT role,content,timestamp,agent_id FROM messages ORDER BY timestamp,id")?;
            let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?, row.get::<_, String>(3)?)))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };
        let mut pairs = std::collections::HashMap::new();
        for (role, content, timestamp, agent_id) in legacy {
            if content == "__loading__" || content == "（暂无内容）" { continue; }
            let thread_id = desktop_thread(&agent_id);
            ensure_thread(&tx, &thread_id, &agent_id, "desktop")?;
            if role == "user" || !pairs.contains_key(&agent_id) { pairs.insert(agent_id.clone(), Uuid::new_v4().to_string()); }
            let name = agents.get(&agent_id).and_then(|a| a.get("name")).and_then(Value::as_str).unwrap_or(&agent_id);
            insert_message(&tx, &thread_id, &pairs[&agent_id], &role, &agent_id,
                if role == "user" { "User" } else { name }, &json!(content), timestamp, "active")?;
        }
        ensure_thread(&tx, FAMILY_THREAD, "", "group")?;
        let mut pair = Uuid::new_v4().to_string();
        for item in group {
            let role = item.get("role").and_then(Value::as_str).unwrap_or("user");
            let name = item.get("name").and_then(Value::as_str).unwrap_or("User");
            let agent_id = if role == "assistant" {
                item.get("agentId").and_then(Value::as_str).map(str::to_owned).or_else(|| {
                    agents.as_object().and_then(|all| all.iter().find(|(id,a)| id.eq_ignore_ascii_case(name) ||
                        a.get("name").and_then(Value::as_str) == Some(name)).map(|(id,_)| id.clone()))
                }).unwrap_or_else(|| name.to_lowercase())
            } else { String::new() };
            if role == "user" { pair = Uuid::new_v4().to_string(); }
            let content = item.get("content").cloned().unwrap_or(json!(""));
            if text_content(&content) == "__loading__" { continue; }
            insert_message(&tx, FAMILY_THREAD, &pair, role, &agent_id, name, &content,
                item.get("timestamp").and_then(Value::as_i64).unwrap_or(0), "active")?;
        }
        tx.execute("UPDATE memories SET source_message_id=(SELECT id FROM chat_messages WHERE agent_id=memories.agent_id
            AND timestamp=memories.msg_timestamp AND role='assistant' LIMIT 1) WHERE
            (SELECT COUNT(*) FROM chat_messages WHERE agent_id=memories.agent_id AND timestamp=memories.msg_timestamp AND role='assistant')=1",[])?;
        tx.execute("INSERT INTO chat_migrations(version) VALUES(1)", [])?;
        tx.commit()?;
    }
    // 崩溃后保留用户输入和已接收正文，但未提交回复不得进入后续上下文。
    conn.execute("UPDATE chat_messages SET status='interrupted',error='上次回复因应用退出而中断' WHERE status='pending'", [])?;
    conn.execute("UPDATE chat_turns SET status='interrupted',error='上次回复因应用退出而中断' WHERE status='running'", [])?;
    Ok(())
}

pub fn ensure_thread(conn: &Connection, id: &str, agent_id: &str, channel: &str) -> ChatResult<Thread> {
    if !matches!(channel, "desktop" | "group") || (channel == "group" && id != FAMILY_THREAD) ||
        (channel == "desktop" && id != desktop_thread(agent_id)) {
        return Err(ChatError::new("INVALID_THREAD", "会话身份与角色不匹配"));
    }
    conn.execute("INSERT OR IGNORE INTO chat_threads(id,agent_id,channel) VALUES(?1,?2,?3)", params![id,agent_id,channel])?;
    thread(conn, id)
}

pub fn thread(conn: &Connection, id: &str) -> ChatResult<Thread> {
    conn.query_row("SELECT id,agent_id,channel FROM chat_threads WHERE id=?1", [id], |r| Ok(Thread {
        id: r.get(0)?, agent_id: r.get(1)?, channel: r.get(2)?,
    })).optional()?.ok_or_else(|| ChatError::new("NOT_FOUND", "会话不存在"))
}

fn map_message(row: &Row<'_>) -> rusqlite::Result<Message> {
    let content: String = row.get(6)?;
    Ok(Message { id: row.get(0)?, thread_id: row.get(1)?, pair_id: row.get(2)?, role: row.get(3)?,
        agent_id: row.get(4)?, name: row.get(5)?, content: serde_json::from_str(&content).unwrap_or(json!(content)),
        timestamp: row.get(7)?, revision: row.get(8)?, status: row.get(9)?, error: row.get(10)?,reasoning:row.get(11)?,usage:serde_json::from_str(&row.get::<_,String>(12)?).unwrap_or(json!({})) })
}

pub fn message(conn: &Connection, id: i64) -> ChatResult<Message> {
    conn.query_row("SELECT * FROM chat_messages WHERE id=?1 AND status!='deleted'", [id], map_message)
        .optional()?.ok_or_else(|| ChatError::new("NOT_FOUND", "消息不存在或已删除"))
}

pub fn messages(conn: &Connection, thread_id: &str, limit: u32, before_id: Option<i64>) -> ChatResult<Vec<Message>> {
    let mut stmt = conn.prepare("SELECT * FROM chat_messages WHERE thread_id=?1 AND status!='deleted'
        AND (?2 IS NULL OR id<?2) ORDER BY id DESC LIMIT ?3")?;
    let rows = stmt.query_map(params![thread_id,before_id,limit.min(10000)], map_message)?;
    let mut items = rows.collect::<Result<Vec<_>, _>>()?;
    items.reverse();
    Ok(items)
}

pub fn snapshot(conn: &Connection, thread_id: &str) -> ChatResult<Snapshot> {
    Ok(Snapshot { thread: thread(conn, thread_id)?, messages: messages(conn, thread_id, 10000, None)? })
}

#[allow(clippy::too_many_arguments)]
pub fn insert_message(conn: &Connection, thread_id: &str, pair_id: &str, role: &str, agent_id: &str,
    name: &str, content: &Value, timestamp: i64, status: &str) -> ChatResult<Message> {
    conn.execute("INSERT INTO chat_messages(thread_id,pair_id,role,agent_id,name,content,timestamp,status)
        VALUES(?1,?2,?3,?4,?5,?6,?7,?8)", params![thread_id,pair_id,role,agent_id,name,content.to_string(),timestamp,status])?;
    message(conn, conn.last_insert_rowid())
}

#[derive(Debug)]
pub struct PreparedTurn {
    pub user: Option<Message>,
    pub assistant: Message,
    pub current_input: Value,
    pub before_id: Option<i64>,
    pub regenerate: bool,
}

pub fn prepare_turn(conn: &mut Connection, request: &TurnRequest, name: &str) -> ChatResult<PreparedTurn> {
    let current_thread = thread(conn, &request.thread_id)?;
    if current_thread.channel == "desktop" && current_thread.agent_id != request.agent_id {
        return Err(ChatError::new("INVALID_THREAD", "角色与会话不匹配"));
    }
    if request.content.is_some() as u8 + request.regenerate_message_id.is_some() as u8 + request.reply_to_message_id.is_some() as u8 != 1 {
        return Err(ChatError::new("INVALID_INPUT", "必须且只能指定当前输入、重新生成目标或群聊回复目标"));
    }
    let tx = conn.transaction()?;
    let prepared = if let Some(id) = request.regenerate_message_id {
        let old = message(&tx, id)?;
        if old.thread_id != request.thread_id || old.agent_id != request.agent_id || old.role != "assistant" || old.status=="pending" {
            return Err(ChatError::new("INVALID_INPUT", "重新生成目标与当前角色不匹配"));
        }
        let user = {
            let mut stmt = tx.prepare("SELECT * FROM chat_messages WHERE thread_id=?1 AND pair_id=?2 AND role='user' AND status='active' ORDER BY id LIMIT 1")?;
            stmt.query_row(params![old.thread_id,old.pair_id], map_message).optional()?
                .ok_or_else(|| ChatError::new("INVALID_INPUT", "找不到这条回复对应的用户消息"))?
        };
        PreparedTurn { current_input: user.content.clone(), before_id: Some(user.id), user: Some(user), assistant: old, regenerate: true }
    } else {
        let (user, input, pair_id, before_id) = if let Some(id) = request.reply_to_message_id {
            if current_thread.channel != "group" { return Err(ChatError::new("INVALID_INPUT", "仅群聊可以继续回复同一条输入")); }
            let user = message(&tx, id)?;
            if user.thread_id != request.thread_id || user.role != "user" || user.status != "active" {
                return Err(ChatError::new("INVALID_INPUT", "群聊输入不存在"));
            }
            (Some(user.clone()), user.content, user.pair_id, None)
        } else {
            let input = request.content.clone().unwrap();
            validate_content(&input)?;
            let pair_id = Uuid::new_v4().to_string();
            let user = if request.ephemeral { None } else {
                Some(insert_message(&tx, &request.thread_id, &pair_id, "user", "", "User", &input, chrono::Utc::now().timestamp_millis(), "active")?)
            };
            let before_id = user.as_ref().map(|m| m.id);
            (user, input, pair_id, before_id)
        };
        let assistant = insert_message(&tx, &request.thread_id, &pair_id, "assistant", &request.agent_id, name,
            &json!(""), chrono::Utc::now().timestamp_millis(), "pending")?;
        PreparedTurn { user, assistant, current_input: input, before_id, regenerate: false }
    };
    tx.execute("INSERT INTO chat_turns(request_id,thread_id,agent_id,user_message_id,assistant_message_id,regenerate,status,fingerprint)
        VALUES(?1,?2,?3,?4,?5,?6,'running',?7)", params![request.request_id,request.thread_id,request.agent_id,
            prepared.user.as_ref().map(|m| m.id),prepared.assistant.id,prepared.regenerate,fingerprint(request)])?;
    tx.commit()?;
    Ok(prepared)
}

pub fn update_partial(conn: &Connection, id: i64, content: &str) -> ChatResult<()> {
    conn.execute("UPDATE chat_messages SET content=?1 WHERE id=?2 AND status='pending'", params![json!(content).to_string(),id])?;
    Ok(())
}

pub fn finish_turn(conn:&mut Connection,request:&TurnRequest,prepared:&PreparedTurn,content:&str,error:Option<&ChatError>,manifest:&Value)->ChatResult<Message> {
    finish_with_tools(conn,request,prepared,&Completion { content:content.into(),..Default::default() },error,manifest,&super::chat_tools::ToolSession::default())
}

pub fn finish_with_tools(conn:&mut Connection,request:&TurnRequest,prepared:&PreparedTurn,completion:&Completion,error:Option<&ChatError>,manifest:&Value,tools:&super::chat_tools::ToolSession)->ChatResult<Message> {
    let tx=conn.transaction()?;
    let old=message(&tx,prepared.assistant.id)?;
    if (prepared.regenerate && (old.revision!=prepared.assistant.revision || old.status!=prepared.assistant.status)) || (!prepared.regenerate && old.status!="pending") {
        return Err(ChatError::new("STALE_MESSAGE","回复目标已经发生变化，已取消提交"));
    }
    let status=error.map(|e|if e.code=="CANCELLED" {"cancelled"} else {"failed"}).unwrap_or("completed");
    if error.is_none() {
        if prepared.regenerate {
            tx.execute("INSERT OR IGNORE INTO chat_message_revisions(message_id,revision,content,reasoning,usage) VALUES(?1,?2,?3,?4,?5)",
                params![old.id,old.revision,old.content.to_string(),old.reasoning,old.usage.to_string()])?;
            super::diary::remove_sources(&tx,&[old.id])?;
            tx.execute("DELETE FROM chat_tool_calls WHERE message_id=?1",[old.id])?;
        }
        tx.execute("UPDATE chat_messages SET content=?1,status='active',error=NULL,revision=revision+?2,reasoning=?3,usage=?4 WHERE id=?5",
            params![json!(completion.content).to_string(),i64::from(prepared.regenerate),completion.reasoning,completion.usage.to_string(),old.id])?;
        let mut source_ids=prepared.user.as_ref().map(|m|vec![m.id]).unwrap_or_default();source_ids.push(old.id);
        tools.commit(&tx,&request.agent_id,&source_ids,chrono::Utc::now().timestamp_millis())?;
        for trace in &tools.traces {
            tx.execute("INSERT INTO chat_tool_calls(request_id,message_id,call_id,name,arguments,result) VALUES(?1,?2,?3,?4,?5,?6)",
                params![request.request_id,old.id,trace.call.id,trace.call.name,trace.call.arguments.to_string(),trace.result.to_string()])?;
        }
    } else if !prepared.regenerate {
        tx.execute("UPDATE chat_messages SET content=?1,status=?2,error=?3,reasoning=?4,usage=?5 WHERE id=?6",
            params![json!(completion.content).to_string(),status,error.map(|e|e.message.as_str()),completion.reasoning,completion.usage.to_string(),old.id])?;
    }
    tx.execute("UPDATE chat_turns SET status=?1,error=?2,manifest=?3,assistant_revision=?5 WHERE request_id=?4",params![status,error.map(|e|e.message.as_str()),manifest.to_string(),request.request_id,old.revision+i64::from(prepared.regenerate && error.is_none())])?;
    let result=message(&tx,old.id)?;tx.commit()?;Ok(result)
}

fn fingerprint(request:&TurnRequest)->String {
    use sha2::{Digest,Sha256};
    let mut canonical=request.clone();canonical.request_id.clear();
    format!("{:x}",Sha256::digest(serde_json::to_vec(&canonical).unwrap_or_default()))
}

pub fn existing_turn(conn:&Connection,request:&TurnRequest)->ChatResult<Option<Message>> {
    let existing:Option<(String,String,String,i64,Option<String>,Option<i64>)>=conn.query_row("SELECT thread_id,agent_id,status,assistant_message_id,fingerprint,assistant_revision FROM chat_turns WHERE request_id=?1",
        [&request.request_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional()?;
    match existing {
        None=>Ok(None),
        Some((thread,agent,status,id,hash,revision)) if thread==request.thread_id && agent==request.agent_id && status=="completed" && hash.as_deref()==Some(&fingerprint(request))=> {
            let message=message(conn,id)?;
            if message.status!="active" || revision!=Some(message.revision) { return Err(ChatError::new("REQUEST_EXISTS","原回复已删除或变更，不能重放该请求")); }
            Ok(Some(message))
        }
        Some(_)=>Err(ChatError::new("REQUEST_EXISTS","请求身份已使用、内容不匹配或已中断，请重新发送")),
    }
}

pub fn delete_message(conn: &mut Connection, thread_id: &str, id: i64, pair: bool) -> ChatResult<()> {
    let target = message(conn,id)?;
    if target.thread_id != thread_id { return Err(ChatError::new("INVALID_THREAD", "消息不属于当前会话")); }
    let tx = conn.transaction()?;
    let targets = if pair || target.role=="user" {
        let mut stmt = tx.prepare("SELECT id,timestamp,agent_id FROM chat_messages WHERE thread_id=?1 AND pair_id=?2 AND status!='deleted'")?;
        let rows = stmt.query_map(params![thread_id,target.pair_id], |r| Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,String>(2)?)))?;
        rows.collect::<Result<Vec<_>,_>>()?
    } else { vec![(target.id,target.timestamp,target.agent_id)] };
    super::diary::remove_sources(&tx,&targets.iter().map(|t|t.0).collect::<Vec<_>>())?;
    for (message_id,timestamp,agent_id) in targets {
        tx.execute("UPDATE chat_messages SET status='deleted',revision=revision+1,content='\"\"',reasoning='',usage='{}',error=NULL WHERE id=?1", [message_id])?;
        tx.execute("DELETE FROM chat_message_revisions WHERE message_id=?1",[message_id])?;
        tx.execute("DELETE FROM chat_tool_calls WHERE message_id=?1",[message_id])?;
        tx.execute("UPDATE chat_turns SET status='deleted',manifest='{}',error=NULL WHERE assistant_message_id=?1 OR user_message_id=?1",[message_id])?;
        super::diary::dirty(&tx,&agent_id)?;
        // 仅清理当前角色且来源匹配的旧记忆，避免相同时间戳跨角色误删。
        tx.execute("DELETE FROM memories WHERE agent_id=?1 AND (source_message_id=?2 OR (source_message_id IS NULL AND msg_timestamp=?3))", params![agent_id,message_id,timestamp])?;
    }
    tx.commit()?;
    Ok(())
}
