use rusqlite::{params, Connection};
use serde_json::{json, Value};
use super::{context, identity, types::*};

pub fn blocks(text: &str, tag: &str) -> Vec<Value> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut remaining = text;
    let mut values = Vec::new();
    while let Some(start) = remaining.find(&open) {
        let tail = &remaining[start+open.len()..];
        let Some(end) = tail.find(&close) else { break; };
        if end <= 16_000 {
            if let Ok(value) = serde_json::from_str(tail[..end].trim()) { values.push(value); }
        }
        remaining = &tail[end+close.len()..];
    }
    values
}

pub fn persist_database(conn: &Connection, agent_id: &str, timestamp: i64, message_id: i64, text: &str, regenerate: bool) -> ChatResult<()> {
    if regenerate { conn.execute("DELETE FROM memories WHERE agent_id=?1 AND (source_message_id=?2 OR (source_message_id IS NULL AND msg_timestamp=?3))",params![agent_id,message_id,timestamp])?; }
    if let Some(memory) = blocks(text,"MEMORY").first() { context::remember(conn,agent_id,timestamp,Some(message_id),memory)?; }
    for reminder in blocks(text,"REMINDER") {
        let task = reminder.get("task").and_then(Value::as_str).unwrap_or("");
        let time = reminder.get("time").and_then(Value::as_str).unwrap_or("");
        if task.is_empty() || task.len() > 2000 || chrono::NaiveDateTime::parse_from_str(time,"%Y-%m-%d %H:%M:%S").is_err() { continue; }
        conn.execute("INSERT INTO reminders(task,time,timestamp,agent_id,delivered,recur)
            SELECT ?1,?2,?3,?4,0,'none' WHERE NOT EXISTS(SELECT 1 FROM reminders WHERE agent_id=?4 AND task=?1 AND time=?2)",
            params![task,time,timestamp,agent_id])?;
    }
    for topic in blocks(text,"TOPIC") {
        let text = topic.get("topic").and_then(Value::as_str).unwrap_or("");
        let time = topic.get("time").and_then(Value::as_str).unwrap_or("");
        if text.is_empty() || text.len() > 2000 { continue; }
        conn.execute("INSERT INTO topics(topic,revealed,timestamp,agent_id,time)
            SELECT ?1,0,?2,?3,?4 WHERE NOT EXISTS(SELECT 1 FROM topics WHERE agent_id=?3 AND topic=?1 AND time=?4)",params![text,timestamp,agent_id,time])?;
    }
    persist_state_database(conn,agent_id,text)?;
    Ok(())
}

pub fn persist_state_database(conn: &Connection, agent_id: &str, text: &str) -> ChatResult<()> {
    if let Some(state) = blocks(text,"PEROCUE").last().filter(|s| s.is_object()) {
        for key in ["mood","vibe","mind"] {
            if let Some(value) = state.get(key).and_then(Value::as_str).filter(|s| s.len() <= 2000) {
                identity::set_value_in(conn,&format!("ppc.{agent_id}.{key}"),&json!(value))?;
            }
        }
    }
    let texts_key = format!("ppc.{agent_id}.waifu.texts");
    let saved = identity::snapshot_in(conn)?;
    let mut texts: Value = serde_json::from_str(saved.get(&texts_key).and_then(Value::as_str).unwrap_or("{}")).unwrap_or(json!({}));
    if !texts.is_object() { texts = json!({}); }
    for (tag,prefix) in [("IDLE_MESSAGES","idleMessages"),("BACK_MESSAGES","visibilityBack")] {
        if let Some(items) = blocks(text,tag).last().and_then(Value::as_array) {
            if let Some(map) = texts.as_object_mut() {
                if prefix == "idleMessages" { map.retain(|key,_| !key.starts_with("idleMessages_")); }
                for (index,item) in items.iter().filter_map(Value::as_str).filter(|s| s.len() <= 1000).take(20).enumerate() {
                    let key = if prefix == "visibilityBack" { prefix.into() } else { format!("{prefix}_{:02}",index+1) };
                    map.insert(key,json!(item));
                    if prefix == "visibilityBack" { break; }
                }
            }
        }
    }
    if let Some(clicks) = blocks(text,"CLICK_MESSAGES").last() {
        for (index,part) in ["head","chest","body"].iter().enumerate() {
            let values = if let Some(array) = clicks.as_array() {
                array.iter().skip(index*2).take(2).cloned().collect::<Vec<_>>()
            } else { clicks.get(part).and_then(Value::as_array).cloned().unwrap_or_default() };
            for (slot,value) in values.iter().filter_map(Value::as_str).filter(|s| s.len() <= 1000).take(20).enumerate() {
                texts[format!("click_{part}_{:02}",slot+1)] = json!(value);
            }
        }
    }
    identity::set_value_in(conn,&texts_key,&json!(texts.to_string()))
}
