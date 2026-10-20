use std::collections::BTreeMap;
use rusqlite::{params,Connection};
use serde_json::{json,Value};
use super::types::{ChatResult,ToolActivity,ToolCall};

fn writes(name:&str)->bool { matches!(name,"write_diary"|"update_character"|"manage_reminder") }

// 只投影简短的展示结果，不向消息列表发送原始工具参数和整批查询内容。
pub fn describe(call:&ToolCall,result:&Value,committed:Option<bool>)->ToolActivity {
    let (status,summary)=if result.is_null() { ("running",String::new()) }
    else if result["ok"]==false {
        ("failed",result["error"]["message"].as_str().unwrap_or("调用失败").chars().take(160).collect())
    } else if writes(&call.name) && committed!=Some(true) {
        if committed.is_none() { ("staged","等待保存".into()) } else { ("not_applied","本轮未保存".into()) }
    } else {
        let summary=match call.name.as_str() {
            "query_diary"=>format!("找到 {} 篇日记",result["entries"].as_array().map_or(0,Vec::len)),
            "write_diary"=>"日记已保存".into(),
            "update_character"=>match (!call.arguments["state"].is_null(),!call.arguments["interactions"].is_null()) {
                (true,true)=>"状态与台词已更新",(false,true)=>"台词已更新",_=>"状态已更新",
            }.into(),
            "query_reminders"=>format!("找到 {} 项提醒",result["reminders"]["total"].as_u64().unwrap_or(0)),
            "manage_reminder"=>match call.arguments["action"].as_str() {
                Some("delete")=>"提醒已取消",Some("update")=>"提醒已修改",_=>"提醒已添加",
            }.into(),
            _=>"调用完成".into(),
        };
        ("completed",summary)
    };
    ToolActivity {id:call.id.clone(),name:call.name.chars().take(200).collect(),status:status.into(),summary}
}

pub fn recover(items:&mut [ToolActivity],message_status:&str) {
    if message_status=="pending" {return;}
    for item in items {
        if matches!(item.status.as_str(),"running"|"staged") {
            if writes(&item.name) {item.status="not_applied".into();item.summary="本轮未保存".into();}
            else {item.status="interrupted".into();item.summary="调用已中断".into();}
        }
    }
}

pub fn valid(items:&[ToolActivity])->bool {
    let mut ids=std::collections::HashSet::new();
    items.len()<=24 && items.iter().all(|item|!item.id.is_empty() && item.id.len()<=200 && ids.insert(&item.id)
        && item.name.len()<=800 && item.summary.len()<=1000
        && matches!(item.status.as_str(),"running"|"staged"|"completed"|"failed"|"not_applied"|"interrupted"))
}

pub fn migrate(conn:&mut Connection)->ChatResult<()> {
    let done:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM chat_migrations WHERE version=2)",[],|r|r.get(0))?;
    if done {return Ok(());}
    let tx=conn.transaction()?;
    let mut grouped:BTreeMap<i64,Vec<ToolActivity>>=BTreeMap::new();
    {
        let mut stmt=tx.prepare("SELECT c.message_id,c.call_id,c.name,c.arguments,c.result FROM chat_tool_calls c
            JOIN chat_messages m ON m.id=c.message_id WHERE m.status!='deleted' AND m.tool_timeline='[]' ORDER BY c.rowid")?;
        let rows=stmt.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?)))?;
        // 逐条读取旧记录，只保留短摘要，避免一次载入所有原始查询结果。
        for row in rows {
            let (message_id,id,name,arguments,result)=row?;
            if let (Ok(arguments),Ok(result))=(serde_json::from_str::<Value>(&arguments),serde_json::from_str::<Value>(&result)) {
                let items=grouped.entry(message_id).or_default();
                if items.len()<24 && !items.iter().any(|item|item.id==id) {items.push(describe(&ToolCall {id,name,arguments},&result,Some(true)));}
            }
        }
    }
    for (id,items) in grouped {tx.execute("UPDATE chat_messages SET tool_timeline=?1 WHERE id=?2 AND tool_timeline='[]'",params![json!(items).to_string(),id])?;}
    tx.execute("INSERT INTO chat_migrations(version) VALUES(2)",[])?;
    tx.commit()?;Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{chat_tools::ToolSession,repository,tests::{request,state},types::{ChatError,Completion}};

    #[test]
    fn timeline_keeps_committed_history_and_discards_cancelled_writes() {
        let db=state();
        let request=request("pero","记住我喜欢抹茶");
        let prepared=repository::prepare_turn(&mut db.conn.lock().unwrap(),&request,"Pero").unwrap();
        let mut tools=ToolSession::default();
        let call=ToolCall {id:"diary-1".into(),name:"write_diary".into(),arguments:json!({"content":"主人喜欢抹茶。"})};
        let result=tools.execute(&db,"pero",&call).unwrap();
        assert_eq!(describe(&call,&result,None).status,"staged");
        let mut conn=db.conn.lock().unwrap();
        let message=repository::finish_with_tools(&mut conn,&request,&prepared,&Completion {content:"记住啦".into(),..Default::default()},None,&json!({}),&tools).unwrap();
        assert_eq!(message.tool_timeline[0].summary,"日记已保存");
        assert_eq!(repository::snapshot(&conn,"desktop:pero").unwrap().messages.last().unwrap().tool_timeline[0].id,"diary-1");

        // 已存在的旧调用记录可以回填；重复迁移不重复追加。
        conn.execute("UPDATE chat_messages SET tool_timeline='[]' WHERE id=?1",[message.id]).unwrap();
        conn.execute("DELETE FROM chat_migrations WHERE version=2",[]).unwrap();
        migrate(&mut conn).unwrap();migrate(&mut conn).unwrap();
        assert_eq!(repository::message(&conn,message.id).unwrap().tool_timeline.len(),1);
        drop(conn);

        let second=super::super::tests::request("pero","改一下心情");
        let prepared=repository::prepare_turn(&mut db.conn.lock().unwrap(),&second,"Pero").unwrap();
        let mut tools=ToolSession::default();
        tools.execute(&db,"pero",&ToolCall {id:"state-1".into(),name:"update_character".into(),arguments:json!({"state":{"mood":"开心"}})}).unwrap();
        let mut conn=db.conn.lock().unwrap();
        let failed=repository::finish_with_tools(&mut conn,&second,&prepared,&Completion::default(),Some(&ChatError::new("CANCELLED","已停止回复")),&json!({}),&tools).unwrap();
        assert_eq!(failed.tool_timeline[0].status,"not_applied");
        assert_eq!(failed.tool_timeline[0].summary,"本轮未保存");

        let mut retry=super::super::tests::request("pero","占位");retry.content=None;retry.regenerate_message_id=Some(message.id);
        let prepared=repository::prepare_turn(&mut conn,&retry,"Pero").unwrap();
        let retained=repository::finish_with_tools(&mut conn,&retry,&prepared,&Completion::default(),Some(&ChatError::new("CANCELLED","已停止回复")),&json!({}),&tools).unwrap();
        assert_eq!(retained.content,"记住啦");assert_eq!(retained.tool_timeline[0].id,"diary-1");
        repository::delete_message(&mut conn,"desktop:pero",message.id,false).unwrap();
        let raw:String=conn.query_row("SELECT tool_timeline FROM chat_messages WHERE id=?1",[message.id],|r|r.get(0)).unwrap();
        assert_eq!(raw,"[]");
    }

    #[test]
    fn interrupted_timeline_never_claims_a_staged_write_was_saved() {
        let call=ToolCall {id:"state-1".into(),name:"update_character".into(),arguments:json!({"interactions":{"idle":["来聊天嘛"]}})};
        let mut items=vec![describe(&call,&json!({"ok":true,"commit":"after_assistant_persisted"}),None)];
        recover(&mut items,"interrupted");assert_eq!(items[0].status,"not_applied");
        assert_eq!(describe(&call,&json!({"ok":true}),Some(true)).summary,"台词已更新");
        let failed=describe(&call,&json!({"ok":false,"error":{"message":"参数无效"}}),Some(true));
        assert_eq!(failed.status,"failed");assert_eq!(failed.summary,"参数无效");assert!(valid(&[failed]));
    }
}
