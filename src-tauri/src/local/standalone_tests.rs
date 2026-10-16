use serde_json::{json,Value};
use super::{chat_tools::{self,ToolSession},conversation::ConversationService,diary,identity,model_config,prompts,provider_stream::Accumulator,
    repository,runtime::ChatRuntime,tests::{agents,mock_server,request,settings,state},types::*,wire};

fn tool(id:&str,name:&str,arguments:Value)->ToolCall {ToolCall {id:id.into(),name:name.into(),arguments}}
fn answer(content:&str)->String {json!({"choices":[{"message":{"content":content},"finish_reason":"stop"}]}).to_string()}
fn calls(calls:&[ToolCall])->String {
    json!({"choices":[{"message":{"content":"","reasoning_content":"记录真实互动",
        "tool_calls":calls.iter().map(|c|json!({"id":c.id,"type":"function","function":{"name":c.name,"arguments":c.arguments.to_string()}})).collect::<Vec<_>>()},"finish_reason":"tool_calls"}]}).to_string()
}
fn draft(content:&str,date:&str)->diary::Draft {diary::Draft {content:content.into(),date:Some(date.into()),tags:vec!["喜好".into()],id:None,expected_revision:None}}

#[tokio::test]
async fn react_commits_diary_and_unified_character_tool_only_with_final_reply() {
    let writes=[tool("diary-1","write_diary",json!({"content":"今天知道主人喜欢猫，也喜欢抹茶。","date":"2026-10-16","tags":["喜好"]})),
        tool("state-1","update_character",json!({"state":{"mood":"开心","mind":"想多陪陪主人"},"interactions":{"idle":["来聊聊嘛"],"back":["欢迎回来"],"click":{"head":["摸摸头"],"default":["在呢"]}}}))];
    let (base,requests)=mock_server(vec![(200,"application/json",calls(&writes)),(200,"application/json",answer("我记住啦，下次一起喝抹茶！"))]);
    let db=state();let runtime=ChatRuntime::default();let agents=agents();let config=json!({});let settings=settings(base,false);
    let service=ConversationService {db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings};
    let result=service.execute(request("pero","我喜欢猫和抹茶"),|_|{},|_,_|Ok(json!({}))).await.unwrap();
    assert_eq!(result.message.content,"我记住啦，下次一起喝抹茶！");
    assert_eq!(result.state["diaryWrites"],1);
    assert_eq!(result.message.reasoning,"记录真实互动");assert_eq!(result.message.usage["steps"].as_array().unwrap().len(),2);
    let first=requests.recv().unwrap();assert_eq!(first["tools"].as_array().unwrap().len(),5);
    assert_eq!(first["tools"][2]["function"]["name"],"update_character");
    let second=requests.recv().unwrap();let messages=second["messages"].as_array().unwrap();
    assert_eq!(messages.iter().filter(|m|m["role"]=="tool").count(),2);
    assert!(second.to_string().contains("after_assistant_persisted"));
    let conn=db.conn.lock().unwrap();
    let entries=db.diary.query(&conn,"pero",&diary::Query {keyword:"抹茶".into(),..Default::default()}).unwrap();
    assert_eq!(entries.len(),1);assert_eq!(entries[0].source_message_ids,vec![result.snapshot.messages[0].id,result.message.id]);
    assert!(db.diary.query(&conn,"nana",&Default::default()).unwrap().is_empty());
    let saved=identity::snapshot_in(&conn).unwrap();assert_eq!(saved["ppc.pero.mood"],"开心");
    let texts:Value=serde_json::from_str(saved["ppc.pero.waifu.texts"].as_str().unwrap()).unwrap();
    assert_eq!(texts["idleMessages_01"],"来聊聊嘛");assert_eq!(texts["click_head_01"],"摸摸头");assert_eq!(texts["visibilityBack"],json!(["欢迎回来"]));
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM chat_tool_calls",[],|r|r.get::<_,i64>(0)).unwrap(),2);
}

#[tokio::test]
async fn tool_drafts_are_discarded_on_upstream_failure_and_tool_limit() {
    for limit in [false,true] {
        let writes=[tool("write","write_diary",json!({"content":"不能遗留的草稿"})),tool("update","update_character",json!({"state":{"mood":"不能提交"}})),
            tool("reminder","manage_reminder",json!({"action":"create","task":"不能留下的提醒","time":(chrono::Local::now()+chrono::Duration::days(2)).format("%Y-%m-%d %H:%M:%S").to_string()}))];
        let mut replies=vec![(200,"application/json",calls(&writes))];
        if !limit {replies.push((500,"application/json",json!({"error":{"message":"test-key 不可用"}}).to_string()));}
        let (base,requests)=mock_server(replies);let db=state();let runtime=ChatRuntime::default();let agents=agents();let config=json!({});let mut settings=settings(base,false);
        if limit {settings.max_steps=1;}
        let service=ConversationService {db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings};
        let error=service.execute(request("pero","失败场景"),|_|{},|_,_|Ok(json!({}))).await.unwrap_err();
        assert_eq!(error.code,if limit {"TOOL_LIMIT"} else {"UPSTREAM_ERROR"});assert!(!error.message.contains("test-key"));
        requests.recv().unwrap();if !limit {requests.recv().unwrap();}
        let conn=db.conn.lock().unwrap();assert!(db.diary.query(&conn,"pero",&Default::default()).unwrap().is_empty());
        assert!(identity::snapshot_in(&conn).unwrap()["ppc.pero.mood"].is_null());
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM chat_tool_calls",[],|r|r.get::<_,i64>(0)).unwrap(),0);
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM reminders",[],|r|r.get::<_,i64>(0)).unwrap(),0);
    }
}

#[tokio::test]
async fn cancellation_after_tool_dispatch_discards_staged_writes() {
    let writes=[tool("write","write_diary",json!({"content":"被取消的草稿"})),
        tool("reminder","manage_reminder",json!({"action":"create","task":"被取消的提醒草稿","time":(chrono::Local::now()+chrono::Duration::days(2)).format("%Y-%m-%d %H:%M:%S").to_string()}))];
    let (base,requests)=mock_server(vec![(200,"application/json",calls(&writes))]);
    let db=state();let runtime=ChatRuntime::default();let agents=agents();let config=json!({});let settings=settings(base,false);
    let service=ConversationService {db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings};let turn=request("pero","停止");let id=turn.request_id.clone();
    let error=service.execute(turn,|event| {if event.kind=="tool" {runtime.cancel(&id).unwrap();}},|_,_|Ok(json!({}))).await.unwrap_err();
    assert_eq!(error.code,"CANCELLED");requests.recv().unwrap();
    let conn=db.conn.lock().unwrap();assert!(db.diary.query(&conn,"pero",&Default::default()).unwrap().is_empty());
    assert_eq!(repository::snapshot(&conn,"desktop:pero").unwrap().messages[1].status,"cancelled");
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM reminders",[],|r|r.get::<_,i64>(0)).unwrap(),0);
}

#[test]
fn tools_are_scoped_validated_and_idempotent() {
    let db=state();let mut session=ToolSession::default();
    let call=tool("same","write_diary",json!({"content":"只写一次"}));
    assert_eq!(session.execute(&db,"pero",&call).unwrap()["ok"],true);
    assert_eq!(session.execute(&db,"pero",&call).unwrap()["ok"],true);
    assert_eq!(session.traces.len(),1);
    assert!(session.execute(&db,"pero",&tool("same","write_diary",json!({"content":"偷换内容"}))).is_err());
    assert_eq!(session.execute(&db,"pero",&tool("foreign","write_diary",json!({"agentId":"nana","content":"越权"}))).unwrap()["ok"],false);
    assert_eq!(session.execute(&db,"pero",&tool("unknown","execute_code",json!({}))).unwrap()["ok"],false);
    let conn=db.conn.lock().unwrap();session.commit(&conn,"pero",&[],1).unwrap();
    assert_eq!(db.diary.query(&conn,"pero",&Default::default()).unwrap().len(),1);
    assert!(db.diary.query(&conn,"nana",&Default::default()).unwrap().is_empty());
}

#[test]
fn diary_revision_date_text_index_and_source_deletion_are_consistent() {
    let db=state();let mut conn=db.conn.lock().unwrap();
    let id=diary::apply(&conn,"pero",&draft("主人喜欢抹茶","2026-10-15"),&[10,11],1).unwrap();
    assert_eq!(db.diary.query(&conn,"pero",&diary::Query {keyword:"抹茶".into(),..Default::default()}).unwrap().len(),1);
    let update=diary::Draft {id:Some(id),expected_revision:Some(1),date:None,..draft("主人现在更喜欢咖啡","2026-10-16")};
    assert!(diary::apply(&conn,"nana",&update,&[],2).is_err());
    diary::apply(&conn,"pero",&update,&[20,21],2).unwrap();assert!(diary::apply(&conn,"pero",&update,&[],3).is_err());
    let entries=db.diary.query(&conn,"pero",&diary::Query {keyword:"咖啡".into(),from:Some("2026-10-15".into()),to:Some("2026-10-15".into()),..Default::default()}).unwrap();
    assert_eq!(entries.len(),1);assert_eq!(entries[0].revision,2);assert_eq!(entries[0].date,"2026-10-15");
    assert!(db.diary.query(&conn,"pero",&diary::Query {keyword:"抹茶".into(),..Default::default()}).unwrap().is_empty());
    {let tx=conn.transaction().unwrap();diary::remove_sources(&tx,&[21]).unwrap();tx.commit().unwrap();}
    assert!(db.diary.query(&conn,"pero",&Default::default()).unwrap().is_empty());
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM memory_tdb_pending",[],|r|r.get::<_,i64>(0)).unwrap(),0);
}

#[test]
fn diary_persists_after_reopen_and_retries_pending_sync_after_storage_failure() {
    let db=state();let conn=db.conn.lock().unwrap();
    diary::apply(&conn,"pero",&draft("重启以后仍记得抹茶","2026-10-16"),&[],1).unwrap();
    let root=std::env::temp_dir().join(format!("ppc-reopen-{}",uuid::Uuid::new_v4()));
    std::fs::write(&root,"临时阻挡目录").unwrap();
    let store=diary::DiaryStore::new(root.clone());
    assert!(store.query(&conn,"pero",&Default::default()).is_err());
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM memory_tdb_pending",[],|r|r.get::<_,i64>(0)).unwrap(),1);
    std::fs::remove_file(&root).unwrap();
    assert_eq!(store.query(&conn,"pero",&Default::default()).unwrap().len(),1);drop(store);
    let reopened=diary::DiaryStore::new(root.clone());
    assert_eq!(reopened.query(&conn,"pero",&Default::default()).unwrap()[0].content,"重启以后仍记得抹茶");
    diary::delete(&conn,"pero",1).unwrap();assert!(reopened.query(&conn,"pero",&Default::default()).unwrap().is_empty());
    drop(reopened);std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn failed_diary_commit_rolls_back_character_and_reply_atomically() {
    let db=state();{
        let conn=db.conn.lock().unwrap();diary::apply(&conn,"pero",&draft("原日记","2026-10-16"),&[],1).unwrap();
    }
    let writes=[tool("state","update_character",json!({"state":{"mood":"不应提交"}})),
        tool("revise","write_diary",json!({"id":1,"expectedRevision":1,"content":"本轮修订"}))];
    let (base,requests)=mock_server(vec![(200,"application/json",calls(&writes)),(200,"application/json",answer("提交会冲突"))]);
    let runtime=ChatRuntime::default();let agents=agents();let config=json!({});let settings=settings(base,false);
    let service=ConversationService {db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings};
    let result=service.execute(request("pero","修改日记"),|event| {
        if event.kind=="step" {let conn=db.conn.lock().unwrap();conn.execute("UPDATE memories SET revision=2,content='其他会话的合法修订' WHERE id=1",[]).unwrap();diary::dirty(&conn,"pero").unwrap();}
    },|_,_|Ok(json!({}))).await;
    assert_eq!(result.unwrap_err().code,"STALE_DIARY");requests.recv().unwrap();requests.recv().unwrap();
    let conn=db.conn.lock().unwrap();assert!(identity::snapshot_in(&conn).unwrap()["ppc.pero.mood"].is_null());
    assert_eq!(db.diary.query(&conn,"pero",&Default::default()).unwrap()[0].content,"其他会话的合法修订");
    assert_eq!(repository::snapshot(&conn,"desktop:pero").unwrap().messages[1].status,"failed");
}

#[tokio::test]
async fn changed_request_and_deleted_reply_cannot_replay_or_resurrect() {
    let (base,requests)=mock_server(vec![(200,"application/json",answer("已完成"))]);
    let db=state();let runtime=ChatRuntime::default();let agents=agents();let config=json!({});let settings=settings(base,false);
    let service=ConversationService {db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings};let turn=request("pero","原输入");
    let result=service.execute(turn.clone(),|_|{},|_,_|Ok(json!({}))).await.unwrap();requests.recv().unwrap();
    let mut changed=turn.clone();changed.content=Some(json!("替换输入"));
    assert_eq!(service.execute(changed,|_|{},|_,_|Ok(json!({}))).await.unwrap_err().code,"REQUEST_EXISTS");
    {let mut conn=db.conn.lock().unwrap();diary::apply(&conn,"pero",&draft("来源关联","2026-10-16"),&[result.message.id],1).unwrap();
        repository::delete_message(&mut conn,"desktop:pero",result.snapshot.messages[0].id,false).unwrap();
        assert!(repository::snapshot(&conn,"desktop:pero").unwrap().messages.is_empty());
        assert!(db.diary.query(&conn,"pero",&Default::default()).unwrap().is_empty());
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM chat_messages WHERE content!='\"\"'",[],|r|r.get::<_,i64>(0)).unwrap(),0);}
    assert_eq!(service.execute(turn,|_|{},|_,_|Ok(json!({}))).await.unwrap_err().code,"REQUEST_EXISTS");
}

#[test]
fn prompt_drafts_publish_rollback_and_literal_variables_have_clear_boundaries() {
    let db=state();let mut conn=db.conn.lock().unwrap();let mut bundle=prompts::Bundle::default();
    bundle.frameworks[0].blocks[0].template="陪伴 {{character.name}}".into();
    let draft=prompts::mutate(&mut conn,0,"draft",Some(bundle.clone()),"","").unwrap();
    assert_eq!(draft.active.id,"builtin");assert_eq!(draft.revision,1);
    assert!(prompts::mutate(&mut conn,0,"publish",None,"过期发布","").is_err());
    let published=prompts::mutate(&mut conn,1,"publish",None,"新版","").unwrap();assert_ne!(published.active.id,"builtin");
    let rollback=prompts::mutate(&mut conn,2,"rollback",None,"","builtin").unwrap();assert_eq!(rollback.active.id,"builtin");
    let mut variables=prompts::variables();variables.insert("character.name".into(),json!("{{环境变量不执行}}"));
    assert_eq!(prompts::render("你好 {{character.name}}",&variables).unwrap(),"你好 {{环境变量不执行}}");
    assert!(prompts::render("{{未知}}",&variables).is_err());
    bundle.frameworks[0].blocks.retain(|b|b.source!="current");assert!(prompts::validate(&bundle).is_err());
}

#[test]
fn provider_requests_preserve_native_tools_and_reasoning_and_protect_managed_fields() {
    let mut settings=settings("https://api.deepseek.com".into(),true);settings.model="deepseek-reasoner".into();settings.reasoning_effort=Some("high".into());
    let messages=vec![ModelMessage::new("system",json!("系统")),ModelMessage::new("user",json!("你好"))];let tools=chat_tools::definitions();
    let body=wire::body(&settings,&messages,&tools).unwrap();assert_eq!(body["thinking"]["type"],"enabled");assert_eq!(body["max_tokens"],4096);
    assert!(body["max_completion_tokens"].is_null());assert!(body["temperature"].is_null());
    for format in ["responses","anthropic","gemini"] {
        settings.api_format=format.into();let body=wire::body(&settings,&messages,&tools).unwrap();
        assert!(!body["tools"].is_null());
        if format=="responses" {assert_eq!(body["tools"][0]["strict"],false);assert_eq!(body["store"],false);}
        if format=="anthropic" {assert!(body["thinking"]["budget_tokens"].as_u64().unwrap()<body["max_tokens"].as_u64().unwrap());}
        if format=="gemini" {assert_eq!(body["tools"][0]["functionDeclarations"][0]["parameters"]["type"],"OBJECT");assert!(!body.to_string().contains("additionalProperties"));}
    }
    for extra in [json!({"messages":[]}),json!({"tools":[]}),json!({"store":true}),json!({"generationConfig":{"candidateCount":2}}),json!({"max_tokens":999999})] {
        settings.extra_body=extra.as_object().unwrap().clone();assert!(model_config::validate(&settings).is_err());
    }
    settings.extra_body=json!({"generationConfig":{"topK":20}}).as_object().unwrap().clone();
    let body=wire::body(&settings,&messages,&tools).unwrap();assert_eq!(body["generationConfig"]["topK"],20);assert_eq!(body["generationConfig"]["maxOutputTokens"],4096);
    settings.headers.insert("Authorization".into(),json!("Bearer 私密"));assert_eq!(model_config::redact(&settings,"test-key Bearer 私密"),"[已隐藏] [已隐藏]");
    settings.headers.insert("Host".into(),json!("别的站点"));assert!(model_config::validate(&settings).is_err());
    let mut openai=super::provider::ModelSettings {model:"gpt-5".into(),reasoning_effort:Some("off".into()),..Default::default()};
    let mut history=messages.clone();history.push(ModelMessage {role:"assistant".into(),content:json!("历史回复"),reasoning:"切换服务前的思考".into(),..Default::default()});
    let body=wire::body(&openai,&history,&tools).unwrap();assert_eq!(body["max_completion_tokens"],4096);assert!(body["messages"][2]["reasoning_content"].is_null());
    openai.api_format="responses".into();let body=wire::body(&openai,&history,&tools).unwrap();assert_eq!(body["input"][2]["content"],"历史回复");
    openai.api_format="anthropic".into();openai.model="claude-sonnet-4-5".into();openai.reasoning_effort=Some("high".into());
    assert_eq!(wire::body(&openai,&messages,&tools).unwrap()["thinking"]["type"],"enabled");
}

#[tokio::test]
async fn streamed_chat_tools_replay_without_spurious_provider_fields() {
    let call=tool("call1","write_diary",json!({"content":"流式日记"}));
    let first=format!("data: {}\n\ndata: [DONE]\n\n",json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":call.id,"function":{"name":call.name,"arguments":call.arguments.to_string()}}]},"finish_reason":"tool_calls"}]}));
    let final_reply=format!("data: {}\n\ndata: [DONE]\n\n",json!({"choices":[{"delta":{"content":"记下来啦"},"finish_reason":"stop"}]}));
    let (base,requests)=mock_server(vec![(200,"text/event-stream",first),(200,"text/event-stream",final_reply)]);
    let db=state();let runtime=ChatRuntime::default();let agents=agents();let config=json!({});let settings=settings(base,true);
    let service=ConversationService {db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings};
    assert_eq!(service.execute(request("pero","记住今天"),|_|{},|_,_|Ok(json!({}))).await.unwrap().message.content,"记下来啦");
    requests.recv().unwrap();let second=requests.recv().unwrap();
    assert!(!second.to_string().contains("reasoning_details"));
    let conn=db.conn.lock().unwrap();assert_eq!(db.diary.query(&conn,"pero",&Default::default()).unwrap()[0].content,"流式日记");
}

#[test]
fn four_stream_protocols_reassemble_tools_and_keep_signatures() {
    let mut chat=Accumulator::default();
    for event in [json!({"choices":[{"delta":{"reasoning_content":"思考","tool_calls":[{"index":0,"id":"call1","function":{"name":"write_diary","arguments":"{\"content\":\""}}]}}]}),
        json!({"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"猫猫\"}"}}]},"finish_reason":"tool_calls"}]})] {
        chat.event("chat_completions",&event.to_string(),|_,_|Ok(())).unwrap();
    }
    let result=chat.finish("chat_completions").unwrap();assert_eq!(result.calls[0].arguments["content"],"猫猫");assert_eq!(result.reasoning,"思考");
    let mut claude=Accumulator::default();
    for event in [json!({"type":"content_block_start","index":0,"content_block":{"type":"thinking","thinking":"","signature":""}}),
        json!({"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":"思考"}}),
        json!({"type":"content_block_delta","index":0,"delta":{"type":"signature_delta","signature":"必须保留"}}),
        json!({"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"call1","name":"query_diary","input":{}}}),
        json!({"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"keyword\":\"抹茶\"}"}}),
        json!({"type":"message_delta","delta":{"stop_reason":"tool_use"}}),json!({"type":"message_stop"})] {
        claude.event("anthropic",&event.to_string(),|_,_|Ok(())).unwrap();
    }
    let result=claude.finish("anthropic").unwrap();assert_eq!(result.native[0]["signature"],"必须保留");assert_eq!(result.calls[0].arguments["keyword"],"抹茶");
    let mut gemini=Accumulator::default();gemini.event("gemini",&json!({"candidates":[{"content":{"parts":[{"functionCall":{"name":"query_diary","args":{}},"thoughtSignature":"签名"}]},"finishReason":"STOP"}]}).to_string(),|_,_|Ok(())).unwrap();
    let result=gemini.finish("gemini").unwrap();assert_eq!(result.native[0]["thoughtSignature"],"签名");assert_eq!(result.calls.len(),1);
    let mut responses=Accumulator::default();responses.event("responses",&json!({"type":"response.completed","response":{"status":"completed","output":[{"type":"reasoning","encrypted_content":"密文","summary":[{"text":"摘要"}]},{"type":"function_call","call_id":"call1","name":"query_diary","arguments":"{}"}]}}).to_string(),|_,_|Ok(())).unwrap();
    let result=responses.finish("responses").unwrap();assert_eq!(result.native[0]["encrypted_content"],"密文");assert_eq!(result.calls.len(),1);
    for format in ["chat_completions","responses","anthropic","gemini"] {assert_eq!(Accumulator::default().finish(format).unwrap_err().code,"STREAM_INTERRUPTED");}
}

#[test]
fn context_preserves_current_once_and_reports_budget_without_running_templates() {
    let db=state();let conn=db.conn.lock().unwrap();let thread=repository::thread(&conn,"desktop:pero").unwrap();let agents=agents();
    let input=json!("{{不执行}} 当前输入");let config=json!({});
    let (messages,manifest)=super::context::compile(&conn,super::context::ContextInput {thread:&thread,agent_id:"pero",identity:&agents,config:&config,current:&input,before_id:None,rounds:40,budget:4000,diary:&[]}).unwrap();
    assert_eq!(messages.iter().filter(|m|m.content==input).count(),1);assert!(messages[0].content.as_str().unwrap().contains("工具"));
    assert!(manifest["estimatedTokens"].as_u64().unwrap()<=4000);
    assert!(super::context::compile(&conn,super::context::ContextInput {thread:&thread,agent_id:"pero",identity:&agents,config:&config,current:&input,before_id:None,rounds:40,budget:5,diary:&[]}).is_err());
}

#[tokio::test]
async fn native_protocol_react_roundtrips_keep_call_ids_and_signed_reasoning() {
    for format in ["responses","anthropic","gemini"] {
        let (first,final_reply)=match format {
            "responses"=>(json!({"status":"completed","output":[{"type":"reasoning","id":"rs1","encrypted_content":"密文","summary":[{"type":"summary_text","text":"查日记"}]},
                {"type":"function_call","call_id":"call1","name":"query_diary","arguments":"{}"}]}),
                json!({"status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":"还没有日记，我们开始聊吧"}]}]})),
            "anthropic"=>(json!({"content":[{"type":"thinking","thinking":"查日记","signature":"签名"},{"type":"tool_use","id":"call1","name":"query_diary","input":{}}],"stop_reason":"tool_use"}),
                json!({"content":[{"type":"text","text":"还没有日记，我们开始聊吧"}],"stop_reason":"end_turn"})),
            _=>(json!({"candidates":[{"content":{"role":"model","parts":[{"functionCall":{"id":"call1","name":"query_diary","args":{}},"thoughtSignature":"签名"}]},"finishReason":"STOP"}]}),
                json!({"candidates":[{"content":{"role":"model","parts":[{"text":"还没有日记，我们开始聊吧"}]},"finishReason":"STOP"}]})),
        };
        let (base,requests)=mock_server(vec![(200,"application/json",first.to_string()),(200,"application/json",final_reply.to_string())]);
        let db=state();let runtime=ChatRuntime::default();let agents=agents();let config=json!({});let mut settings=settings(base,false);settings.api_format=format.into();
        let service=ConversationService {db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings};
        assert_eq!(service.execute(request("pero","你还记得什么"),|_|{},|_,_|Ok(json!({}))).await.unwrap().message.content,"还没有日记，我们开始聊吧");
        requests.recv().unwrap();let second=requests.recv().unwrap();
        match format {
            "responses"=> {let input=second["input"].as_array().unwrap();assert!(input.iter().any(|i|i["encrypted_content"]=="密文"));assert!(input.iter().any(|i|i["type"]=="function_call_output" && i["call_id"]=="call1"));}
            "anthropic"=> {let messages=second["messages"].as_array().unwrap();assert_eq!(messages[messages.len()-2]["content"][0]["signature"],"签名");assert_eq!(messages.last().unwrap()["content"][0]["tool_use_id"],"call1");}
            _=> {let messages=second["contents"].as_array().unwrap();assert_eq!(messages[messages.len()-2]["parts"][0]["thoughtSignature"],"签名");assert_eq!(messages.last().unwrap()["parts"][0]["functionResponse"]["id"],"call1");}
        }
    }
}

#[tokio::test]
async fn model_discovery_normalizes_native_ids_and_handles_pagination() {
    for format in ["chat_completions","anthropic","gemini"] {
        let replies=match format {
            "gemini"=>vec![(200,"application/json",json!({"models":[{"name":"models/gemini-first"}],"nextPageToken":"next"}).to_string()),
                (200,"application/json",json!({"models":[{"name":"models/gemini-second"}]}).to_string())],
            "anthropic"=>vec![(200,"application/json",json!({"data":[{"id":"claude-first"}],"has_more":true,"last_id":"claude-first"}).to_string()),
                (200,"application/json",json!({"data":[{"id":"claude-second"}],"has_more":false}).to_string())],
            _=>vec![(200,"application/json",json!({"data":[{"id":"compatible-model"}]}).to_string())],
        };
        let count=replies.len();let (base,requests)=mock_server(replies);let mut settings=settings(base,false);settings.api_format=format.into();
        let result=super::provider::list_models(&settings).await.unwrap();assert_eq!(result["data"].as_array().unwrap().len(),count);
        assert!(!result["data"][0]["id"].as_str().unwrap().starts_with("models/"));
        for _ in 0..count {assert_eq!(requests.recv().unwrap(),Value::Null);}
    }
}
