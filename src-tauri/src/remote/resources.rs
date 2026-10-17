use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use tauri::{AppHandle,Manager};
use sha2::{Digest,Sha256};
use crate::{mode::{Coordinator,Token},local::types::*};
use super::{transport,operations,cache};

#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Upload {pub name:String,pub mime_type:String,pub bytes:Vec<u8>}
#[tauri::command]
pub async fn remote_upload(app:AppHandle,token:Token,request_id:String,thread_id:String,file:Upload)->ChatResult<Value> {
    upload_inner(app,token,request_id,thread_id,file).await.map_err(transport::before_send)
}
async fn upload_inner(app:AppHandle,token:Token,request_id:String,thread_id:String,file:Upload)->ChatResult<Value> {
    operations::segment(&thread_id)?;
    if file.bytes.is_empty() || file.bytes.len()>transport::MAX_BODY || file.name.is_empty() || file.name.len()>240 || file.name.contains(['/', '\\']) || file.name.chars().any(char::is_control)
        || file.mime_type.len()>128 || file.mime_type.contains(['\r','\n']) {return Err(ChatError::new("INVALID_FILE","附件为空、过大或文件名无效；当前附件上限 8 MiB"));}
    let part=reqwest::multipart::Part::bytes(file.bytes.clone()).file_name(file.name.clone()).mime_str(&file.mime_type)
        .map_err(|_|ChatError::new("INVALID_FILE","附件媒体类型无效"))?;
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();
    let session=tokio::select! {biased;_ =changes.changed()=>return Err(super::inactive()),result=super::session(&app,&token)=>result?,};
    cache::prepare(&app,&token,&request_id,&json!({"kind":"upload","threadId":thread_id,"name":file.name,"mimeType":file.mime_type,"sha256":format!("{:x}",Sha256::digest(&file.bytes))}))?;
    let result=tokio::select! {biased;
        _=changes.changed()=>return Err(super::inactive().with_outcome("unknown")),
        result=async {
            let response=session.client.post(format!("{}/api/attachments",session.profile.endpoint)).bearer_auth(&session.secret)
                .timeout(std::time::Duration::from_secs(90)).multipart(reqwest::multipart::Form::new().text("threadId",thread_id).part("file",part))
                .send().await.map_err(transport::send_error)?;
            transport::status(response.status())?;transport::envelope(&transport::body(response).await?)
        }=>result,
    };
    let data=cache::settle(&app,&token,&request_id,result)?;
    Ok(json!({"modeToken":token,"requestId":request_id,"data":data}))
}

#[derive(Deserialize,Serialize)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum Resource {
    Attachment {id:String},AgentAvatar {id:String},
    Workspace {#[serde(rename="agentId")] agent_id:String,#[serde(rename="threadId")] thread_id:String,path:String,version:String},
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Range {pub start:u64,pub end:u64}
#[tauri::command]
pub async fn remote_resource(app:AppHandle,token:Token,resource:Resource,range:Option<Range>)->ChatResult<Value> {
    let (path,params)=match &resource {
        Resource::Attachment {id}=>(format!("/attachments/{}/content",operations::segment(id)?),Vec::new()),
        Resource::AgentAvatar {id}=>(format!("/agents/{}/avatar",operations::segment(id)?),Vec::new()),
        Resource::Workspace {agent_id,thread_id,path,version}=> {
            operations::segment(agent_id)?;operations::segment(thread_id)?;
            if path.is_empty() || path.len()>2048 || version.is_empty() || version.len()>256 {return Err(ChatError::new("INVALID_FILE","工作区资源必须指定路径与版本"));}
            ("/workspace/content".into(),vec![("agentId",agent_id.as_str()),("threadId",thread_id.as_str()),("path",path.as_str()),("version",version.as_str())])
        },
    };
    if range.as_ref().is_some_and(|r|r.end<r.start || r.end-r.start>=transport::MAX_BODY as u64) {return Err(ChatError::new("INVALID_FILE","资源分段范围无效或过大"));}
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();
    let result=tokio::select! {biased;
        _=changes.changed()=>return Err(super::inactive()),
        result=async {
            let session=super::session(&app,&token).await?;
            let mut url=reqwest::Url::parse(&format!("{}/api{path}",session.profile.endpoint)).map_err(|_|transport::network())?;
            if !params.is_empty() {url.query_pairs_mut().extend_pairs(params);}
            let mut request=session.client.get(url).bearer_auth(&session.secret).timeout(std::time::Duration::from_secs(60));
            if let Some(range)=&range {request=request.header("Range",format!("bytes={}-{}",range.start,range.end));}
            let response=request.send().await.map_err(|_|transport::network())?;transport::status(response.status())?;
            if range.is_some() && response.status()!=reqwest::StatusCode::PARTIAL_CONTENT {return Err(ChatError::new("REMOTE_RANGE_UNSUPPORTED","服务端未按资源范围响应，已停止读取"));}
            let mime=response.headers().get("content-type").and_then(|v|v.to_str().ok()).unwrap_or("application/octet-stream").to_owned();
            let content_range=response.headers().get("content-range").and_then(|v|v.to_str().ok()).map(str::to_owned);
            let bytes=transport::body(response).await?;
            Ok::<_,ChatError>(json!({"modeToken":token,"mimeType":mime,"contentRange":content_range,"bytes":bytes}))
        }=>result?,
    };
    super::validate_token(&app,&token)?;Ok(result)
}
