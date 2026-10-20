//! 文件通过系统保存入口交付；二进制下载不经过前端内存。
use tauri::{AppHandle,Manager};
use serde_json::{json,Value};
use crate::{mode::Token,local::types::*};

#[cfg(target_os="android")]
struct Native(tauri::plugin::PluginHandle<tauri::Wry>);
pub fn init()->tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("remote-files").setup(|app,api| {
        super::downloads::init(app);
        #[cfg(target_os="android")]
        app.manage(Native(api.register_android_plugin("com.peroperochat.mobile","RemoteFilesPlugin")?));
        #[cfg(not(target_os="android"))]
        let _=(app,api);
        Ok(())
    }).build()
}

fn filename(name:&str)->String {
    let mut bytes=0;
    let name:String=name.chars().filter(|c|!c.is_control() && !['/','\\',':','*','?','"','<','>','|'].contains(c)).take(120).take_while(|c| {bytes+=c.len_utf8();bytes<=160}).collect();
    let name=name.trim().trim_matches('.');
    if name.is_empty() {"infos-file".into()} else {name.into()}
}

pub(super) async fn cancel_save(app:&AppHandle,id:&str) {
    #[cfg(target_os="android")]
    {
        let native=app.state::<Native>().0.clone();let args=json!({"id":id});
        let _=tauri::async_runtime::spawn_blocking(move ||native.run_mobile_plugin::<Value>("cancelSave",args)).await;
    }
    #[cfg(not(target_os="android"))]
    let _=(app,id);
}

pub(super) async fn save_staged(app:&AppHandle,token:&Token,id:&str,staged:super::downloads::Staged,name:&str,mime:&str,cancel:tokio::sync::watch::Receiver<bool>)->ChatResult<Value> {
    let name=filename(name);
    #[cfg(target_os="android")]
    {
        let native=app.state::<Native>().0.clone();
        let control=app.path().app_data_dir().map_err(|_|super::inactive())?.join("data/control/runtime.sqlite");
        let args=json!({"id":id,"path":staged.0.to_string_lossy(),"name":name,"mimeType":mime,"controlPath":control.to_string_lossy(),"modeToken":serde_json::to_string(token).map_err(|_|super::inactive())?});
        tauri::async_runtime::spawn_blocking(move || {
            // 保存选择器和复制完成之前由此闭包持有临时文件，取消时也不会提前删除复制源。
            let _owned=staged;
            if *cancel.borrow() {return Err(ChatError::new("DOWNLOAD_CANCELLED","已取消保存"));}
            native.run_mobile_plugin::<Value>("saveFile",args).map_err(|_|ChatError::new("FILE_EXPORT_ERROR","系统文件保存未完成"))
        }).await.map_err(|_|ChatError::new("FILE_EXPORT_ERROR","文件保存进程中断"))?
    }
    #[cfg(not(target_os="android"))]
    {
        let _=(id,mime);let app=app.clone();let token=token.clone();
        tauri::async_runtime::spawn_blocking(move || {
            use std::io::{Read,Write};
            #[cfg(target_os="ios")]
            let directory=app.path().document_dir().map(|path|path.join("infOS导出"));
            #[cfg(not(target_os="ios"))]
            let directory=app.path().download_dir();
            let directory=directory.map_err(|_|ChatError::new("FILE_EXPORT_ERROR","无法打开文件保存目录"))?;
            std::fs::create_dir_all(&directory).map_err(|_|ChatError::new("FILE_EXPORT_ERROR","无法创建文件保存目录"))?;
            let path=directory.join(format!("infos-{}-{name}",uuid::Uuid::new_v4()));
            let mut source=std::fs::File::open(&staged.0).map_err(|_|ChatError::new("FILE_EXPORT_ERROR","下载临时文件已失效"))?;
            let mut output=std::fs::OpenOptions::new().write(true).create_new(true).open(&path).map_err(|_|ChatError::new("FILE_EXPORT_ERROR","无法创建目标文件"))?;
            let partial=super::downloads::Staged(path.clone());
            let result=(||->ChatResult<()> {
                let mut buffer=vec![0u8;256*1024];
                loop {
                    if *cancel.borrow() {return Err(ChatError::new("DOWNLOAD_CANCELLED","已取消保存"));}
                    super::validate_token(&app,&token)?;
                    let n=source.read(&mut buffer).map_err(|_|ChatError::new("FILE_EXPORT_ERROR","下载文件读取失败"))?;if n==0 {break;}
                    output.write_all(&buffer[..n]).map_err(|_|ChatError::new("FILE_EXPORT_ERROR","目标文件写入失败，请检查剩余空间"))?;
                }
                output.sync_all().map_err(|_|ChatError::new("FILE_EXPORT_ERROR","文件保存未完成"))?;
                if *cancel.borrow() {return Err(ChatError::new("DOWNLOAD_CANCELLED","已取消保存"));}
                super::validate_token(&app,&token)
            })();
            drop(output);result?;partial.keep();drop(staged);
            Ok(json!({"saved":true,"path":path.to_string_lossy()}))
        }).await.map_err(|_|ChatError::new("FILE_EXPORT_ERROR","文件保存进程中断"))?
    }
}

#[cfg(any(target_os="android",test))]
#[no_mangle]
pub extern "system" fn Java_com_peroperochat_mobile_RemoteFilesPlugin_nativeGuard(mut env:jni::JNIEnv<'_>,_:jni::objects::JObject<'_>,path:jni::objects::JString<'_>,token:jni::objects::JString<'_>,action:jni::objects::JObject<'_>)->jni::sys::jboolean {
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||->Option<()> {
        let path:String=env.get_string(&path).ok()?.into();let raw:String=env.get_string(&token).ok()?.into();
        let token:Token=serde_json::from_str(&raw).ok()?;
        if !matches!(token.target,crate::mode::Target::Remote {..}) {return None;}
        let _gate=crate::mode::lock_existing(std::path::Path::new(&path),Some(&token),false).ok()?;
        env.call_method(&action,"run","()V",&[]).ok()?;Some(())
    })).ok().flatten().is_some();
    if env.exception_check().unwrap_or(false) {let _=env.exception_clear();return 0;}
    u8::from(result)
}
#[tauri::command]
pub async fn remote_export_text(app:AppHandle,token:Token,name:String,content:String)->ChatResult<Value> {
    super::validate_token(&app,&token)?;
    if content.len()>super::transport::MAX_BODY {return Err(ChatError::new("INVALID_FILE","导出内容超过 8 MiB，请缩小范围"));}
    let name=filename(&name);
    #[cfg(target_os="android")]
    let data={
        let native=app.state::<Native>().0.clone();let args=json!({"name":name,"content":content});
        tauri::async_runtime::spawn_blocking(move ||native.run_mobile_plugin::<Value>("saveText",args))
            .await.map_err(|_|ChatError::new("FILE_EXPORT_ERROR","文件保存进程中断"))?
            .map_err(|_|ChatError::new("FILE_EXPORT_ERROR","系统文件保存未完成"))?
    };
    #[cfg(not(target_os="android"))]
    let data={
        use std::io::Write;
        #[cfg(target_os="ios")]
        let directory=app.path().document_dir();
        #[cfg(not(target_os="ios"))]
        let directory=app.path().download_dir();
        let directory=directory.map_err(|_|ChatError::new("FILE_EXPORT_ERROR","此平台无法直接保存，请使用系统分享或复制文字"))?;
        let path=directory.join(format!("infos-{}-{name}",uuid::Uuid::new_v4()));
        let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(&path).map_err(|_|ChatError::new("FILE_EXPORT_ERROR","无法在下载目录创建导出文件"))?;
        if file.write_all(content.as_bytes()).and_then(|_|file.sync_all()).is_err() {drop(file);let _=std::fs::remove_file(&path);return Err(ChatError::new("FILE_EXPORT_ERROR","导出写入失败，已移除不完整文件"));}
        json!({"saved":true,"path":path.to_string_lossy()})
    };
    super::validate_token(&app,&token)?;Ok(json!({"modeToken":token,"data":data}))
}

#[cfg(test)]
mod tests {
    #[test]
    fn sanitizes_display_names_without_interpreting_paths() {
        assert_eq!(super::filename("../报告.pdf"),"报告.pdf");
        assert_eq!(super::filename("\r\n./\\:*?\"<>|"),"infos-file");
        assert!(super::filename(&"名".repeat(200)).chars().count()<=120);
    }
}
