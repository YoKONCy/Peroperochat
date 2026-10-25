use rusqlite::{params,Connection,OptionalExtension,TransactionBehavior};
use serde_json::Value;
use tauri::{AppHandle,Manager};
use sha2::{Digest,Sha256};
use crate::{mode::{Coordinator,Token,Target},local::types::*};

pub(super) fn connection(app:&AppHandle,token:&Token)->ChatResult<Connection> {
    let root=app.path().app_data_dir().map_err(|_|ChatError::new("STORAGE_ERROR","远程缓存目录不可用"))?.join("data");
    connection_at(&root,token)
}
pub(super) fn connection_at(root:&std::path::Path,token:&Token)->ChatResult<Connection> {
    let Target::Remote {profile_id}= &token.target else {return Err(ChatError::new("MODE_INACTIVE","当前不是远程模式"));};
    if uuid::Uuid::parse_str(profile_id).is_err() {return Err(ChatError::new("INVALID_PROFILE","远程分区身份无效"));}
    let dir=root.join("remote").join(profile_id);
    std::fs::create_dir_all(&dir).map_err(|_|ChatError::new("STORAGE_ERROR","无法创建远程缓存目录"))?;
    let mut conn=Connection::open(dir.join("client.sqlite"))?;conn.busy_timeout(std::time::Duration::from_secs(5))?;
    // 界面与原生接收进程共享分区，迁移检查与变更必须在同一写事务中串行执行。
    let tx=conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute_batch("CREATE TABLE IF NOT EXISTS projections(key TEXT PRIMARY KEY,value TEXT NOT NULL,touched INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS requests(id TEXT PRIMARY KEY,fingerprint TEXT NOT NULL,state TEXT NOT NULL,created INTEGER NOT NULL,result TEXT);
        CREATE TABLE IF NOT EXISTS projection_reads(key TEXT PRIMARY KEY,ticket INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS cache_clock(id INTEGER PRIMARY KEY,value INTEGER NOT NULL);
        INSERT OR IGNORE INTO cache_clock VALUES(1,0);
        CREATE TABLE IF NOT EXISTS preferences(key TEXT PRIMARY KEY,value TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS resource_cache(key TEXT PRIMARY KEY,mime TEXT NOT NULL,content_range TEXT,bytes BLOB NOT NULL,touched INTEGER NOT NULL);")?;
    for column in ["context","owner"] {
        let exists:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM pragma_table_info('requests') WHERE name=?1)",[column],|r|r.get(0))?;
        if !exists {tx.execute_batch(&format!("ALTER TABLE requests ADD COLUMN {column} TEXT"))?;}
    }
    tx.commit()?;
    Ok(conn)
}
pub fn begin_read(app:&AppHandle,token:&Token,key:&str)->ChatResult<i64> {
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(token),false)?;let mut conn=connection(app,token)?;
    let tx=conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute("UPDATE cache_clock SET value=value+1 WHERE id=1",[])?;
    let ticket=tx.query_row("SELECT value FROM cache_clock WHERE id=1",[],|r|r.get(0))?;
    tx.execute("INSERT INTO projection_reads VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET ticket=excluded.ticket",params![key,ticket])?;
    if let Some(head)=head_key(key) {tx.execute("INSERT INTO projection_reads VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET ticket=excluded.ticket",params![head,ticket])?;}
    tx.execute("DELETE FROM projection_reads WHERE key NOT IN (SELECT key FROM projection_reads ORDER BY ticket DESC LIMIT 128)",[])?;
    tx.commit()?;Ok(ticket)
}
pub fn save(app:&AppHandle,token:&Token,key:&str,ticket:i64,value:&Value)->ChatResult<bool> {
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(token),false)?;let mut conn=connection(app,token)?;
    save_projection(&mut conn,key,ticket,value)
}
fn save_projection(conn:&mut Connection,key:&str,ticket:i64,value:&Value)->ChatResult<bool> {
    let tx=conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let current:Option<i64>=tx.query_row("SELECT ticket FROM projection_reads WHERE key=?1",[key],|r|r.get(0)).optional()?;
    if current!=Some(ticket) {return Ok(false);}
    if let Some(head)=head_key(key) {
        let newest:Option<i64>=tx.query_row("SELECT ticket FROM projection_reads WHERE key=?1",[head],|row|row.get(0)).optional()?;
        if newest!=Some(ticket) {return Ok(false);}
    }
    let raw=value.to_string();
    // 离线缓存额度不限制在线阅读；大型单条回复照常返回，不保留过期副本。
    if raw.len()>super::transport::MAX_BODY {
        tx.execute("DELETE FROM projections WHERE key=?1",[key])?;
        if let Some(head)=head_key(key) {
            let keys=tx.prepare("SELECT key FROM projections")?.query_map([],|row|row.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()?;
            for saved_key in keys {if head_key(&saved_key).as_ref()==Some(&head) {tx.execute("DELETE FROM projections WHERE key=?1",[saved_key])?;}}
        }
        tx.commit()?;return Ok(true);
    }
    if let (Some(thread),Some(total))=(head_key(key),value["totalMessages"].as_u64()) {
        let rows=tx.prepare("SELECT key,value FROM projections")?.query_map([],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?)))?.collect::<Result<Vec<_>,_>>()?;
        let shrunk=rows.iter().any(|(saved_key,raw)|head_key(saved_key).as_ref()==Some(&thread) && serde_json::from_str::<Value>(raw).ok().and_then(|value|value["totalMessages"].as_u64()).is_some_and(|old|total<old));
        if shrunk {
            let id=thread.trim_start_matches("conversation-head:");
            for (saved_key,_) in rows {if serde_json::from_str::<Value>(&saved_key).ok().is_some_and(|query|query["kind"]=="conversation" && query["id"]==id) {tx.execute("DELETE FROM projections WHERE key=?1",[saved_key])?;}}
            let keys=tx.prepare("SELECT key FROM projection_reads")?.query_map([],|row|row.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()?;
            for saved_key in keys {if saved_key!=key && serde_json::from_str::<Value>(&saved_key).ok().is_some_and(|query|query["kind"]=="conversation" && query["id"]==id) {tx.execute("DELETE FROM projection_reads WHERE key=?1",[saved_key])?;}}
        }
    }
    let touched:i64=tx.query_row("SELECT MAX(?1,COALESCE(MAX(touched),0)+1) FROM projections",[chrono::Utc::now().timestamp_millis()],|row|row.get(0))?;
    tx.execute("INSERT INTO projections VALUES(?1,?2,?3) ON CONFLICT(key) DO UPDATE SET value=excluded.value,touched=excluded.touched",params![key,raw,touched])?;
    tx.execute("DELETE FROM projections WHERE key NOT IN (SELECT key FROM projections ORDER BY touched DESC LIMIT 256)",[])?;
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
    load_projection(&conn,key)
}
fn head_key(key:&str)->Option<String> {
    let query:Value=serde_json::from_str(key).ok()?;
    (query["kind"]=="conversation" && query["beforeCursor"].is_null()).then(||query["id"].as_str().map(|id|format!("conversation-head:{id}"))).flatten()
}
fn load_projection(conn:&Connection,key:&str)->ChatResult<Option<Value>> {
    let mut raw:Option<String>=conn.query_row("SELECT value FROM projections WHERE key=?1",[key],|row|row.get(0)).optional()?;
    if let Some(head)=head_key(key) {
        // 首屏页长可能不同，离线始终取同一会话最近确认的首屏，不退回更旧的小页。
        let rows=conn.prepare("SELECT key,value FROM projections ORDER BY touched DESC")?.query_map([],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?)))?.collect::<Result<Vec<_>,_>>()?;
        if let Some((_,value))=rows.into_iter().find(|(saved_key,_)|head_key(saved_key).as_ref()==Some(&head)) {raw=Some(value);}
    }
    raw.map(|raw|serde_json::from_str(&raw).map_err(|_|ChatError::new("STORAGE_ERROR","远程投影缓存损坏"))).transpose()
}
pub(super) fn offline_error(error:&ChatError)->bool {
    matches!(error.code.as_str(),"REMOTE_NETWORK"|"REMOTE_HTTP"|"REMOTE_RATE_LIMIT")
}
pub(super) struct Blob {pub mime:String,pub content_range:Option<String>,pub bytes:Vec<u8>}
pub(super) fn load_blob(app:&AppHandle,token:&Token,key:&str)->ChatResult<Option<Blob>> {
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(token),false)?;let conn=connection(app,token)?;
    let result=conn.query_row("SELECT mime,content_range,bytes FROM resource_cache WHERE key=?1",[key],|row|Ok(Blob {mime:row.get(0)?,content_range:row.get(1)?,bytes:row.get(2)?})).optional()?;
    if result.is_some() {conn.execute("UPDATE resource_cache SET touched=?2 WHERE key=?1",params![key,chrono::Utc::now().timestamp_millis()])?;}
    Ok(result)
}
pub(super) fn save_blob(app:&AppHandle,token:&Token,key:&str,ticket:i64,blob:&Blob)->ChatResult<bool> {
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(token),false)?;let mut conn=connection(app,token)?;
    save_blob_at(&mut conn,key,ticket,blob,64*1024*1024)
}
fn save_blob_at(conn:&mut Connection,key:&str,ticket:i64,blob:&Blob,budget:usize)->ChatResult<bool> {
    let tx=conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let current:Option<i64>=tx.query_row("SELECT ticket FROM projection_reads WHERE key=?1",[format!("blob:{key}")],|row|row.get(0)).optional()?;
    if current!=Some(ticket) {return Ok(false);}
    if blob.bytes.len()>super::transport::MAX_BODY || blob.bytes.len()>budget {return Ok(true);}
    tx.execute("INSERT INTO resource_cache VALUES(?1,?2,?3,?4,?5) ON CONFLICT(key) DO UPDATE SET mime=excluded.mime,content_range=excluded.content_range,bytes=excluded.bytes,touched=excluded.touched",params![key,blob.mime,blob.content_range,blob.bytes,chrono::Utc::now().timestamp_millis()])?;
    tx.execute("DELETE FROM resource_cache WHERE key NOT IN (SELECT key FROM resource_cache ORDER BY touched DESC,rowid DESC LIMIT 256)",[])?;
    loop {
        let size:i64=tx.query_row("SELECT COALESCE(SUM(length(bytes)),0) FROM resource_cache",[],|row|row.get(0))?;
        if size<=budget as i64 {break;}
        tx.execute("DELETE FROM resource_cache WHERE key=(SELECT key FROM resource_cache ORDER BY touched,rowid LIMIT 1)",[])?;
    }
    tx.commit()?;Ok(true)
}
pub(super) fn forget_query(app:&AppHandle,token:&Token,key:&str,ticket:i64,thread:Option<&str>)->ChatResult<()> {
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(token),false)?;let mut conn=connection(app,token)?;
    let tx=conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if current_read(&tx,key,ticket)? {forget_projection(&tx,key,thread)?;}
    tx.commit()?;Ok(())
}
fn current_read(conn:&Connection,key:&str,ticket:i64)->ChatResult<bool> {
    let current:Option<i64>=conn.query_row("SELECT ticket FROM projection_reads WHERE key=?1",[key],|row|row.get(0)).optional()?;
    Ok(current==Some(ticket))
}
fn current_resource_read(conn:&Connection,resource:&Value,key:&str,ticket:i64)->ChatResult<bool> {
    if !current_read(conn,key,ticket)? {return Ok(false);}
    let info=format!("resource-info:{resource}");
    let rows=conn.prepare("SELECT key,ticket FROM projection_reads WHERE ticket>?1")?.query_map([ticket],|row|Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?)))?.collect::<Result<Vec<_>,_>>()?;
    Ok(!rows.into_iter().any(|(key,_)|key==info || key.strip_prefix("blob:").and_then(|raw|serde_json::from_str::<Value>(raw).ok()).is_some_and(|value|value["resource"]==*resource)))
}
fn scoped_query(key:&str)->Option<Value> {serde_json::from_str(key.strip_prefix("resource-info:").or_else(||key.strip_prefix("resource-resolve:")).unwrap_or(key)).ok()}
fn forget_projection(conn:&Connection,key:&str,thread:Option<&str>)->ChatResult<()> {
    conn.execute("DELETE FROM projections WHERE key=?1",[key])?;
    conn.execute("DELETE FROM projection_reads WHERE key=?1",[key])?;
    if let Some(thread)=thread {
        let rows=conn.prepare("SELECT key,value FROM projections")?.query_map([],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?)))?.collect::<Result<Vec<_>,_>>()?;
        for (key,raw) in rows {
            let Some(query)=scoped_query(&key) else {continue};
            if query["id"]==thread || query["threadId"]==thread {
                conn.execute("DELETE FROM projections WHERE key=?1",[key])?;
            } else if query["kind"]=="threads" {
                let Ok(mut value)=serde_json::from_str::<Value>(&raw) else {continue};
                if let Some(items)=value.get_mut("items").and_then(Value::as_array_mut) {
                    let old=items.len();items.retain(|item|item["id"]!=thread);let removed=old-items.len();
                    if removed>0 {
                        if let Some(total)=value["total"].as_u64() {value["total"]=Value::from(total.saturating_sub(removed as u64));}
                        conn.execute("UPDATE projections SET value=?2 WHERE key=?1",params![key,value.to_string()])?;
                    }
                }
            }
        }
        conn.execute("DELETE FROM resource_cache WHERE json_extract(key,'$.resource.threadId')=?1",[thread])?;
        let keys=conn.prepare("SELECT key FROM projection_reads")?.query_map([],|row|row.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()?;
        for key in keys {
            if scoped_query(&key).is_some_and(|query|query["id"]==thread || query["threadId"]==thread || query["kind"]=="threads") || key==format!("conversation-head:{thread}") {
                conn.execute("DELETE FROM projection_reads WHERE key=?1",[key])?;
            }
        }
    }
    Ok(())
}
fn confirmed_action(conn:&Connection,context:&Value,data:&Value)->ChatResult<()> {
    if context["kind"]!="action" {return Ok(());}
    let input=&context["input"];
    if input["kind"]=="delete_thread" {
        if let Some(id)=input["id"].as_str() {forget_projection(conn,"",Some(id))?;}
    } else if matches!(input["kind"].as_str(),Some("rewind"|"edit_message")) {
        let id=input["threadId"].as_str().or_else(||input["id"].as_str()).unwrap_or("");
        let keys=conn.prepare("SELECT key FROM projections")?.query_map([],|row|row.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()?;
        for key in keys {
            let Ok(query)=serde_json::from_str::<Value>(&key) else {continue};
            if (query["id"]==id && matches!(query["kind"].as_str(),Some("conversation"|"queue"|"queue_live"|"thread"))) || query["threadId"]==id {
                conn.execute("DELETE FROM projections WHERE key=?1",[key])?;
            }
        }
    } else if input["kind"]=="rename_thread" {
        let id=input["id"].as_str().unwrap_or("");let title=data["thread"]["title"].as_str().or_else(||input["title"].as_str()).unwrap_or("");
        let rows=conn.prepare("SELECT key,value FROM projections")?.query_map([],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?)))?.collect::<Result<Vec<_>,_>>()?;
        for (key,raw) in rows {
            let Ok(query)=serde_json::from_str::<Value>(&key) else {continue};let Ok(mut value)=serde_json::from_str::<Value>(&raw) else {continue};
            if query["kind"]=="thread" && query["id"]==id {value["thread"]["title"]=Value::from(title);}
            else if query["kind"]=="threads" {if let Some(items)=value["items"].as_array_mut() {for item in items {if item["id"]==id {item["title"]=Value::from(title);}}}}
            else {continue;}
            conn.execute("UPDATE projections SET value=?2 WHERE key=?1",params![key,value.to_string()])?;
        }
    }
    Ok(())
}
pub(super) fn forget_resource(app:&AppHandle,token:&Token,resource:&Value,read_key:&str,ticket:i64)->ChatResult<()> {
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(token),false)?;let mut conn=connection(app,token)?;
    let tx=conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    // 晚到的旧 404 不能清掉后发请求已经确认的新头像或文件。
    if !current_resource_read(&tx,resource,read_key,ticket)? {return Ok(());}
    let keys=tx.prepare("SELECT key FROM resource_cache")?.query_map([],|row|row.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()?;
    for key in keys {
        if serde_json::from_str::<Value>(&key).ok().is_some_and(|value|value["resource"]==*resource) {tx.execute("DELETE FROM resource_cache WHERE key=?1",[key])?;}
    }
    tx.execute("DELETE FROM projections WHERE key=?1",[format!("resource-info:{resource}")])?;
    let keys=tx.prepare("SELECT key FROM projection_reads")?.query_map([],|row|row.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()?;
    for key in keys {
        if key==format!("resource-info:{resource}") || key.strip_prefix("blob:").and_then(|key|serde_json::from_str::<Value>(key).ok()).is_some_and(|value|value["resource"]==*resource) {tx.execute("DELETE FROM projection_reads WHERE key=?1",[key])?;}
    }
    tx.commit()?;Ok(())
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
    // 写操作只撤销在途读票据。离线副本不作为操作依据，也不因一次发送丢掉其他会话。
    conn.execute_batch("DELETE FROM projection_reads;")?;Ok(())
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
        if let Ok(data)=&result {
            let raw:Option<String>=tx.query_row("SELECT context FROM requests WHERE id=?1",[id],|row|row.get(0))?;
            if let Some(context)=raw.and_then(|raw|serde_json::from_str::<Value>(&raw).ok()) {confirmed_action(&tx,&context,data)?;}
        }
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
    fn offline_store()->(std::path::PathBuf,Token,Connection) {
        let root=std::env::temp_dir().join(format!("ppc-offline-{}",uuid::Uuid::new_v4()));
        let token=Token {installation_id:"测试安装".into(),epoch:1,target:Target::Remote {profile_id:uuid::Uuid::new_v4().to_string()}};
        let conn=connection_at(&root,&token).unwrap();(root,token,conn)
    }
    fn ticket(conn:&Connection,key:&str,value:i64) {
        conn.execute("INSERT OR REPLACE INTO projection_reads VALUES(?1,?2)",params![key,value]).unwrap();
        if let Some(head)=head_key(key) {conn.execute("INSERT OR REPLACE INTO projection_reads VALUES(?1,?2)",params![head,value]).unwrap();}
    }
    #[test]
    fn writes_preserve_offline_reading_but_reject_older_network_results() {
        let (root,token,mut conn)=offline_store();let key=r#"{"kind":"agents"}"#;
        ticket(&conn,key,1);save_projection(&mut conn,key,1,&serde_json::json!([{"id":"pero"}])).unwrap();
        invalidate(&conn).unwrap();
        assert!(load_projection(&conn,key).unwrap().is_some());
        assert!(!save_projection(&mut conn,key,1,&serde_json::json!([])).unwrap());
        ticket(&conn,key,2);assert!(save_projection(&mut conn,key,2,&serde_json::json!([{"id":"nana"}])).unwrap());
        drop(conn);let conn=connection_at(&root,&token).unwrap();assert_eq!(load_projection(&conn,key).unwrap().unwrap()[0]["id"],"nana");
    }
    #[test]
    fn latest_head_wins_across_page_sizes_and_rewind_discards_old_history() {
        let (_,_,mut conn)=offline_store();
        let small=r#"{"kind":"conversation","id":"a","beforeCursor":null,"pageSize":24}"#;
        let large=r#"{"kind":"conversation","id":"a","beforeCursor":null,"pageSize":60}"#;
        let older=r#"{"kind":"conversation","id":"a","beforeCursor":"20","pageSize":24}"#;
        ticket(&conn,small,1);save_projection(&mut conn,small,1,&serde_json::json!({"totalMessages":30,"text":"旧页"})).unwrap();
        ticket(&conn,older,2);save_projection(&mut conn,older,2,&serde_json::json!({"totalMessages":30})).unwrap();
        ticket(&conn,large,3);save_projection(&mut conn,large,3,&serde_json::json!({"totalMessages":31,"text":"最新回复"})).unwrap();
        assert_eq!(load_projection(&conn,small).unwrap().unwrap()["text"],"最新回复");
        assert!(!save_projection(&mut conn,small,1,&serde_json::json!({"totalMessages":30})).unwrap());
        ticket(&conn,small,4);save_projection(&mut conn,small,4,&serde_json::json!({"totalMessages":10})).unwrap();
        assert!(load_projection(&conn,older).unwrap().is_none());
        assert!(!save_projection(&mut conn,older,2,&serde_json::json!({"totalMessages":30})).unwrap());
        assert_eq!(load_projection(&conn,large).unwrap().unwrap()["totalMessages"],10);
    }
    #[test]
    fn confirmed_deletion_only_removes_its_thread_and_list_entry() {
        let (_,_,mut conn)=offline_store();
        let first=r#"{"kind":"thread","id":"a"}"#;let second=r#"{"kind":"thread","id":"b"}"#;let list=r#"{"kind":"threads","agentId":"pero","page":1,"pageSize":30}"#;
        for (n,key,value) in [(1,first,serde_json::json!({"thread":{"id":"a"}})),(2,second,serde_json::json!({"thread":{"id":"b"}})),(3,list,serde_json::json!({"items":[{"id":"a"},{"id":"b"}],"total":2}))] {ticket(&conn,key,n);save_projection(&mut conn,key,n,&value).unwrap();}
        confirmed_action(&conn,&serde_json::json!({"kind":"action","input":{"kind":"delete_thread","id":"a"}}),&Value::Null).unwrap();
        assert!(load_projection(&conn,first).unwrap().is_none());assert!(load_projection(&conn,second).unwrap().is_some());
        let value=load_projection(&conn,list).unwrap().unwrap();assert_eq!(value["items"],serde_json::json!([{"id":"b"}]));assert_eq!(value["total"],1);
    }
    #[test]
    fn binary_cache_is_bounded_replaceable_and_isolated_by_connection() {
        let (root,token,mut conn)=offline_store();
        let blob=Blob {mime:"image/png".into(),content_range:None,bytes:vec![7;6]};
        for (n,key) in [(1,"a"),(2,"b")] {ticket(&conn,&format!("blob:{key}"),n);assert!(save_blob_at(&mut conn,key,n,&blob,10).unwrap());}
        let size:i64=conn.query_row("SELECT SUM(length(bytes)) FROM resource_cache",[],|row|row.get(0)).unwrap();assert!(size<=10);
        assert!(!save_blob_at(&mut conn,"b",1,&blob,10).unwrap());
        ticket(&conn,"blob:b",3);let new=Blob {mime:"image/png".into(),content_range:None,bytes:vec![9;4]};save_blob_at(&mut conn,"b",3,&new,10).unwrap();
        drop(conn);let conn=connection_at(&root,&token).unwrap();let bytes:Vec<u8>=conn.query_row("SELECT bytes FROM resource_cache WHERE key='b'",[],|row|row.get(0)).unwrap();assert_eq!(bytes,vec![9;4]);
        let mut other=token;other.target=Target::Remote {profile_id:uuid::Uuid::new_v4().to_string()};
        let conn=connection_at(&root,&other).unwrap();let count:i64=conn.query_row("SELECT COUNT(*) FROM resource_cache",[],|row|row.get(0)).unwrap();assert_eq!(count,0);
    }
    #[test]
    fn auth_identity_and_deleted_resources_never_fall_back_to_cached_success() {
        for code in ["REMOTE_AUTH_REQUIRED","REMOTE_FORBIDDEN","REMOTE_NOT_FOUND","REMOTE_IDENTITY_CHANGED","REMOTE_PROTOCOL","MODE_INACTIVE"] {assert!(!offline_error(&ChatError::new(code,"测试")));}
        assert!(offline_error(&ChatError::new("REMOTE_NETWORK","测试")));
    }
    #[test]
    fn late_file_failure_cannot_retire_newer_file_metadata_or_bytes() {
        let (_,_,conn)=offline_store();let resource=serde_json::json!({"kind":"agent_avatar","id":"pero"});
        let key=format!("blob:{}",serde_json::json!({"resource":resource,"range":null}));
        ticket(&conn,&key,1);assert!(current_resource_read(&conn,&resource,&key,1).unwrap());
        ticket(&conn,&format!("resource-info:{resource}"),2);assert!(!current_resource_read(&conn,&resource,&key,1).unwrap());
        ticket(&conn,&key,3);assert!(current_resource_read(&conn,&resource,&key,3).unwrap());
    }
    #[test]
    fn large_online_projection_does_not_fail_when_offline_cache_is_full() {
        let mut conn=Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE projections(key TEXT PRIMARY KEY,value TEXT NOT NULL,touched INTEGER NOT NULL);
            CREATE TABLE projection_reads(key TEXT PRIMARY KEY,ticket INTEGER NOT NULL);
            INSERT INTO projection_reads VALUES('conversation',2);
            INSERT INTO projections VALUES('conversation','{}',0);").unwrap();
        let value=serde_json::json!({"content":"长".repeat(super::super::transport::MAX_BODY/3+1)});
        assert!(!save_projection(&mut conn,"conversation",1,&value).unwrap());
        assert!(save_projection(&mut conn,"conversation",2,&value).unwrap());
        let count:i64=conn.query_row("SELECT COUNT(*) FROM projections",[],|row|row.get(0)).unwrap();
        assert_eq!(count,0);
        assert!(save_projection(&mut conn,"conversation",2,&serde_json::json!({"content":"短回复"})).unwrap());
    }
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
