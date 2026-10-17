//! 持久化运行目标。所有本地提交和系统入口共用控制库中的短事务门。
use std::{path::{Path, PathBuf}, sync::Arc, time::Duration};
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use crate::local::{runtime::ChatRuntime, types::*};

tokio::task_local! {
    /// IPC 到达时的原生运行令牌，跨 await 保持；不从当前 UI 状态重新推断。
    pub static LOCAL_TOKEN: Token;
    /// 只由持有切换锁的协调器设置，用于闭门状态准备独立存储。
    pub static LOCAL_INITIALIZING: bool;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag="mode", rename_all="camelCase", deny_unknown_fields)]
pub enum Target {
    Standalone,
    Remote { #[serde(rename="profileId")] profile_id:String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Token { pub installation_id:String, pub target:Target, pub epoch:u64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Snapshot {
    pub schema_version:u32, pub revision:u64, pub installation_id:String, pub runtime_owner_id:String, pub selected_target:Target,
    pub active_target:Option<Target>, pub activation_epoch:u64, pub phase:String,
    pub switch_id:Option<String>, pub last_error:Option<String>,
}
impl Snapshot {
    pub fn token(&self)->ChatResult<Token> {
        if self.phase!="stable" || self.active_target.as_ref()!=Some(&self.selected_target) {return Err(closed());}
        Ok(Token { installation_id:self.installation_id.clone(), target:self.active_target.clone().ok_or_else(closed)?, epoch:self.activation_epoch })
    }
}
fn closed()->ChatError {ChatError::new("MODE_INACTIVE","当前运行目标未就绪或正在切换")}

#[derive(Clone)]
pub struct Coordinator {
    path:Arc<PathBuf>, pub switch_lock:Arc<tokio::sync::Mutex<()>>,
    changes:tokio::sync::watch::Sender<u64>,
    owner:Arc<String>,
}

/// 连接持有 BEGIN IMMEDIATE，跨进程串行化校验与短副作用；释放时回滚只读事务。
pub struct Gate { pub connection:Connection, pub snapshot:Snapshot }
impl Drop for Gate {fn drop(&mut self) {let _=self.connection.execute_batch("ROLLBACK");}}

pub fn open_existing(path:&Path)->ChatResult<Connection> {
    let conn=Connection::open_with_flags(path,OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    conn.busy_timeout(Duration::from_secs(5))?;
    Ok(conn)
}
fn read(conn:&Connection)->ChatResult<Snapshot> {
    let raw:String=conn.query_row("SELECT value FROM runtime_control WHERE id=1",[],|r|r.get(0))?;
    let state:Snapshot=serde_json::from_str(&raw).map_err(|_|ChatError::new("MODE_CORRUPT","运行控制数据损坏，后台入口已关闭"))?;
    if state.schema_version!=1 || state.activation_epoch==0 || uuid::Uuid::parse_str(&state.installation_id).is_err() || uuid::Uuid::parse_str(&state.runtime_owner_id).is_err() {
        return Err(ChatError::new("MODE_CORRUPT","运行控制版本或身份无效"));
    }
    Ok(state)
}
fn write(conn:&Connection,state:&Snapshot)->ChatResult<()> {
    conn.execute("UPDATE runtime_control SET value=?1 WHERE id=1",[serde_json::to_string(state).map_err(|_|closed())?])?;Ok(())
}
pub fn lock_existing(path:&Path,expected:Option<&Token>,standalone:bool)->ChatResult<Gate> {
    let conn=open_existing(path)?;
    conn.execute_batch("BEGIN IMMEDIATE")?;
    let snapshot=read(&conn)?;
    let token=snapshot.token()?;
    if expected.is_some_and(|value|value!=&token) || (standalone && token.target!=Target::Standalone) {return Err(closed());}
    Ok(Gate {connection:conn,snapshot})
}
pub fn inactive_local_gate(path:&Path)->ChatResult<Gate> {
    let conn=open_existing(path)?;conn.execute_batch("BEGIN IMMEDIATE")?;let snapshot=read(&conn)?;
    if snapshot.phase=="stable" && snapshot.active_target==Some(Target::Standalone) {return Err(closed());}
    Ok(Gate {connection:conn,snapshot})
}

impl Coordinator {
    pub fn initialize(path:PathBuf)->ChatResult<Self> {
        std::fs::create_dir_all(path.parent().ok_or_else(closed)?).map_err(|_|ChatError::new("STORAGE_ERROR","无法创建运行控制目录"))?;
        let conn=Connection::open(&path)?;conn.busy_timeout(Duration::from_secs(5))?;
        conn.execute_batch("PRAGMA journal_mode=WAL;
            CREATE TABLE IF NOT EXISTS runtime_control(id INTEGER PRIMARY KEY CHECK(id=1),value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS remote_profiles(id TEXT PRIMARY KEY,value TEXT NOT NULL);")?;
        let owner=uuid::Uuid::new_v4().to_string();
        let initial=Snapshot {schema_version:1,revision:1,installation_id:uuid::Uuid::new_v4().to_string(),runtime_owner_id:owner.clone(),selected_target:Target::Standalone,
            active_target:None,activation_epoch:1,phase:"activating".into(),switch_id:Some(uuid::Uuid::new_v4().to_string()),last_error:None};
        conn.execute("INSERT OR IGNORE INTO runtime_control(id,value) VALUES(1,?1)",[serde_json::to_string(&initial).unwrap()])?;
        let (changes,_)=tokio::sync::watch::channel(0);
        let result=Self {path:Arc::new(path),switch_lock:Arc::new(tokio::sync::Mutex::new(())),changes,owner:Arc::new(owner)};
        // 每次进程启动先撤销旧代次；未完成切换按已选择目标恢复，后台入口保持关闭。
        result.update_with(true,|state| {
            state.runtime_owner_id=(*result.owner).clone();
            state.activation_epoch=state.activation_epoch.checked_add(1).ok_or_else(closed)?;
            state.active_target=None;state.phase="activating".into();
            state.switch_id=Some(uuid::Uuid::new_v4().to_string());Ok(())
        })?;
        Ok(result)
    }
    pub fn path(&self)->&Path {&self.path}
    pub fn snapshot(&self)->ChatResult<Snapshot> {
        let state=read(&open_existing(self.path())?)?;if &state.runtime_owner_id!=self.owner.as_ref() {return Err(closed());}Ok(state)
    }
    pub fn token(&self)->ChatResult<Token> {self.snapshot()?.token()}
    pub fn gate(&self,expected:Option<&Token>,standalone:bool)->ChatResult<Gate> {
        let gate=lock_existing(self.path(),expected,standalone)?;if &gate.snapshot.runtime_owner_id!=self.owner.as_ref() {return Err(closed());}Ok(gate)
    }
    pub fn local_gate(&self)->ChatResult<Gate> {
        let expected=LOCAL_TOKEN.try_with(Clone::clone).ok();self.gate(expected.as_ref(),true)
    }
    pub fn initialization_gate(&self)->ChatResult<Gate> {
        let conn=open_existing(self.path())?;conn.execute_batch("BEGIN IMMEDIATE")?;
        let snapshot=read(&conn)?;
        if &snapshot.runtime_owner_id!=self.owner.as_ref() || !LOCAL_INITIALIZING.try_with(|allowed|*allowed).unwrap_or(false) || snapshot.active_target.is_some()
            || snapshot.selected_target!=Target::Standalone || !["activating","quiescing"].contains(&snapshot.phase.as_str()) || snapshot.switch_id.is_none() {
            return Err(closed());
        }
        Ok(Gate {connection:conn,snapshot})
    }
    fn cleanup_gate(&self)->ChatResult<Gate> {
        let conn=open_existing(self.path())?;conn.execute_batch("BEGIN IMMEDIATE")?;let snapshot=read(&conn)?;
        if &snapshot.runtime_owner_id!=self.owner.as_ref() || snapshot.active_target.is_some() || snapshot.phase=="stable" {return Err(closed());}
        Ok(Gate {connection:conn,snapshot})
    }
    pub fn subscribe(&self)->tokio::sync::watch::Receiver<u64> {self.changes.subscribe()}
    fn update(&self,change:impl FnOnce(&mut Snapshot)->ChatResult<()>)->ChatResult<Snapshot> {
        self.update_with(false,change)
    }
    fn update_with(&self,takeover:bool,change:impl FnOnce(&mut Snapshot)->ChatResult<()>)->ChatResult<Snapshot> {
        let mut conn=open_existing(self.path())?;let tx=conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let mut state=read(&tx)?;
        if !takeover && &state.runtime_owner_id!=self.owner.as_ref() {return Err(closed());}
        change(&mut state)?;state.revision=state.revision.checked_add(1).ok_or_else(closed)?;write(&tx,&state)?;tx.commit()?;
        self.changes.send_replace(state.activation_epoch);Ok(state)
    }
    pub fn begin_switch(&self,target:Target)->ChatResult<Snapshot> {
        self.update(|state| {
            state.activation_epoch=state.activation_epoch.checked_add(1).ok_or_else(closed)?;
            state.selected_target=target;state.active_target=None;state.phase="quiescing".into();
            state.switch_id=Some(uuid::Uuid::new_v4().to_string());state.last_error=None;Ok(())
        })
    }
    pub fn activate(&self)->ChatResult<Snapshot> {
        self.update(|state| {state.active_target=Some(state.selected_target.clone());state.phase="stable".into();state.switch_id=None;state.last_error=None;Ok(())})
    }
    pub fn fail(&self)->ChatResult<Snapshot> {
        self.update(|state| {state.active_target=None;state.phase="recovery_required".into();state.last_error=Some("模式切换尚未完成，请重试恢复；后台执行保持关闭".into());Ok(())})
    }
}

#[tauri::command]
pub fn mode_snapshot(mode:tauri::State<Coordinator>)->ChatResult<Snapshot> {mode.snapshot()}

#[tauri::command]
pub async fn mode_switch(app:AppHandle,target:Target)->ChatResult<Snapshot> {
    let mode=app.state::<Coordinator>();let _switch=mode.switch_lock.lock().await;
    if let Target::Remote {profile_id}= &target {crate::remote::profiles::get(&mode,profile_id)?;}
    if let Ok(state)=mode.snapshot() {if state.phase=="stable" && state.active_target.as_ref()==Some(&target) {return Ok(state);}}
    let state=mode.begin_switch(target)?;
    let _=app.emit("mode-event",&state);
    let result=async {
        let runtime=app.state::<ChatRuntime>();runtime.cancel_all()?;
        crate::remote::stop_all(&app);
        // 不持有控制事务等待异步请求退出。门已关闭，任何迟到提交均被拒绝。
        tokio::time::timeout(Duration::from_secs(10),async {
            while !runtime.is_idle()? {tokio::time::sleep(Duration::from_millis(20)).await;}Ok::<_,ChatError>(())
        }).await.map_err(|_|ChatError::new("MODE_BUSY","旧请求尚未退出，运行门保持关闭，请重试切换"))??;
        cleanup_local(&app)?;
        crate::local::reminder_platform::suspend_async(app.clone()).await?;
        if mode.snapshot()?.selected_target==Target::Standalone {prepare_standalone(&app)?;}
        mode.activate()
    }.await;
    match result {
        Ok(state)=> {let _=app.emit("mode-event",&state);Ok(state)},
        Err(error)=> {if let Ok(state)=mode.fail() {let _=app.emit("mode-event",&state);}Err(error)}
    }
}

/// 仅协调器在门关闭时执行确定的终态清理，不保存旧回复或工具草稿。
pub fn cleanup_local(app:&AppHandle)->ChatResult<()> {
    let mode=app.state::<Coordinator>();let _gate=mode.cleanup_gate()?;
    let db=app.state::<crate::db::DbState>();
    let mut conn=db.conn.lock().map_err(|_|closed())?;
    cleanup_storage(&mut conn)
}
fn cleanup_storage(conn:&mut Connection)->ChatResult<()> {
    let tx=conn.transaction()?;
    // 迁移中途退出也可以恢复；不存在的独立表不为远程启动而创建。
    for (table,sql) in [
        ("chat_messages","UPDATE chat_messages SET status='cancelled',content='\"\"',reasoning='',error='运行模式已切换' WHERE status='pending'"),
        ("chat_turns","UPDATE chat_turns SET status='cancelled',error='运行模式已切换' WHERE status='running'"),
        ("local_group_batches",r#"UPDATE local_group_batches SET status='failed',error='{"code":"CANCELLED","message":"运行模式已切换"}' WHERE status='running'"#),
        ("reminder_delivery","DELETE FROM reminder_delivery"),
        ("reminder_dispatch","UPDATE reminder_dispatch SET registered_version=NULL,mode=NULL"),
    ] {
        let exists:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",[table],|r|r.get(0))?;
        if exists {tx.execute(sql,[])?;}
    }
    let clock:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='reminder_dispatch_clock')",[],|r|r.get(0))?;
    if clock {tx.execute("UPDATE reminder_dispatch_clock SET epoch=?1 WHERE singleton=1",[uuid::Uuid::new_v4().to_string()])?;}
    tx.commit()?;Ok(())
}

pub fn prepare_standalone(app:&AppHandle)->ChatResult<()> {
    LOCAL_INITIALIZING.sync_scope(true,||crate::local::initialize(app))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->Coordinator {
        let dir=std::env::temp_dir().join(format!("ppc-mode-{}",uuid::Uuid::new_v4()));
        Coordinator::initialize(dir.join("runtime.sqlite")).unwrap()
    }
    #[test]
    fn cold_entry_fails_closed_and_old_epoch_never_recovers() {
        let mode=fixture();assert!(mode.gate(None,true).is_err());mode.activate().unwrap();let old=mode.token().unwrap();
        mode.begin_switch(Target::Remote {profile_id:"profile".into()}).unwrap();assert!(mode.gate(Some(&old),true).is_err());
        mode.activate().unwrap();assert!(mode.gate(None,true).is_err());
        mode.begin_switch(Target::Standalone).unwrap();mode.activate().unwrap();
        assert!(mode.gate(Some(&old),true).is_err());assert!(mode.gate(None,true).is_ok());
    }
    #[test]
    fn restart_keeps_remote_selection_but_revokes_execution_until_activation() {
        let mode=fixture();mode.begin_switch(Target::Remote {profile_id:"a".into()}).unwrap();mode.activate().unwrap();
        let old=mode.token().unwrap();let next=Coordinator::initialize(mode.path().to_owned()).unwrap();
        assert!(next.token().is_err());assert_eq!(next.snapshot().unwrap().selected_target,old.target);
        next.activate().unwrap();assert_ne!(next.token().unwrap().epoch,old.epoch);
        assert!(mode.token().is_err());assert!(mode.gate(None,false).is_err());assert!(mode.begin_switch(Target::Standalone).is_err());
    }
    #[test]
    fn local_preparation_requires_closed_local_target_and_partial_schema_can_be_cleaned() {
        let mode=fixture();
        assert!(mode.initialization_gate().is_err());
        assert!(inactive_local_gate(mode.path()).is_ok());
        LOCAL_INITIALIZING.sync_scope(true,||assert!(mode.initialization_gate().is_ok()));
        mode.activate().unwrap();assert!(inactive_local_gate(mode.path()).is_err());
        mode.begin_switch(Target::Remote {profile_id:"r".into()}).unwrap();
        LOCAL_INITIALIZING.sync_scope(true,||assert!(mode.initialization_gate().is_err()));
        let mut conn=Connection::open_in_memory().unwrap();cleanup_storage(&mut conn).unwrap();
        conn.execute_batch("CREATE TABLE chat_turns(status TEXT,error TEXT);INSERT INTO chat_turns VALUES('running',NULL);").unwrap();
        cleanup_storage(&mut conn).unwrap();let status:String=conn.query_row("SELECT status FROM chat_turns",[],|r|r.get(0)).unwrap();assert_eq!(status,"cancelled");
    }
    #[test]
    fn native_commit_gate_serializes_switch_across_connections() {
        let mode=fixture();mode.activate().unwrap();let gate=mode.gate(None,true).unwrap();
        let other=mode.clone();let (send,recv)=std::sync::mpsc::channel();
        let worker=std::thread::spawn(move || {other.begin_switch(Target::Remote {profile_id:"b".into()}).unwrap();send.send(()).unwrap();});
        assert!(recv.recv_timeout(Duration::from_millis(60)).is_err());drop(gate);
        recv.recv_timeout(Duration::from_secs(3)).unwrap();worker.join().unwrap();
        assert!(mode.gate(None,true).is_err());
    }
}
