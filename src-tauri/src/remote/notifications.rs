//! 通知属于远程实例；本机只缓存最近待阅记录，明确阅读后才发送服务端回执。
use serde::Deserialize;
use serde_json::{json,Value};
use tauri::{AppHandle,Manager};
use crate::{mode::{Coordinator,Token},local::types::*};

fn table(conn:&rusqlite::Connection)->ChatResult<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS notifications(principal TEXT NOT NULL,id TEXT NOT NULL,revision INTEGER NOT NULL,value TEXT NOT NULL,touched INTEGER NOT NULL,PRIMARY KEY(principal,id))")?;Ok(())
}
fn key(value:&str)->ChatResult<()> {
    if value.is_empty() || value.len()>300 || value.chars().any(char::is_control) {return Err(ChatError::new("REMOTE_PROTOCOL","远程通知身份无效"));}Ok(())
}
// 调用方持有当前运行门，不能在这里递归获取同一 SQLite 控制事务。
pub(super) fn capture(app:&AppHandle,token:&Token,payload:&Value)->ChatResult<()> {
    if payload["action"]!="durable_notification" {return Ok(());}
    let principal=payload["principalId"].as_str().ok_or_else(super::inactive)?;
    let id=payload["notificationId"].as_str().ok_or_else(super::inactive)?;
    key(principal)?;key(id)?;
    let revision=payload["revision"].as_u64().filter(|value|*value>0 && *value<=9_007_199_254_740_991).ok_or_else(super::inactive)?;
    let text=|name:&str,limit:usize|payload[name].as_str().unwrap_or("").chars().take(limit).collect::<String>();
    let value=json!({"principalId":principal,"notificationId":id,"revision":revision,"title":text("title",256),"body":text("body",4096),"createdAt":text("createdAt",100),"level":text("level",20),"replayed":payload["replayed"]==true});
    let mut conn=super::cache::connection(app,token)?;table(&conn)?;
    let tx=conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    tx.execute("INSERT INTO notifications VALUES(?1,?2,?3,?4,?5) ON CONFLICT(principal,id) DO UPDATE SET revision=excluded.revision,value=excluded.value,touched=excluded.touched WHERE excluded.revision>notifications.revision",
        rusqlite::params![principal,id,revision as i64,value.to_string(),chrono::Utc::now().timestamp_millis()])?;
    // 缓存只保留最近 256 条；未确认记录仍由服务端保存，不伪造已读。
    tx.execute("DELETE FROM notifications WHERE rowid NOT IN (SELECT rowid FROM notifications ORDER BY touched DESC,rowid DESC LIMIT 256)",[])?;
    tx.commit()?;Ok(())
}
#[tauri::command]
pub fn remote_notifications(app:AppHandle,token:Token)->ChatResult<Value> {
    super::validate_token(&app,&token)?;
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(&token),false)?;
    let conn=super::cache::connection(&app,&token)?;table(&conn)?;
    let mut stmt=conn.prepare("SELECT value FROM notifications ORDER BY touched DESC,rowid DESC LIMIT 256")?;
    let rows=stmt.query_map([],|row|row.get::<_,String>(0))?;
    let mut items=Vec::new();
    for row in rows {items.push(serde_json::from_str::<Value>(&row?).map_err(|_|ChatError::new("STORAGE_ERROR","通知缓存损坏，请重新连接刷新"))?);}
    Ok(json!({"modeToken":token,"data":{"items":items,"limit":256}}))
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Receipt {notification_id:String,revision:u64}
#[tauri::command]
pub async fn remote_ack_notification(app:AppHandle,token:Token,principal_id:String,receipt:Receipt)->ChatResult<Value> {
    key(&principal_id)?;key(&receipt.notification_id)?;
    if receipt.revision==0 || receipt.revision>9_007_199_254_740_991 {return Err(super::inactive());}
    {
        let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(&token),false)?;
        let conn=super::cache::connection(&app,&token)?;table(&conn)?;
        let matches:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM notifications WHERE principal=?1 AND id=?2 AND revision=?3)",rusqlite::params![principal_id,receipt.notification_id,receipt.revision as i64],|row|row.get(0))?;
        if !matches {return Err(ChatError::new("REMOTE_CONFLICT","通知已更新，请重新阅读后确认"));}
    }
    let runtime=app.state::<super::Runtime>();let _input=runtime.input_lock.lock().await;
    let handle=super::gateway_handle(&app,&token)?;
    let changed=handle.principal!=principal_id;
    if changed {handle.request("gateway.select_principal",json!({"principalId":principal_id})).await?;}
    let result=handle.request("gateway.notifications.ack",json!({"principalId":principal_id,"receipts":[{"notificationId":receipt.notification_id,"revision":receipt.revision}]})).await;
    if changed && handle.request("gateway.select_principal",json!({"principalId":handle.principal})).await.is_err() {handle.stop();}
    let data=result?;
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(&token),false)?;
    let conn=super::cache::connection(&app,&token)?;
    conn.execute("DELETE FROM notifications WHERE principal=?1 AND id=?2 AND revision=?3",rusqlite::params![principal_id,receipt.notification_id,receipt.revision as i64])?;
    Ok(json!({"modeToken":token,"data":data}))
}
