use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct Memory {
    pub id: Option<i64>,
    pub content: String,
    pub tags: Vec<String>,
    pub importance: i32,
    pub timestamp: i64,
    pub msg_timestamp: Option<i64>,
    #[serde(alias = "type")]
    pub type_: String, // "event", "fact", etc. mapped from "type"
    pub source: String,
    pub agent_id: String,
    #[serde(default)]
    pub source_message_id: Option<i64>,
    #[serde(default)]
    pub diary_date: Option<String>,
    #[serde(default="memory_revision")]
    pub revision:i64,
    #[serde(default)]
    pub source_message_ids:Vec<i64>,
}

fn memory_revision()->i64 { 1 }

#[derive(Debug, Serialize, Deserialize)]
pub struct Reminder {
    pub id: Option<i64>,
    pub task: String,
    pub time: String,
    pub timestamp: i64,
    pub agent_id: String,
    pub delivered: Option<bool>,
    pub recur: Option<String>,
    pub recur_until: Option<String>,
    #[serde(default="memory_revision")]
    pub revision:i64,
    #[serde(default)]
    pub delivery_error:Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Topic {
    pub id: Option<i64>,
    pub topic: String,
    pub revealed: bool,
    pub timestamp: i64,
    pub agent_id: String,
    #[serde(default)]
    pub time: Option<String>,
}

pub struct DbState {
    pub conn: Mutex<Connection>,
    pub diary: crate::local::diary::DiaryStore,
    pub mode: std::sync::OnceLock<crate::mode::Coordinator>,
}

pub fn init_db(app: &AppHandle) -> Result<DbState, Box<dyn std::error::Error>> {
    let app_dir = app.path().app_data_dir()?;
    let data_dir = app_dir.join("data");

    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
    }

    let db_path = data_dir.join("peropero.db");

    // Migration: Check if old DB exists in root and move it
    let old_db_path = app_dir.join("peropero.db");
    if old_db_path.exists() && !db_path.exists() {
        let _ = std::fs::rename(old_db_path, &db_path);
    }

    let conn = Connection::open(db_path)?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;

    // Initialize tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS memories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            tags TEXT,
            importance INTEGER,
            timestamp INTEGER,
            msg_timestamp INTEGER,
            type TEXT,
            source TEXT,
            agent_id TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp INTEGER,
            agent_id TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS reminders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task TEXT NOT NULL,
            time TEXT NOT NULL,
            timestamp INTEGER,
            agent_id TEXT,
            delivered INTEGER DEFAULT 0,
            recur TEXT DEFAULT 'none',
            recur_until TEXT
        )",
        [],
    )?;
    ensure_reminders_columns(&conn)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS topics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            topic TEXT NOT NULL,
            revealed BOOLEAN DEFAULT 0,
            timestamp INTEGER,
            agent_id TEXT
        )",
        [],
    )?;

    let has_topic_time = {
        let mut stmt = conn.prepare("PRAGMA table_info(topics)")?;
        let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
        let found = columns.filter_map(Result::ok).any(|name| name == "time");
        found
    };
    if !has_topic_time {
        conn.execute("ALTER TABLE topics ADD COLUMN time TEXT", [])?;
    }

    Ok(DbState {
        conn: Mutex::new(conn),
        diary: crate::local::diary::DiaryStore::new(data_dir.join("diary")),
        mode: std::sync::OnceLock::new(),
    })
}

// --- Memory Commands ---

#[tauri::command]
pub fn get_memories(state:tauri::State<DbState>,agent_id:String,limit:u32,offset:u32, mode_token:crate::mode::Token)->Result<Vec<Memory>,crate::local::types::ChatError> {
    crate::mode::LOCAL_TOKEN.sync_scope(mode_token, || {
    let conn=crate::local::db_lock(&state)?;
    Ok(state.diary.query(&conn,&agent_id,&crate::local::diary::Query { limit:limit as usize,offset:offset as usize,..Default::default() })?
        .into_iter().map(|e|Memory { id:Some(e.id),content:e.content,tags:e.tags,importance:5,timestamp:e.timestamp,msg_timestamp:Some(e.timestamp),
            type_:"event".into(),source:"diary".into(),agent_id:e.agent_id,source_message_id:e.source_message_ids.last().copied(),diary_date:Some(e.date),revision:e.revision,source_message_ids:e.source_message_ids }).collect())

    })
}

#[tauri::command]
pub fn delete_memory(state:tauri::State<DbState>,runtime:tauri::State<crate::local::runtime::ChatRuntime>,agent_id:String,id:i64, mode_token:crate::mode::Token)->Result<(),crate::local::types::ChatError> {
    crate::mode::LOCAL_TOKEN.sync_scope(mode_token, || {
    let _lease=runtime.maintenance()?;
    let mut conn=crate::local::db_lock(&state)?;
    let tx=conn.transaction()?;crate::local::diary::delete(&tx,&agent_id,id)?;tx.commit()?;
    state.diary.synchronize(&conn)

    })
}

fn ensure_reminders_columns(conn: &Connection) -> SqlResult<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(reminders)")?;
    let cols_iter = stmt.query_map([], |row| {
        let name: String = row.get(1)?;
        Ok(name)
    })?;
    let mut has_delivered = false;
    let mut has_recur = false;
    let mut has_recur_until = false;
    for c in cols_iter {
        if let Ok(n) = c {
            match n.as_str() {
                "delivered" => has_delivered = true,
                "recur" => has_recur = true,
                "recur_until" => has_recur_until = true,
                _ => {}
            }
        }
    }
    if !has_delivered {
        conn.execute("ALTER TABLE reminders ADD COLUMN delivered INTEGER DEFAULT 0", [])?;
    }
    if !has_recur {
        conn.execute("ALTER TABLE reminders ADD COLUMN recur TEXT DEFAULT 'none'", [])?;
    }
    if !has_recur_until {
        conn.execute("ALTER TABLE reminders ADD COLUMN recur_until TEXT", [])?;
    }
    Ok(())
}

// --- Topic Commands ---

#[tauri::command]
pub fn get_topics(state: tauri::State<DbState>, agent_id: String, mode_token:crate::mode::Token) -> Result<Vec<Topic>, String> {
    crate::mode::LOCAL_TOKEN.sync_scope(mode_token, || {
    let conn = crate::local::db_lock(&state).map_err(|e|e.message)?;
    let mut stmt = conn.prepare(
        "SELECT id, topic, revealed, timestamp, agent_id, time FROM topics WHERE agent_id = ?1 ORDER BY timestamp DESC"
    ).map_err(|e| e.to_string())?;

    let iter = stmt.query_map(params![agent_id], |row| {
        Ok(Topic {
            id: row.get(0)?,
            topic: row.get(1)?,
            revealed: row.get(2)?,
            timestamp: row.get(3)?,
            agent_id: row.get(4)?,
            time: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for item in iter {
        items.push(item.map_err(|e| e.to_string())?);
    }
    Ok(items)

    })
}

#[tauri::command]
pub fn update_topic_status(state: tauri::State<DbState>, id: i64, revealed: bool, mode_token:crate::mode::Token) -> Result<(), String> {
    crate::mode::LOCAL_TOKEN.sync_scope(mode_token, || {
    let conn = crate::local::db_lock(&state).map_err(|e|e.message)?;
    conn.execute("UPDATE topics SET revealed = ?1 WHERE id = ?2", params![revealed, id])
        .map_err(|e| e.to_string())?;
    Ok(())

    })
}

#[tauri::command]
pub fn delete_topic(state: tauri::State<DbState>, id: i64, mode_token:crate::mode::Token) -> Result<(), String> {
    crate::mode::LOCAL_TOKEN.sync_scope(mode_token, || {
    let conn = crate::local::db_lock(&state).map_err(|e|e.message)?;
    conn.execute("DELETE FROM topics WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())

    })
}
