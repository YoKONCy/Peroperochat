use chrono::{Duration, Local, NaiveDateTime, TimeZone};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
#[cfg(not(target_os="android"))]
use tauri_plugin_notification::NotificationExt;
use super::{db_lock, identity, types::*};
use crate::db::DbState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all="camelCase")]
pub struct Entry {
    pub id:i64, pub task:String, pub time:String, pub recur:String, pub recur_until:Option<String>,
    pub delivered:bool, pub revision:i64, pub delivery_error:Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all="camelCase")]
pub struct Query { pub keyword:String, pub include_delivered:bool, pub limit:Option<usize> }

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all="camelCase")]
pub struct Change {
    pub action:String, pub id:Option<i64>, pub expected_revision:Option<i64>, pub task:Option<String>,
    pub time:Option<String>, pub recur:Option<String>, pub recur_until:Option<String>,
}

#[derive(Debug, Clone)]
pub struct Draft { pub change:Change, pub value:Option<Entry> }

pub fn initialize(conn:&Connection)->ChatResult<()> {
    let columns={let mut stmt=conn.prepare("PRAGMA table_info(reminders)")?;
        let rows=stmt.query_map([],|r|r.get::<_,String>(1))?;rows.collect::<Result<Vec<_>,_>>()?};
    for (name,kind) in [("revision","INTEGER NOT NULL DEFAULT 1"),("delivery_error","TEXT")] {
        if !columns.iter().any(|column|column==name) {conn.execute(&format!("ALTER TABLE reminders ADD COLUMN {name} {kind}"),[])?;}
    }
    conn.execute_batch("CREATE INDEX IF NOT EXISTS reminders_pending ON reminders(agent_id,delivered,time);
        CREATE TABLE IF NOT EXISTS reminder_sequence(singleton INTEGER PRIMARY KEY CHECK(singleton=1),last_id INTEGER NOT NULL);
        INSERT OR IGNORE INTO reminder_sequence(singleton,last_id) SELECT 1,COALESCE(MAX(id),0) FROM reminders;")?;
    super::reminder_dispatch::initialize(conn)?;
    Ok(())
}

fn read(row:&rusqlite::Row<'_>)->rusqlite::Result<Entry> {
    Ok(Entry {id:row.get(0)?,task:row.get(1)?,time:row.get(2)?,recur:row.get::<_,Option<String>>(3)?.unwrap_or_else(||"none".into()),
        recur_until:row.get(4)?,delivered:row.get::<_,Option<bool>>(5)?.unwrap_or(false),revision:row.get(6)?,delivery_error:row.get(7)?})
}
const COLUMNS:&str="id,task,time,recur,recur_until,delivered,revision,delivery_error";

pub fn entry(conn:&Connection,agent:&str,id:i64)->ChatResult<Entry> {
    conn.query_row(&format!("SELECT {COLUMNS} FROM reminders WHERE agent_id=?1 AND id=?2"),params![agent,id],read).optional()?
        .ok_or_else(||ChatError::new("REMINDER_NOT_FOUND","提醒不存在或不属于当前角色"))
}

pub fn query(conn:&Connection,agent:&str,input:&Query)->ChatResult<Value> {
    let limit=input.limit.unwrap_or(30);
    if !(1..=50).contains(&limit) || input.keyword.len()>500 {return Err(ChatError::new("INVALID_INPUT","提醒查询范围无效"));}
    let mut stmt=conn.prepare(&format!("SELECT {COLUMNS} FROM reminders WHERE agent_id=?1 AND (?2 OR COALESCE(delivered,0)=0)
        AND instr(lower(task),lower(?3))>0 ORDER BY COALESCE(delivered,0),time,id LIMIT ?4"))?;
    let rows=stmt.query_map(params![agent,input.include_delivered,input.keyword.trim(),limit as i64],read)?;
    let entries=rows.collect::<Result<Vec<_>,_>>()?;
    let total:i64=conn.query_row("SELECT COUNT(*) FROM reminders WHERE agent_id=?1 AND (?2 OR COALESCE(delivered,0)=0) AND instr(lower(task),lower(?3))>0",
        params![agent,input.include_delivered,input.keyword.trim()],|r|r.get(0))?;
    let mut entries=json!(entries);super::reminder_dispatch::decorate(conn,&mut entries)?;
    Ok(json!({"entries":entries,"total":total,"now":Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),"utcOffset":Local::now().format("%:z").to_string()}))
}

pub(super) fn parse(value:&str)->ChatResult<NaiveDateTime> {
    let date=NaiveDateTime::parse_from_str(value,"%Y-%m-%d %H:%M:%S").map_err(|_|ChatError::new("INVALID_REMINDER_TIME","时间须为设备本地时间 YYYY-MM-DD HH:mm:ss"))?;
    if date.format("%Y-%m-%d %H:%M:%S").to_string()!=value || Local.from_local_datetime(&date).single().is_none() {
        return Err(ChatError::new("INVALID_REMINDER_TIME","该本地时间不存在或有夏令时歧义，请换一个明确时间"));
    }
    Ok(date)
}

pub fn prepare(conn:&Connection,agent:&str,change:Change,now:NaiveDateTime)->ChatResult<Draft> {
    if !["create","update","delete"].contains(&change.action.as_str()) {return Err(ChatError::new("INVALID_INPUT","提醒操作无效"));}
    let mut value=if change.action=="create" {
        if change.id.is_some() || change.expected_revision.is_some() {return Err(ChatError::new("INVALID_INPUT","新提醒不应提供身份或修订号"));}
        Entry {id:0,task:String::new(),time:String::new(),recur:"none".into(),recur_until:None,delivered:false,revision:1,delivery_error:None}
    } else {
        let current=entry(conn,agent,change.id.ok_or_else(||ChatError::new("INVALID_INPUT","请提供提醒身份"))?)?;
        if change.expected_revision!=Some(current.revision) {return Err(ChatError::new("STALE_REMINDER","提醒已改变，请重新查询后操作"));}
        current
    };
    if change.action=="delete" {return Ok(Draft {change,value:None});}
    if let Some(task)=&change.task {value.task=task.trim().into();}
    if let Some(time)=&change.time {value.time=time.trim().into();}
    if let Some(recur)=&change.recur {value.recur=recur.clone();}
    if let Some(until)=&change.recur_until {value.recur_until=if until.trim().is_empty() {None} else {Some(until.trim().into())};}
    if value.task.is_empty() || value.task.len()>2000 || !["none","daily","weekly"].contains(&value.recur.as_str()) {
        return Err(ChatError::new("INVALID_INPUT","提醒内容为空、过长或重复规则无效"));
    }
    let time=parse(&value.time)?;
    if time<=now {return Err(ChatError::new("REMINDER_IN_PAST","提醒时间已过，请选择未来时间"));}
    if let Some(until)=&value.recur_until {if value.recur=="none" || parse(until)?<time {return Err(ChatError::new("INVALID_INPUT","重复截止时间须不早于首次提醒，且仅用于重复提醒"));}}
    Ok(Draft {change,value:Some(value)})
}

pub fn commit(conn:&Connection,agent:&str,draft:&Draft,now:NaiveDateTime)->ChatResult<()> {
    // 在最终事务内重新检查时间和修订，等待模型回复期间的编辑不会被覆盖。
    let checked=prepare(conn,agent,draft.change.clone(),now)?;
    let value=draft.value.as_ref();
    match draft.change.action.as_str() {
        "delete"=> {conn.execute("DELETE FROM reminders WHERE agent_id=?1 AND id=?2",params![agent,draft.change.id])?;}
        "create"=> {
            let value=value.unwrap();
            let exists:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM reminders WHERE agent_id=?1 AND task=?2 AND time=?3 AND recur=?4 AND COALESCE(delivered,0)=0)",
                params![agent,value.task,value.time,value.recur],|r|r.get(0))?;
            if exists {return Err(ChatError::new("REMINDER_EXISTS","同一提醒已存在，请查询后修改"));}
            conn.execute("UPDATE reminder_sequence SET last_id=MAX(last_id,(SELECT COALESCE(MAX(id),0) FROM reminders))+1 WHERE singleton=1",[])?;
            conn.execute("INSERT INTO reminders(id,task,time,timestamp,agent_id,delivered,recur,recur_until,revision) SELECT last_id,?1,?2,?3,?4,0,?5,?6,1 FROM reminder_sequence WHERE singleton=1",
                params![value.task,value.time,chrono::Utc::now().timestamp_millis(),agent,value.recur,value.recur_until])?;
        }
        _=> {let value=checked.value.as_ref().unwrap();conn.execute("UPDATE reminders SET task=?1,time=?2,recur=?3,recur_until=?4,delivered=0,revision=revision+1,delivery_error=NULL WHERE agent_id=?5 AND id=?6",
            params![value.task,value.time,value.recur,value.recur_until,agent,value.id])?;}
    }
    Ok(())
}

pub fn deliver(conn:&Connection,agent:&str,id:i64,now:NaiveDateTime,send:impl FnOnce(&Entry)->Result<(),String>)->ChatResult<bool> {
    let current=entry(conn,agent,id)?;
    if current.delivered {return Ok(false);}
    let due=match parse(&current.time) {Ok(time) if time<=now=>time,Ok(_)=>return Ok(false),Err(error)=> {
        conn.execute("UPDATE reminders SET delivery_error=?1 WHERE id=?2",params![error.message,id])?;return Ok(false);
    }};
    if let Err(error)=send(&current) {conn.execute("UPDATE reminders SET delivery_error=?1 WHERE id=?2",params![error,id])?;return Ok(false);}
    let days=match current.recur.as_str() {"daily"=>1,"weekly"=>7,_=>0};
    let next=if days>0 {let interval=days*86400;due.checked_add_signed(Duration::seconds(((now-due).num_seconds()/interval+1)*interval))} else {None};
    let next=next.filter(|next|current.recur_until.as_ref().map(|until|parse(until).is_ok_and(|until|*next<=until)).unwrap_or(true));
    if let Some(next)=next {conn.execute("UPDATE reminders SET time=?1,revision=revision+1,delivery_error=NULL WHERE id=?2",params![next.format("%Y-%m-%d %H:%M:%S").to_string(),id])?;}
    else {conn.execute("UPDATE reminders SET delivered=1,revision=revision+1,delivery_error=NULL WHERE id=?1",[id])?;}
    Ok(true)
}

#[tauri::command]
pub fn local_reminder_query(app:AppHandle,db:tauri::State<DbState>,agent_id:String,input:Query)->ChatResult<Value> {
    if identity::agents(&app)?.get(&agent_id).is_none() {return Err(ChatError::new("AGENT_NOT_FOUND","角色不存在"));}
    query(&*db_lock(&db)?,&agent_id,&input)
}

#[tauri::command]
pub async fn local_manage_reminder(app:AppHandle,db:tauri::State<'_,DbState>,agent_id:String,change:Change)->ChatResult<Value> {
    if identity::agents(&app)?.get(&agent_id).is_none() {return Err(ChatError::new("AGENT_NOT_FOUND","角色不存在"));}
    let result={let mut conn=db_lock(&db)?;let tx=conn.transaction()?;
        let now=Local::now().naive_local();let draft=prepare(&tx,&agent_id,change,now)?;commit(&tx,&agent_id,&draft,now)?;
        let result=query(&tx,&agent_id,&Query::default())?;tx.commit()?;result};
    if let Err(error)=super::reminder_platform::synchronize(&app) {eprintln!("提醒系统调度等待重试：{}",error.message);}
    let _=app.emit("ppc:reminders-updated",json!({"agentId":agent_id}));Ok(result)
}

#[cfg(not(target_os="android"))]
fn notify(app:&AppHandle,id:i64,title:&str,body:&str)->Result<(),String> {
    #[cfg(any(target_os="android",target_os="ios"))]
    let result=app.notification().builder().id((id%i32::MAX as i64) as i32).title(title).body(body).show().map_err(|error|error.to_string());
    #[cfg(not(any(target_os="android",target_os="ios")))]
    let result={
        let _=id;
        // 插件的桌面实现会忽略异步发送错误，直接使用相同底层库取得实际提交结果。
        let mut notification=notify_rust::Notification::new();notification.summary(title).body(body);
        #[cfg(windows)]
        if let Ok(exe)=tauri::utils::platform::current_exe() {
            let development=exe.parent().is_some_and(|path|path.ends_with("target/debug") || path.ends_with("target/release"));
            if !development {notification.app_id(&app.config().identifier);}
        }
        #[cfg(target_os="macos")]
        let _=notify_rust::set_application(if tauri::is_dev() {"com.apple.Terminal"} else {&app.config().identifier});
        #[cfg(not(any(windows,target_os="macos")))]
        let _=app;
        notification.show().map(|_|()).map_err(|error|error.to_string())
    };
    result.map_err(|error| {eprintln!("系统通知发送失败：{error}");"系统通知发送失败，将稍后重试".into()})
}

pub fn start(app:AppHandle) {
    tauri::async_runtime::spawn(async move {loop {
        let result=(||->ChatResult<()> {
            #[cfg(target_os="android")]
            {
                super::reminder_platform::synchronize(&app)?;
                let agents=identity::agents(&app)?;
                if let Some(agents)=agents.as_object() {for id in agents.keys() {let _=app.emit("ppc:reminders-updated",json!({"agentId":id}));}}
                return Ok(());
            }
            #[cfg(not(target_os="android"))]
            {
            let db=app.state::<DbState>();
            let agents=identity::agents(&app)?;
            let permission=app.notification().permission_state().map(|state|state==tauri::plugin::PermissionState::Granted).unwrap_or(false);
            let conn=db_lock(&db)?;
            let ids=agents.as_object().map(|roles|roles.keys().collect::<Vec<_>>()).unwrap_or_default();
            let candidates={let mut stmt=conn.prepare("SELECT agent_id,id FROM reminders WHERE COALESCE(delivered,0)=0 AND time<=?1 AND agent_id IN (SELECT value FROM json_each(?2)) ORDER BY time LIMIT 100")?;
                let rows=stmt.query_map(params![Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),json!(ids).to_string()],|r|Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?)))?;
                rows.collect::<Result<Vec<_>,_>>()?};
            for (agent,id) in candidates {
                let Some(character)=agents.get(&agent) else {continue};
                // 发送与确认共用数据库锁，取消或编辑不会插入这两个步骤之间。
                let notified=deliver(&conn,&agent,id,Local::now().naive_local(),|entry| {
                    if !permission {return Err("系统通知权限未开启，提醒仍保留待处理".into());}
                    notify(&app,id,&format!("{} 的提醒",character["name"].as_str().unwrap_or(&agent)),&entry.task)
                })?;
                if notified {let _=app.emit("ppc:reminder-due",json!({"agentId":agent,"id":id}));}
                let _=app.emit("ppc:reminders-updated",json!({"agentId":agent}));
            }
            Ok(())
            }
        })();
        if let Err(error)=result {eprintln!("提醒检查失败：{}",error.message);}
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    }});
}
