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
}

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
    })
}

// --- Memory Commands ---

#[tauri::command]
pub fn get_memories(state: tauri::State<DbState>, agent_id: String, limit: u32, offset: u32) -> Result<Vec<Memory>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, content, tags, importance, timestamp, msg_timestamp, type, source, agent_id, source_message_id
         FROM memories WHERE agent_id = ?1 ORDER BY timestamp DESC LIMIT ?2 OFFSET ?3"
    ).map_err(|e| e.to_string())?;
    
    let memory_iter = stmt.query_map(params![agent_id, limit, offset], |row| {
        let tags_str: String = row.get(2)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        
        Ok(Memory {
            id: row.get(0)?,
            content: row.get(1)?,
            tags,
            importance: row.get(3)?,
            timestamp: row.get(4)?,
            msg_timestamp: row.get(5)?,
            type_: row.get(6)?,
            source: row.get(7)?,
            agent_id: row.get(8)?,
            source_message_id: row.get(9)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut memories = Vec::new();
    for memory in memory_iter {
        memories.push(memory.map_err(|e| e.to_string())?);
    }
    
    Ok(memories)
}

#[tauri::command]
pub fn delete_memory(state: tauri::State<DbState>, id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM memories WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// --- Reminder Commands ---

#[tauri::command]
pub fn get_reminders(state: tauri::State<DbState>, agent_id: String) -> Result<Vec<Reminder>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, task, time, timestamp, agent_id, delivered, recur, recur_until FROM reminders WHERE agent_id = ?1 ORDER BY time ASC"
    ).map_err(|e| e.to_string())?;
    
    let iter = stmt.query_map(params![agent_id], |row| {
        Ok(Reminder {
            id: row.get(0)?,
            task: row.get(1)?,
            time: row.get(2)?,
            timestamp: row.get(3)?,
            agent_id: row.get(4)?,
            delivered: {
                let v: i64 = row.get(5).unwrap_or(0);
                Some(v != 0)
            },
            recur: row.get(6).ok(),
            recur_until: row.get(7).ok(),
        })
    }).map_err(|e| e.to_string())?;
    
    let mut items = Vec::new();
    for item in iter {
        items.push(item.map_err(|e| e.to_string())?);
    }
    Ok(items)
}

#[tauri::command]
pub fn delete_reminder(state: tauri::State<DbState>, id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM reminders WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
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
pub fn get_topics(state: tauri::State<DbState>, agent_id: String) -> Result<Vec<Topic>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
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
}

#[tauri::command]
pub fn update_topic_status(state: tauri::State<DbState>, id: i64, revealed: bool) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE topics SET revealed = ?1 WHERE id = ?2", params![revealed, id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_topic(state: tauri::State<DbState>, id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM topics WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
