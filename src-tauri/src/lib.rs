use tauri::{AppHandle, Emitter, Manager};
use rusqlite::params;
use chrono::Local;
use crate::db::{DbState, Reminder};

mod db;
mod local;

#[tauri::command]
async fn save_api_key(app: AppHandle, key: String) -> Result<(), local::types::ChatError> {
    local::identity::set_config(&app, "api_key", key.trim())
}

#[tauri::command]
async fn get_api_key(app: AppHandle) -> Result<String, local::types::ChatError> {
    local::identity::config(&app, "api_key")
}

#[tauri::command]
async fn save_config(app: AppHandle, key: String, value: String) -> Result<(), local::types::ChatError> {
    local::identity::set_config(&app, &key, &value)
}

#[tauri::command]
async fn get_config(app: AppHandle, key: String) -> Result<String, local::types::ChatError> {
    local::identity::config(&app, &key)
}

#[tauri::command]
async fn fetch_models(api_base: String, api_key: String) -> Result<serde_json::Value, local::types::ChatError> {
    local::provider::list_models(&api_base, &api_key).await
}

// -----------------------------------------------------------------------
// 安卓系统级闹铃管理 (通过 Kotlin 原生插件桥接 AlarmManager)
// -----------------------------------------------------------------------

/// 注册系统级精确闹铃（仅安卓生效，桌面端为空操作）
/// 前端调用此 command 后，安卓系统会在指定时间唤醒 AlarmReceiver 发送通知
#[tauri::command]
#[allow(unused_variables)]
async fn schedule_alarm(
    app: AppHandle,
    reminder_id: i64,
    trigger_at_ms: i64,
    task: String,
    agent_name: String,
) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        // 通过事件机制通知 Kotlin 原生层注册系统闹铃
        let task_log = task.clone();
        let app_clone = app.clone();
        app.run_on_main_thread(move || {
            let payload = serde_json::json!({
                "reminderId": reminder_id as i32,
                "triggerAtMs": trigger_at_ms,
                "task": task,
                "agentName": agent_name
            });
            let _ = app_clone.emit("ppc:schedule-alarm", payload);
        }).map_err(|e| format!("主线程调度失败: {}", e))?;

        println!("[Alarm] 已请求注册安卓闹铃: id={}, time={}, task={}", reminder_id, trigger_at_ms, task_log);
    }

    #[cfg(not(target_os = "android"))]
    {
        // 桌面端：闹铃由 Rust 轮询器处理，此处仅打印日志
        let _ = &app;
        println!("[Alarm] 桌面端忽略闹铃注册: id={}, task={}", reminder_id, task);
    }

    Ok(())
}

/// 取消已注册的系统级闹铃
#[tauri::command]
async fn cancel_alarm(
    app: AppHandle,
    reminder_id: i64,
) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        let app_clone = app.clone();
        app.run_on_main_thread(move || {
            let payload = serde_json::json!({
                "reminderId": reminder_id as i32
            });
            let _ = app_clone.emit("ppc:cancel-alarm", payload);
        }).map_err(|e| format!("主线程调度失败: {}", e))?;

        println!("[Alarm] 已请求取消安卓闹铃: id={}", reminder_id);
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = &app;
        println!("[Alarm] 桌面端忽略闹铃取消: id={}", reminder_id);
    }

    Ok(())
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
        start_background_reminder_poller(app.handle().clone());
        local::environment::start(app.handle().clone());
        Ok(())
    })
    .manage(local::runtime::ChatRuntime::default())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_haptics::init())
    .invoke_handler(tauri::generate_handler![
        db::get_memories,
        db::delete_memory,
        db::get_reminders,
        db::delete_reminder,
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
        local::identity::local_character_projection,
        local::blackjack::local_blackjack_snapshot,
        local::blackjack::local_blackjack_play,
        local::identity::local_save_model_preferences,
        local::backup::local_export_data,
        local::backup::local_import_data,
        local::backup::local_reset_data,
        schedule_alarm,
        cancel_alarm
    ])
            .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

// 启动后台提醒轮询：定期检查数据库中的提醒，到时后发送系统通知并删除该提醒
fn start_background_reminder_poller(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            // 当前时间（与前端一致的格式：YYYY-MM-DD HH:mm:ss）
            let now_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

            // 查询并触发提醒
            let state: tauri::State<DbState> = app.state();
            let (due_items, should_sleep) = {
                match state.conn.lock() {
                    Ok(conn) => {
                        let mut items: Vec<Reminder> = Vec::new();
                        if let Ok(mut stmt) = conn.prepare(
                            "SELECT id, task, time, timestamp, agent_id, delivered, recur, recur_until FROM reminders ORDER BY time ASC"
                        ) {
                            let rows = stmt.query_map([], |row| {
                                Ok(Reminder {
                                    id: row.get(0)?,
                                    task: row.get(1)?,
                                    time: row.get(2)?,
                                    timestamp: row.get(3)?,
                                    agent_id: row.get(4)?,
                                    delivered: {
                                        let v: i64 = row.get(5).unwrap_or(0);
                                        Some(v != 0)
                                    },
                                    recur: row.get(6).ok(),
                                    recur_until: row.get(7).ok(),
                                })
                            });
                            if let Ok(iter) = rows {
                                for r in iter {
                                    if let Ok(rem) = r {
                                        items.push(rem);
                                    }
                                }
                            }
                        }
                        (items, false)
                    }
                    Err(_) => (Vec::new(), true),
                }
            };

            if should_sleep {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                continue;
            }

            for rem in due_items {
                // 只触发到当前时间为止的提醒（字符串比较在此格式下等价于时间比较）
                if rem.time <= now_str {
                    // 已通知过则跳过
                    if rem.delivered.unwrap_or(false) {
                        continue;
                    }
                    // 发送事件给前端（由前端通过插件显示系统通知）
                    let _ = app.emit("ppc:reminder-due", serde_json::json!({
                        "id": rem.id,
                        "task": rem.task,
                        "time": rem.time,
                        "agent_id": rem.agent_id
                    }));

                    // 标记已通知；如为循环提醒则计算下次时间并重置 delivered
                    if let Ok(conn) = state.conn.lock() {
                        let recur = rem.recur.clone().unwrap_or_else(|| "none".to_string());
                        if recur == "none" {
                            let _ = conn.execute("UPDATE reminders SET delivered = 1 WHERE id = ?1", params![rem.id]);
                        } else {
                            use chrono::{NaiveDateTime, Duration};
                            if let Ok(dt) = NaiveDateTime::parse_from_str(&rem.time, "%Y-%m-%d %H:%M:%S") {
                                let next_dt = match recur.as_str() {
                                    "daily" => dt + Duration::days(1),
                                    "weekly" => dt + Duration::weeks(1),
                                    _ => dt,
                                };
                                let mut allow_reschedule = true;
                                if let Some(until_str) = rem.recur_until.clone() {
                                    if let Ok(until_dt) = NaiveDateTime::parse_from_str(&until_str, "%Y-%m-%d %H:%M:%S") {
                                        if next_dt > until_dt {
                                            allow_reschedule = false;
                                        }
                                    }
                                }
                                if allow_reschedule && (recur == "daily" || recur == "weekly") {
                                    let nstr = next_dt.format("%Y-%m-%d %H:%M:%S").to_string();
                                    let _ = conn.execute(
                                        "UPDATE reminders SET time = ?1, delivered = 0 WHERE id = ?2",
                                        params![nstr, rem.id]
                                    );
                                } else {
                                    let _ = conn.execute("UPDATE reminders SET delivered = 1 WHERE id = ?1", params![rem.id]);
                                }
                            } else {
                                let _ = conn.execute("UPDATE reminders SET delivered = 1 WHERE id = ?1", params![rem.id]);
                            }
                        }
                    }
                } else {
                    // 因为结果按时间升序，遇到未到期的即可提前结束
                    break;
                }
            }

            // 休眠一段时间后再次检查
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });
}
