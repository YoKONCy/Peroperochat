use chrono::{Local,NaiveDateTime,TimeZone};
use rusqlite::params;
use serde_json::{json,Value};
use super::{identity,reminder_dispatch as dispatch,reminders,tests::state};

fn millis(value:&str)->i64 {Local.from_local_datetime(&NaiveDateTime::parse_from_str(value,"%Y-%m-%d %H:%M:%S").unwrap()).single().unwrap().timestamp_millis()}
fn seed(conn:&rusqlite::Connection,id:i64,recur:&str) {
    identity::set_value_in(conn,"ppc.agents",&json!(json!({"pero":{"name":"Pero"},"nana":{"name":"Nana"}}).to_string())).unwrap();
    conn.execute("INSERT INTO reminders(id,task,time,timestamp,agent_id,delivered,recur,revision) VALUES(?1,'浇花','2030-01-02 12:00:00',1,'pero',0,?2,1)",params![id,recur]).unwrap();
}
fn token(conn:&rusqlite::Connection,id:i64)->String {
    let plan=dispatch::plan(conn,millis("2030-01-01 12:00:00")).unwrap();
    plan["jobs"].as_array().unwrap().iter().find(|job|job["id"]==id).unwrap()["token"].as_str().unwrap().into()
}
fn invoke(conn:&mut rusqlite::Connection,command:&str,input:Value)->Value {dispatch::execute(conn,command,input,millis("2030-01-02 13:00:00")).unwrap()}

#[test]
fn dispatch_outbox_rolls_back_with_reminders_and_preserves_registration_versions() {
    let db=state();let mut conn=db.conn.lock().unwrap();
    {let tx=conn.transaction().unwrap();seed(&tx,1,"none");assert_eq!(dispatch::plan(&tx,millis("2030-01-01 12:00:00")).unwrap()["jobs"].as_array().unwrap().len(),1);}
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM reminder_dispatch",[],|row|row.get::<_,i64>(0)).unwrap(),0);
    seed(&conn,1,"none");let first=token(&conn,1);
    invoke(&mut conn,"registered",json!([{"id":1,"token":first,"mode":"exact","error":null}]));
    assert_eq!(reminders::query(&conn,"pero",&Default::default()).unwrap()["entries"][0]["systemMode"],"exact");
    conn.execute("UPDATE reminders SET task='用户刚修改',revision=2 WHERE id=1",[]).unwrap();let second=token(&conn,1);assert_ne!(first,second);
    invoke(&mut conn,"registered",json!([{"id":1,"token":first,"mode":"exact","error":null}]));
    assert_eq!(reminders::query(&conn,"pero",&Default::default()).unwrap()["entries"][0]["systemMode"],"pending");
    invoke(&mut conn,"registered",json!([{"id":1,"token":second,"mode":"blocked","error":"通知权限未开启"}]));
    let blocked=reminders::query(&conn,"pero",&Default::default()).unwrap();assert_eq!(blocked["entries"][0]["systemMode"],"blocked");
    assert_eq!(blocked["entries"][0]["backgroundError"],"通知权限未开启");
}

#[test]
fn cancelled_replaced_and_deleted_role_callbacks_never_claim_tasks() {
    let db=state();let mut conn=db.conn.lock().unwrap();seed(&conn,1,"none");let old=token(&conn,1);
    conn.execute("UPDATE reminders SET task='新的内容',revision=2 WHERE id=1",[]).unwrap();
    assert!(invoke(&mut conn,"claim",json!({"id":1,"token":old})).is_null());
    let current=token(&conn,1);conn.execute("DELETE FROM reminders WHERE id=1",[]).unwrap();
    assert!(invoke(&mut conn,"claim",json!({"id":1,"token":current})).is_null());
    let plan=invoke(&mut conn,"plan",json!({}));assert_eq!(plan["cancellations"][0]["dismiss"],true);
    seed(&conn,1,"none");let restored=token(&conn,1);assert_ne!(restored,current);
    assert!(invoke(&mut conn,"claim",json!({"id":1,"token":current})).is_null());
    identity::set_value_in(&conn,"ppc.agents",&json!(json!({"nana":{"name":"Nana"}}).to_string())).unwrap();
    assert!(invoke(&mut conn,"claim",json!({"id":1,"token":restored})).is_null());
    assert!(invoke(&mut conn,"plan",json!({}))["jobs"].as_array().unwrap().is_empty());
}

#[test]
fn durable_claim_and_receipt_advance_recurrence_once() {
    let db=state();let mut conn=db.conn.lock().unwrap();seed(&conn,1,"daily");let current=token(&conn,1);
    let claim=invoke(&mut conn,"claim",json!({"id":1,"token":current}));assert_eq!(claim["body"],"浇花");
    assert!(invoke(&mut conn,"claim",json!({"id":1,"token":current})).is_null());
    let finished=invoke(&mut conn,"finish",json!({"id":1,"token":current,"lease":claim["lease"]}));assert_eq!(finished,true);
    assert_eq!(reminders::entry(&conn,"pero",1).unwrap().time,"2030-01-03 12:00:00");
    assert_eq!(invoke(&mut conn,"finish",json!({"id":1,"token":current,"lease":claim["lease"]})),false);
    let plan=invoke(&mut conn,"plan",json!({}));assert_eq!(plan["jobs"][0]["dismissPrevious"],false);
    assert_ne!(plan["jobs"][0]["token"],current);
}

#[test]
fn failed_receipt_keeps_pending_and_crash_lease_is_recoverable() {
    let db=state();let mut conn=db.conn.lock().unwrap();seed(&conn,1,"none");let current=token(&conn,1);
    let claim=invoke(&mut conn,"claim",json!({"id":1,"token":current}));
    assert_eq!(invoke(&mut conn,"finish",json!({"id":1,"token":current,"lease":claim["lease"],"error":"通知权限被拒绝"})),true);
    assert!(!reminders::entry(&conn,"pero",1).unwrap().delivered);
    assert!(invoke(&mut conn,"claim",json!({"id":1,"token":current})).is_null());
    let later=millis("2030-01-02 13:02:00");
    let again=dispatch::execute(&mut conn,"claim",json!({"id":1,"token":current}),later).unwrap();assert!(!again.is_null());
    let after_crash=dispatch::execute(&mut conn,"claim",json!({"id":1,"token":current}),later+61_000).unwrap();
    assert_ne!(again["lease"],after_crash["lease"]);
    assert_eq!(dispatch::execute(&mut conn,"finish",json!({"id":1,"token":current,"lease":again["lease"]}),later+62_000).unwrap(),false);
    assert_eq!(dispatch::execute(&mut conn,"finish",json!({"id":1,"token":current,"lease":after_crash["lease"]}),later+62_000).unwrap(),true);
    assert!(reminders::entry(&conn,"pero",1).unwrap().delivered);
    assert_eq!(dispatch::execute(&mut conn,"plan",json!({}),later+62_000).unwrap()["cancellations"][0]["dismiss"],false);
}

#[test]
fn concurrent_edit_after_claim_rejects_old_success_receipt() {
    let db=state();let mut conn=db.conn.lock().unwrap();seed(&conn,1,"none");let current=token(&conn,1);
    let claim=invoke(&mut conn,"claim",json!({"id":1,"token":current}));
    conn.execute("UPDATE reminders SET task='最新内容',time='2030-01-04 12:00:00',revision=2 WHERE id=1",[]).unwrap();
    assert_eq!(invoke(&mut conn,"finish",json!({"id":1,"token":current,"lease":claim["lease"]})),false);
    let latest=reminders::entry(&conn,"pero",1).unwrap();assert!(!latest.delivered);assert_eq!(latest.task,"最新内容");assert_eq!(latest.revision,2);
}

#[test]
fn cold_database_reopen_restores_jobs_and_keeps_receipts_without_app_handle() {
    let db=state();let conn=db.conn.lock().unwrap();seed(&conn,1,"none");let expected=token(&conn,1);
    let directory=std::env::temp_dir().join(format!("ppc-reminder-{}",uuid::Uuid::new_v4()));std::fs::create_dir(&directory).unwrap();
    let path=directory.join("peropero.db");conn.execute("VACUUM INTO ?1",[path.to_string_lossy().to_string()]).unwrap();drop(conn);
    {
        let mut cold=rusqlite::Connection::open(&path).unwrap();
        let plan=invoke(&mut cold,"plan",json!({}));assert_eq!(plan["jobs"][0]["token"],expected);
        let claim=invoke(&mut cold,"claim",json!({"id":1,"token":expected}));assert!(!claim.is_null());
    }
    {
        let mut cold=rusqlite::Connection::open(&path).unwrap();
        assert!(invoke(&mut cold,"claim",json!({"id":1,"token":expected})).is_null());
        let claim=dispatch::execute(&mut cold,"claim",json!({"id":1,"token":expected}),millis("2030-01-02 13:02:00")).unwrap();
        assert!(!claim.is_null());
    }
    std::fs::remove_file(path).unwrap();std::fs::remove_dir(directory).unwrap();
}
