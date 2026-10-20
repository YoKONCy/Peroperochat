//! Tauri 只负责控制原生通知服务；冷启动接收器不依赖 AppHandle。
use serde_json::{json,Value};
use tauri::{AppHandle,Manager};
use crate::{local::types::*,mode::Token};

#[cfg(target_os="android")]
struct Native(tauri::plugin::PluginHandle<tauri::Wry>);
pub fn init()->tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("remote-background").setup(|app,api| {
        #[cfg(target_os="android")]
        app.manage(Native(api.register_android_plugin("com.peroperochat.mobile","RemoteReceiverPlugin")?));
        #[cfg(not(target_os="android"))]
        let _=(app,api);
        Ok(())
    }).build()
}
fn execute(app:&AppHandle,command:&str,input:&Value)->ChatResult<Value> {
    let root=app.path().app_data_dir().map_err(|_|super::inactive())?.join("data");
    super::background_store::execute(&root,command,input)
}
async fn native(app:AppHandle,command:&'static str,args:Value)->ChatResult<Value> {
    #[cfg(target_os="android")]
    {
        let task=tauri::async_runtime::spawn_blocking(move ||app.state::<Native>().0.run_mobile_plugin::<Value>(command,args)
            .map_err(|_|ChatError::new("BACKGROUND_SYNC","原生通知服务暂不可用，请打开连接设置重试")));
        tokio::time::timeout(std::time::Duration::from_secs(10),task).await.map_err(|_|ChatError::new("BACKGROUND_SYNC","原生通知服务响应超时"))?
            .map_err(|_|ChatError::new("BACKGROUND_SYNC","原生通知服务调用中断"))?
    }
    #[cfg(not(target_os="android"))]
    {let _=(app,command,args);Ok(json!({"platform":if cfg!(target_os="ios") {"ios"} else {"desktop"},"supported":false,"state":"unsupported"}))}
}
pub async fn suspend_async(app:AppHandle)->ChatResult<()> {native(app,"stop",json!({})).await?;Ok(())}
pub async fn resume_async(app:AppHandle) {if let Err(error)=native(app,"synchronize",json!({})).await {eprintln!("远程后台接收等待恢复：{}",error.message);}}
pub fn reauthorized(app:&AppHandle)->ChatResult<()> {
    let token=app.state::<crate::mode::Coordinator>().token()?;
    execute(app,"reauthorized",&json!({"modeToken":token}))?;Ok(())
}
/// 调用方已经持有运行门，这里不能再递归获取控制事务。
pub(super) fn authorization_failed(app:&AppHandle,token:&Token) {
    if let Ok(root)=app.path().app_data_dir() {
        if let Ok(conn)=super::background_store::connection(&root.join("data"),token) {
            let _=conn.execute("UPDATE notification_runtime SET blocked='REMOTE_AUTH_REQUIRED',state='blocked',updated=?1 WHERE id=1",[chrono::Utc::now().timestamp_millis()]);
        }
    }
}

#[tauri::command]
pub async fn remote_background_runtime(app:AppHandle,token:Token)->ChatResult<Value> {
    super::validate_token(&app,&token)?;
    let data=native(app.clone(),"status",json!({"modeToken":serde_json::to_string(&token).map_err(|_|super::inactive())?})).await?;
    super::validate_token(&app,&token)?;Ok(json!({"modeToken":token,"data":data}))
}
#[tauri::command]
pub async fn remote_configure_background(app:AppHandle,token:Token,enabled:bool)->ChatResult<Value> {
    super::validate_token(&app,&token)?;
    #[cfg(not(target_os="android"))]
    {let _=enabled;return Err(ChatError::new("UNSUPPORTED_PLATFORM","此平台暂未提供原生后台接收"));}
    #[cfg(target_os="android")]
    {
        execute(&app,"configure",&json!({"modeToken":token,"enabled":enabled}))?;
        native(app.clone(),"synchronize",json!({"modeToken":token})).await?;
        remote_background_runtime(app,token).await
    }
}
#[tauri::command]
pub async fn remote_background_settings(app:AppHandle,token:Token,kind:String)->ChatResult<Value> {
    super::validate_token(&app,&token)?;
    if !["battery","notifications"].contains(&kind.as_str()) {return Err(super::inactive());}
    native(app.clone(),"openSettings",json!({"kind":kind})).await?;
    super::validate_token(&app,&token)?;Ok(json!({"modeToken":token}))
}
#[tauri::command]
pub fn remote_consume_notification_open(app:AppHandle,token:Token)->ChatResult<Value> {
    super::validate_token(&app,&token)?;
    let data=execute(&app,"consumeOpen",&json!({"modeToken":token}))?;
    Ok(json!({"modeToken":token,"data":data}))
}
