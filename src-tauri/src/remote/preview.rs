//! HTML 预览只转发短期票据内的资源，沿用原生 TLS 信任；不向页面发送设备凭据。
use std::{collections::HashMap,sync::Mutex,time::{Duration,Instant}};
use tauri::{AppHandle,Manager,http::{Request,Response}};
use crate::{local::types::*,mode::{Coordinator,Token}};

#[derive(Clone)]
struct Entry { token:Token, endpoint:String, client:reqwest::Client, expires:Instant }
#[derive(Default)]
pub struct Runtime(Mutex<HashMap<String,Entry>>);
pub fn clear(app:&AppHandle) {if let Ok(mut entries)=app.state::<Runtime>().0.lock() {entries.clear();}}
pub(super) fn register(app:&AppHandle,session:&super::Session,path:&str)->ChatResult<()> {
    let ticket=ticket(path)?;
    let runtime=app.state::<Runtime>();let mut entries=runtime.0.lock().map_err(|_|super::inactive())?;
    entries.retain(|_,entry|entry.expires>Instant::now());
    if entries.len()>=16 {if let Some(oldest)=entries.iter().min_by_key(|(_,entry)|entry.expires).map(|(id,_)|id.clone()) {entries.remove(&oldest);}}
    entries.insert(ticket.to_owned(),Entry {token:session.token.clone(),endpoint:session.profile.endpoint.clone(),client:session.client.clone(),expires:Instant::now()+Duration::from_secs(600)});
    Ok(())
}
fn ticket(path:&str)->ChatResult<&str> {
    let ticket=path.strip_prefix("/html-previews/").and_then(|value|value.split_once('/')).map(|(ticket,_)|ticket).ok_or_else(super::inactive)?;
    if ticket.len()!=64 || !ticket.bytes().all(|byte|byte.is_ascii_hexdigit()) {return Err(super::inactive());}
    Ok(ticket)
}
fn target(endpoint:&str,path:&str,query:Option<&str>)->ChatResult<reqwest::Url> {
    let id=ticket(path)?;
    let base=reqwest::Url::parse(endpoint).map_err(|_|super::inactive())?;
    let mut url=reqwest::Url::parse(&format!("{}{path}",endpoint.trim_end_matches('/'))).map_err(|_|super::inactive())?;
    let prefix=format!("{}/html-previews/{id}/",base.path().trim_end_matches('/'));
    if base.origin()!=url.origin() || !url.path().starts_with(&prefix) || path.contains('\\') {return Err(super::inactive());}
    url.set_query(query);Ok(url)
}
fn policy()->String {
    // 自定义协议的本地入口是唯一允许的页内资源来源；保留 infOS 的脚本沙箱。
    let root="ppc-preview: http://ppc-preview.localhost https://ppc-preview.localhost";
    format!("sandbox allow-scripts; default-src 'none'; script-src 'unsafe-inline' 'unsafe-eval' {root} https:; style-src 'unsafe-inline' {root} https:; img-src {root} data: blob: https: http:; font-src {root} data: https:; media-src {root} data: blob: https:; connect-src {root}; frame-src 'none'; base-uri 'none'; form-action 'none'")
}
pub async fn respond(app:&AppHandle,request:Request<Vec<u8>>)->Response<Vec<u8>> {
    match read(app,request).await {
        Ok(response)=>response,
        Err(_)=>Response::builder().status(410).header("Cache-Control","no-store").header("Access-Control-Allow-Origin","*").body(Vec::new()).expect("预览错误响应有效"),
    }
}
async fn read(app:&AppHandle,request:Request<Vec<u8>>)->ChatResult<Response<Vec<u8>>> {
    if !matches!(request.method().as_str(),"GET"|"HEAD") {return Err(super::inactive());}
    let path=request.uri().path();let id=ticket(path)?;
    let entry=app.state::<Runtime>().0.lock().map_err(|_|super::inactive())?.get(id).filter(|entry|entry.expires>Instant::now()).cloned().ok_or_else(super::inactive)?;
    super::validate_token(app,&entry.token)?;
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();
    tokio::select! {biased;
        _=changes.changed()=>Err(super::inactive()),
        result=async {
            let url=target(&entry.endpoint,path,request.uri().query())?;
            let mut remote=entry.client.request(request.method().clone(),url).timeout(Duration::from_secs(30));
            if let Some(range)=request.headers().get("Range") {remote=remote.header("Range",range);}
            let response=remote.send().await.map_err(super::transport::send_error).map_err(|error| {super::notify_error(app,&entry.token,&error);error})?;
            let response=super::transport::checked(response).await?;
            let mut output=Response::builder().status(response.status().as_u16());
            for header in ["Content-Type","Content-Range","Accept-Ranges"] {if let Some(value)=response.headers().get(header) {output=output.header(header,value);}}
            let bytes=super::transport::body(response).await?;
            super::validate_token(app,&entry.token)?;
            output.header("Cache-Control","no-store").header("Access-Control-Allow-Origin","*").header("Referrer-Policy","no-referrer").header("X-Content-Type-Options","nosniff").header("Content-Security-Policy",policy()).body(bytes).map_err(|_|super::inactive())
        }=>result,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preview_cannot_escape_ticket_or_deployment_prefix() {
        let id="a".repeat(64);let path=format!("/html-previews/{id}/index.html");
        assert_eq!(target("https://example.com/infos",&path,None).unwrap().path(),format!("/infos{path}"));
        for suffix in ["../other/file","%2e%2e/other/file","../../api/agents","\\..\\api"] {
            assert!(target("https://example.com/infos",&format!("/html-previews/{id}/{suffix}"),None).is_err());
        }
        assert!(target("https://example.com","/api/agents",None).is_err());
        assert!(ticket("/html-previews/invalid/index.html").is_err());
    }
}
