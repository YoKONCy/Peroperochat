use chrono::{Duration, NaiveDateTime};
use serde_json::{json, Value};
use super::{chat_tools::ToolSession, reminders::{self, Change, Query}, tests::state, types::*};

fn time(value:&str)->NaiveDateTime {NaiveDateTime::parse_from_str(value,"%Y-%m-%d %H:%M:%S").unwrap()}
fn change(value:Value)->Change {serde_json::from_value(value).unwrap()}
fn create(conn:&mut rusqlite::Connection,task:&str,recur:&str,until:&str)->i64 {
    let now=time("2030-01-01 12:00:00");
    let draft=reminders::prepare(conn,"pero",change(json!({"action":"create","task":task,"time":"2030-01-02 12:00:00","recur":recur,"recurUntil":until})),now).unwrap();
    let tx=conn.transaction().unwrap();reminders::commit(&tx,"pero",&draft,now).unwrap();tx.commit().unwrap();
    let data=reminders::query(conn,"pero",&Query::default()).unwrap();data["entries"].as_array().unwrap().iter().find(|e|e["task"]==task).unwrap()["id"].as_i64().unwrap()
}

#[test]
fn reminder_crud_is_scoped_and_rejects_stale_edits() {
    let db=state();let mut conn=db.conn.lock().unwrap();let id=create(&mut conn,"浇花","none","");
    let now=time("2030-01-01 12:00:00");
    assert_eq!(reminders::query(&conn,"nana",&Query::default()).unwrap()["total"],0);
    assert_eq!(reminders::prepare(&conn,"nana",change(json!({"action":"delete","id":id,"expectedRevision":1})),now).unwrap_err().code,"REMINDER_NOT_FOUND");
    let update=reminders::prepare(&conn,"pero",change(json!({"action":"update","id":id,"expectedRevision":1,"task":"给窗边的花浇水"})),now).unwrap();
    {let tx=conn.transaction().unwrap();reminders::commit(&tx,"pero",&update,now).unwrap();tx.commit().unwrap();}
    assert_eq!(reminders::entry(&conn,"pero",id).unwrap().revision,2);
    assert_eq!(reminders::prepare(&conn,"pero",change(json!({"action":"delete","id":id,"expectedRevision":1})),now).unwrap_err().code,"STALE_REMINDER");
    let delete=reminders::prepare(&conn,"pero",change(json!({"action":"delete","id":id,"expectedRevision":2})),now).unwrap();
    {let tx=conn.transaction().unwrap();reminders::commit(&tx,"pero",&delete,now).unwrap();tx.commit().unwrap();}
    assert!(create(&mut conn,"新的提醒","none","")>id);
}

#[test]
fn reminder_validation_rejects_past_invalid_and_duplicate_requests() {
    let db=state();let mut conn=db.conn.lock().unwrap();let now=time("2030-01-01 12:00:00");
    for value in [json!({"action":"create","task":"过去","time":"2029-01-01 12:00:00"}),
        json!({"action":"create","task":"格式错误","time":"明天下午"}),json!({"action":"create","task":" ","time":"2030-01-02 12:00:00"}),
        json!({"action":"create","task":"规则错误","time":"2030-01-02 12:00:00","recur":"hourly"}),
        json!({"action":"create","task":"截止太早","time":"2030-01-02 12:00:00","recur":"daily","recurUntil":"2030-01-01 12:00:00"})] {
        assert!(reminders::prepare(&conn,"pero",change(value),now).is_err());
    }
    create(&mut conn,"浇花","none","");
    let draft=reminders::prepare(&conn,"pero",change(json!({"action":"create","task":"浇花","time":"2030-01-02 12:00:00"})),now).unwrap();
    {let tx=conn.transaction().unwrap();assert_eq!(reminders::commit(&tx,"pero",&draft,now).unwrap_err().code,"REMINDER_EXISTS");}
    assert_eq!(reminders::query(&conn,"pero",&Query::default()).unwrap()["total"],1);
}

#[test]
fn reminder_delivery_failure_remains_pending_and_recurring_delivery_skips_missed_periods() {
    let db=state();let mut conn=db.conn.lock().unwrap();let id=create(&mut conn,"每天喝水","daily","");let now=time("2030-01-08 13:00:00");
    assert!(!reminders::deliver(&conn,"pero",id,now,|_|Err("没有通知权限".into())).unwrap());
    let pending=reminders::entry(&conn,"pero",id).unwrap();assert!(!pending.delivered);assert_eq!(pending.revision,1);assert_eq!(pending.delivery_error.as_deref(),Some("没有通知权限"));
    assert!(reminders::deliver(&conn,"pero",id,now,|_|Ok(())).unwrap());
    let next=reminders::entry(&conn,"pero",id).unwrap();assert_eq!(next.time,"2030-01-09 12:00:00");assert!(!next.delivered);assert_eq!(next.revision,2);assert!(next.delivery_error.is_none());
    assert!(!reminders::deliver(&conn,"pero",id,now,|_|panic!("未到期提醒不应再次发送")).unwrap());
}

#[test]
fn reminder_one_shot_and_expired_recurrence_finish_only_after_success() {
    let db=state();let mut conn=db.conn.lock().unwrap();
    for (task,recur,until) in [("一次提醒","none",""),("截止提醒","weekly","2030-01-02 13:00:00")] {
        let id=create(&mut conn,task,recur,until);
        assert!(reminders::deliver(&conn,"pero",id,time("2030-01-02 12:00:00"),|_|Ok(())).unwrap());
        assert!(reminders::entry(&conn,"pero",id).unwrap().delivered);
        assert!(!reminders::deliver(&conn,"pero",id,time("2030-01-03 12:00:00"),|_|panic!("已送达提醒不应重复发送")).unwrap());
    }
    assert_eq!(reminders::query(&conn,"pero",&Query::default()).unwrap()["total"],0);
}

#[test]
fn reminder_tool_commit_rolls_back_when_a_concurrent_edit_changes_revision() {
    let db=state();let id={let mut conn=db.conn.lock().unwrap();create(&mut conn,"已有提醒","none","")};
    let future=(chrono::Local::now()+Duration::days(2)).format("%Y-%m-%d %H:%M:%S").to_string();
    let mut session=ToolSession::default();
    for (call_id,arguments) in [("create",json!({"action":"create","task":"事务草稿","time":future})),
        ("edit",json!({"action":"update","id":id,"expectedRevision":1,"task":"不应覆盖的新内容"}))] {
        let result=session.execute(&db,"pero",&ToolCall {id:call_id.into(),name:"manage_reminder".into(),arguments}).unwrap();assert_eq!(result["ok"],true);
    }
    let mut conn=db.conn.lock().unwrap();
    assert_eq!(reminders::query(&conn,"pero",&Query::default()).unwrap()["total"],1);
    conn.execute("UPDATE reminders SET revision=2,task='用户手动编辑' WHERE id=?1",[id]).unwrap();
    {let tx=conn.transaction().unwrap();assert_eq!(session.commit(&tx,"pero",&[],0).unwrap_err().code,"STALE_REMINDER");}
    assert_eq!(reminders::query(&conn,"pero",&Query::default()).unwrap()["total"],1);
    assert_eq!(reminders::entry(&conn,"pero",id).unwrap().task,"用户手动编辑");
}

#[tokio::test]
async fn reminder_react_turn_commits_once_and_replay_does_not_duplicate() {
    use super::{conversation::ConversationService, runtime::ChatRuntime, tests::{agents,mock_server,request,settings}};
    let future=(chrono::Local::now()+Duration::days(2)).format("%Y-%m-%d %H:%M:%S").to_string();
    let tools=json!({"choices":[{"message":{"content":"","tool_calls":[{"id":"clock","type":"function","function":{"name":"query_reminders","arguments":"{}"}},
        {"id":"create","type":"function","function":{"name":"manage_reminder","arguments":json!({"action":"create","task":"带伞","time":future}).to_string()}}]},"finish_reason":"tool_calls"}]}).to_string();
    let answer=json!({"choices":[{"message":{"content":"记下啦，到时提醒你带伞。"},"finish_reason":"stop"}]}).to_string();
    let (base,requests)=mock_server(vec![(200,"application/json",tools),(200,"application/json",answer)]);
    let db=state();let runtime=ChatRuntime::default();let agents=agents();let config=json!({});let settings=settings(base,false);
    let service=ConversationService {db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings};let turn=request("pero","提醒我带伞");
    let result=service.execute(turn.clone(),|_|{},|_,_|Ok(json!({}))).await.unwrap();assert_eq!(result.state["reminderChanges"],1);
    service.execute(turn,|_|{},|_,_|Ok(json!({}))).await.unwrap();
    requests.recv().unwrap();let followup=requests.recv().unwrap();assert!(followup.to_string().contains("utcOffset"));
    let conn=db.conn.lock().unwrap();assert_eq!(reminders::query(&conn,"pero",&Query::default()).unwrap()["total"],1);
    assert_eq!(reminders::query(&conn,"nana",&Query::default()).unwrap()["total"],0);
}
