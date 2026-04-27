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
    pub type_: String, // "event", "fact", etc. mapped from "type"
    pub source: String,
    pub agent_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Option<i64>,
    pub role: String,
    pub content: String,
    pub timestamp: i64,
    pub agent_id: String,
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
}

pub struct DbState {
    pub conn: Mutex<Connection>,
}

pub fn init_db(app: &AppHandle) -> SqlResult<DbState> {
    let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
    let data_dir = app_dir.join("data");
    
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir).expect("failed to create data dir");
    }
    
    let db_path = data_dir.join("peropero.db");
    
    // Migration: Check if old DB exists in root and move it
    let old_db_path = app_dir.join("peropero.db");
    if old_db_path.exists() && !db_path.exists() {
        let _ = std::fs::rename(old_db_path, &db_path);
    }
    
    let conn = Connection::open(db_path)?;
    
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

    // 清理已有的重复消息（保留每组中 id 最大的记录），再建唯一索引
    conn.execute(
        "DELETE FROM messages WHERE id NOT IN (
            SELECT MAX(id) FROM messages GROUP BY agent_id, timestamp, role
        )",
        [],
    )?;

    // 防止消息重复插入的唯一索引
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_msg_unique ON messages(agent_id, timestamp, role)",
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

    Ok(DbState {
        conn: Mutex::new(conn),
    })
}

// --- Memory Commands ---

#[tauri::command]
pub fn add_memory(state: tauri::State<DbState>, memory: Memory) -> Result<i64, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let tags_json = serde_json::to_string(&memory.tags).unwrap_or("[]".to_string());
    
    conn.execute(
        "INSERT INTO memories (content, tags, importance, timestamp, msg_timestamp, type, source, agent_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            memory.content,
            tags_json,
            memory.importance,
            memory.timestamp,
            memory.msg_timestamp,
            memory.type_,
            memory.source,
            memory.agent_id
        ],
    ).map_err(|e| e.to_string())?;
    
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn get_memories(state: tauri::State<DbState>, agent_id: String, limit: u32, offset: u32) -> Result<Vec<Memory>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, content, tags, importance, timestamp, msg_timestamp, type, source, agent_id
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
        })
    }).map_err(|e| e.to_string())?;
    
    let mut memories = Vec::new();
    for memory in memory_iter {
        memories.push(memory.map_err(|e| e.to_string())?);
    }
    
    Ok(memories)
}

#[tauri::command]
pub fn get_relevant_memories(state: tauri::State<DbState>, agent_id: String, query: String, limit: u32) -> Result<Vec<Memory>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    // Simple LIKE search for now
    let search_pattern = format!("%{}%", query);
    
    let mut stmt = conn.prepare(
        "SELECT id, content, tags, importance, timestamp, msg_timestamp, type, source, agent_id
         FROM memories 
         WHERE agent_id = ?1 AND content LIKE ?2
         ORDER BY importance DESC, timestamp DESC 
         LIMIT ?3"
    ).map_err(|e| e.to_string())?;
    
    let memory_iter = stmt.query_map(params![agent_id, search_pattern, limit], |row| {
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

#[tauri::command]
pub fn delete_memories_by_msg_timestamp(state: tauri::State<DbState>, msg_timestamp: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM memories WHERE msg_timestamp = ?1", params![msg_timestamp])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn clear_memories(state: tauri::State<DbState>, agent_id: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM memories WHERE agent_id = ?1", params![agent_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// --- Message Commands ---

#[tauri::command]
pub fn save_message(state: tauri::State<DbState>, message: ChatMessage) -> Result<i64, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    // 使用 INSERT OR IGNORE 防止重复插入（配合 idx_msg_unique 唯一索引）
    conn.execute(
        "INSERT OR IGNORE INTO messages (role, content, timestamp, agent_id) VALUES (?1, ?2, ?3, ?4)",
        params![message.role, message.content, message.timestamp, message.agent_id],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn get_messages(state: tauri::State<DbState>, agent_id: String, limit: u32, offset: u32) -> Result<Vec<ChatMessage>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    
    // Get total count first to handle offset from end if needed, 
    // but typically we want latest messages.
    // For chat UI, we usually want the last N messages, ordered by time ASC.
    // So we select top N order by time DESC, then reverse in Rust.
    
    let mut stmt = conn.prepare(
        "SELECT id, role, content, timestamp, agent_id
         FROM messages WHERE agent_id = ?1 
         ORDER BY timestamp DESC LIMIT ?2 OFFSET ?3"
    ).map_err(|e| e.to_string())?;
    
    let msg_iter = stmt.query_map(params![agent_id, limit, offset], |row| {
        Ok(ChatMessage {
            id: row.get(0)?,
            role: row.get(1)?,
            content: row.get(2)?,
            timestamp: row.get(3)?,
            agent_id: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut messages = Vec::new();
    for msg in msg_iter {
        messages.push(msg.map_err(|e| e.to_string())?);
    }
    
    // Reverse to get chronological order
    messages.reverse();
    
    Ok(messages)
}

#[tauri::command]
pub fn clear_messages(state: tauri::State<DbState>, agent_id: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM messages WHERE agent_id = ?1", params![agent_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_message_by_timestamp(state: tauri::State<DbState>, agent_id: String, timestamp: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM messages WHERE agent_id = ?1 AND timestamp = ?2", 
        params![agent_id, timestamp]
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// --- Reminder Commands ---

#[tauri::command]
pub fn add_reminder(state: tauri::State<DbState>, reminder: Reminder) -> Result<i64, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let delivered = reminder.delivered.unwrap_or(false);
    let recur = reminder.recur.unwrap_or_else(|| "none".to_string());
    let recur_until = reminder.recur_until;
    conn.execute(
        "INSERT INTO reminders (task, time, timestamp, agent_id, delivered, recur, recur_until) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![reminder.task, reminder.time, reminder.timestamp, reminder.agent_id, if delivered {1} else {0}, recur, recur_until],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

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
pub fn add_topic(state: tauri::State<DbState>, topic: Topic) -> Result<i64, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO topics (topic, revealed, timestamp, agent_id) VALUES (?1, ?2, ?3, ?4)",
        params![topic.topic, topic.revealed, topic.timestamp, topic.agent_id],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn get_topics(state: tauri::State<DbState>, agent_id: String) -> Result<Vec<Topic>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, topic, revealed, timestamp, agent_id FROM topics WHERE agent_id = ?1 ORDER BY timestamp DESC"
    ).map_err(|e| e.to_string())?;
    
    let iter = stmt.query_map(params![agent_id], |row| {
        Ok(Topic {
            id: row.get(0)?,
            topic: row.get(1)?,
            revealed: row.get(2)?,
            timestamp: row.get(3)?,
            agent_id: row.get(4)?,
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
