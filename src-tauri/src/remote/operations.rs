//! 有限的主体接口目录；不暴露任意 HTTP 代理或子应用路径。
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use crate::local::types::*;

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum Query {
    Agents,
    Tools,
    Agent {id:String},
    AgentState {id:String,#[serde(default,rename="threadId")] thread_id:Option<String>},
    AgentTexts {id:String},
    Threads {#[serde(rename="agentId")] agent_id:String,page:u32,#[serde(rename="pageSize")] page_size:u32},
    Thread {id:String},
    Conversation {id:String,#[serde(rename="beforeCursor")] before_cursor:Option<String>,#[serde(rename="pageSize")] page_size:u32},
    ThreadTools {id:String},
    Queue {id:String},
    QueueLive {id:String},
    ActiveExecutions,
    Approvals,
    Questions,
    Tasks {#[serde(rename="agentId")] agent_id:Option<String>,status:Option<String>,page:u32,#[serde(rename="pageSize")] page_size:u32},
    Task {id:String},
    TaskProjection {id:String},
    Plan {#[serde(rename="threadId")] thread_id:String},
    Reminders {#[serde(rename="agentId")] agent_id:Option<String>},
    Rooms,
    Room {id:String,limit:u32},
    RoomMembers {id:String},
    Memories {#[serde(rename="agentId")] agent_id:String,query:String,page:u32,#[serde(rename="pageSize")] page_size:u32},
    MemorySource {id:String},
    Documents {query:String,page:u32,#[serde(rename="pageSize")] page_size:u32},
    Document {id:String},
    Attachment {id:String},
    WorkspaceTree {#[serde(rename="agentId")] agent_id:String,#[serde(rename="threadId")] thread_id:String,path:String},
    WorkspaceFile {#[serde(rename="agentId")] agent_id:String,#[serde(rename="threadId")] thread_id:String,path:String},
}
pub struct Route {pub method:reqwest::Method,pub path:String,pub params:Vec<(String,String)>,pub body:Option<Value>}

pub async fn fetch_query<F,Fut>(query:&Query,mut execute:F)->ChatResult<Value>
where F:FnMut(Route,usize)->Fut,Fut:std::future::Future<Output=ChatResult<Value>> {
    let mut query=query.clone();
    loop {
        // 长对话自动缩小单页，游标与消息保持完整；单条长回复使用独立读取额度。
        let limit=match &query {
            Query::Conversation {page_size,..} if *page_size>1=>super::transport::MAX_BODY,
            Query::Conversation {..}|Query::TaskProjection {..}|Query::Queue {..}|Query::QueueLive {..}|Query::Room {..}|Query::Thread {..}=>super::transport::MAX_PROJECTION_BODY,
            _=>super::transport::MAX_BODY,
        };
        match execute(query.route()?,limit).await {
            Err(error) if error.code=="REMOTE_TOO_LARGE"=> {
                match &mut query {
                    Query::Conversation {page_size,..} if *page_size>1=>*page_size=(*page_size/2).max(1),
                    Query::Room {limit,..} if *limit>1=>*limit=(*limit/2).max(1),
                    _=>return Err(error),
                }
            },
            result=>return result,
        }
    }
}
fn route(path:String)->Route {Route {method:reqwest::Method::GET,path,params:Vec::new(),body:None}}
pub fn segment(value:&str)->ChatResult<String> {
    if value.is_empty() || matches!(value,"."|"..") || value.len()>256 || value.chars().any(char::is_control) {return Err(ChatError::new("INVALID_INPUT","远程对象身份无效"));}
    let mut url=reqwest::Url::parse("http://localhost/").unwrap();url.path_segments_mut().unwrap().push(value);
    Ok(url.path()[1..].to_owned())
}
fn page(route:&mut Route,n:u32,size:u32)->ChatResult<()> {
    if n==0 || !(1..=100).contains(&size) {return Err(ChatError::new("INVALID_INPUT","远程分页范围无效"));}
    route.params.extend([("page".into(),n.to_string()),("pageSize".into(),size.to_string())]);Ok(())
}
fn param(route:&mut Route,key:&str,value:&str)->ChatResult<()> {
    if value.len()>2048 || value.chars().any(|v|v=='\0') {return Err(ChatError::new("INVALID_INPUT","远程查询参数过长或无效"));}
    route.params.push((key.into(),value.into()));Ok(())
}
impl Query {
    pub fn route(&self)->ChatResult<Route> {
        let mut r=match self {
            Self::Agents=>route("/agents".into()),
            Self::Tools=> {let mut r=route("/agents/tools".into());param(&mut r,"includeAdvanced","true")?;r},
            Self::Agent {id}=>route(format!("/agents/{}",segment(id)?)),
            Self::AgentState {id,thread_id}=> {let mut r=route(format!("/agents/{}/pet-state",segment(id)?));if let Some(thread)=thread_id {segment(thread)?;param(&mut r,"threadId",thread)?;}r},
            Self::AgentTexts {id}=>route(format!("/agents/{}/texts",segment(id)?)),
            Self::Threads {agent_id,page:n,page_size}=> {let mut r=route("/chat/threads".into());param(&mut r,"agentId",agent_id)?;param(&mut r,"channel","desktop")?;page(&mut r,*n,*page_size)?;r},
            Self::Thread {id}=> {let mut r=route(format!("/chat/threads/{}",segment(id)?));param(&mut r,"pageSize","1")?;r},
            Self::Conversation {id,before_cursor,page_size}=> {
                if !(1..=100).contains(page_size) {return Err(ChatError::new("INVALID_INPUT","历史分页范围无效"));}
                let mut r=route(format!("/chat/threads/{}/projection",segment(id)?));param(&mut r,"pageSize",&page_size.to_string())?;
                if let Some(cursor)=before_cursor {param(&mut r,"beforeCursor",cursor)?;}r
            },
            Self::ThreadTools {id}=>route(format!("/chat/threads/{}/tools",segment(id)?)),
            Self::Queue {id}=>route(format!("/chat/queue/{}",segment(id)?)),
            Self::QueueLive {id}=>route(format!("/chat/queue/{}/live",segment(id)?)),
            Self::ActiveExecutions=>route("/chat/tasks".into()),
            Self::Approvals=>route("/approvals".into()),Self::Questions=>route("/agent-inputs".into()),
            Self::Tasks {agent_id,status,page:n,page_size}=> {let mut r=route("/background-tasks".into());page(&mut r,*n,*page_size)?;
                if let Some(id)=agent_id {param(&mut r,"agentId",id)?;}if let Some(status)=status {param(&mut r,"status",status)?;}r},
            Self::Task {id}=>route(format!("/background-tasks/{}",segment(id)?)),
            Self::TaskProjection {id}=>route(format!("/background-tasks/{}/projection",segment(id)?)),
            Self::Plan {thread_id}=>route(format!("/task-plans/{}",segment(thread_id)?)),
            Self::Reminders {agent_id}=> {let mut r=route("/scheduler/reminders".into());if let Some(id)=agent_id {param(&mut r,"agentId",id)?;}r},
            Self::Rooms=>route("/stronghold/rooms".into()),
            Self::Room {id,limit}=> {if !(1..=100).contains(limit) {return Err(ChatError::new("INVALID_INPUT","房间历史范围无效"));}
                let mut r=route(format!("/stronghold/rooms/{}/projection",segment(id)?));param(&mut r,"limit",&limit.to_string())?;r},
            Self::RoomMembers {id}=>route(format!("/stronghold/rooms/{}/members",segment(id)?)),
            Self::Memories {agent_id,query,page:n,page_size}=> {let mut r=route("/memories".into());param(&mut r,"agentId",agent_id)?;param(&mut r,"query",query)?;page(&mut r,*n,*page_size)?;r},
            Self::MemorySource {id}=>route(format!("/memories/{}/source",segment(id)?)),
            Self::Documents {query,page:n,page_size}=> {let mut r=route("/knowledge/documents".into());param(&mut r,"query",query)?;page(&mut r,*n,*page_size)?;r},
            Self::Document {id}=>route(format!("/knowledge/documents/{}",segment(id)?)),
            Self::Attachment {id}=>route(format!("/attachments/{}",segment(id)?)),
            Self::WorkspaceTree {agent_id,thread_id,path}|Self::WorkspaceFile {agent_id,thread_id,path}=> {
                let mut r=route(if matches!(self,Self::WorkspaceTree {..}) {"/workspace/tree"} else {"/workspace/file"}.into());
                param(&mut r,"agentId",agent_id)?;param(&mut r,"threadId",thread_id)?;param(&mut r,"path",path)?;r
            },
        };
        if matches!(self,Self::Questions|Self::Approvals) {param(&mut r,"status","pending")?;}
        Ok(r)
    }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum Action {
    CreateThread {#[serde(rename="agentId")] agent_id:String,title:String},
    RenameThread {id:String,title:String},DeleteThread {id:String},
    StopChat {#[serde(rename="threadId")] thread_id:String},
    QueueItem {#[serde(rename="threadId")] thread_id:String,id:String,revision:u64,action:String,content:Option<String>},
    QueueControl {#[serde(rename="threadId")] thread_id:String,paused:bool},
    QueueOrder {#[serde(rename="threadId")] thread_id:String,order:Vec<QueuePosition>},
    InteractionMode {id:String,mode:String},ExecutionMode {id:String,mode:String},
    ThreadTools {id:String,#[serde(rename="disabledTools")] disabled_tools:Vec<String>},
    EditMessage {#[serde(rename="threadId")] thread_id:String,id:String,content:String,#[serde(default,rename="expectedRevision")] expected_revision:Option<u64>},
    RewindPreview {id:String,#[serde(rename="messageId")] message_id:Option<u64>,#[serde(rename="wholeThread")] whole_thread:bool},
    Rewind {id:String,#[serde(rename="messageId")] message_id:Option<u64>,#[serde(rename="wholeThread")] whole_thread:bool,#[serde(default,rename="preserveInputAttachments")] preserve_input_attachments:bool},
    TaskControl {id:String,action:String},
    DispatchTask {#[serde(rename="agentId")] agent_id:String,instruction:String,title:Option<String>,#[serde(rename="targetThreadId")] target_thread_id:Option<String>},
    RoomMessage {id:String,content:String,mentions:Vec<String>},
}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueuePosition {id:String,revision:u64}
impl Action {
    pub fn route(&self)->ChatResult<Route> {
        let (method,path,body)=match self {
            Self::CreateThread {agent_id,title}=> ("POST","/chat/threads".into(),json!({"agentId":agent_id,"title":title,"channel":"desktop","platform":"peroperochat"})),
            Self::RenameThread {id,title}=> {if title.encode_utf16().count()>100 {return Err(invalid());}("PATCH",format!("/chat/threads/{}",segment(id)?),json!({"title":title}))},
            Self::DeleteThread {id}=> ("DELETE",format!("/chat/threads/{}",segment(id)?),json!({})),
            Self::StopChat {thread_id}=> ("POST","/chat/stop".into(),json!({"threadId":thread_id})),
            Self::QueueItem {thread_id,id,revision,action,content}=> {
                if *revision==0 || *revision>9_007_199_254_740_991 || !["edit","cancel","immediate","continue"].contains(&action.as_str()) ||
                    (action=="edit" && content.as_ref().is_none_or(|value|value.trim().is_empty() || value.len()>128*1024)) {return Err(invalid());}
                let mut body=json!({"revision":revision,"action":action});
                if action=="edit" {body["content"]=json!(content);}
                ("PATCH",format!("/chat/queue/{}/items/{}",segment(thread_id)?,segment(id)?),body)
            },
            Self::QueueControl {thread_id,paused}=> ("POST",format!("/chat/queue/{}/control",segment(thread_id)?),json!({"paused":paused})),
            Self::QueueOrder {thread_id,order}=> {
                let mut ids=std::collections::HashSet::new();
                if order.len()>100 || order.iter().any(|item|segment(&item.id).is_err() || item.revision==0 || item.revision>9_007_199_254_740_991 || !ids.insert(&item.id)) {return Err(invalid());}
                ("POST",format!("/chat/queue/{}/order",segment(thread_id)?),json!({"order":order}))
            },
            Self::InteractionMode {id,mode}=> {if !["minimal","chat","work"].contains(&mode.as_str()) {return Err(invalid());}("PUT",format!("/chat/threads/{}/interaction-mode",segment(id)?),json!({"interactionMode":mode}))},
            Self::ExecutionMode {id,mode}=> {if !["manual","auto","auto_approve","full"].contains(&mode.as_str()) {return Err(invalid());}("PUT",format!("/chat/threads/{}/execution-mode",segment(id)?),json!({"executionMode":mode}))},
            Self::ThreadTools {id,disabled_tools}=> {if disabled_tools.len()>500 {return Err(invalid());}("PUT",format!("/chat/threads/{}/tools",segment(id)?),json!({"disabledTools":disabled_tools}))},
            Self::EditMessage {thread_id,id,content,expected_revision}=> {
                if content.trim().is_empty() || content.len()>128*1024 || expected_revision.is_none_or(|revision|revision==0) {return Err(ChatError::new("INVALID_INPUT","消息正文或修订无效，请刷新会话后重试"));}
                ("PATCH",format!("/chat/threads/{}/messages/{}",segment(thread_id)?,segment(id)?),json!({"content":content,"expectedRevision":expected_revision}))
            },
            Self::RewindPreview {id,message_id,whole_thread}|Self::Rewind {id,message_id,whole_thread,..}=> {
                if !*whole_thread && message_id.is_none_or(|id|id==0) {return Err(invalid());}
                let name=if matches!(self,Self::Rewind {..}) {"rewind"} else {"rewind-preview"};
                let mut body=json!({"wholeThread":whole_thread});if let Some(id)=message_id {body["messageId"]=json!(id);}
                if let Self::Rewind {preserve_input_attachments,..}=self {body["preserveInputAttachments"]=json!(preserve_input_attachments);}
                ("POST",format!("/chat/threads/{}/{name}",segment(id)?),body)
            },
            Self::TaskControl {id,action}=> {if !["pause","resume","resume-interrupted","retry","cancel","read"].contains(&action.as_str()) {return Err(invalid());}("POST",format!("/background-tasks/{}/{}",segment(id)?,action),json!({}))},
            Self::DispatchTask {agent_id,instruction,title,target_thread_id}=> {
                if instruction.trim().is_empty() || instruction.len()>16000 {return Err(invalid());}
                let mut body=json!({"agentId":agent_id,"instruction":instruction});
                if let Some(title)=title {body["title"]=json!(title);}if let Some(id)=target_thread_id {body["targetThreadId"]=json!(id);}
                ("POST","/background-tasks".into(),body)
            },
            Self::RoomMessage {id,content,mentions}=> {if content.trim().is_empty() || mentions.len()>32 {return Err(invalid());}
                ("POST",format!("/stronghold/rooms/{}/messages",segment(id)?),json!({"content":content,"mentions":mentions,"role":"user","senderId":"user"}))},
        };
        if body.to_string().len()>128*1024 {return Err(invalid());}
        Ok(Route {method:method.parse().unwrap(),path,params:Vec::new(),body:Some(body)})
    }
}
fn invalid()->ChatError {ChatError::new("INVALID_INPUT","远程操作参数无效或超出范围")}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn oversized_history_retries_smaller_pages_without_losing_cursor() {
        let query=Query::Conversation {id:"thread-a".into(),before_cursor:Some("123".into()),page_size:24};
        let mut attempts=Vec::new();
        let result=fetch_query(&query,|route,limit| {
            assert_eq!(route.method,reqwest::Method::GET);
            assert_eq!(route.path,"/chat/threads/thread-a/projection");
            assert!(route.params.contains(&("beforeCursor".into(),"123".into())));
            let size=route.params.iter().find(|(key,_)|key=="pageSize").unwrap().1.parse::<u32>().unwrap();
            attempts.push(size);
            std::future::ready(if size>1 {Err(ChatError::new("REMOTE_TOO_LARGE","内容较多"))} else {
                assert_eq!(limit,super::super::transport::MAX_PROJECTION_BODY);
                Ok(json!({"pageSize":1,"hasMoreBefore":true,"beforeCursor":"122","messages":[{"messageId":"122"}]}))
            })
        }).await.unwrap();
        assert_eq!(attempts,vec![24,12,6,3,1]);assert_eq!(result["beforeCursor"],"122");assert_eq!(result["hasMoreBefore"],true);
    }
    #[tokio::test]
    async fn paging_does_not_retry_auth_network_or_write_failures() {
        let query=Query::Conversation {id:"t".into(),before_cursor:None,page_size:24};let mut calls=0;
        let result=fetch_query(&query,|_,_| {calls+=1;std::future::ready(Err(ChatError::new("REMOTE_AUTH_REQUIRED","需要登录")))}).await;
        assert_eq!(calls,1);assert_eq!(result.unwrap_err().code,"REMOTE_AUTH_REQUIRED");
    }
    #[test]
    fn typed_routes_encode_ids_and_keep_bounds_and_explicit_rewind() {
        assert_eq!(segment("a/b?#").unwrap(),"a%2Fb%3F%23");
        assert!(Query::Threads {agent_id:"pero".into(),page:0,page_size:20}.route().is_err());
        assert!(Action::TaskControl {id:"t".into(),action:"reset".into()}.route().is_err());
        assert!(Action::Rewind {id:"t".into(),message_id:None,whole_thread:false,preserve_input_attachments:false}.route().is_err());
        assert!(serde_json::from_value::<Action>(json!({"kind":"delete_message","threadId":"t","id":"1"})).is_err());
        assert!(serde_json::from_value::<Action>(json!({"kind":"delete_message_pair","threadId":"t","id":"1"})).is_err());
        let rewind=Action::Rewind {id:"t".into(),message_id:Some(1),whole_thread:false,preserve_input_attachments:true}.route().unwrap();
        assert_eq!(rewind.body.unwrap()["preserveInputAttachments"],true);
        assert!(Action::EditMessage {thread_id:"t".into(),id:"1".into(),content:"新正文".into(),expected_revision:None}.route().is_err());
        assert!(Action::EditMessage {thread_id:"t".into(),id:"1".into(),content:" ".into(),expected_revision:Some(1)}.route().is_err());
    }
    #[test]
    fn state_and_queue_routes_preserve_thread_identity_and_revisions() {
        let state=Query::AgentState {id:"nana".into(),thread_id:Some("thread-2".into())}.route().unwrap();
        assert_eq!(state.params,vec![("threadId".into(),"thread-2".into())]);
        assert_eq!(Query::Queue {id:"a/b".into()}.route().unwrap().path,"/chat/queue/a%2Fb");
        let action=Action::QueueItem {thread_id:"t".into(),id:"m".into(),revision:2,action:"immediate".into(),content:None}.route().unwrap();
        assert_eq!(action.body.unwrap(),json!({"revision":2,"action":"immediate"}));
        assert!(Action::QueueItem {thread_id:"t".into(),id:"m".into(),revision:0,action:"cancel".into(),content:None}.route().is_err());
        assert!(Action::QueueItem {thread_id:"t".into(),id:"m".into(),revision:1,action:"edit".into(),content:Some(" ".into())}.route().is_err());
        assert!(Action::QueueOrder {thread_id:"t".into(),order:vec![QueuePosition {id:"m".into(),revision:1},QueuePosition {id:"m".into(),revision:2}]}.route().is_err());
    }
}
