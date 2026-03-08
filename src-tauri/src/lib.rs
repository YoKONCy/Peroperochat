use tauri::{AppHandle, Emitter, Manager};
use tauri::WindowEvent;
use rusqlite::params;
use chrono::Local;
use crate::db::DbState;
use crate::db::Reminder;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri_plugin_store::StoreExt;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use futures::StreamExt;

mod db;


// 定义消息结构 - 支持多模态内容（文本 + 图片）
// content 可以是字符串，也可以是数组格式（OpenAI 多模态格式）
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    role: String,
    #[serde(deserialize_with = "deserialize_content")]
    content: serde_json::Value,
}

// 自定义反序列化函数：将字符串转换为 JSON Value，同时支持数组格式
fn deserialize_content<'de, D>(deserializer: D) -> Result<serde_json::Value, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(value)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    api_base: String,
    api_key: String,
}

// 定义流式响应的数据结构
#[derive(Clone, Serialize)]
struct StreamPayload {
    content: String,
    is_done: bool,
    error: Option<String>,
}

#[tauri::command]
async fn save_api_key(app: AppHandle, key: String) -> Result<(), String> {
    let store = app.store("data/settings.json").map_err(|e| e.to_string())?;
    store.set("api_key", serde_json::json!(key));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_api_key(app: AppHandle) -> Result<String, String> {
    let store = app.store("data/settings.json").map_err(|e| e.to_string())?;
    if let Some(val) = store.get("api_key") {
        if let Some(k) = val.as_str() {
            return Ok(k.to_string());
        }
    }
    Ok("".to_string())
}


#[tauri::command]
async fn save_config(app: AppHandle, key: String, value: String) -> Result<(), String> {
    let store = app.store("data/settings.json").map_err(|e| e.to_string())?;
    store.set(key, serde_json::json!(value));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_config(app: AppHandle, key: String) -> Result<String, String> {
    let store = app.store("data/settings.json").map_err(|e| e.to_string())?;
    if let Some(val) = store.get(key) {
        if let Some(v) = val.as_str() {
            return Ok(v.to_string());
        }
    }
    Ok("".to_string())
}

#[tauri::command]
async fn clear_config(app: AppHandle) -> Result<(), String> {
    let store = app.store("data/settings.json").map_err(|e| e.to_string())?;
    store.clear();
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn fetch_models(api_base: String, api_key: String) -> Result<serde_json::Value, String> {
    let client = Client::new();
    let mut url = api_base.trim_end_matches('/').to_string();
    if !url.ends_with("/v1") {
        url.push_str("/v1");
    }
    url.push_str("/models");

    let res = client.get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Request failed with status: {}", res.status()));
    }

    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    Ok(json)
}

#[tauri::command]
async fn chat_stream(
    app: AppHandle,
    request: ChatRequest,
) -> Result<(), String> {
    let client = Client::new();
    
    // 获取 API Key (优先使用请求中的，如果没有则从 Store 读取)
    let mut api_key = request.api_key.clone();
    if api_key.is_empty() {
         if let Ok(store) = app.store("data/settings.json") {
             if let Some(val) = store.get("api_key") {
                 if let Some(k) = val.as_str() {
                     api_key = k.to_string();
                 }
             }
         }
    }

    if api_key.is_empty() {
        return Err("API Key is missing (please set it in settings)".to_string());
    }

    // 处理 API Base URL，确保格式正确
    let mut url = request.api_base.trim_end_matches('/').to_string();
    if !url.ends_with("/v1") {
        url.push_str("/v1");
    }
    url.push_str("/chat/completions");

    // 构造请求体
    let body = serde_json::json!({
        "model": request.model,
        "messages": request.messages,
        "temperature": request.temperature,
        "stream": true, // 强制开启流式
    });

    println!("Sending request to: {}", url);

    // 发起请求
    let res = client.post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("API Request failed with status: {}", res.status()));
    }

    // 处理流式响应
    let mut stream = res.bytes_stream();
    
    // 在异步任务中处理流，避免阻塞主线程
    tauri::async_runtime::spawn(async move {
        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    let chunk = String::from_utf8_lossy(&bytes);
                    // 解析 SSE 格式 (data: {...})
                    for line in chunk.lines() {
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                let _ = app.emit("chat-stream-event", StreamPayload {
                                    content: "".into(),
                                    is_done: true,
                                    error: None,
                                });
                                break;
                            }
                            
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                                    let _ = app.emit("chat-stream-event", StreamPayload {
                                        content: content.to_string(),
                                        is_done: false,
                                        error: None,
                                    });
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = app.emit("chat-stream-event", StreamPayload {
                        content: "".into(),
                        is_done: true,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
    });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    // 拦截窗口关闭事件：隐藏到托盘而不是退出
    .on_window_event(|window, event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            let _ = window.hide();
            api.prevent_close();
        }
    })
    .setup(|app| {
        // 初始化数据库
        let db_state = db::init_db(app.handle()).expect("failed to init db");
        app.manage(db_state);

        // 创建系统托盘（最小实现）
        // 仅在桌面平台启用
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let show = MenuItemBuilder::with_id("show", "显示窗口").build(app).ok();
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app).ok();
            if let (Some(show), Some(quit)) = (show, quit) {
                if let Ok(menu) = MenuBuilder::new(app).items(&[&show, &quit]).build() {
                    let _ = TrayIconBuilder::with_id("main")
                        .menu(&menu)
                        .on_menu_event(|app: &AppHandle, event: tauri::menu::MenuEvent| {
                            match event.id.as_ref() {
                                "show" => {
                                    if let Some(w) = app.get_webview_window("main") {
                                        let _ = w.show();
                                        let _ = w.set_focus();
                                    }
                                }
                                "quit" => {
                                    app.exit(0);
                                }
                                _ => {}
                            }
                        })
                        .build(app);
                }
            }
        }

        // 启动后台提醒轮询器
        start_background_reminder_poller(app.handle().clone());
        Ok(())
    })
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_haptics::init())
    .plugin(tauri_plugin_store::Builder::default().build())
    .invoke_handler(tauri::generate_handler![
        db::add_memory,
        db::get_memories,
        db::delete_memory,
        db::delete_memories_by_msg_timestamp,
        db::clear_memories,
        db::get_relevant_memories,
        db::save_message,
        db::get_messages,
        db::clear_messages,
        db::delete_message_by_timestamp,
        db::add_reminder,
        db::get_reminders,
        db::delete_reminder,
        db::add_topic,
        db::get_topics,
        db::update_topic_status,
        db::delete_topic,
        save_api_key, 
        get_api_key, 
        save_config,
        get_config,
        clear_config,
        fetch_models,
        chat_stream
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
