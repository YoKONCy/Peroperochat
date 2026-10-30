//! 文件流只经过原生层；临时文件不进入会话缓存，失败、取消和切换模式时清理。
use std::{path::PathBuf,sync::Mutex,time::{Duration,Instant}};
use serde_json::{json,Value};
use tauri::{AppHandle,Emitter,Manager};
use tokio::io::AsyncWriteExt;
use crate::{local::types::*,mode::{Coordinator,Token}};
use super::resources::Resource;

const MAX_DOWNLOAD:u64=2*1024*1024*1024;
#[derive(Default)]
pub(super) struct Downloads(Mutex<Option<(Token,String,tokio::sync::watch::Sender<bool>)>>);
struct Job {app:AppHandle,id:String}
impl Drop for Job {
    fn drop(&mut self) {
        if let Ok(mut slot)=self.app.state::<Downloads>().0.lock() {
            if slot.as_ref().is_some_and(|(_,id,_)|id==&self.id) {*slot=None;}
        }
    }
}
pub(super) struct Staged(pub PathBuf);
impl Staged {pub fn keep(mut self) {self.0=PathBuf::new();}}
impl Drop for Staged {
    fn drop(&mut self) {
        if self.0.as_os_str().is_empty() {return;}
        if std::fs::remove_file(&self.0).is_err() && self.0.exists() {
            // Windows 的异步写入句柄可能稍晚释放；清理线程不读取文件，也不继续下载。
            let path=self.0.clone();std::thread::spawn(move || {
                for _ in 0..20 {std::thread::sleep(Duration::from_millis(100));if std::fs::remove_file(&path).is_ok() || !path.exists() {break;}}
            });
        }
    }
}
fn failed()->ChatError {ChatError::new("FILE_DOWNLOAD_ERROR","文件下载失败，请检查网络和剩余空间后重试")}
fn cancelled()->ChatError {ChatError::new("DOWNLOAD_CANCELLED","已取消保存")}
fn check_size(size:u64,total:Option<u64>)->ChatResult<()> {
    if size>MAX_DOWNLOAD || total.is_some_and(|n|n>MAX_DOWNLOAD || size>n) {return Err(ChatError::new("REMOTE_TOO_LARGE","文件超过下载限制（2 GiB），或服务端返回的长度异常"));}
    Ok(())
}
async fn receive(mut response:reqwest::Response,file:&mut tokio::fs::File,mut report:impl FnMut(u64,Option<u64>))->ChatResult<()> {
    let total=response.content_length();check_size(0,total)?;
    let mut received=0u64;let mut last=Instant::now();report(received,total);
    while let Some(chunk)=tokio::time::timeout(Duration::from_secs(45),response.chunk()).await.map_err(|_|failed())?.map_err(|_|failed())? {
        received=received.checked_add(chunk.len() as u64).ok_or_else(failed)?;check_size(received,total)?;
        file.write_all(&chunk).await.map_err(|_|failed())?;
        if last.elapsed()>=Duration::from_millis(150) {report(received,total);last=Instant::now();}
    }
    if total.is_some_and(|n|n!=received) {return Err(failed());}
    file.flush().await.map_err(|_|failed())?;file.sync_all().await.map_err(|_|failed())?;
    report(received,total);Ok(())
}
fn progress(app:&AppHandle,token:&Token,id:&str,phase:&str,received:u64,total:Option<u64>) {
    let _=app.emit("remote-event",json!({"modeToken":token,"kind":"download","data":{"id":id,"phase":phase,"received":received,"total":total}}));
}
#[tauri::command]
pub async fn remote_cancel_download(app:AppHandle,token:Token,id:String)->ChatResult<()> {
    super::validate_token(&app,&token)?;
    if let Ok(slot)=app.state::<Downloads>().0.lock() {
        if let Some((owner,job,signal))=slot.as_ref() {if owner==&token && job==&id {signal.send_replace(true);}}
    }
    Ok(())
}
#[tauri::command]
pub async fn remote_download_resource(app:AppHandle,token:Token,id:String,resource:Resource,name:String)->ChatResult<Value> {
    super::validate_token(&app,&token)?;
    uuid::Uuid::parse_str(&id).map_err(|_|ChatError::new("INVALID_FILE","下载任务标识无效"))?;
    let (path,mut params)=resource.route()?;params.push(("download".into(),"1".into()));
    let (signal,mut cancel)=tokio::sync::watch::channel(false);
    {
        let state=app.state::<Downloads>();let mut slot=state.0.lock().map_err(|_|failed())?;
        if slot.is_some() {return Err(ChatError::new("DOWNLOAD_BUSY","请先完成或取消当前文件保存"));}
        *slot=Some((token.clone(),id.clone(),signal));
    }
    let _job=Job {app:app.clone(),id:id.clone()};
    let mode=app.state::<Coordinator>();let mut changes=mode.subscribe();
    let work_cancel=cancel.clone();
    let result=tokio::select! {biased;
        _=changes.changed()=>Err(super::inactive()),
        _=cancel.changed()=>Err(cancelled()),
        result=async {
            let session=super::session(&app,&token).await?;
            let mut url=reqwest::Url::parse(&format!("{}/api{path}",session.profile.endpoint)).map_err(|_|failed())?;
            url.query_pairs_mut().extend_pairs(params);
            let response=session.client.get(url).bearer_auth(&session.secret)
                .header("Accept-Encoding","identity").timeout(Duration::from_secs(30*60)).send().await.map_err(super::transport::send_error)?;
            let response=super::checked_response(&session,response).await?;
            if response.status()!=reqwest::StatusCode::OK {return Err(failed());}
            let total=response.content_length();check_size(0,total)?;
            let mime=response.headers().get("content-type").and_then(|v|v.to_str().ok()).unwrap_or("application/octet-stream").to_owned();
            let directory=app.path().app_cache_dir().map_err(|_|failed())?.join("remote-transfers");
            tokio::fs::create_dir_all(&directory).await.map_err(|_|failed())?;
            let path=directory.join(format!("{}.part",uuid::Uuid::new_v4()));
            let raw=std::fs::OpenOptions::new().write(true).create_new(true).open(&path).map_err(|_|failed())?;
            let staged=Staged(path);
            let mut file=tokio::fs::File::from_std(raw);
            let mut received=0;
            receive(response,&mut file,|size,total| {received=size;progress(&app,&token,&id,"downloading",size,total);}).await?;
            drop(file);
            super::validate_token(&app,&token)?;
            if *work_cancel.borrow() {return Err(cancelled());}
            progress(&app,&token,&id,"saving",received,total);
            super::files::save_staged(&app,&token,&id,staged,&name,&mime,work_cancel.clone()).await
        }=>result,
    };
    if result.is_err() {super::files::cancel_save(&app,&id).await;}
    let data=result?;super::validate_token(&app,&token)?;
    Ok(json!({"modeToken":token,"data":data}))
}

pub(super) fn init(app:&AppHandle) {
    app.manage(Downloads::default());
    // 上次进程被系统终止时只可能遗留此专用目录中的临时文件。
    if let Ok(root)=app.path().app_cache_dir() {
        if let Ok(entries)=std::fs::read_dir(root.join("remote-transfers")) {
            for entry in entries.flatten() {
                let path=entry.path();
                if path.extension().is_some_and(|v|v=="part") && path.file_stem().and_then(|v|v.to_str()).is_some_and(|v|uuid::Uuid::parse_str(v).is_ok()) {let _=std::fs::remove_file(path);}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unbounded_or_inconsistent_lengths() {
        assert!(check_size(MAX_DOWNLOAD,None).is_ok());
        assert!(check_size(MAX_DOWNLOAD+1,None).is_err());
        assert!(check_size(0,Some(MAX_DOWNLOAD+1)).is_err());
        assert!(check_size(4,Some(3)).is_err());
        assert!(check_size(0,Some(0)).is_ok());
    }
    #[test]
    fn removes_only_owned_temporary_file() {
        let directory=std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir(&directory).unwrap();let path=directory.join("download.part");
        std::fs::write(&path,b"partial").unwrap();{let _file=Staged(path.clone());}
        assert!(!path.exists());std::fs::remove_dir(directory).unwrap();
    }
    async fn response(raw:&'static [u8])->reqwest::Response {
        use tokio::io::AsyncReadExt;
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut socket,_)=listener.accept().await.unwrap();let mut request=[0u8;2048];let _=socket.read(&mut request).await;
            socket.write_all(raw).await.unwrap();socket.shutdown().await.unwrap();
        });
        reqwest::Client::builder().no_proxy().build().unwrap().get(format!("http://{address}/file")).send().await.unwrap()
    }
    #[tokio::test]
    async fn streams_exact_binary_without_json_or_text_conversion() {
        let path=std::env::temp_dir().join(format!("{}.part",uuid::Uuid::new_v4()));let owned=Staged(path.clone());
        let mut file=tokio::fs::File::create(&path).await.unwrap();let mut progress=Vec::new();
        let input=response(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\n\x00\xff\x01\r\n").await;
        receive(input,&mut file,|n,_|progress.push(n)).await.unwrap();drop(file);
        assert_eq!(std::fs::read(&path).unwrap(),b"\x00\xff\x01\r\n");assert_eq!(progress.last(),Some(&5));drop(owned);
    }
    #[tokio::test]
    async fn interrupted_response_cannot_be_reported_as_complete() {
        let path=std::env::temp_dir().join(format!("{}.part",uuid::Uuid::new_v4()));let owned=Staged(path.clone());
        let mut file=tokio::fs::File::create(&path).await.unwrap();
        let input=response(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\npartial").await;
        assert!(receive(input,&mut file,|_,_|{}).await.is_err());drop(file);drop(owned);assert!(!path.exists());
    }
}
