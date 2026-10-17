use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{mpsc, Mutex};
use rusqlite::Connection;
use serde_json::{json, Value};
use super::{context, conversation::ConversationService, identity, provider::ModelSettings, repository, runtime::ChatRuntime, types::*};
use crate::db::DbState;

pub(super) fn database() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE messages(id INTEGER PRIMARY KEY,role TEXT,content TEXT,timestamp INTEGER,agent_id TEXT);
        CREATE TABLE memories(id INTEGER PRIMARY KEY,content TEXT,tags TEXT,importance INTEGER,timestamp INTEGER,msg_timestamp INTEGER,type TEXT,source TEXT,agent_id TEXT);
        CREATE TABLE reminders(id INTEGER PRIMARY KEY,task TEXT,time TEXT,timestamp INTEGER,agent_id TEXT,delivered INTEGER DEFAULT 0,recur TEXT DEFAULT 'none',recur_until TEXT);
        CREATE TABLE topics(id INTEGER PRIMARY KEY,topic TEXT,revealed INTEGER,timestamp INTEGER,agent_id TEXT,time TEXT);").unwrap();
    conn
}

pub(super) fn agents() -> Value {
    json!({"pero":{"name":"佩洛","persona_prompt":"你是佩洛"},"nana":{"name":"娜娜","persona_prompt":"你是娜娜"}})
}

pub(super) fn state() -> DbState {
    let mut conn = database();
    repository::migrate(&mut conn,&[],&agents()).unwrap();
    for id in ["pero","nana"] { repository::ensure_thread(&conn,&repository::desktop_thread(id),id,"desktop").unwrap(); }
    DbState { conn:Mutex::new(conn),diary:super::diary::DiaryStore::temporary(),mode:std::sync::OnceLock::new() }
}

pub(super) fn request(agent_id: &str, content: &str) -> TurnRequest {
    TurnRequest { request_id:uuid::Uuid::new_v4().to_string(),thread_id:repository::desktop_thread(agent_id),agent_id:agent_id.into(),
        content:Some(json!(content)),regenerate_message_id:None,reply_to_message_id:None,ephemeral:false }
}

pub(super) fn settings(base: String, stream: bool) -> ModelSettings {
    ModelSettings { base,api_key:"test-key".into(),model:"测试模型".into(),temperature:Some(0.6),top_p:Some(0.8),
        frequency_penalty:Some(0.2),presence_penalty:Some(0.3),stream,memory_rounds:40,..Default::default() }
}

#[test]
fn persistent_mode_fences_local_storage_and_runtime_even_after_returning_to_local() {
    let path=std::env::temp_dir().join(format!("ppc-local-gate-{}",uuid::Uuid::new_v4())).join("runtime.sqlite");
    let mode=crate::mode::Coordinator::initialize(path).unwrap();mode.activate().unwrap();
    let old_token=mode.token().unwrap();
    let db=state();assert!(db.mode.set(mode.clone()).is_ok());let runtime=ChatRuntime::with_mode(mode.clone());
    let request_id=uuid::Uuid::new_v4().to_string();let lease=runtime.reserve("desktop:pero",&request_id).unwrap();
    {let conn=super::db_lock(&db).unwrap();conn.execute("INSERT INTO topics(topic,agent_id) VALUES('保留的独立记录','pero')",[]).unwrap();}
    mode.begin_switch(crate::mode::Target::Remote {profile_id:uuid::Uuid::new_v4().to_string()}).unwrap();
    runtime.cancel_all().unwrap();assert!(*lease.cancel.borrow());assert!(super::db_lock(&db).is_err());assert!(runtime.maintenance().is_err());
    drop(lease);assert!(runtime.is_idle().unwrap());mode.activate().unwrap();assert!(super::db_lock(&db).is_err());
    mode.begin_switch(crate::mode::Target::Standalone).unwrap();mode.activate().unwrap();
    crate::mode::LOCAL_TOKEN.sync_scope(old_token,|| {
        assert!(super::db_lock(&db).is_err());assert!(runtime.maintenance().is_err());
    });
    let conn=super::db_lock(&db).unwrap();let count:i64=conn.query_row("SELECT COUNT(*) FROM topics",[],|r|r.get(0)).unwrap();assert_eq!(count,1);
}

// 使用真实 TCP 和 reqwest 验证请求体、流式拆包、HTTP 错误及持久化，不访问外网。
pub(super) fn mock_server(responses: Vec<(u16,&'static str,String)>) -> (String,mpsc::Receiver<Value>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (sender,receiver) = mpsc::channel();
    std::thread::spawn(move || {
        for (status,kind,body) in responses {
            let (mut socket,_) = listener.accept().unwrap();
            socket.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
            let mut reader = BufReader::new(socket.try_clone().unwrap());
            let mut length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" { break; }
                if let Some((name,value)) = line.split_once(':') {
                    if name.eq_ignore_ascii_case("content-length") { length = value.trim().parse().unwrap(); }
                }
            }
            let mut bytes = vec![0;length];
            reader.read_exact(&mut bytes).unwrap();
            sender.send(if bytes.is_empty() {Value::Null} else {serde_json::from_slice(&bytes).unwrap()}).unwrap();
            write!(socket,"HTTP/1.1 {status} OK\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).unwrap();
            // 故意拆在中文 UTF-8 的字节内部，客户端必须通过事件缓冲器恢复。
            for chunk in body.as_bytes().chunks(7) { if socket.write_all(chunk).is_err() { break; } }
        }
    });
    (format!("http://{address}/v1"),receiver)
}

fn sse(text: &str) -> String {
    format!("data: {}\n\ndata: [DONE]\n\n",json!({"choices":[{"delta":{"content":text}}]}))
}

#[test]
fn migration_is_one_time_and_preserves_roles_pairs_and_group_identity() {
    let mut conn = database();
    conn.execute("INSERT INTO messages VALUES(1,'user','你好',100,'pero'),(2,'assistant','你好主人',101,'pero'),
        (3,'user','娜娜你好',102,'nana'),(4,'assistant','__loading__',103,'nana')",[]).unwrap();
    let group = vec![json!({"role":"user","name":"User","content":"大家好","timestamp":104}),
        json!({"role":"assistant","name":"娜娜","content":"你好","timestamp":105})];
    repository::migrate(&mut conn,&group,&agents()).unwrap();
    let messages = repository::snapshot(&conn,"desktop:pero").unwrap().messages;
    assert_eq!(messages.len(),2);
    assert_eq!(messages[0].pair_id,messages[1].pair_id);
    assert_eq!(messages[1].timestamp,101);
    assert_eq!(repository::snapshot(&conn,"desktop:nana").unwrap().messages.len(),1);
    assert_eq!(repository::snapshot(&conn,repository::FAMILY_THREAD).unwrap().messages[1].agent_id,"nana");
    repository::migrate(&mut conn,&group,&agents()).unwrap();
    assert_eq!(repository::snapshot(&conn,"desktop:pero").unwrap().messages.len(),2);
    conn.execute("DELETE FROM chat_messages",[]).unwrap();
    repository::migrate(&mut conn,&group,&agents()).unwrap();
    assert!(repository::snapshot(&conn,"desktop:pero").unwrap().messages.is_empty());
}

#[tokio::test]
async fn full_conversation_regeneration_idempotence_and_pair_deletion() {
    let first_reply = "主人你好🐱<MEMORY>{\"content\":\"用户喜欢猫\",\"tags\":[\"喜好\"],\"importance\":8,\"type\":\"preference\"}</MEMORY>";
    let (base,requests) = mock_server(vec![(200,"text/event-stream",sse(first_reply)),
        (401,"application/json",json!({"error":{"message":"test-key 已失效"}}).to_string()),
        (200,"text/event-stream",sse("重新生成的回复"))]);
    let settings = settings(base,true);
    let db = state();
    let runtime = ChatRuntime::default();
    let agents = agents();
    let config = json!({});
    let service = ConversationService { db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings };
    let events = Mutex::new(Vec::new());
    let original = request("pero","你好");
    let result = service.execute(original.clone(),|event| events.lock().unwrap().push(event),|_,_| Ok(json!({}))).await.unwrap();
    let id = result.message.id;
    let timestamp = result.message.timestamp;
    assert_eq!(result.message.content,"主人你好🐱");
    assert_eq!(result.message.status,"active");
    assert_eq!(result.snapshot.messages.len(),2);
    assert_eq!(events.lock().unwrap().iter().filter(|e| e.kind == "done").count(),1);
    let wire = requests.recv().unwrap();
    assert_eq!(wire["messages"].as_array().unwrap().iter().filter(|m| m["role"] == "user").count(),1);
    assert_eq!(wire["top_p"],0.8);
    assert_eq!(wire["frequency_penalty"],0.2);
    assert_eq!(wire["presence_penalty"],0.3);
    assert_eq!(wire["stream"],true);
    assert_eq!(service.execute(original, |_| {}, |_,_| Ok(json!({}))).await.unwrap().message.id,id);
    let mut regenerate = request("pero","");
    regenerate.content = None;
    regenerate.regenerate_message_id = Some(id);
    let error = service.execute(regenerate.clone(), |_| {}, |_,_| Ok(json!({}))).await.unwrap_err();
    assert_eq!(error.code,"UPSTREAM_ERROR");
    assert!(!error.message.contains("test-key"));
    assert_eq!(repository::message(&db.conn.lock().unwrap(),id).unwrap().content,"主人你好🐱");
    requests.recv().unwrap();
    regenerate.request_id = uuid::Uuid::new_v4().to_string();
    let regenerated = service.execute(regenerate, |_| {}, |_,_| Ok(json!({}))).await.unwrap();
    assert_eq!(regenerated.message.id,id);
    assert_eq!(regenerated.message.revision,2);
    assert_eq!(regenerated.message.timestamp,timestamp);
    assert_eq!(regenerated.snapshot.messages.len(),2);
    let wire = requests.recv().unwrap();
    assert_eq!(wire["messages"].as_array().unwrap().iter().filter(|m| m["role"] == "user").count(),1);
    assert!(!wire.to_string().contains("主人你好🐱"));
    let mut conn = db.conn.lock().unwrap();
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM chat_message_revisions",[],|r| r.get::<_,i64>(0)).unwrap(),1);
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM memories",[],|r| r.get::<_,i64>(0)).unwrap(),0);
    super::diary::apply(&conn,"nana",&super::diary::Draft {content:"其他角色的事实".into(),date:None,tags:vec![],id:None,expected_revision:None},&[],timestamp).unwrap();
    repository::delete_message(&mut conn,"desktop:pero",id,true).unwrap();
    assert!(repository::snapshot(&conn,"desktop:pero").unwrap().messages.is_empty());
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM memories WHERE agent_id='nana'",[],|r| r.get::<_,i64>(0)).unwrap(),1);
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM chat_messages WHERE status='deleted'",[],|r| r.get::<_,i64>(0)).unwrap(),2);
}

#[tokio::test]
async fn eof_is_failure_and_failed_partial_never_enters_context() {
    let partial = format!("data: {}\n\n",json!({"choices":[{"delta":{"content":"未完成的回复"}}]}));
    let (base,requests) = mock_server(vec![(200,"text/event-stream",partial),
        (200,"application/json",json!({"choices":[{"message":{"content":"恢复成功"},"finish_reason":"stop"}]}).to_string())]);
    let mut settings = settings(base,true);
    let db = state(); let runtime = ChatRuntime::default(); let agents = agents(); let config = json!({});
    let service = ConversationService { db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings };
    let error = service.execute(request("pero","第一次"), |_| {}, |_,_| Ok(json!({}))).await.unwrap_err();
    assert_eq!(error.code,"STREAM_INTERRUPTED");
    requests.recv().unwrap();
    let snapshot = repository::snapshot(&db.conn.lock().unwrap(),"desktop:pero").unwrap();
    assert_eq!(snapshot.messages[1].content,"未完成的回复");
    assert_eq!(snapshot.messages[1].status,"failed");
    settings.stream = false;
    let service = ConversationService { db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings };
    assert_eq!(service.execute(request("pero","第二次"), |_| {}, |_,_| Ok(json!({}))).await.unwrap().message.content,"恢复成功");
    let wire = requests.recv().unwrap();
    assert_eq!(wire["stream"],false);
    assert!(!wire.to_string().contains("未完成的回复"));
}

#[tokio::test]
async fn cancellation_aborts_inflight_io_and_releases_the_thread() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let settings = settings(format!("http://{}/v1",listener.local_addr().unwrap()),true);
    let (started,accepted) = mpsc::channel();
    std::thread::spawn(move || {
        let (mut socket,_) = listener.accept().unwrap();
        let mut reader = BufReader::new(socket.try_clone().unwrap());
        let mut length = 0;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            if line == "\r\n" { break; }
            if let Some((name,value)) = line.split_once(':') {
                if name.eq_ignore_ascii_case("content-length") { length = value.trim().parse().unwrap(); }
            }
        }
        let mut body = vec![0;length];
        reader.read_exact(&mut body).unwrap();
        let _ = started.send(());
        let _ = socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\n\r\n");
        std::thread::sleep(std::time::Duration::from_secs(2));
    });
    let db = state(); let runtime = ChatRuntime::default(); let agents = agents(); let config = json!({});
    let service = ConversationService { db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings };
    let request = request("pero","停止测试");
    let request_id = request.request_id.clone();
    let result = tokio::join!(
        service.execute(request, |_| {}, |_,_| Ok(json!({}))),
        async {
            tokio::task::spawn_blocking(move || accepted.recv_timeout(std::time::Duration::from_secs(3)).unwrap()).await.unwrap();
            runtime.cancel(&request_id).unwrap();
        }
    ).0;
    assert_eq!(result.unwrap_err().code,"CANCELLED");
    assert_eq!(repository::snapshot(&db.conn.lock().unwrap(),"desktop:pero").unwrap().messages[1].status,"cancelled");
    assert!(runtime.reserve("desktop:pero","后续请求").is_ok());
}

#[test]
fn thread_ownership_group_perspective_and_crash_recovery() {
    let db = state();
    let mut conn = db.conn.lock().unwrap();
    let mut wrong = request("nana","不能串角色"); wrong.thread_id = "desktop:pero".into();
    assert_eq!(repository::prepare_turn(&mut conn,&wrong,"娜娜").unwrap_err().code,"INVALID_THREAD");
    let input = repository::insert_message(&conn,repository::FAMILY_THREAD,"群聊轮次","user","","User",&json!("大家好"),1,"active").unwrap();
    repository::insert_message(&conn,repository::FAMILY_THREAD,"群聊轮次","assistant","pero","佩洛",&json!("你好"),2,"active").unwrap();
    let (model,manifest) = context::compile(&conn,context::ContextInput { thread:&repository::thread(&conn,repository::FAMILY_THREAD).unwrap(),
        agent_id:"nana",identity:&agents(),config:&json!({}),current:&input.content,before_id:None,rounds:40,budget:16000,diary:&[] }).unwrap();
    assert_eq!(model.last().unwrap().role,"user");
    assert_eq!(model.last().unwrap().content,"[佩洛]：你好");
    assert_eq!(model.iter().filter(|m| m.content == "大家好").count(),1);
    assert_eq!(manifest["historyMessageIds"].as_array().unwrap().len(),2);
    let prepared = repository::prepare_turn(&mut conn,&request("pero","断电前的输入"),"佩洛").unwrap();
    repository::update_partial(&conn,prepared.assistant.id,"已收到一部分").unwrap();
    repository::migrate(&mut conn,&[],&agents()).unwrap();
    let message = repository::message(&conn,prepared.assistant.id).unwrap();
    assert_eq!(message.status,"interrupted");
    assert_eq!(message.content,"已收到一部分");
}

#[test]
fn runtime_leases_and_strict_metadata_validation() {
    let runtime = ChatRuntime::default();
    runtime.cancel("提前取消").unwrap();
    let early = runtime.reserve("desktop:pero","提前取消").unwrap();
    assert!(*early.cancel.borrow());
    assert!(runtime.reserve("desktop:pero","并发请求").is_err());
    assert!(runtime.maintenance().is_err());
    drop(early);
    let maintenance = runtime.maintenance().unwrap();
    assert!(runtime.reserve("desktop:nana","维护期间").is_err());
    drop(maintenance);
    assert!(runtime.reserve("desktop:nana","维护之后").is_ok());
    assert_eq!(context::visible_text("<DIV>普通示例</DIV><MEMORY>{}</MEMORY>"),"<DIV>普通示例</DIV>");
    assert!(identity::normalize_agents(json!({"pero":{"name":"佩洛","system_prompt":[]}})).is_err());
}

#[test]
fn role_save_derives_texts_atomically_and_preserves_other_interactions() {
    let db=state();
    let mut conn=db.conn.lock().unwrap();
    identity::set_value_in(&conn,"ppc.agents",&json!(agents().to_string())).unwrap();
    identity::set_value_in(&conn,"ppc.pero.waifu.texts",&json!(json!({"idleMessages_03":"旧台词","click_head_01":"摸摸"}).to_string())).unwrap();
    let updated=identity::save_agent_in(&mut conn,"pero",json!({"name":"佩洛","welcome_message":"欢迎回来","idle_messages_text":" 第一句 \n\n 第二句 "}),false).unwrap();
    assert_eq!(updated["pero"]["idle_messages"],json!(["第一句","第二句"]));
    assert!(updated["pero"].get("idle_messages_text").is_none());
    let config=identity::snapshot_in(&conn).unwrap();
    let texts:Value=serde_json::from_str(config["ppc.pero.waifu.texts"].as_str().unwrap()).unwrap();
    assert_eq!(texts["welcome"],"欢迎回来");
    assert_eq!(texts["click_head_01"],"摸摸");
    assert!(texts.get("idleMessages_03").is_none());
    assert!(identity::save_agent_in(&mut conn,"pero",json!({"name":"覆盖"}),true).is_err());
    assert!(identity::save_agent_in(&mut conn,"../bad",json!({"name":"非法"}),false).is_err());
    assert_eq!(identity::snapshot_in(&conn).unwrap(),config);
}

#[test]
fn builtin_persona_update_only_changes_requested_roles_once() {
    let db=state();let mut conn=db.conn.lock().unwrap();
    let before=json!({"pero":{"name":"自定义显示名","persona_prompt":"旧 Pero 人设","avatar":"我的头像","system_prompt":"保留系统设定"},
        "nana":{"name":"Nana","persona_prompt":"旧 Nana 人设"},"custom":{"name":"其他角色","persona_prompt":"保留自定义人设"}});
    identity::set_value_in(&conn,"ppc.agents",&json!(before.to_string())).unwrap();
    identity::update_builtin_personas(&mut conn).unwrap();
    let mut updated=identity::agents_from(&identity::snapshot_in(&conn).unwrap()).unwrap();
    assert!(updated["pero"]["persona_prompt"].as_str().unwrap().contains("可爱但克制"));
    assert!(updated["nana"]["persona_prompt"].as_str().unwrap().contains("不禁夸"));
    assert!(!updated.to_string().contains("owner_appellation"));assert!(!updated.to_string().contains("owner_name"));
    assert_eq!(updated["pero"]["name"],before["pero"]["name"]);assert_eq!(updated["pero"]["avatar"],"我的头像");
    assert_eq!(updated["pero"]["system_prompt"],"保留系统设定");assert_eq!(updated["custom"]["persona_prompt"],"保留自定义人设");
    updated["pero"]["persona_prompt"]=json!("更新后再次编辑的人设");updated.as_object_mut().unwrap().remove("nana");
    identity::set_value_in(&conn,"ppc.agents",&json!(updated.to_string())).unwrap();
    identity::update_builtin_personas(&mut conn).unwrap();
    let repeated=identity::agents_from(&identity::snapshot_in(&conn).unwrap()).unwrap();
    assert_eq!(repeated["pero"]["persona_prompt"],"更新后再次编辑的人设");assert!(repeated.get("nana").is_none());
}

#[test]
fn builtin_avatar_defaults_only_fill_missing_fields() {
    let projected=identity::normalize_agents(json!({"pero":{"name":"Pero"},"nana":{"name":"Nana"},"custom":{"name":"自定义"}})).unwrap();
    assert_eq!(projected["pero"]["avatar"],"/avatars/pero.jpg");
    assert_eq!(projected["nana"]["avatar"],"/avatars/nana.jpg");
    assert!(projected["custom"].get("avatar").is_none());
    let customized=identity::normalize_agents(json!({"pero":{"name":"Pero","avatar":"data:image/png;base64,自定义头像"},"nana":{"name":"Nana","avatar":""}})).unwrap();
    assert_eq!(customized["pero"]["avatar"],"data:image/png;base64,自定义头像");
    assert_eq!(customized["nana"]["avatar"],"");
    assert_eq!(identity::normalize_agents(projected.clone()).unwrap(),projected);
}

#[tokio::test]
async fn group_batch_owns_order_input_idempotence_and_whole_batch_lease() {
    use super::group::{self,GroupRequest};
    let (base,requests)=mock_server(vec![(200,"text/event-stream",sse("娜娜先回复")),(200,"text/event-stream",sse("佩洛接着回复"))]);
    let db=state();
    group::initialize(&db.conn.lock().unwrap()).unwrap();
    let runtime=ChatRuntime::default();let agents=agents();let settings=settings(base,true);
    let config=json!({"ppc.group.replySettings":json!([{"id":"nana","enabled":true},{"id":"pero","enabled":true}]).to_string()});
    let service=ConversationService { db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings };
    let request_id=uuid::Uuid::new_v4().to_string();
    let make_request=||GroupRequest { request_id:request_id.clone(),thread_id:repository::FAMILY_THREAD.into(),content:json!("大家好") };
    let speakers=Mutex::new(Vec::new());
    let result=group::execute(&service,make_request(),|event| {
        assert_eq!(event.request_id,request_id);
        assert!(runtime.reserve(repository::FAMILY_THREAD,"插入请求").is_err());
        assert!(runtime.maintenance().is_err());
        if event.kind=="started" { speakers.lock().unwrap().push(event.agent_id); }
    },|_,_|Ok(json!({}))).await.unwrap();
    assert_eq!(*speakers.lock().unwrap(),vec!["nana","pero"]);
    assert_eq!(result.snapshot.messages.len(),3);
    assert_eq!(result.snapshot.messages.iter().filter(|m|m.role=="user").count(),1);
    requests.recv().unwrap();
    let second=requests.recv().unwrap();
    assert!(second.to_string().contains("娜娜先回复"));
    let repeated=group::execute(&service,make_request(),|_|{},|_,_|Ok(json!({}))).await.unwrap();
    assert_eq!(repeated.message.id,result.message.id);
    assert_eq!(repeated.snapshot.messages.len(),3);
    assert!(runtime.reserve(repository::FAMILY_THREAD,"后续请求").is_ok());
}

#[tokio::test]
async fn group_cancellation_stops_remaining_members_without_duplicating_input() {
    use super::group::{self,GroupRequest};
    let (base,requests)=mock_server(vec![(200,"text/event-stream",sse("第一位回复"))]);
    let db=state();group::initialize(&db.conn.lock().unwrap()).unwrap();
    let runtime=ChatRuntime::default();let agents=agents();let settings=settings(base,true);let config=json!({});
    let service=ConversationService { db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings };
    let request_id=uuid::Uuid::new_v4().to_string();
    let error=group::execute(&service,GroupRequest { request_id:request_id.clone(),thread_id:repository::FAMILY_THREAD.into(),content:json!("大家好") },|event| {
        if event.kind=="done" { runtime.cancel(&request_id).unwrap(); }
    },|_,_|Ok(json!({}))).await.unwrap_err();
    assert_eq!(error.code,"CANCELLED");
    requests.recv().unwrap();
    let snapshot=repository::snapshot(&db.conn.lock().unwrap(),repository::FAMILY_THREAD).unwrap();
    assert_eq!(snapshot.messages.len(),2);
    assert_eq!(snapshot.messages[1].status,"active");
}

#[test]
fn group_settings_preserve_disabled_members_and_normalize_role_changes() {
    let settings=super::group::normalize(&agents(),&json!([{"id":"removed","enabled":true},{"id":"pero","enabled":false},{"id":"pero","enabled":true}])).unwrap();
    assert_eq!(settings.len(),2);
    assert_eq!(settings[0].id,"pero");
    assert!(!settings[0].enabled);
    assert_eq!(settings[1].id,"nana");
}
