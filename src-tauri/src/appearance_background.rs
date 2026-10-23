//! 本机背景与材质。图片独立存储，提交受运行令牌约束，不接触远程服务端配置。
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};
use crate::{local::types::*, mode::{Coordinator, Target, Token}};

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Scope { App, Character { #[serde(rename = "agentId")] agent_id: String } }
impl Scope {
    fn key(&self) -> ChatResult<String> {
        match self {
            Self::App => Ok("app".into()),
            Self::Character { agent_id } if !agent_id.is_empty() && agent_id.len() <= 256 && !agent_id.chars().any(|c| c.is_control() || matches!(c, '/' | '\\')) => Ok(format!("character:{agent_id}")),
            _ => Err(ChatError::new("INVALID_INPUT", "请先选择一个角色")),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Material {
    opacity: f64, blur: f64, brightness: f64, saturation: f64, contrast: f64,
    overlay_opacity: f64, surface_opacity: f64, surface_blur: f64,
    position_x: f64, position_y: f64, fit: String,
}
impl Default for Material {
    fn default() -> Self {
        Self { opacity: 0.95, blur: 0.0, brightness: 1.0, saturation: 1.0, contrast: 1.0,
            overlay_opacity: 0.18, surface_opacity: 0.8, surface_blur: 12.0,
            position_x: 50.0, position_y: 50.0, fit: "cover".into() }
    }
}
impl Material {
    fn validate(&self) -> ChatResult<()> {
        let ranges = [(self.opacity,0.0,1.0),(self.blur,0.0,24.0),(self.brightness,0.6,1.4),
            (self.saturation,0.0,1.6),(self.contrast,0.6,1.4),(self.overlay_opacity,0.0,0.85),
            (self.surface_opacity,0.55,1.0),(self.surface_blur,0.0,20.0),
            (self.position_x,0.0,100.0),(self.position_y,0.0,100.0)];
        if ranges.iter().any(|(value,min,max)| !value.is_finite() || value < min || value > max) || !["cover","contain"].contains(&self.fit.as_str()) {
            return Err(ChatError::new("INVALID_INPUT", "背景参数无效，请重新调整"));
        }
        Ok(())
    }
}
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings { inherit: bool, enabled: bool, light: Material, dark: Material }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageUpload { mime: String, bytes: Vec<u8> }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Save {
    scope: Scope, expected_revision: i64, settings: Settings,
    #[serde(default)] image: Option<ImageUpload>, #[serde(default)] remove_image: bool,
}

fn with_connection<T>(app: &AppHandle, token: &Token, operation: impl FnOnce(&mut Connection) -> ChatResult<T>) -> ChatResult<T> {
    let mode = app.state::<Coordinator>();
    let _gate = mode.gate(Some(token), false)?;
    let data = app.path().app_data_dir().map_err(|_| ChatError::new("STORAGE_ERROR", "背景存储暂不可用"))?.join("data");
    let path = match &token.target {
        Target::Standalone => data.join("ui").join("standalone-backgrounds.sqlite"),
        Target::Remote { profile_id } => {
            if uuid::Uuid::parse_str(profile_id).is_err() { return Err(ChatError::new("INVALID_PROFILE", "请重新选择连接")); }
            data.join("remote").join(profile_id).join("backgrounds.sqlite")
        }
    };
    std::fs::create_dir_all(path.parent().ok_or_else(|| ChatError::new("STORAGE_ERROR", "背景存储暂不可用"))?)
        .map_err(|_| ChatError::new("STORAGE_ERROR", "无法创建背景存储"))?;
    let mut conn = Connection::open(path)?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    conn.execute_batch("CREATE TABLE IF NOT EXISTS backgrounds(scope TEXT PRIMARY KEY,revision INTEGER NOT NULL,settings TEXT NOT NULL,mime TEXT,image BLOB);")?;
    operation(&mut conn)
}
fn metadata(conn: &Connection, scope: &Scope) -> ChatResult<Value> {
    let key = scope.key()?;
    let row: Option<(i64,String,Option<String>,bool)> = conn.query_row(
        "SELECT revision,settings,mime,image IS NOT NULL FROM backgrounds WHERE scope=?1", [&key],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
    let Some((revision,raw,mime,has_image)) = row else {
        return Ok(json!({"revision":0,"settings":Settings { inherit: matches!(scope,Scope::Character {..}), ..Settings::default() },"mime":null,"hasImage":false}));
    };
    // 保留版本号，损坏的参数也能通过保存默认值恢复，不陷入无法重置的死循环。
    let decoded = serde_json::from_str::<Settings>(&raw).ok()
        .filter(|settings| settings.light.validate().is_ok() && settings.dark.validate().is_ok());
    let recovered = decoded.is_none();
    let settings = decoded.unwrap_or_else(|| Settings { inherit: matches!(scope,Scope::Character {..}), ..Settings::default() });
    Ok(json!({"revision":revision,"settings":settings,"mime":mime,"hasImage":has_image,"settingsRecovered":recovered}))
}
fn image_valid(image: &ImageUpload) -> bool {
    let bytes = &image.bytes;
    if bytes.len() < 12 || bytes.len() > 8 * 1024 * 1024 { return false; }
    match image.mime.as_str() {
        "image/png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "image/jpeg" => bytes.starts_with(b"\xff\xd8\xff"),
        "image/webp" => bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP",
        _ => false,
    }
}

#[tauri::command]
pub async fn appearance_background_read(app: AppHandle, token: Token, scope: Scope) -> ChatResult<Value> {
    tauri::async_runtime::spawn_blocking(move || with_connection(&app, &token, |conn| metadata(conn, &scope)))
        .await.map_err(|_| ChatError::new("STORAGE_ERROR", "背景暂时无法读取"))?
}
#[tauri::command]
pub async fn appearance_background_image(app: AppHandle, token: Token, scope: Scope, revision: i64) -> ChatResult<tauri::ipc::Response> {
    tauri::async_runtime::spawn_blocking(move || with_connection(&app, &token, |conn| {
        let bytes: Option<Vec<u8>> = conn.query_row("SELECT image FROM backgrounds WHERE scope=?1 AND revision=?2 AND image IS NOT NULL", params![scope.key()?,revision], |r| r.get(0)).optional()?;
        Ok(tauri::ipc::Response::new(bytes.ok_or_else(|| ChatError::new("BACKGROUND_CHANGED", "背景图片已变化，请重新打开"))?))
    })).await.map_err(|_| ChatError::new("STORAGE_ERROR", "背景图片暂时无法读取"))?
}
#[tauri::command]
pub async fn appearance_background_save(app: AppHandle, token: Token, input: Save) -> ChatResult<Value> {
    input.settings.light.validate()?; input.settings.dark.validate()?;
    if input.expected_revision < 0 || input.image.as_ref().is_some_and(|image| !image_valid(image)) || input.image.is_some() && input.remove_image {
        return Err(ChatError::new("INVALID_INPUT", "请选择有效的 PNG、JPEG 或 WebP 图片"));
    }
    tauri::async_runtime::spawn_blocking(move || with_connection(&app, &token, |conn| {
        let key = input.scope.key()?;
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let current: i64 = tx.query_row("SELECT revision FROM backgrounds WHERE scope=?1", [&key], |r|r.get(0)).optional()?.unwrap_or(0);
        if current != input.expected_revision { return Err(ChatError::new("BACKGROUND_CHANGED", "背景设置已在别处更新，请重新打开此页")); }
        let revision = current.checked_add(1).ok_or_else(|| ChatError::new("STORAGE_ERROR", "背景设置暂时无法保存"))?;
        let raw = serde_json::to_string(&input.settings).map_err(|_| ChatError::new("INVALID_INPUT", "背景设置无效"))?;
        tx.execute("INSERT INTO backgrounds(scope,revision,settings) VALUES(?1,?2,?3) ON CONFLICT(scope) DO UPDATE SET revision=excluded.revision,settings=excluded.settings", params![key,revision,raw])?;
        if let Some(image) = &input.image { tx.execute("UPDATE backgrounds SET image=?2,mime=?3 WHERE scope=?1",params![key,image.bytes,image.mime])?; }
        else if input.remove_image { tx.execute("UPDATE backgrounds SET image=NULL,mime=NULL WHERE scope=?1",[&key])?; }
        let result = metadata(&tx, &input.scope)?;
        tx.commit()?;
        Ok(result)
    })).await.map_err(|_| ChatError::new("STORAGE_ERROR", "背景设置暂时无法保存"))?
}
