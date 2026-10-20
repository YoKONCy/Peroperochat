use rusqlite::{params,Connection,OptionalExtension,TransactionBehavior};
use serde_json::Value;
use tauri::{AppHandle,Manager};
use sha2::{Digest,Sha256};
use crate::{mode::{Coordinator,Token,Target},local::types::*};

pub(super) fn connection(app:&AppHandle,token:&Token)->ChatResult<Connection> {
    let Target::Remote {profile_id}= &token.target else {return Err(ChatError::new("MODE_INACTIVE","当前不是远程模式"));};
    if uuid::Uuid::parse_str(profile_id).is_err() {return Err(ChatError::new("INVALID_PROFILE","远程分区身份无效"));}
    let dir=app.path().app_data_dir().map_err(|_|ChatError::new("STORAGE_ERROR","远程缓存目录不可用"))?.join("data/remote").join(profile_id);
    std::fs::create_dir_all(&dir).map_err(|_|ChatError::new("STORAGE_ERROR","无法创建远程缓存目录"))?;
    let conn=Connection::open(dir.join("client.sqlite"))?;conn.busy_timeout(std::time::Duration::from_secs(5))?;
    conn.execute_batch("CREATE TABLE IF NOT EXISTS projections(key TEXT PRIMARY KEY,value TEXT NOT NULL,touched INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS requests(id TEXT PRIMARY KEY,fingerprint TEXT NOT NULL,state TEXT NOT NULL,created INTEGER NOT NULL,result TEXT);
        CREATE TABLE IF NOT EXISTS projection_reads(key TEXT PRIMARY KEY,ticket INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS cache_clock(id INTEGER PRIMARY KEY,value INTEGER NOT NULL);
        INSERT OR IGNORE INTO cache_clock VALUES(1,0);
        CREATE TABLE IF NOT EXISTS preferences(key TEXT PRIMARY KEY,value TEXT NOT NULL);")?;
    for column in ["context","owner"] {
        let exists:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM pragma_table_info('requests') WHERE name=?1)",[column],|r|r.get(0))?;
        if !exists {conn.execute_batch(&format!("ALTER TABLE requests ADD COLUMN {column} TEXT"))?;}
    }
    Ok(conn)
}
pub fn begin_read(app:&AppHandle,token:&Token,key:&str)->ChatResult<i64> {
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(token),false)?;let mut conn=connection(app,token)?;
    let tx=conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute("UPDATE cache_clock SET value=value+1 WHERE id=1",[])?;
    let ticket=tx.query_row("SELECT value FROM cache_clock WHERE id=1",[],|r|r.get(0))?;
    tx.execute("INSERT INTO projection_reads VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET ticket=excluded.ticket",params![key,ticket])?;
    tx.execute("DELETE FROM projection_reads WHERE key NOT IN (SELECT key FROM projection_reads ORDER BY ticket DESC LIMIT 128)",[])?;
    tx.commit()?;Ok(ticket)
}
pub fn save(app:&AppHandle,token:&Token,key:&str,ticket:i64,value:&Value)->ChatResult<bool> {
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(token),false)?;let mut conn=connection(app,token)?;
    let tx=conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let current:Option<i64>=tx.query_row("SELECT ticket FROM projection_reads WHERE key=?1",[key],|r|r.get(0)).optional()?;
    if current!=Some(ticket) {return Ok(false);}
    let raw=value.to_string();if raw.len()>super::transport::MAX_BODY {return Err(ChatError::new("REMOTE_TOO_LARGE","投影缓存过大"));}
    tx.execute("INSERT INTO projections VALUES(?1,?2,?3) ON CONFLICT(key) DO UPDATE SET value=excluded.value,touched=excluded.touched",params![key,raw,chrono::Utc::now().timestamp_millis()])?;
    tx.execute("DELETE FROM projections WHERE key NOT IN (SELECT key FROM projections ORDER BY touched DESC LIMIT 64)",[])?;
    // 总缓存按实际字节限制；不把媒体和全部服务端数据库同步到手机。
    loop {
        let size:i64=tx.query_row("SELECT COALESCE(SUM(length(CAST(value AS BLOB))),0) FROM projections",[],|r|r.get(0))?;
        if size<=32*1024*1024 {break;}
        tx.execute("DELETE FROM projections WHERE key=(SELECT key FROM projections ORDER BY touched LIMIT 1)",[])?;
    }
    tx.commit()?;Ok(true)
}
pub fn load(app:&AppHandle,token:&Token,key:&str)->ChatResult<Option<Value>> {
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(token),false)?;let conn=connection(app,token)?;
    let raw:Option<String>=conn.query_row("SELECT value FROM projections WHERE key=?1",[key],|r|r.get(0)).optional()?;
    raw.map(|raw|serde_json::from_str(&raw).map_err(|_|ChatError::new("STORAGE_ERROR","远程投影缓存损坏"))).transpose()
}
pub fn prepare(app:&AppHandle,token:&Token,id:&str,payload:&Value)->ChatResult<()> {
    if uuid::Uuid::parse_str(id).is_err() {return Err(ChatError::new("INVALID_INPUT","远程操作身份无效"));}
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(token),false)?;let mut conn=connection(app,token)?;
    let tx=conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    record(&tx,id,payload)?;
    tx.execute("UPDATE requests SET owner=?2 WHERE id=?1",params![id,serde_json::to_string(token).map_err(|_|super::inactive())?])?;
    invalidate(&tx)?;tx.commit()?;Ok(())
}
fn record(conn:&Connection,id:&str,payload:&Value)->ChatResult<()> {
    let mut context=payload.clone();
    if let Some(object)=context.as_object_mut() {object.remove("requestId");}
    if let Some(object)=context.get_mut("input").and_then(Value::as_object_mut) {object.remove("requestId");}
    let raw=context.to_string();
    if raw.len()>256*1024 {return Err(ChatError::new("INVALID_INPUT","远程操作恢复上下文过大"));}
    let fingerprint=format!("{:x}",Sha256::digest(raw.as_bytes()));
    let old:Option<(String,String)>=conn.query_row("SELECT fingerprint,state FROM requests WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
    if let Some((hash,state))=old {
        return Err(if hash!=fingerprint {ChatError::new("REQUEST_ID_REUSED","同一操作身份不能替换内容")}
            else {ChatError::new("REQUEST_ALREADY_SENT",format!("此操作已登记（{state}），请核对服务端状态；不会重复发送"))});
    }
    let pending:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM requests WHERE fingerprint=?1 AND state IN ('sending','unknown'))",[&fingerprint],|r|r.get(0))?;
    if pending {return Err(ChatError::new("REQUEST_UNRESOLVED","相同内容仍有待核对操作，请先读取服务端状态并明确核对，不会重复发送").with_outcome("not_sent"));}
    let (count,bytes):(i64,i64)=conn.query_row("SELECT COUNT(*),COALESCE(SUM(length(CAST(context AS BLOB))),0) FROM requests WHERE state IN ('sending','unknown')",[],|r|Ok((r.get(0)?,r.get(1)?)))?;
    if count>=100 || bytes+raw.len() as i64>8*1024*1024 {return Err(ChatError::new("REMOTE_PENDING_LIMIT","待核对操作过多，请先核对服务端状态").with_outcome("not_sent"));}
    conn.execute("INSERT INTO requests(id,fingerprint,state,created,result,context) VALUES(?1,?2,'sending',?3,NULL,?4)",params![id,fingerprint,chrono::Utc::now().timestamp_millis(),raw])?;
    // 请求身份保留为小型墓碑，不删除未决记录，不因缓存清理允许旧身份重新发送。
    Ok(())
}
fn invalidate(conn:&Connection)->ChatResult<()> {
    // 写入前后同时失效投影与读票据，拒绝跨写操作晚到的旧查询。
    conn.execute_batch("DELETE FROM projections; DELETE FROM projection_reads;")?;Ok(())
}
pub fn settle(app:&AppHandle,token:&Token,id:&str,result:ChatResult<Value>)->ChatResult<Value> {
    settle_when(app,token,id,result,None)
}
pub fn confirm(app:&AppHandle,token:&Token,id:&str,result:ChatResult<Value>)->ChatResult<Value> {
    settle_when(app,token,id,result,Some("unknown"))
}
fn settle_when(app:&AppHandle,token:&Token,id:&str,result:ChatResult<Value>,expected:Option<&str>)->ChatResult<Value> {
    let result=result.map_err(|error|if error.outcome.is_some() {error} else {error.with_outcome("unknown")});
    let state=match &result {Ok(_)=>"completed",Err(error)=>error.outcome.as_deref().unwrap_or("unknown")};
    let receipt=match &result {
        Ok(data) if data.to_string().len()<=256*1024=>serde_json::json!({"data":data}),
        Ok(_)=>serde_json::json!({"resultRequiresRefresh":true}),
        Err(error)=>serde_json::json!({"error":error}),
    };
    let stored=(||->ChatResult<()> {
        let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(token),false)?;let mut conn=connection(app,token)?;
        let tx=conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current:String=tx.query_row("SELECT state FROM requests WHERE id=?1",[id],|r|r.get(0))?;
        if expected.is_some_and(|state|state!=current) {return Err(ChatError::new("REQUEST_STATE_CHANGED","操作已由另一次核对处理，请刷新记录"));}
        let raw:Option<String>=tx.query_row("SELECT result FROM requests WHERE id=?1",[id],|r|r.get(0))?;
        let mut saved=raw.and_then(|raw|serde_json::from_str::<Value>(&raw).ok()).unwrap_or_else(||serde_json::json!({}));
        saved["receipt"]=receipt;
        tx.execute("UPDATE requests SET state=?2,result=?3 WHERE id=?1",params![id,state,saved.to_string()])?;
        // 已终结操作只保留最近的详细回执；墓碑身份和指纹长期保留，未决上下文不清理。
        tx.execute("UPDATE requests SET context=NULL,result=NULL,owner=NULL WHERE state NOT IN ('sending','unknown') AND id NOT IN (SELECT id FROM requests WHERE state NOT IN ('sending','unknown') ORDER BY created DESC LIMIT 64)",[])?;
        invalidate(&tx)?;tx.commit()?;Ok(())
    })();
    stored.map_err(|error|error.with_outcome("unknown"))?;result
}
pub fn pending(app:&AppHandle,token:&Token)->ChatResult<Value> {
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(token),false)?;let conn=connection(app,token)?;
    let owner=serde_json::to_string(token).map_err(|_|super::inactive())?;
    conn.execute("UPDATE requests SET state='unknown' WHERE state='sending' AND (owner IS NULL OR owner<>?1)",[owner])?;
    let mut stmt=conn.prepare("SELECT id,state,created,fingerprint,context,result FROM requests WHERE state IN ('sending','unknown') ORDER BY created DESC LIMIT 1000")?;
    let rows=stmt.query_map([],|r| {
        let context:Option<String>=r.get(4)?;let result:Option<String>=r.get(5)?;
        Ok(serde_json::json!({"requestId":r.get::<_,String>(0)?,"state":r.get::<_,String>(1)?,"createdAt":r.get::<_,i64>(2)?,"fingerprint":r.get::<_,String>(3)?,
            "context":context.and_then(|raw|serde_json::from_str::<Value>(&raw).ok()),"result":result.and_then(|raw|serde_json::from_str::<Value>(&raw).ok())}))
    })?;
    Ok(Value::Array(rows.collect::<Result<Vec<_>,_>>()?))
}
pub fn review(app:&AppHandle,token:&Token,id:&str,fingerprint:&str,note:&str)->ChatResult<()> {
    if uuid::Uuid::parse_str(id).is_err() || fingerprint.len()!=64 || note.trim().is_empty() || note.len()>2048 {return Err(ChatError::new("INVALID_INPUT","请提供已核对操作的身份、指纹和说明"));}
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(token),false)?;let conn=connection(app,token)?;
    // 只记录用户已经核对；不冒充服务端确认，也不重新发送原操作。
    let changed=conn.execute("UPDATE requests SET state='reviewed',result=json_set(COALESCE(result,'{}'),'$.reviewNote',?3) WHERE id=?1 AND fingerprint=?2 AND state='unknown'",params![id,fingerprint,note])?;
    if changed!=1 {return Err(ChatError::new("REMOTE_CONFLICT","操作仍在发送、已经核对或身份已改变，请刷新"));}Ok(())
}
pub fn request(app:&AppHandle,token:&Token,id:&str)->ChatResult<Value> {
    if uuid::Uuid::parse_str(id).is_err() {return Err(ChatError::new("INVALID_INPUT","远程操作身份无效"));}
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(token),false)?;let conn=connection(app,token)?;
    let row:Option<(String,Option<String>,Option<String>)>=conn.query_row("SELECT state,context,result FROM requests WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
    let (state,context,result)=row.ok_or_else(||ChatError::new("REMOTE_NOT_FOUND","远程操作记录不存在"))?;
    Ok(serde_json::json!({"state":state,"context":context.and_then(|raw|serde_json::from_str::<Value>(&raw).ok()),"result":result.and_then(|raw|serde_json::from_str::<Value>(&raw).ok())}))
}
pub fn principal(app:&AppHandle,token:&Token,value:Option<&str>)->ChatResult<Option<String>> {
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(token),false)?;let conn=connection(app,token)?;
    if let Some(value)=value {conn.execute("INSERT INTO preferences VALUES('principal',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[value])?;}
    Ok(conn.query_row("SELECT value FROM preferences WHERE key='principal'",[],|r|r.get(0)).optional()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn store(path:&std::path::Path)->Connection {
        let conn=Connection::open(path).unwrap();conn.execute_batch("CREATE TABLE IF NOT EXISTS requests(id TEXT PRIMARY KEY,fingerprint TEXT NOT NULL,state TEXT NOT NULL,created INTEGER NOT NULL,result TEXT,context TEXT,owner TEXT)").unwrap();conn
    }
    #[test]
    fn request_tombstones_survive_restart_and_partition_ids_do_not_share_authority() {
        let root=std::env::temp_dir().join(format!("ppc-request-{}",uuid::Uuid::new_v4()));std::fs::create_dir_all(&root).unwrap();
        let id=uuid::Uuid::new_v4().to_string();let payload=serde_json::json!({"content":"只发送一次"});
        let first=root.join("first.sqlite");let conn=store(&first);record(&conn,&id,&payload).unwrap();drop(conn);
        let conn=store(&first);assert_eq!(record(&conn,&id,&payload).unwrap_err().code,"REQUEST_ALREADY_SENT");
        assert_eq!(record(&conn,&id,&serde_json::json!({"content":"替换内容"})).unwrap_err().code,"REQUEST_ID_REUSED");
        conn.execute("UPDATE requests SET state='completed'",[]).unwrap();assert!(record(&conn,&id,&payload).is_err());
        let other=store(&root.join("second.sqlite"));assert!(record(&other,&id,&payload).is_ok());
    }
}
