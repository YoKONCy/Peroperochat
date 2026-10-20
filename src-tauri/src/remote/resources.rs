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

#[derive(Deserialize,Serialize)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum Resource {
    Attachment {id:String},AgentAvatar {id:String},
    Workspace {#[serde(rename="agentId")] agent_id:String,#[serde(rename="threadId")] thread_id:String,path:String,version:String},
}
#[derive(Deserialize)]
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
            Self::Workspace {agent_id,thread_id,path,version}=> {
                operations::segment(agent_id)?;operations::segment(thread_id)?;
                if path.is_empty() || path.len()>2048 || version.is_empty() || version.len()>256 {return Err(ChatError::new("INVALID_FILE","工作区资源必须指定路径与版本"));}
                ("/workspace/content".into(),vec![("agentId".into(),agent_id.clone()),("threadId".into(),thread_id.clone()),("path".into(),path.clone()),("version".into(),version.clone())])
            },
        })
    }
}
#[tauri::command]
pub async fn remote_resource_info(app:AppHandle,token:Token,resource:Resource)->ChatResult<Value> {
    let (path,mut params)=resource.route()?;params.push(("download".into(),"1".into()));
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();
    let data=tokio::select! {biased;
        _=changes.changed()=>return Err(super::inactive()),
        result=async {
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
        }=>result?,
    };
    super::validate_token(&app,&token)?;Ok(json!({"modeToken":token,"data":data}))
}
#[tauri::command]
pub async fn remote_resource(app:AppHandle,token:Token,resource:Resource,range:Option<Range>)->ChatResult<Value> {
    let (path,params)=resource.route()?;
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
            let response=request.send().await.map_err(|_|transport::network())?;
            let response=super::checked_response(&session,response).await?;
            if range.is_some() && response.status()!=reqwest::StatusCode::PARTIAL_CONTENT {return Err(ChatError::new("REMOTE_RANGE_UNSUPPORTED","服务端未按资源范围响应，已停止读取"));}
            let mime=response.headers().get("content-type").and_then(|v|v.to_str().ok()).unwrap_or("application/octet-stream").to_owned();
            let content_range=response.headers().get("content-range").and_then(|v|v.to_str().ok()).map(str::to_owned);
            let bytes=transport::body(response).await?;
            if let Some(range)=range.as_ref() {verify_range(range,content_range.as_deref(),bytes.len())?;}
            Ok::<_,ChatError>(json!({"modeToken":token,"mimeType":mime,"contentRange":content_range,"bytes":bytes}))
        }=>result?,
    };
    super::validate_token(&app,&token)?;Ok(result)
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
    }
    #[test]
    fn preserves_utf8_attachment_names_and_literal_plus() {
        assert_eq!(content_name(Some("attachment; filename=report.pdf; filename*=UTF-8''%E6%8A%A5%E5%91%8A+1.pdf")),Some("报告+1.pdf".into()));
        assert_eq!(content_name(Some("inline; filename=\"hello.txt\"")),Some("hello.txt".into()));
        assert!(content_name(Some("attachment; filename*=UTF-8''bad%2G")).is_none());
    }
}
