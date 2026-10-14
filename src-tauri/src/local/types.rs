use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatError {
    pub code: String,
    pub message: String,
}

impl ChatError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self { code: code.into(), message: message.into() }
    }
}

impl From<rusqlite::Error> for ChatError {
    fn from(error: rusqlite::Error) -> Self {
        eprintln!("本地数据库操作失败：{error}");
        Self::new("STORAGE_ERROR", "本地数据读写失败，请检查磁盘空间后重试")
    }
}

pub type ChatResult<T> = Result<T, ChatError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub id: String,
    pub agent_id: String,
    pub channel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: i64,
    pub thread_id: String,
    pub pair_id: String,
    pub role: String,
    pub agent_id: String,
    pub name: String,
    pub content: Value,
    pub timestamp: i64,
    pub revision: i64,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TurnRequest {
    pub request_id: String,
    pub thread_id: String,
    pub agent_id: String,
    pub content: Option<Value>,
    pub regenerate_message_id: Option<i64>,
    pub reply_to_message_id: Option<i64>,
    #[serde(default)]
    pub ephemeral: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub thread: Thread,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnResult {
    pub request_id: String,
    pub message: Message,
    pub snapshot: Snapshot,
    pub state: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatEvent {
    pub request_id: String,
    pub thread_id: String,
    pub agent_id: String,
    pub kind: String,
    pub message: Option<Message>,
    pub user: Option<Message>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelMessage {
    pub role: String,
    pub content: Value,
}

pub fn text_content(value: &Value) -> String {
    if let Some(text) = value.as_str() { return text.to_owned(); }
    value.as_array().map(|parts| parts.iter()
        .filter_map(|part| part.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>().join("\n")).unwrap_or_default()
}

pub fn validate_content(value: &Value) -> ChatResult<()> {
    if let Some(text) = value.as_str() {
        if !text.trim().is_empty() && text.len() <= 100_000 { return Ok(()); }
    } else if let Some(parts) = value.as_array() {
        if !parts.is_empty() && parts.len() <= 10 && parts.iter().all(|part| {
            match part.get("type").and_then(Value::as_str) {
                Some("text") => part.get("text").and_then(Value::as_str)
                    .map(|text| text.len() <= 100_000).unwrap_or(false),
                Some("image_url") => part.pointer("/image_url/url").and_then(Value::as_str)
                    .map(|url| url.len() <= 14 * 1024 * 1024 &&
                        (url.starts_with("data:image/") || url.starts_with("https://") || url.starts_with("http://")))
                    .unwrap_or(false),
                _ => false,
            }
        }) && parts.iter().any(|part| part.get("type").and_then(Value::as_str) == Some("image_url") ||
            part.get("text").and_then(Value::as_str).is_some_and(|text| !text.trim().is_empty())) {
            return Ok(());
        }
    }
    Err(ChatError::new("INVALID_INPUT", "消息为空、过长或图片格式不受支持"))
}
