use tauri::{AppHandle, Manager};




mod db;
mod local;

#[tauri::command]
async fn save_api_key(app: AppHandle, key: String) -> Result<(), local::types::ChatError> {
    if key.len()>16000 || key.contains(['\r','\n']) {return Err(local::types::ChatError::new("INVALID_CONFIG","API 密钥格式无效"));}
    local::identity::set_config(&app, "api_key", key.trim())
}

#[tauri::command]
async fn get_api_key(app: AppHandle) -> Result<String, local::types::ChatError> {
    local::identity::config(&app, "api_key")
}

#[tauri::command]
async fn save_config(app: AppHandle, key: String, value: String) -> Result<(), local::types::ChatError> {
    if key!="ppc.activeAgent" && !key.starts_with("ppc.ui.") {
        return Err(local::types::ChatError::new("INVALID_CONFIG","业务配置请通过对应的管理接口保存"));
    }
    if value.len()>8000 { return Err(local::types::ChatError::new("INVALID_CONFIG","界面配置过大")); }
    if key=="ppc.activeAgent" && local::identity::agents(&app)?.get(&value).is_none() {
        return Err(local::types::ChatError::new("AGENT_NOT_FOUND","角色不存在"));
    }
    local::identity::set_config(&app, &key, &value)
}

#[tauri::command]
async fn get_config(app: AppHandle, key: String) -> Result<String, local::types::ChatError> {
    local::identity::config(&app, &key)
}

#[tauri::command]
async fn fetch_models(app:AppHandle,api_base:String,api_key:String,api_format:Option<String>,headers:Option<serde_json::Map<String,serde_json::Value>>)->Result<serde_json::Value,local::types::ChatError> {
    let mut settings=local::identity::model_settings(&local::identity::snapshot(&app)?)?;
    settings.base=api_base;settings.api_key=api_key;
    if let Some(format)=api_format {settings.api_format=format;}
    if let Some(headers)=headers {settings.headers=headers;}
    local::provider::list_models(&settings).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
        // 初始化数据库
        let db_state = db::init_db(app.handle())?;
        app.manage(db_state);

        local::initialize(app.handle()).map_err(|error| error.message)?;
        // 启动后台提醒轮询器
        local::reminders::start(app.handle().clone());
        local::environment::start(app.handle().clone());
        Ok(())
    })
    .manage(local::runtime::ChatRuntime::default())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_haptics::init())
    .plugin(local::reminder_platform::init())
    .invoke_handler(tauri::generate_handler![
        db::get_memories,
        db::delete_memory,
        local::reminders::local_reminder_query,
        local::reminders::local_manage_reminder,
        local::reminder_platform::local_reminder_runtime,
        local::reminder_platform::local_request_reminder_precision,
        local::reminder_platform::local_open_reminder_notifications,
        db::get_topics,
        db::update_topic_status,
        db::delete_topic,
        save_api_key, 
        get_api_key, 
        save_config,
        get_config,
        fetch_models,
        local::local_agents,
        local::local_delete_agent,
        local::local_snapshot,
        local::local_delete_message,
        local::local_cancel_chat,
        local::local_chat,
        local::group::local_group_chat,
        local::group::local_group_settings,
        local::identity::local_role_action,
        local::settings::local_save_settings,
        local::management::local_model_configuration,
        local::management::local_diary_query,
        local::management::local_manage_diary,
        local::management::local_diary_day,
        local::management::local_prompt_preview,
        local::prompts::local_prompt_workspace,
        local::prompts::local_manage_prompts,
        local::identity::local_character_projection,
        local::blackjack::local_blackjack_snapshot,
        local::blackjack::local_blackjack_play,
        local::identity::local_save_model_preferences,
        local::backup::local_export_data,
        local::backup::local_import_data,
        local::backup::local_reset_data,

    ])
            .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
