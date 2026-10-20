use tauri::{AppHandle, Manager};




mod db;
mod local;
mod mode;
mod remote;

#[tauri::command]
async fn save_api_key(app: AppHandle, key: String, mode_token:crate::mode::Token) -> Result<(), local::types::ChatError> {
    crate::mode::LOCAL_TOKEN.scope(mode_token, async {
    if key.len()>16000 || key.contains(['\r','\n']) {return Err(local::types::ChatError::new("INVALID_CONFIG","API 密钥格式无效"));}
    local::identity::set_config(&app, "api_key", key.trim())

    }).await
}

#[tauri::command]
async fn get_api_key(app: AppHandle, mode_token:crate::mode::Token) -> Result<String, local::types::ChatError> {
    crate::mode::LOCAL_TOKEN.scope(mode_token, async {
    local::identity::config(&app, "api_key")

    }).await
}

#[tauri::command]
async fn save_config(app: AppHandle, key: String, value: String, mode_token:crate::mode::Token) -> Result<(), local::types::ChatError> {
    crate::mode::LOCAL_TOKEN.scope(mode_token, async {
    if key=="ppc.ui.appearance" || (key!="ppc.activeAgent" && !key.starts_with("ppc.ui.")) {
        return Err(local::types::ChatError::new("INVALID_CONFIG","业务配置请通过对应的管理接口保存"));
    }
    if value.len()>8000 { return Err(local::types::ChatError::new("INVALID_CONFIG","界面配置过大")); }
    if key=="ppc.activeAgent" && local::identity::agents(&app)?.get(&value).is_none() {
        return Err(local::types::ChatError::new("AGENT_NOT_FOUND","角色不存在"));
    }
    local::identity::set_config(&app, &key, &value)

    }).await
}

#[tauri::command]
async fn get_config(app: AppHandle, key: String, mode_token:crate::mode::Token) -> Result<String, local::types::ChatError> {
    crate::mode::LOCAL_TOKEN.scope(mode_token, async {
    local::identity::config(&app, &key)

    }).await
}

#[tauri::command]
async fn fetch_models(app:AppHandle,api_base:String,api_key:String,api_format:Option<String>,headers:Option<serde_json::Map<String,serde_json::Value>>, mode_token:crate::mode::Token)->Result<serde_json::Value,local::types::ChatError> {
    crate::mode::LOCAL_TOKEN.scope(mode_token, async {
    let mode=app.state::<mode::Coordinator>();let mut changes=mode.subscribe();let token=mode.token()?;
    {let _gate=mode.gate(Some(&token),true)?;}
    let mut settings=local::identity::model_settings(&local::identity::snapshot(&app)?)?;
    settings.base=api_base;settings.api_key=api_key;
    if let Some(format)=api_format {settings.api_format=format;}
    if let Some(headers)=headers {settings.headers=headers;}
    tokio::select! {biased;
        _=changes.changed()=>Err(local::types::ChatError::new("MODE_INACTIVE","运行模式已切换")),
        result=local::provider::list_models(&settings)=>result,
    }

    }).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
        #[cfg(mobile)]
        app.handle().plugin(tauri_plugin_barcode_scanner::init())?;
        // 控制状态先关闭旧代次，存储迁移期间任何冷启动入口都不能恢复业务。
        let control_path=app.path().app_data_dir()?.join("data/control/runtime.sqlite");
        let mode=mode::Coordinator::initialize(control_path).map_err(|error|error.message)?;
        app.manage(mode.clone());
        app.manage(local::runtime::ChatRuntime::with_mode(mode.clone()));
        // 初始化数据库
        let db_state = db::init_db(app.handle())?;
        app.manage(db_state);

        app.state::<db::DbState>().mode.set(mode.clone()).map_err(|_|"运行控制重复初始化")?;
        // 原生插件需要主事件循环处理回调；初始化返回后再撤销系统登记并开放运行门。
        let handle=app.handle().clone();
        let startup_switch=mode.switch_lock.clone().try_lock_owned().map_err(|_|"运行控制初始化冲突")?;
        tauri::async_runtime::spawn(async move {
            let _switch=startup_switch;
            use tauri::Emitter;
            let result=async {
                remote::background::suspend_async(handle.clone()).await?;
                local::reminder_platform::suspend_async(handle.clone()).await?;
                let selection=mode.snapshot()?.selected_target;
                match selection {
                    mode::Target::Standalone=>mode::prepare_standalone(&handle)?,
                    mode::Target::Remote {profile_id}=> {remote::profiles::get(&mode,&profile_id)?;},
                }
                mode::cleanup_local(&handle)?;
                mode.activate()
            }.await;
            let state=match result {Ok(state)=>Some(state),Err(error)=> {eprintln!("运行模式恢复失败：{}",error.message);mode.fail().ok()}};
            if let Some(state)=state {let _=handle.emit("mode-event",state);}
            remote::background::resume_async(handle).await;
        });
        // 启动后台提醒轮询器
        local::reminders::start(app.handle().clone());
        local::environment::start(app.handle().clone());
        remote::start(app.handle().clone());
        Ok(())
    })
    .manage(remote::Runtime::default())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_haptics::init())
    .plugin(local::reminder_platform::init())
    .plugin(remote::vault::init())
    .plugin(remote::background::init())
    .plugin(remote::files::init())
    .invoke_handler(tauri::generate_handler![
        mode::mode_snapshot,
        mode::mode_switch,
        remote::profiles::remote_profiles,
        remote::profiles::remote_decode_pairing,
        remote::profiles::remote_save_profile,
        remote::profiles::remote_delete_profile,
        remote::remote_query,
        remote::management::remote_manage,
        remote::management::remote_manage_upload,
        remote::management::remote_voice_tts,
        remote::files::remote_export_text,
        remote::downloads::remote_download_resource,
        remote::downloads::remote_cancel_download,
        remote::remote_action,
        remote::remote_chat,
        remote::remote_connect,
        remote::remote_session_status,
        remote::remote_logout_device,
        remote::notifications::remote_notifications,
        remote::notifications::remote_ack_notification,
        remote::background::remote_background_runtime,
        remote::background::remote_configure_background,
        remote::background::remote_background_settings,
        remote::background::remote_consume_notification_open,
        remote::remote_subscribe,
        remote::remote_surface_input,
        remote::remote_pending_requests,
        remote::remote_review_request,
        remote::remote_reconcile_request,
        remote::presentation::remote_ui_read,
        remote::presentation::remote_ui_write,
        remote::resources::remote_upload,
        remote::resources::remote_resource,
        remote::resources::remote_resource_info,
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
        local::appearance::local_appearance,
        local::appearance::local_save_appearance,
        local::appearance::local_room_background,
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
