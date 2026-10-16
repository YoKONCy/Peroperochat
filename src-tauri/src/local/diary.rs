use std::{collections::{HashMap,HashSet}, path::PathBuf, sync::Mutex};
use rusqlite::{params,Connection,OptionalExtension};
use serde::{Deserialize,Serialize};
use serde_json::json;
use triviumdb::Database;
use super::types::*;

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Entry {
    pub id:i64, pub agent_id:String, pub date:String, pub content:String, pub tags:Vec<String>,
    pub timestamp:i64, pub revision:i64, pub source_message_ids:Vec<i64>,
}

#[derive(Debug,Clone,Default,Serialize,Deserialize)]
#[serde(default,rename_all="camelCase",deny_unknown_fields)]
pub struct Query { pub keyword:String,pub from:Option<String>,pub to:Option<String>,pub limit:usize,pub offset:usize }

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Draft {
    pub content:String,
    #[serde(default)] pub date:Option<String>,
    #[serde(default)] pub tags:Vec<String>,
    #[serde(default)] pub id:Option<i64>,
    #[serde(default)] pub expected_revision:Option<i64>,
}

pub fn initialize(conn:&Connection)->ChatResult<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS memory_tdb_pending(agent_id TEXT PRIMARY KEY);")?;
    let columns={ let mut stmt=conn.prepare("PRAGMA table_info(memories)")?;
        let rows=stmt.query_map([],|r|r.get::<_,String>(1))?; rows.collect::<Result<HashSet<_>,_>>()? };
    for (name,kind) in [("revision","INTEGER NOT NULL DEFAULT 1"),("diary_date","TEXT"),("source_message_ids","TEXT NOT NULL DEFAULT '[]'")] {
        if !columns.contains(name) { conn.execute(&format!("ALTER TABLE memories ADD COLUMN {name} {kind}"),[])?; }
    }
    Ok(())
}

fn date_valid(date:&str)->bool { date.len()==10 && chrono::NaiveDate::parse_from_str(date,"%Y-%m-%d").is_ok() }
pub fn validate(draft:&Draft)->ChatResult<()> {
    if draft.content.trim().is_empty() || draft.content.len()>16_000 || draft.tags.len()>16 || draft.tags.iter().any(|t|t.len()>100)
        || draft.date.as_ref().is_some_and(|d| !date_valid(d)) || draft.id.is_some()!=draft.expected_revision.is_some()
        || draft.id.is_some_and(|id|id<=0) || draft.expected_revision.is_some_and(|r|r<1) {
        return Err(ChatError::new("INVALID_TOOL_INPUT","日记内容、日期或修订参数无效"));
    }
    Ok(())
}

pub fn dirty(conn:&Connection,agent_id:&str)->ChatResult<()> {
    if agent_id.is_empty() {return Ok(());}
    conn.execute("INSERT OR IGNORE INTO memory_tdb_pending(agent_id) VALUES(?1)",[agent_id])?; Ok(())
}

pub fn apply(conn:&Connection,agent_id:&str,draft:&Draft,source_ids:&[i64],timestamp:i64)->ChatResult<i64> {
    validate(draft)?;
    let date=draft.date.clone().unwrap_or_else(||chrono::Local::now().format("%Y-%m-%d").to_string());
    let id=if let Some(id)=draft.id {
        let previous:Option<(String,i64)> = conn.query_row("SELECT agent_id,revision FROM memories WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
        if previous!=Some((agent_id.into(),draft.expected_revision.unwrap())) { return Err(ChatError::new("STALE_DIARY","日记不存在、身份不匹配或已被修改")); }
        let (original_date,raw):(Option<String>,String)=conn.query_row("SELECT diary_date,source_message_ids FROM memories WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?)))?;
        let mut origins:Vec<i64>=serde_json::from_str(&raw).map_err(|_|storage_error("日记来源记录损坏"))?;
        for source in source_ids { if !origins.contains(source) { origins.push(*source); } }
        let date=if draft.date.is_none() { original_date.unwrap_or(date) } else { date };
        conn.execute("UPDATE memories SET content=?1,tags=?2,diary_date=?3,revision=revision+1,source_message_ids=?5 WHERE id=?4",
            params![draft.content,json!(draft.tags).to_string(),date,id,json!(origins).to_string()])?;
        id
    } else {
        conn.execute("INSERT INTO memories(content,tags,importance,timestamp,msg_timestamp,type,source,agent_id,source_message_id,diary_date,source_message_ids)
            VALUES(?1,?2,5,?3,?3,'event','diary',?4,?5,?6,?7)",
            params![draft.content,json!(draft.tags).to_string(),timestamp,agent_id,source_ids.last(),date,json!(source_ids).to_string()])?;
        conn.last_insert_rowid()
    };
    dirty(conn,agent_id)?; Ok(id)
}

pub fn delete(conn:&Connection,agent_id:&str,id:i64)->ChatResult<()> {
    let count=conn.execute("DELETE FROM memories WHERE id=?1 AND agent_id=?2",params![id,agent_id])?;
    if count==0 { return Err(ChatError::new("NOT_FOUND","日记不存在或不属于当前角色")); }
    dirty(conn,agent_id)
}

pub fn remove_sources(conn:&Connection,ids:&[i64])->ChatResult<()> {
    let mut stmt=conn.prepare("SELECT id,agent_id,source_message_id,source_message_ids FROM memories")?;
    let rows=stmt.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<i64>>(2)?,r.get::<_,String>(3)?)))?;
    let mut targets=Vec::new();
    for row in rows { let (id,agent,source,raw)=row?; let origins:Vec<i64>=serde_json::from_str(&raw).map_err(|_|ChatError::new("STORAGE_ERROR","记忆来源记录损坏，删除已取消"))?;
        if source.is_some_and(|s|ids.contains(&s)) || origins.iter().any(|s|ids.contains(s)) { targets.push((id,agent)); } }
    drop(stmt);
    for (id,agent) in targets { delete(conn,&agent,id)?; }
    Ok(())
}

fn entries(conn:&Connection,agent_id:&str)->ChatResult<Vec<Entry>> {
    let mut stmt=conn.prepare("SELECT id,agent_id,diary_date,content,tags,timestamp,revision,source_message_ids,source_message_id FROM memories WHERE agent_id=?1")?;
    let rows=stmt.query_map([agent_id],|r| {
        let timestamp:i64=r.get(5)?; let date:Option<String>=r.get(2)?;
        let tags:String=r.get(4)?; let origins:String=r.get(7)?; let source:Option<i64>=r.get(8)?;
        let mut source_message_ids:Vec<i64>=serde_json::from_str(&origins).map_err(|e|rusqlite::Error::FromSqlConversionFailure(7,rusqlite::types::Type::Text,Box::new(e)))?;
        if let Some(source)=source {if !source_message_ids.contains(&source) {source_message_ids.push(source);}}
        Ok(Entry { id:r.get(0)?,agent_id:r.get(1)?,date:date.unwrap_or_else(||chrono::DateTime::from_timestamp_millis(timestamp)
            .map(|d|d.with_timezone(&chrono::Local).format("%Y-%m-%d").to_string()).unwrap_or_else(||"1970-01-01".into())),
            content:r.get(3)?,tags:serde_json::from_str(&tags).unwrap_or_default(),timestamp,revision:r.get(6)?,
            source_message_ids })
    })?;
    Ok(rows.collect::<Result<Vec<_>,_>>()?)
}

fn storage_error(error:impl std::fmt::Display)->ChatError {
    eprintln!("日记存储操作失败：{error}"); ChatError::new("DIARY_STORAGE_ERROR","日记存储暂时不可用，待提交记录已保留")
}

pub struct DiaryStore { root:PathBuf, stores:Mutex<HashMap<String,Database<f32>>>, temporary:bool }
impl DiaryStore {
    pub fn new(root:PathBuf)->Self { Self { root,stores:Mutex::new(HashMap::new()),temporary:false } }
    #[cfg(test)] pub fn temporary()->Self { let mut store=Self::new(std::env::temp_dir().join(format!("ppc-diary-{}",uuid::Uuid::new_v4()))); store.temporary=true; store }

    // SQLite 保留与消息事务一致的恢复日志；TDB 负责实际查询，失败的同步不会清除日志。
    pub fn query(&self,conn:&Connection,agent_id:&str,query:&Query)->ChatResult<Vec<Entry>> {
        if agent_id.is_empty() || agent_id.len()>128 || query.keyword.len()>1000 || query.from.as_ref().is_some_and(|s|!date_valid(s)) || query.to.as_ref().is_some_and(|s|!date_valid(s))
            || query.offset>100_000 || query.limit>10_000 || matches!((&query.from,&query.to),(Some(from),Some(to)) if from>to) { return Err(ChatError::new("INVALID_INPUT","日记查询参数无效")); }
        let mut stores=self.stores.lock().map_err(|_|storage_error("日记锁不可用"))?;
        let first=!stores.contains_key(agent_id);
        if first {
            // 固定长度摘要既避免目录穿越，也避免长角色 ID 超出文件名长度限制。
            use sha2::Digest;
            let name=format!("{:x}",sha2::Sha256::digest(agent_id.as_bytes()));
            let directory=self.root.join(name);
            std::fs::create_dir_all(&directory).map_err(storage_error)?;
            let config=triviumdb::database::Config { dim:1,auto_build_quiver:false,load_text_index:false,
                memory_limit:64*1024*1024,payload_cache_bytes:4*1024*1024,..Default::default() };
            let path=directory.join("memory.tdb");
            stores.insert(agent_id.into(),Database::open_with_config(path.to_str().ok_or_else(||storage_error("路径无效"))?,config).map_err(storage_error)?);
        }
        let store=stores.get_mut(agent_id).unwrap();
        let pending:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM memory_tdb_pending WHERE agent_id=?1)",[agent_id],|r|r.get(0))?;
        if first || pending {
            // 首次重建也先登记恢复任务，避免失败后缓存中的半成品被当作已同步数据读取。
            dirty(conn,agent_id)?;
            let records=entries(conn,agent_id)?;
            let wanted:HashSet<_>=records.iter().map(|e|e.id as u64).collect();
            for id in store.all_node_ids() { if !wanted.contains(&id) { store.delete(id).map_err(storage_error)?; } }
            for entry in records {
                let id=entry.id as u64;
                // TDB 当前要求固定维度；此槽位仅占位，不生成 Embedding，也不参与检索。
                store.upsert_with_id(id,&[0.0],json!(entry)).map_err(storage_error)?;
            }
            store.flush().map_err(storage_error)?;
            conn.execute("DELETE FROM memory_tdb_pending WHERE agent_id=?1",[agent_id])?;
        }
        let ids=store.all_node_ids();
        let keyword=query.keyword.trim().to_lowercase();
        let mut result=Vec::new();
        for id in ids { if let Some(payload)=store.get_payload(id) {
            let entry:Entry=serde_json::from_value(payload).map_err(storage_error)?;
            // 普通日记直接检查 TDB 的当前正文；不依赖修订后可能残留旧词的追加式索引。
            if entry.agent_id==agent_id && (keyword.is_empty() || entry.content.to_lowercase().contains(&keyword) || entry.tags.iter().any(|tag|tag.to_lowercase().contains(&keyword)))
                && query.from.as_ref().is_none_or(|d|entry.date>=*d) && query.to.as_ref().is_none_or(|d|entry.date<=*d) { result.push(entry); }
        } }
        result.sort_by(|a,b|b.date.cmp(&a.date).then(b.timestamp.cmp(&a.timestamp)).then(b.id.cmp(&a.id)));
        Ok(result.into_iter().skip(query.offset).take(if query.limit==0 { 10 } else { query.limit }).collect())
    }

    pub fn synchronize(&self,conn:&Connection)->ChatResult<()> {
        let agents={ let mut stmt=conn.prepare("SELECT agent_id FROM memory_tdb_pending")?;
            let rows=stmt.query_map([],|r|r.get::<_,String>(0))?; rows.collect::<Result<Vec<_>,_>>()? };
        for agent in agents { self.query(conn,&agent,&Query { limit:1,..Default::default() })?; } Ok(())
    }
}

impl Drop for DiaryStore {
    fn drop(&mut self) {
        if let Ok(stores)=self.stores.get_mut() { for store in stores.values_mut() { let _=store.close(); } stores.clear(); }
        if self.temporary { let _=std::fs::remove_dir_all(&self.root); }
    }
}
