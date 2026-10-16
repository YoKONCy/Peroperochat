use serde_json::{json,Value};
use tauri::AppHandle;
#[cfg(target_os="android")]
use tauri::Manager;
use super::types::*;

#[cfg(target_os="android")]
struct Native(tauri::plugin::PluginHandle<tauri::Wry>);

pub fn init()->tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("reminder-platform").setup(|app,api| {
        #[cfg(target_os="android")]
        app.manage(Native(api.register_android_plugin("com.peroperochat.mobile","ReminderPlugin")?));
        #[cfg(not(target_os="android"))]
        let _=(app,api);
        Ok(())
    }).build()
}

pub fn synchronize(app:&AppHandle)->ChatResult<Value> {
    #[cfg(target_os="android")]
    {
        let database=app.path().app_data_dir().map_err(|_|ChatError::new("BACKGROUND_SYNC","无法读取提醒数据库路径"))?.join("data/peropero.db");
        app.state::<Native>().0.run_mobile_plugin("synchronize",json!({"databasePath":database.to_string_lossy()}))
            .map_err(|error| {eprintln!("安卓系统调度同步失败：{error}");ChatError::new("BACKGROUND_SYNC","提醒已保存，系统调度等待恢复")})
    }
    #[cfg(not(target_os="android"))]
    {let _=app;Ok(json!({"platform":"desktop","background":false}))}
}

#[tauri::command]
pub async fn local_reminder_runtime(app:AppHandle)->ChatResult<Value> {
    #[cfg(target_os="android")]
    {app.state::<Native>().0.run_mobile_plugin("status",json!({})).map_err(|_|ChatError::new("BACKGROUND_SYNC","无法读取系统提醒权限"))}
    #[cfg(not(target_os="android"))]
    {let _=app;Ok(json!({"platform":if cfg!(target_os="ios") {"ios"} else {"desktop"},"background":false}))}
}

#[tauri::command]
pub async fn local_request_reminder_precision(app:AppHandle)->ChatResult<Value> {
    #[cfg(target_os="android")]
    {app.state::<Native>().0.run_mobile_plugin("requestPrecision",json!({})).map_err(|_|ChatError::new("PERMISSION_ERROR","无法打开系统闹钟与提醒设置"))}
    #[cfg(not(target_os="android"))]
    {let _=app;Err(ChatError::new("UNSUPPORTED_PLATFORM","此平台不需要安卓精确提醒权限"))}
}

#[tauri::command]
pub async fn local_open_reminder_notifications(app:AppHandle)->ChatResult<Value> {
    #[cfg(target_os="android")]
    {app.state::<Native>().0.run_mobile_plugin("openNotificationSettings",json!({})).map_err(|_|ChatError::new("PERMISSION_ERROR","无法打开系统通知设置"))}
    #[cfg(not(target_os="android"))]
    {let _=app;Ok(Value::Null)}
}
