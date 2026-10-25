//! 远程语音保留服务端预设与角色绑定，取消和播放状态只属于当前手机连接。
use std::{collections::HashMap, sync::Mutex, time::{Duration, Instant}};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};
use tokio::sync::watch;
use sha2::{Digest, Sha256};
use crate::{local::types::*, mode::{Coordinator, Token}};
use super::{cache, transport};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Input {
    pub text: String,
    #[serde(default, skip_serializing_if="Option::is_none")] pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if="Option::is_none")] pub preset_id: Option<String>,
    #[serde(default, skip_serializing_if="Option::is_none")] pub voice: Option<String>,
    #[serde(default, skip_serializing_if="Option::is_none")] pub speed: Option<f64>,
    #[serde(default, skip_serializing_if="Option::is_none")] pub format: Option<String>,
}
impl Input {
    fn body(&self)->ChatResult<Value> {
        let invalid=||ChatError::new("INVALID_INPUT","请检查朗读文字与声音设置");
        if self.text.trim().is_empty() || self.text.chars().count()>100_000 || self.text.contains('\0')
            || self.agent_id.as_ref().is_some_and(|v|v.chars().count()>120 || v.contains('\0'))
            || self.preset_id.as_ref().is_some_and(|v|v.chars().count()>120 || v.contains('\0'))
            || self.voice.as_ref().is_some_and(|v|v.chars().count()>300 || v.contains('\0'))
            || self.speed.is_some_and(|v|!v.is_finite() || !(0.25..=4.0).contains(&v))
            || self.format.as_ref().is_some_and(|v|!["mp3","wav","flac","opus","aac","pcm"].contains(&v.as_str())) {return Err(invalid());}
        serde_json::to_value(self).map_err(|_|invalid())
    }
}
struct Entry { token:Token, updated:Instant, signal:watch::Sender<bool>, running:bool }
#[derive(Default)]
pub struct Calls(Mutex<HashMap<String,Entry>>);
fn cancelled()->ChatError {ChatError::new("CANCELLED","已停止朗读").with_outcome("failed")}
fn signal(app:&AppHandle,token:&Token,id:&str,start:bool)->ChatResult<watch::Receiver<bool>> {
    let mode=app.state::<Coordinator>();let _gate=mode.gate(Some(token),false)?;
    app.state::<Calls>().signal(token,id,start)
}
impl Calls {
fn signal(&self,token:&Token,id:&str,start:bool)->ChatResult<watch::Receiver<bool>> {
    if uuid::Uuid::parse_str(id).is_err() {return Err(ChatError::new("INVALID_INPUT","朗读请求无效"));}
    let mut entries=self.0.lock().map_err(|_|super::inactive())?;
    entries.retain(|_,entry|entry.token==*token && (entry.running || entry.updated.elapsed()<Duration::from_secs(300)));
    if !entries.contains_key(id) && entries.len()>=128 {return Err(ChatError::new("REMOTE_RATE_LIMIT","朗读请求过多，请稍后重试"));}
    let entry=entries.entry(id.to_owned()).or_insert_with(||Entry {token:token.clone(),updated:Instant::now(),signal:watch::channel(false).0,running:false});
    if start {
        if *entry.signal.borrow() {return Err(cancelled());}
        if entry.running {return Err(ChatError::new("REQUEST_ALREADY_SENT","这段朗读已经在生成"));}
        entry.running=true;
    } else {entry.signal.send_replace(true);}
    entry.updated=Instant::now();Ok(entry.signal.subscribe())
}
}
struct Active { app:AppHandle, id:String, token:Token }
impl Drop for Active {
    fn drop(&mut self) {if let Ok(mut entries)=self.app.state::<Calls>().0.lock() {if entries.get(&self.id).is_some_and(|entry|entry.token==self.token) {entries.remove(&self.id);}}}
}
#[tauri::command]
pub fn remote_cancel_tts(app:AppHandle,token:Token,request_id:String)->ChatResult<()> {signal(&app,&token,&request_id,false).map(|_|())}
pub(super) async fn synthesize(app:AppHandle,token:Token,request_id:String,input:Input)->ChatResult<tauri::ipc::Response> {
    let body=input.body()?;
    let mut cancel=signal(&app,&token,&request_id,true)?;
    let _active=Active {app:app.clone(),id:request_id.clone(),token:token.clone()};
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();
    let session=tokio::select! {biased;_=cancel.changed()=>return Err(cancelled()),_=changes.changed()=>return Err(super::inactive()),result=super::session(&app,&token)=>result?,};
    cache::prepare(&app,&token,&request_id,&json!({"kind":"management","operation":"voice.tts","sha256":format!("{:x}",Sha256::digest(body.to_string().as_bytes()))}))?;
    let result=tokio::select! {biased;
        _=cancel.changed()=>Err(cancelled()),
        _=changes.changed()=>return Err(super::inactive().with_outcome("unknown")),
        result=async {
            let response=session.client.post(format!("{}/api/voice/tts",session.profile.endpoint)).bearer_auth(&session.secret)
                .timeout(Duration::from_secs(310)).json(&body).send().await.map_err(transport::send_error)?;
            let response=super::checked_response(&session,response).await?;
            let mime=response.headers().get("content-type").and_then(|v|v.to_str().ok()).unwrap_or("").to_owned();
            if !mime.starts_with("audio/") {return Err(ChatError::new("REMOTE_PROTOCOL","朗读服务未返回音频"));}
            // infOS 音频合同上限为 32 MiB，不套用普通 JSON 的 8 MiB 限制。
            let bytes=transport::body_with_limit(response,32*1024*1024).await?;
            if bytes.is_empty() {return Err(ChatError::new("REMOTE_PROTOCOL","朗读服务未返回音频"));}
            Ok::<_,ChatError>((mime,bytes))
        }=>result,
    };
    cache::settle(&app,&token,&request_id,result.as_ref().map(|_|Value::Null).map_err(Clone::clone))?;
    let (mime,bytes)=result?;super::validate_token(&app,&token)?;audio_packet(&mime,bytes)
}
fn audio_packet(mime:&str,bytes:Vec<u8>)->ChatResult<tauri::ipc::Response> {
    if mime.len()>256 || !mime.is_ascii() {return Err(ChatError::new("REMOTE_PROTOCOL","朗读服务未返回音频"));}
    // 二进制音频不转成数千万个 JSON 数字，避免手机主线程解析时卡顿。
    let mut packet=Vec::with_capacity(6+mime.len()+bytes.len());packet.extend_from_slice(b"PCTS");
    packet.extend_from_slice(&(mime.len() as u16).to_be_bytes());packet.extend_from_slice(mime.as_bytes());packet.extend(bytes);
    Ok(tauri::ipc::Response::new(packet))
}
pub(super) fn validate_reference(bytes:&[u8])->ChatResult<()> {
    let wav=bytes.len()>=12 && bytes.starts_with(b"RIFF") && &bytes[8..12]==b"WAVE";
    let mp3=bytes.starts_with(b"ID3") || bytes.len()>=2 && bytes[0]==0xff && bytes[1]&0xe0==0xe0;
    if bytes.is_empty() || bytes.len()>7*1024*1024 || !(wav || mp3) {return Err(ChatError::new("INVALID_FILE","请选择不超过 7MB 的 WAV 或 MP3 参考音频"));}Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn synthesis_preserves_bindings_and_preset_format_without_forcing_defaults() {
        let input:Input=serde_json::from_value(json!({"text":"你好","agentId":"pero","presetId":"mimo"})).unwrap();
        let body=input.body().unwrap();assert_eq!(body["agentId"],"pero");assert_eq!(body["presetId"],"mimo");assert!(body.get("format").is_none());assert!(body.get("speed").is_none());
        assert!(serde_json::from_value::<Input>(json!({"text":"你好","url":"https://example.invalid"})).is_err());
        for format in ["mp3","wav","flac","opus","aac","pcm"] {assert!(serde_json::from_value::<Input>(json!({"text":"你好","format":format})).unwrap().body().is_ok());}
        for value in [json!({"text":" "}),json!({"text":"字".repeat(100001)}),json!({"text":"你好","speed":0.1}),json!({"text":"你好","format":"html"})] {assert!(serde_json::from_value::<Input>(value).unwrap().body().is_err());}
    }
    #[test]
    fn reference_audio_is_bounded_and_cannot_upload_arbitrary_files() {
        assert!(validate_reference(b"RIFF1234WAVEdata").is_ok());assert!(validate_reference(b"ID3audio").is_ok());
        for bytes in [b"<html>".as_slice(),b"RIFF1234WEBP".as_slice(),b"".as_slice()] {assert!(validate_reference(bytes).is_err());}
        assert!(validate_reference(&vec![0;7*1024*1024+1]).is_err());
    }
    #[test]
    fn cancel_before_registration_and_old_connection_cannot_restart_speech() {
        let calls=Calls::default();
        let token=Token {installation_id:uuid::Uuid::new_v4().to_string(),epoch:1,target:crate::mode::Target::Remote {profile_id:uuid::Uuid::new_v4().to_string()}};
        let id=uuid::Uuid::new_v4().to_string();
        calls.signal(&token,&id,false).unwrap();
        assert_eq!(calls.signal(&token,&id,true).unwrap_err().code,"CANCELLED");
        let active=uuid::Uuid::new_v4().to_string();let receiver=calls.signal(&token,&active,true).unwrap();
        assert!(!*receiver.borrow());assert_eq!(calls.signal(&token,&active,true).unwrap_err().code,"REQUEST_ALREADY_SENT");
        calls.signal(&token,&active,false).unwrap();assert!(*receiver.borrow());
        let next=Token {epoch:2,..token};let next_id=uuid::Uuid::new_v4().to_string();
        calls.signal(&next,&next_id,true).unwrap();
        assert!(!calls.0.lock().unwrap().contains_key(&active));
        assert!(calls.signal(&next,"任意字符串",false).is_err());
    }
}
