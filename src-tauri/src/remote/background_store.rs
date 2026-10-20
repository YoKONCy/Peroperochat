//! 无界面的远程通知入口；不接管运行目标，不初始化 Tauri 或独立模式业务。
use std::path::Path;
use rusqlite::{Connection,OptionalExtension};
use serde_json::{json,Value};
use crate::{local::types::*,mode::{self,Token,Target}};

pub(super) fn root(path:&str)->ChatResult<&Path> {
    let path=Path::new(path);
    if !path.is_absolute() || path.file_name().and_then(|s|s.to_str())!=Some("data") || !path.join("control/runtime.sqlite").is_file() {
        return Err(ChatError::new("MODE_INACTIVE","后台运行控制尚未初始化"));
    }
    Ok(path)
}
fn table(conn:&Connection)->ChatResult<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS notification_runtime(id INTEGER PRIMARY KEY CHECK(id=1),enabled INTEGER NOT NULL DEFAULT 0,blocked TEXT NOT NULL DEFAULT '',state TEXT NOT NULL DEFAULT 'stopped',updated INTEGER NOT NULL DEFAULT 0,visible_until INTEGER NOT NULL DEFAULT 0,launch TEXT);
        INSERT OR IGNORE INTO notification_runtime(id) VALUES(1);")?;
    super::notifications::table(conn)
}
pub(super) fn gate(root:&Path,expected:Option<&Token>)->ChatResult<mode::Gate> {
    let gate=mode::lock_existing(&root.join("control/runtime.sqlite"),expected,false)?;
    if !matches!(gate.snapshot.selected_target,Target::Remote {..}) {return Err(super::inactive());}
    Ok(gate)
}
pub(super) fn profile(gate:&mode::Gate)->ChatResult<super::profiles::Profile> {
    let Target::Remote {profile_id}=&gate.snapshot.selected_target else {return Err(super::inactive());};
    if uuid::Uuid::parse_str(profile_id).is_err() {return Err(super::inactive());}
    let raw:String=gate.connection.query_row("SELECT value FROM remote_profiles WHERE id=?1",[profile_id],|row|row.get(0))?;
    let profile:super::profiles::Profile=serde_json::from_str(&raw).map_err(|_|super::inactive())?;
    if profile.id!=*profile_id {return Err(super::inactive());}
    Ok(profile)
}
pub(super) fn connection(root:&Path,token:&Token)->ChatResult<Connection> {
    let conn=super::cache::connection_at(root,token)?;table(&conn)?;Ok(conn)
}
fn enabled(conn:&Connection)->ChatResult<bool> {
    Ok(conn.query_row("SELECT enabled=1 AND blocked='' FROM notification_runtime WHERE id=1",[],|r|r.get(0))?)
}
pub(super) fn runnable(root:&Path,token:&Token)->ChatResult<super::profiles::Profile> {
    let gate=gate(root,Some(token))?;let profile=profile(&gate)?;
    if profile.session_owner.is_some() || profile.device.as_ref().is_some_and(|device|device.expires_at<=chrono::Utc::now().timestamp_millis()) {
        return Err(ChatError::new("REMOTE_AUTH_REQUIRED","后台接收需要有效的保持登录授权"));
    }
    if !enabled(&connection(root,token)?)? {return Err(super::inactive());}
    Ok(profile)
}
pub(super) fn principal(conn:&Connection)->ChatResult<Option<String>> {
    Ok(conn.query_row("SELECT value FROM preferences WHERE key='principal'",[],|r|r.get(0)).optional()?)
}
pub(super) fn state(root:&Path,token:&Token,state:&str,blocked:&str)->ChatResult<()> {
    let _gate=gate(root,Some(token))?;
    connection(root,token)?.execute("UPDATE notification_runtime SET state=?1,blocked=CASE WHEN ?2='' THEN blocked ELSE ?2 END,updated=?3 WHERE id=1",rusqlite::params![state,blocked,chrono::Utc::now().timestamp_millis()])?;Ok(())
}
pub(super) fn capture(root:&Path,token:&Token,principal:&str,value:&Value)->ChatResult<()> {
    let _gate=gate(root,Some(token))?;
    if value["payload"]["action"]!="durable_notification" {return Ok(());}
    if value["protocolVersion"]!=1 || value["type"]!="push" || value["payload"]["principalId"]!=principal
        || !value["id"].as_str().is_some_and(|id|!id.is_empty() && id.len()<=1024) {return Err(ChatError::new("REMOTE_PROTOCOL","后台通知身份不匹配"));}
    super::notifications::capture_in(&mut connection(root,token)?,&value["payload"])
}

/// Java 回调只显示系统通知，不能重入运行门或修改业务状态。
pub(super) fn deliver(root:&Path,token:&Token,show:&mut impl FnMut(&Value)->ChatResult<bool>)->ChatResult<()> {
    let _gate=gate(root,Some(token))?;let mut conn=connection(root,token)?;
    if !enabled(&conn)? {return Err(super::inactive());}
    let mut stmt=conn.prepare("SELECT n.value FROM notifications n LEFT JOIN notification_receipts r ON r.principal=n.principal AND r.id=n.id WHERE n.revision>COALESCE(r.delivered,0) AND n.revision>COALESCE(r.read_revision,0) ORDER BY n.touched DESC LIMIT 8")?;
    let rows=stmt.query_map([],|r|r.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()?;drop(stmt);
    for raw in rows {
        let mut notice:Value=serde_json::from_str(&raw).map_err(|_|super::inactive())?;
        let visible:i64=conn.query_row("SELECT visible_until FROM notification_runtime WHERE id=1",[],|r|r.get(0))?;
        notice["modeToken"]=json!(token);
        notice["foregroundHint"]=json!(visible>chrono::Utc::now().timestamp_millis());
        // 界面可见时由收件箱展示；接收与系统通知都不发送已读回执。
        if show(&notice)? {
            let tx=conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            tx.execute("INSERT INTO notification_receipts(principal,id,delivered) VALUES(?1,?2,?3) ON CONFLICT(principal,id) DO UPDATE SET delivered=MAX(delivered,excluded.delivered)",rusqlite::params![notice["principalId"].as_str(),notice["notificationId"].as_str(),notice["revision"].as_i64()])?;
            tx.commit()?;
        }
    }
    Ok(())
}

pub(super) fn execute(root:&Path,command:&str,input:&Value)->ChatResult<Value> {
    let expected=input.get("modeToken").map(|v|serde_json::from_value::<Token>(v.clone())).transpose().map_err(|_|super::inactive())?;
    let gate=match gate(root,expected.as_ref()) {Ok(gate)=>gate,Err(_) if command=="plan" || command=="visible"=>return Ok(json!({"active":false})),Err(error)=>return Err(error)};
    let token=gate.snapshot.token()?;let profile=profile(&gate)?;let conn=connection(root,&token)?;
    let now=chrono::Utc::now().timestamp_millis();
    match command {
        "configure"=> {
            if expected.is_none() {return Err(super::inactive());}
            let enable=input["enabled"].as_bool().ok_or_else(super::inactive)?;
            if enable && (profile.session_owner.is_some() || profile.device.as_ref().is_some_and(|device|device.expires_at<=now)) {return Err(ChatError::new("REMOTE_AUTH_REQUIRED","请先使用保持登录重新授权，再开启后台接收"));}
            conn.execute("UPDATE notification_runtime SET enabled=?1,blocked='',state='stopped',updated=?2 WHERE id=1",rusqlite::params![enable,now])?;
        },
        "visible"=> {conn.execute("UPDATE notification_runtime SET visible_until=?1 WHERE id=1",[if input["visible"]==true {now+35_000} else {0}])?;},
        "block"=> {
            if expected.is_none() {return Err(super::inactive());}
            conn.execute("UPDATE notification_runtime SET blocked='REMOTE_AUTH_REQUIRED',state='blocked',updated=?1 WHERE id=1",[now])?;
        },
        "reauthorized"=> {
            if expected.is_none() {return Err(super::inactive());}
            conn.execute("UPDATE notification_runtime SET blocked='',state='stopped',updated=?1 WHERE id=1",[now])?;
        },
        "open"=> {
            // 点击必须来自当前代次；进入冷启动后只交接一次，绝不自动切换运行模式或确认已读。
            if expected.is_none() {return Err(super::inactive());}
            let principal=input["principalId"].as_str().ok_or_else(super::inactive)?;
            let id=input["notificationId"].as_str().ok_or_else(super::inactive)?;
            let exists:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM notifications WHERE principal=?1 AND id=?2)",rusqlite::params![principal,id],|r|r.get(0))?;
            if !exists {return Ok(Value::Null);}
            conn.execute("UPDATE notification_runtime SET launch=?1 WHERE id=1",[json!({"principalId":principal,"notificationId":id,"expiresAt":now+600_000,"installationId":token.installation_id,"epoch":token.epoch,"owner":gate.snapshot.runtime_owner_id}).to_string()])?;
            return Ok(Value::Null);
        },
        "consumeOpen"=> {
            if expected.is_none() {return Err(super::inactive());}
            let launch:Option<String>=conn.query_row("SELECT launch FROM notification_runtime WHERE id=1",[],|r|r.get(0))?;
            conn.execute("UPDATE notification_runtime SET launch=NULL WHERE id=1",[])?;
            return Ok(launch.and_then(|raw|serde_json::from_str::<Value>(&raw).ok()).filter(|v| {
                let same_epoch=v["epoch"].as_u64()==Some(token.epoch) && v["owner"]==gate.snapshot.runtime_owner_id;
                let cold_handoff=v["epoch"].as_u64().and_then(|n|n.checked_add(1))==Some(token.epoch) && v["owner"]!=gate.snapshot.runtime_owner_id;
                v["installationId"]==token.installation_id && v["expiresAt"].as_i64().is_some_and(|at|at>now) && (same_epoch || cold_handoff)
            }).unwrap_or(Value::Null));
        },
        "status"|"plan"=>{},_=>return Err(ChatError::new("INVALID_INPUT","未知后台通知操作")),
    }
    let (setting,blocked,state,updated):(bool,String,String,i64)=conn.query_row("SELECT enabled,blocked,state,updated FROM notification_runtime WHERE id=1",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?;
    let eligible=profile.session_owner.is_none() && profile.device.as_ref().is_none_or(|device|device.expires_at>now);
    Ok(json!({"active":setting && blocked.is_empty() && eligible,"enabled":setting,"eligible":eligible,"blocked":blocked,
        "state":if now-updated>90_000 {"stopped"} else {&state},"updatedAt":updated,"modeToken":token,
        "credentialId":profile.credential_key(),"principalId":principal(&conn)?}))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->(std::path::PathBuf,mode::Coordinator,Token) {
        let root=std::env::temp_dir().join(format!("ppc-background-{}",uuid::Uuid::new_v4())).join("data");
        let mode=mode::Coordinator::initialize(root.join("control/runtime.sqlite")).unwrap();let id=uuid::Uuid::new_v4().to_string();
        mode::open_existing(mode.path()).unwrap().execute("INSERT INTO remote_profiles VALUES(?1,?2)",rusqlite::params![id,json!({"id":id,"serverId":"server","endpoint":"https://example.com","displayName":"测试","appVersion":"1"}).to_string()]).unwrap();
        mode.begin_switch(Target::Remote {profile_id:id}).unwrap();mode.activate().unwrap();let token=mode.token().unwrap();(root,mode,token)
    }
    #[test]
    fn background_is_opt_in_and_old_epoch_cannot_deliver_or_restart() {
        let (root,mode,token)=fixture();assert_eq!(execute(&root,"plan",&json!({})).unwrap()["active"],false);
        execute(&root,"configure",&json!({"modeToken":token,"enabled":true})).unwrap();assert!(runnable(&root,&token).is_ok());
        mode.begin_switch(Target::Standalone).unwrap();mode.activate().unwrap();
        assert_eq!(execute(&root,"plan",&json!({})).unwrap()["active"],false);assert!(runnable(&root,&token).is_err());
        assert!(deliver(&root,&token,&mut |_|panic!("旧代次不能展示通知")).is_err());
    }
    #[test]
    fn replay_is_deduplicated_without_read_ack_and_launch_is_one_shot() {
        let (root,_mode,token)=fixture();execute(&root,"configure",&json!({"modeToken":token,"enabled":true})).unwrap();
        let event=json!({"protocolVersion":1,"type":"push","id":"e","payload":{"action":"durable_notification","principalId":"custom","notificationId":"n","revision":1,"body":"测试"}});
        capture(&root,&token,"custom",&event).unwrap();capture(&root,&token,"custom",&event).unwrap();
        let mut count=0;let mut show=|_:&Value|{count+=1;Ok(true)};
        deliver(&root,&token,&mut show).unwrap();deliver(&root,&token,&mut show).unwrap();assert_eq!(count,1);
        let conn=connection(&root,&token).unwrap();let read:i64=conn.query_row("SELECT read_revision FROM notification_receipts",[],|r|r.get(0)).unwrap();assert_eq!(read,0);
        execute(&root,"open",&json!({"modeToken":token,"principalId":"custom","notificationId":"n"})).unwrap();
        assert_eq!(execute(&root,"consumeOpen",&json!({"modeToken":token})).unwrap()["notificationId"],"n");
        assert!(execute(&root,"consumeOpen",&json!({"modeToken":token})).unwrap().is_null());
        assert!(capture(&root,&token,"other",&event).is_err());
    }
    #[test]
    fn auth_block_persists_until_explicit_retry_and_foreground_receipt_is_not_read() {
        let (root,_mode,token)=fixture();execute(&root,"configure",&json!({"modeToken":token,"enabled":true})).unwrap();
        execute(&root,"block",&json!({"modeToken":token})).unwrap();
        state(&root,&token,"reconnecting","").unwrap();assert_eq!(execute(&root,"plan",&json!({})).unwrap()["active"],false);
        execute(&root,"configure",&json!({"modeToken":token,"enabled":true})).unwrap();
        execute(&root,"visible",&json!({"visible":true})).unwrap();
        let event=json!({"protocolVersion":1,"type":"push","id":"e","payload":{"action":"durable_notification","principalId":"a","notificationId":"n","revision":1}});
        capture(&root,&token,"a",&event).unwrap();deliver(&root,&token,&mut |notice| {assert_eq!(notice["foregroundHint"],true);Ok(true)}).unwrap();
        let conn=connection(&root,&token).unwrap();
        assert_eq!(conn.query_row("SELECT read_revision FROM notification_receipts",[],|r|r.get::<_,i64>(0)).unwrap(),0);
        conn.execute("UPDATE notification_receipts SET read_revision=1",[]).unwrap();
        capture(&root,&token,"a",&event).unwrap();assert_eq!(conn.query_row("SELECT COUNT(*) FROM notifications",[],|r|r.get::<_,i64>(0)).unwrap(),0);
        let mut update=event.clone();update["payload"]["revision"]=json!(2);capture(&root,&token,"a",&update).unwrap();
        assert_eq!(conn.query_row("SELECT revision FROM notifications",[],|r|r.get::<_,i64>(0)).unwrap(),2);
    }
    #[test]
    fn notification_click_survives_one_cold_start_but_not_mode_roundtrip() {
        let (root,mode,token)=fixture();
        let event=json!({"protocolVersion":1,"type":"push","id":"e","payload":{"action":"durable_notification","principalId":"a","notificationId":"n","revision":1}});
        capture(&root,&token,"a",&event).unwrap();execute(&root,"open",&json!({"modeToken":token,"principalId":"a","notificationId":"n"})).unwrap();
        let next=mode::Coordinator::initialize(mode.path().to_owned()).unwrap();next.activate().unwrap();let current=next.token().unwrap();
        assert_eq!(execute(&root,"consumeOpen",&json!({"modeToken":current})).unwrap()["notificationId"],"n");
        execute(&root,"open",&json!({"modeToken":current,"principalId":"a","notificationId":"n"})).unwrap();
        next.begin_switch(Target::Standalone).unwrap();next.activate().unwrap();next.begin_switch(current.target.clone()).unwrap();next.activate().unwrap();
        assert!(execute(&root,"consumeOpen",&json!({"modeToken":next.token().unwrap()})).unwrap().is_null());
    }
}
