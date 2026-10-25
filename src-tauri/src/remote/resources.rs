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
            let response=super::checked_response(&session,response).await?;transport::envelope(&transport::body(response).await?)
        }=>result,
    };
    let data=cache::settle(&app,&token,&request_id,result)?;
    Ok(json!({"modeToken":token,"requestId":request_id,"data":data}))
}

#[derive(Clone,Deserialize,Serialize)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum Resource {
    Attachment {id:String},AgentAvatar {id:String},OwnerAvatar {#[serde(default)] version:Option<String>},
    Workspace {#[serde(rename="agentId")] agent_id:String,#[serde(rename="threadId")] thread_id:String,path:String,version:String},
    Reference {#[serde(rename="agentId")] agent_id:String,#[serde(rename="threadId")] thread_id:String,#[serde(default,rename="taskId")] task_id:Option<String>,source:String},
}
#[derive(Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Range {pub start:u64,pub end:u64}
fn verify_range(range:&Range,header:Option<&str>,length:usize)->ChatResult<()> {
    let parsed=header.and_then(|s|s.strip_prefix("bytes ")).and_then(|s|s.split_once('/')).and_then(|(span,total)| {
        let (start,end)=span.split_once('-')?;Some((start.parse::<u64>().ok()?,end.parse::<u64>().ok()?,total.parse::<u64>().ok()?))
    });
    if !parsed.is_some_and(|(start,end,total)|start==range.start && end>=start && end<=range.end && end<total && end-start+1==length as u64) {
        return Err(ChatError::new("REMOTE_PROTOCOL","服务端返回的文件范围不匹配，请重新打开文件"));
    }
    Ok(())
}
fn content_name(header:Option<&str>)->Option<String> {
    let header=header.filter(|s|s.len()<=4096)?;
    let fields:Vec<_>=header.split(';').filter_map(|part|part.trim().split_once('=')).collect();
    if let Some((_,value))=fields.iter().find(|(key,_)|key.eq_ignore_ascii_case("filename*")) {
        if value.get(..7).is_some_and(|s|s.eq_ignore_ascii_case("UTF-8''")) {
            let mut result=Vec::new();let mut bytes=value.as_bytes()[7..].iter().copied();
            while let Some(byte)=bytes.next() {
                if byte==b'%' {
                    let a=(bytes.next()? as char).to_digit(16)?;let b=(bytes.next()? as char).to_digit(16)?;
                    result.push((a*16+b) as u8);
                } else {result.push(byte);}
            }
            return String::from_utf8(result).ok();
        }
    }
    fields.iter().find(|(key,_)|key.eq_ignore_ascii_case("filename")).map(|(_,value)|value.trim_matches('"').to_string())
}
impl Resource {
    pub(super) fn route(&self)->ChatResult<(String,Vec<(String,String)>)> {
        Ok(match self {
            Self::Attachment {id}=>(format!("/attachments/{}/content",operations::segment(id)?),vec![]),
            Self::AgentAvatar {id}=>(format!("/agents/{}/avatar",operations::segment(id)?),vec![]),
            Self::OwnerAvatar {version}=> {
                if version.as_ref().is_some_and(|value|uuid::Uuid::parse_str(value).is_err()) {return Err(ChatError::new("INVALID_FILE","用户头像版本无效"));}
                ("/system/user-avatar/content".into(),vec![])
            },
            Self::Workspace {agent_id,thread_id,path,version}=> {
                operations::segment(agent_id)?;operations::segment(thread_id)?;
                if path.is_empty() || path.len()>2048 || version.is_empty() || version.len()>256 {return Err(ChatError::new("INVALID_FILE","工作区资源必须指定路径与版本"));}
                ("/workspace/content".into(),vec![("agentId".into(),agent_id.clone()),("threadId".into(),thread_id.clone()),("path".into(),path.clone()),("version".into(),version.clone())])
            },
            Self::Reference {agent_id,thread_id,task_id,source}=> {
                operations::segment(agent_id)?;operations::segment(thread_id)?;
                if source.is_empty() || source.len()>2048 || source.chars().any(char::is_control) || !source.starts_with("workspace:") {
                    return Err(ChatError::new("INVALID_FILE","资源引用必须由服务端解析为当前会话工作区文件"));
                }
                let mut params=vec![("agentId".into(),agent_id.clone()),("threadId".into(),thread_id.clone()),("source".into(),source.clone())];
                if let Some(id)=task_id {operations::segment(id)?;params.push(("taskId".into(),id.clone()));}
                ("/attachments/reference-content".into(),params)
            },
        })
    }
}

async fn cached_metadata<F>(app:&AppHandle,token:&Token,key:&str,cached_only:Option<bool>,resource:Option<&Value>,fetch:F)->ChatResult<Value>
where F:std::future::Future<Output=ChatResult<Value>> {
    super::validate_token(app,token)?;
    if cached_only==Some(true) {
        let data=cache::load(app,token,key)?.ok_or_else(||ChatError::new("REMOTE_NOT_CACHED","这份文件还没有保存在本机，联网后打开一次吧"))?;
        return Ok(json!({"modeToken":token,"cached":true,"data":data}));
    }
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();let ticket=cache::begin_read(app,token,key)?;
    let result=tokio::select! {biased;_ =changes.changed()=>return Err(super::inactive()),result=fetch=>result,};
    match result {
        Ok(data)=> {
            if !cache::save(app,token,key,ticket,&data)? {return Err(ChatError::new("REMOTE_STALE_READ","正在同步最新内容"));}
            Ok(json!({"modeToken":token,"cached":false,"data":data}))
        },
        Err(error)=> {
            super::validate_token(app,token)?;
            if cache::offline_error(&error) {if let Ok(Some(data))=cache::load(app,token,key) {return Ok(json!({"modeToken":token,"cached":true,"data":data}));}}
            if matches!(error.code.as_str(),"REMOTE_NOT_FOUND"|"REMOTE_FORBIDDEN") {
                if let Some(resource)=resource {let _=cache::forget_resource(app,token,resource,key,ticket);}
                else {let _=cache::forget_query(app,token,key,ticket,None);}
            }
            Err(error)
        },
    }
}
// 引用解析由 infOS 校验会话与路径权限；离线只复用同一连接中已确认的解析结果。
#[tauri::command]
pub async fn remote_resolve_resource(app:AppHandle,token:Token,agent_id:String,thread_id:String,task_id:Option<String>,source:String,cached_only:Option<bool>)->ChatResult<Value> {
    operations::segment(&agent_id)?;operations::segment(&thread_id)?;
    if source.is_empty() || source.len()>2048 || source.chars().any(char::is_control) {return Err(ChatError::new("INVALID_FILE","文件引用无效或过长"));}
    if let Some(id)=&task_id {operations::segment(id)?;}
    let key=format!("resource-resolve:{}",json!({"agentId":agent_id,"threadId":thread_id,"taskId":task_id,"source":source}));
    cached_metadata(&app,&token,&key,cached_only,None,async {
            let session=super::session(&app,&token).await?;
            let mut input=json!({"agentId":agent_id,"threadId":thread_id,"source":source});
            if let Some(id)=task_id {input["taskId"]=json!(id);}
            let response=session.client.post(format!("{}/api/attachments/resolve-reference",session.profile.endpoint))
                .bearer_auth(&session.secret).timeout(std::time::Duration::from_secs(35))
                .json(&input)
                .send().await.map_err(|_|transport::network())?;
            let response=super::checked_response(&session,response).await?;
            transport::envelope(&transport::body(response).await?)
    }).await
}

// HTML 文件使用 infOS 已有的短期预览票据，支持同一工作区内的相对资源，不向页面暴露设备凭据。
#[tauri::command]
pub async fn remote_html_preview(app:AppHandle,token:Token,resource:Resource)->ChatResult<Value> {
    resource.route()?;
    let encode=|value:&str|value.bytes().map(|byte|if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {(byte as char).to_string()} else {format!("%{byte:02X}")}).collect::<String>();
    let input=match resource {
        Resource::Attachment {id}=>json!({"id":id}),
        Resource::Workspace {agent_id,thread_id,path,..}=>json!({"agentId":agent_id,"threadId":thread_id,"source":format!("workspace:{}",encode(&path))}),
        Resource::Reference {agent_id,thread_id,task_id,source}=> {
            let mut input=json!({"agentId":agent_id,"threadId":thread_id,"source":source});
            if let Some(id)=task_id {input["taskId"]=json!(id);}input
        },
        _=>return Err(ChatError::new("INVALID_FILE","此资源不支持 HTML 页面预览")),
    };
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();
    let url=tokio::select! {biased;
        _=changes.changed()=>return Err(super::inactive()),
        result=async {
            let session=super::session(&app,&token).await?;
            let response=session.client.post(format!("{}/api/attachments/browser-preview",session.profile.endpoint))
                .bearer_auth(&session.secret).timeout(std::time::Duration::from_secs(35)).json(&input)
                .send().await.map_err(|_|transport::network())?;
            let response=super::checked_response(&session,response).await?;
            let data=transport::envelope(&transport::body(response).await?)?;
            let path=data.get("path").and_then(Value::as_str).ok_or_else(||ChatError::new("REMOTE_PROTOCOL","服务端未提供页面预览地址"))?;
            let ticket=path.strip_prefix("/html-previews/").and_then(|value|value.split('/').next()).unwrap_or("");
            if path.len()>8192 || ticket.len()!=64 || !ticket.bytes().all(|byte|byte.is_ascii_hexdigit()) || path.contains(['\r','\n','\\']) {
                return Err(ChatError::new("REMOTE_PROTOCOL","页面预览地址不符合 infOS 票据格式"));
            }
            Ok::<_,ChatError>(format!("{}{path}",session.profile.endpoint.trim_end_matches('/')))
        }=>result?,
    };
    super::validate_token(&app,&token)?;Ok(json!({"modeToken":token,"data":{"url":url}}))
}
#[tauri::command]
pub async fn remote_resource_info(app:AppHandle,token:Token,resource:Resource,cached_only:Option<bool>)->ChatResult<Value> {
    let (path,mut params)=resource.route()?;params.push(("download".into(),"1".into()));
    let descriptor=serde_json::to_value(&resource).map_err(|_|super::inactive())?;
    let key=format!("resource-info:{descriptor}");
    cached_metadata(&app,&token,&key,cached_only,Some(&descriptor),async {
            let session=super::session(&app,&token).await?;
            let mut url=reqwest::Url::parse(&format!("{}/api{path}",session.profile.endpoint)).map_err(|_|transport::network())?;
            url.query_pairs_mut().extend_pairs(params);
            let response=session.client.head(url).bearer_auth(&session.secret).header("Accept-Encoding","identity")
                .timeout(std::time::Duration::from_secs(30)).send().await.map_err(|_|transport::network())?;
            let response=super::checked_response(&session,response).await?;
            // HEAD 的 content_length() 在部分客户端固定为零，必须读取原始响应头。
            let size=response.headers().get("content-length").and_then(|v|v.to_str().ok()).and_then(|v|v.parse::<u64>().ok());
            let mime=response.headers().get("content-type").and_then(|v|v.to_str().ok()).unwrap_or("application/octet-stream");
            let name=content_name(response.headers().get("content-disposition").and_then(|v|v.to_str().ok()));
            Ok::<_,ChatError>(json!({"name":name,"size":size,"mimeType":mime,"ranges":response.headers().get("accept-ranges").is_some_and(|v|v=="bytes")}))
    }).await
}
#[tauri::command]
pub async fn remote_resource(app:AppHandle,token:Token,resource:Resource,range:Option<Range>,cached_only:Option<bool>)->ChatResult<Value> {
    let (path,params)=resource.route()?;
    if range.as_ref().is_some_and(|r|r.end<r.start || r.end-r.start>=transport::MAX_BODY as u64) {return Err(ChatError::new("INVALID_FILE","资源分段范围无效或过大"));}
    super::validate_token(&app,&token)?;
    let descriptor=serde_json::to_value(&resource).map_err(|_|super::inactive())?;
    let key=json!({"resource":descriptor,"range":range}).to_string();
    // 可变工作区引用的分块不能混用不同时间的文件；只有完整文件或固定版本资源才保留离线副本。
    let cacheable=range.is_none() || matches!(resource,Resource::Attachment {..}|Resource::Workspace {..});
    let response=|blob:cache::Blob,cached:bool|json!({"modeToken":token,"cached":cached,"mimeType":blob.mime,"contentRange":blob.content_range,"bytes":blob.bytes});
    if cached_only==Some(true) {
        return if cacheable {cache::load_blob(&app,&token,&key)?} else {None}.map(|blob|response(blob,true)).ok_or_else(||ChatError::new("REMOTE_NOT_CACHED","这份资源还没有保存在本机，联网后打开一次吧"));
    }
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();
    let ticket=cache::begin_read(&app,&token,&format!("blob:{key}"))?;
    let result=tokio::select! {biased;
        _=changes.changed()=>return Err(super::inactive()),
        result=async {
            let session=super::session(&app,&token).await?;
            let mut url=reqwest::Url::parse(&format!("{}/api{path}",session.profile.endpoint)).map_err(|_|transport::network())?;
            if !params.is_empty() {url.query_pairs_mut().extend_pairs(params);}
            let mut request=session.client.get(url).bearer_auth(&session.secret).timeout(std::time::Duration::from_secs(60));
            if let Some(range)=&range {request=request.header("Range",format!("bytes={}-{}",range.start,range.end));}
            let response=request.send().await.map_err(|_|transport::network())?;
            let response=super::checked_response(&session,response).await?;
            if range.is_some() && response.status()!=reqwest::StatusCode::PARTIAL_CONTENT {return Err(ChatError::new("REMOTE_RANGE_UNSUPPORTED","服务端未按资源范围响应，已停止读取"));}
            let mime=response.headers().get("content-type").and_then(|v|v.to_str().ok()).unwrap_or("application/octet-stream").to_owned();
            let content_range=response.headers().get("content-range").and_then(|v|v.to_str().ok()).map(str::to_owned);
            let bytes=transport::body(response).await?;
            if let Some(range)=range.as_ref() {verify_range(range,content_range.as_deref(),bytes.len())?;}
            Ok::<_,ChatError>(cache::Blob {mime,content_range,bytes})
        }=>result,
    };
    super::validate_token(&app,&token)?;
    match result {
        Ok(blob)=> {
            if cacheable && !cache::save_blob(&app,&token,&key,ticket,&blob)? {return Err(ChatError::new("REMOTE_STALE_READ","正在同步最新内容"));}
            Ok(response(blob,false))
        },
        Err(error)=> {
            if cacheable && cache::offline_error(&error) {if let Ok(Some(blob))=cache::load_blob(&app,&token,&key) {return Ok(response(blob,true));}}
            if matches!(error.code.as_str(),"REMOTE_NOT_FOUND"|"REMOTE_FORBIDDEN") {let _=cache::forget_resource(&app,&token,&descriptor,&format!("blob:{key}"),ticket);}
            Err(error)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks_partial_content_including_short_final_chunk() {
        let range=Range {start:100,end:199};
        assert!(verify_range(&range,Some("bytes 100-149/150"),50).is_ok());
        for (header,len) in [(Some("bytes 0-99/200"),100),(Some("bytes 100-199/200"),99),(Some("bytes 100-200/201"),101),(Some("bytes 100-199/199"),100),(None,100)] {
            assert!(verify_range(&range,header,len).is_err());
        }
    }
    #[test]
    fn resource_routes_cannot_be_used_as_arbitrary_url_proxy() {
        assert!(serde_json::from_value::<Resource>(json!({"kind":"url","url":"https://example.com"})).is_err());
        assert!(serde_json::from_value::<Resource>(json!({"kind":"attachment","id":"a","url":"https://example.com"})).is_err());
        let file=Resource::Workspace {agent_id:"agent".into(),thread_id:"thread".into(),path:"报告.pdf".into(),version:"".into()};
        assert!(file.route().is_err());
        let avatar:Resource=serde_json::from_value(json!({"kind":"owner_avatar"})).unwrap();
        assert_eq!(avatar.route().unwrap(),("/system/user-avatar/content".into(),vec![]));
        assert!(serde_json::from_value::<Resource>(json!({"kind":"owner_avatar","url":"https://example.invalid"})).is_err());
        assert!(serde_json::from_value::<Resource>(json!({"kind":"owner_avatar","id":"another-user"})).is_err());
        let updated:Resource=serde_json::from_value(json!({"kind":"owner_avatar","version":uuid::Uuid::new_v4().to_string()})).unwrap();
        assert_eq!(updated.route().unwrap().0,"/system/user-avatar/content");
        assert_ne!(serde_json::to_value(&updated).unwrap(),serde_json::to_value(&avatar).unwrap());
        let invalid:Resource=serde_json::from_value(json!({"kind":"owner_avatar","version":"过期"})).unwrap();assert!(invalid.route().is_err());
    }
    #[test]
    fn preserves_utf8_attachment_names_and_literal_plus() {
        assert_eq!(content_name(Some("attachment; filename=report.pdf; filename*=UTF-8''%E6%8A%A5%E5%91%8A+1.pdf")),Some("报告+1.pdf".into()));
        assert_eq!(content_name(Some("inline; filename=\"hello.txt\"")),Some("hello.txt".into()));
        assert!(content_name(Some("attachment; filename*=UTF-8''bad%2G")).is_none());
    }
}
