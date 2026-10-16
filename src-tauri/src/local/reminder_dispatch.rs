#[cfg(any(target_os="android",test))]
use chrono::{Local,TimeZone};
use rusqlite::{Connection,OptionalExtension};
#[cfg(any(target_os="android",test))]
use rusqlite::params;
#[cfg(any(target_os="android",test))]
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use super::types::*;
#[cfg(any(target_os="android",test))]
use super::{identity,reminders};

pub fn initialize(conn:&Connection)->ChatResult<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS reminder_dispatch_clock(singleton INTEGER PRIMARY KEY CHECK(singleton=1),epoch TEXT NOT NULL,tick INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS reminder_dispatch(id INTEGER PRIMARY KEY,version INTEGER NOT NULL,registered_version INTEGER,mode TEXT,last_error TEXT,reason TEXT NOT NULL DEFAULT 'changed');
        CREATE TABLE IF NOT EXISTS reminder_delivery(id INTEGER PRIMARY KEY,token TEXT NOT NULL,lease TEXT NOT NULL,state TEXT NOT NULL,not_before INTEGER NOT NULL);")?;
    conn.execute("INSERT OR IGNORE INTO reminder_dispatch_clock VALUES(1,?1,0)",[uuid::Uuid::new_v4().to_string()])?;
    conn.execute_batch("CREATE TRIGGER IF NOT EXISTS reminder_dispatch_insert AFTER INSERT ON reminders BEGIN
        UPDATE reminder_dispatch_clock SET tick=tick+1 WHERE singleton=1;
        INSERT OR REPLACE INTO reminder_dispatch(id,version,reason) SELECT NEW.id,tick,'changed' FROM reminder_dispatch_clock WHERE singleton=1;
        END;
        CREATE TRIGGER IF NOT EXISTS reminder_dispatch_update AFTER UPDATE OF task,time,agent_id,delivered,recur,recur_until,revision ON reminders BEGIN
        UPDATE reminder_dispatch_clock SET tick=tick+1 WHERE singleton=1;
        INSERT OR REPLACE INTO reminder_dispatch(id,version,reason) SELECT NEW.id,tick,'changed' FROM reminder_dispatch_clock WHERE singleton=1;
        END;
        CREATE TRIGGER IF NOT EXISTS reminder_dispatch_delete AFTER DELETE ON reminders BEGIN
        UPDATE reminder_dispatch_clock SET tick=tick+1 WHERE singleton=1;
        INSERT OR REPLACE INTO reminder_dispatch(id,version,reason) SELECT OLD.id,tick,'cancelled' FROM reminder_dispatch_clock WHERE singleton=1;
        END;")?;
    let missing={let mut stmt=conn.prepare("SELECT id FROM reminders WHERE id NOT IN (SELECT id FROM reminder_dispatch)")?;
        let rows=stmt.query_map([],|r|r.get::<_,i64>(0))?;rows.collect::<Result<Vec<_>,_>>()?};
    for id in missing {
        conn.execute("UPDATE reminder_dispatch_clock SET tick=tick+1 WHERE singleton=1",[])?;
        conn.execute("INSERT INTO reminder_dispatch(id,version) SELECT ?1,tick FROM reminder_dispatch_clock WHERE singleton=1",[id])?;
    }
    Ok(())
}

// 调度执行仅用于安卓原生桥接及其回归测试；桌面保留共享表与状态投影。
#[cfg(any(target_os="android",test))]
fn token(conn:&Connection,id:i64)->ChatResult<Option<String>> {
    Ok(conn.query_row("SELECT epoch || ':' || version FROM reminder_dispatch,reminder_dispatch_clock WHERE id=?1 AND singleton=1",[id],|r|r.get(0)).optional()?)
}

#[cfg(any(target_os="android",test))]
#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Job {pub id:i64,pub agent_id:String,pub token:String,pub trigger_at_ms:i64,pub title:String,pub body:String,pub dismiss_previous:bool}

#[cfg(any(target_os="android",test))]
pub fn plan(conn:&Connection,now_ms:i64)->ChatResult<Value> {
    let agents=identity::agents_from(&identity::snapshot_in(conn)?)?;
    let ids={let mut stmt=conn.prepare("SELECT id,reason,registered_version=version FROM reminder_dispatch ORDER BY id")?;
        let rows=stmt.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<bool>>(2)?.unwrap_or(false))))?;
        rows.collect::<Result<Vec<_>,_>>()?};
    let mut jobs=Vec::new();let mut cancellations=Vec::new();
    for (id,reason,registered) in ids {
        let current=conn.query_row("SELECT agent_id FROM reminders WHERE id=?1",[id],|r|r.get::<_,String>(0)).optional()?;
        let mut active=false;
        if let Some(agent)=current.as_ref().filter(|agent|agents.get(*agent).is_some()) {
            let entry=reminders::entry(conn,agent,id)?;
            if !entry.delivered {
                match reminders::parse(&entry.time).and_then(|date|Local.from_local_datetime(&date).single().map(|date|date.timestamp_millis())
                    .ok_or_else(||ChatError::new("INVALID_REMINDER_TIME","提醒时间无法转换为设备时间"))) {
                    Ok(due)=> {
                        let not_before:i64=conn.query_row("SELECT not_before FROM reminder_delivery WHERE id=?1 AND token=?2 AND state IN ('claimed','failed')",
                            params![id,token(conn,id)?],|r|r.get(0)).optional()?.unwrap_or(0);
                        jobs.push(Job {id,agent_id:agent.clone(),token:token(conn,id)?.unwrap(),trigger_at_ms:due.max(now_ms+1500).max(not_before),
                            title:format!("{} 的提醒",agents[agent]["name"].as_str().unwrap_or(agent)),body:entry.task,dismiss_previous:reason!="delivered"});active=true;
                    }
                    Err(error)=> {conn.execute("UPDATE reminders SET delivery_error=?1 WHERE id=?2",params![error.message,id])?;}
                }
            }
        }
        if !active {cancellations.push(json!({"id":id,"token":token(conn,id)?,"dismiss":reason!="delivered" || current.as_ref().is_some_and(|agent|agents.get(agent).is_none()),"registered":registered}));}
    }
    Ok(json!({"jobs":jobs,"cancellations":cancellations}))
}

#[cfg(any(target_os="android",test))]
#[derive(Debug,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Registration {pub id:i64,pub token:String,pub mode:String,pub error:Option<String>}

#[cfg(any(target_os="android",test))]
pub fn registered(conn:&Connection,results:&[Registration])->ChatResult<()> {
    for item in results {
        if !["exact","inexact","cancelled","blocked","failed"].contains(&item.mode.as_str()) || item.error.as_ref().is_some_and(|error|error.len()>2000) {
            return Err(ChatError::new("INVALID_INPUT","系统调度回执无效"));
        }
        if token(conn,item.id)?.as_deref()!=Some(item.token.as_str()) {continue;}
        conn.execute("UPDATE reminder_dispatch SET registered_version=CASE WHEN ?1 IS NULL THEN version ELSE NULL END,mode=?2,last_error=?1 WHERE id=?3",
            params![item.error,item.mode,item.id])?;
    }
    Ok(())
}

pub fn decorate(conn:&Connection,entries:&mut Value)->ChatResult<()> {
    if let Some(entries)=entries.as_array_mut() {for entry in entries {
        let Some(id)=entry["id"].as_i64() else {continue};
        let state:Option<(String,Option<String>)>=conn.query_row("SELECT CASE WHEN last_error IS NOT NULL OR registered_version=version THEN COALESCE(mode,'pending') ELSE 'pending' END,last_error FROM reminder_dispatch WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
        if let Some((mode,error))=state {entry["systemMode"]=json!(mode);entry["backgroundError"]=json!(error);}
    }}
    Ok(())
}

#[cfg(any(target_os="android",test))]
pub fn claim(conn:&Connection,id:i64,expected:&str,now_ms:i64)->ChatResult<Value> {
    if token(conn,id)?.as_deref()!=Some(expected) {return Ok(Value::Null);}
    let agents=identity::agents_from(&identity::snapshot_in(conn)?)?;
    let agent:Option<String>=conn.query_row("SELECT agent_id FROM reminders WHERE id=?1 AND COALESCE(delivered,0)=0",[id],|r|r.get(0)).optional()?;
    let Some(agent)=agent.filter(|agent|agents.get(agent).is_some()) else {return Ok(Value::Null)};
    let entry=reminders::entry(conn,&agent,id)?;
    let due=reminders::parse(&entry.time)?;
    let now=Local.timestamp_millis_opt(now_ms).single().ok_or_else(||ChatError::new("INVALID_INPUT","系统时间无效"))?.naive_local();
    if due>now {return Ok(Value::Null);}
    let previous:Option<(String,i64)>=conn.query_row("SELECT state,not_before FROM reminder_delivery WHERE id=?1 AND token=?2",params![id,expected],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
    if previous.is_some_and(|(state,until)|state=="sent" || until>now_ms) {return Ok(Value::Null);}
    let lease=uuid::Uuid::new_v4().to_string();
    conn.execute("INSERT INTO reminder_delivery VALUES(?1,?2,?3,'claimed',?4) ON CONFLICT(id) DO UPDATE SET token=excluded.token,lease=excluded.lease,state='claimed',not_before=excluded.not_before",
        params![id,expected,lease,now_ms+60_000])?;
    Ok(json!({"id":id,"token":expected,"lease":lease,"title":format!("{} 的提醒",agents[&agent]["name"].as_str().unwrap_or(&agent)),"body":entry.task}))
}

#[cfg(any(target_os="android",test))]
pub fn finish(conn:&Connection,id:i64,expected:&str,lease:&str,error:Option<&str>,now_ms:i64)->ChatResult<bool> {
    let valid:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM reminder_delivery WHERE id=?1 AND token=?2 AND lease=?3 AND state='claimed')",params![id,expected,lease],|r|r.get(0))?;
    if !valid || token(conn,id)?.as_deref()!=Some(expected) {return Ok(false);}
    if let Some(error)=error {
        let error=if error.len()>2000 {"系统通知发送失败"} else {error};
        conn.execute("UPDATE reminder_delivery SET state='failed',not_before=?1 WHERE id=?2",params![now_ms+60_000,id])?;
        conn.execute("UPDATE reminders SET delivery_error=?1 WHERE id=?2",params![error,id])?;return Ok(true);
    }
    let agent:String=conn.query_row("SELECT agent_id FROM reminders WHERE id=?1",[id],|r|r.get(0))?;
    let agents=identity::agents_from(&identity::snapshot_in(conn)?)?;
    if agents.get(&agent).is_none() {return Ok(false);}
    let now=Local.timestamp_millis_opt(now_ms).single().ok_or_else(||ChatError::new("INVALID_INPUT","系统时间无效"))?.naive_local();
    if !reminders::deliver(conn,&agent,id,now,|_|Ok(()))? {return Ok(false);}
    conn.execute("UPDATE reminder_delivery SET state='sent',not_before=0 WHERE id=?1",[id])?;
    conn.execute("UPDATE reminder_dispatch SET reason='delivered' WHERE id=?1",[id])?;
    Ok(true)
}

// 原生接收器使用同一个数据库；不需要创建 Tauri、WebView 或会话服务。
#[cfg(any(target_os="android",test))]
pub fn execute(conn:&mut Connection,command:&str,payload:Value,now_ms:i64)->ChatResult<Value> {
    let tx=conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let value=match command {
        "plan"=>plan(&tx,now_ms)?,
        "registered"=> {let results:Vec<Registration>=serde_json::from_value(payload).map_err(|_|ChatError::new("INVALID_INPUT","系统调度回执格式无效"))?;registered(&tx,&results)?;json!(true)}
        "claim"|"finish"=> {
            let id=payload["id"].as_i64().filter(|id|*id>0).ok_or_else(||ChatError::new("INVALID_INPUT","提醒身份无效"))?;
            let token=payload["token"].as_str().filter(|token|token.len()<200).ok_or_else(||ChatError::new("INVALID_INPUT","调度版本无效"))?;
            if command=="claim" {claim(&tx,id,token,now_ms)?} else {
                let lease=payload["lease"].as_str().filter(|lease|lease.len()<100).ok_or_else(||ChatError::new("INVALID_INPUT","执行回执身份无效"))?;
                json!(finish(&tx,id,token,lease,payload["error"].as_str(),now_ms)?)
            }
        }
        _=>return Err(ChatError::new("INVALID_INPUT","未知的原生提醒操作")),
    };
    tx.commit()?;Ok(value)
}
