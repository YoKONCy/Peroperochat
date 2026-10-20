use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use tauri::{AppHandle, Manager};
use crate::db::DbState;
use super::{db_lock, identity, types::*};

const KEY: &str = "ppc.ui.appearance";

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Appearance {
    pub display_mode: String,
    pub palette: String,
    pub glass: u8,
    pub figure_size: u8,
    pub motion: bool,
}
impl Default for Appearance {
    fn default() -> Self {
        Self { display_mode: "illustration".into(), palette: "auto".into(), glass: 80, figure_size: 57, motion: true }
    }
}
impl Appearance {
    fn validate(&self) -> ChatResult<()> {
        if !matches!(self.display_mode.as_str(), "live2d" | "illustration") ||
            !matches!(self.palette.as_str(), "auto" | "sky" | "sakura" | "mint" | "violet") ||
            !(60..=94).contains(&self.glass) || !(44..=64).contains(&self.figure_size) {
            return Err(ChatError::new("INVALID_CONFIG", "角色显示或界面参数无效"));
        }
        Ok(())
    }
}

fn read(config: &Value) -> ChatResult<Appearance> {
    let raw = match config.get(KEY) {
        None => return Ok(Appearance::default()),
        Some(Value::String(value)) => value.as_str(),
        Some(_) => return Err(ChatError::new("INVALID_CONFIG", "显示设置格式无效，请重新保存显示偏好")),
    };
    if raw.is_empty() { return Ok(Appearance::default()); }
    let value: Appearance = serde_json::from_str(raw).map_err(|_| ChatError::new("INVALID_CONFIG", "显示设置无法读取，请重新保存显示偏好"))?;
    value.validate()?;
    Ok(value)
}

#[tauri::command]
pub fn local_appearance(app: AppHandle, mode_token: crate::mode::Token) -> ChatResult<Appearance> {
    crate::mode::LOCAL_TOKEN.sync_scope(mode_token, || read(&identity::snapshot(&app)?))
}

#[tauri::command]
pub fn local_save_appearance(app: AppHandle, appearance: Appearance, mode_token: crate::mode::Token) -> ChatResult<Appearance> {
    crate::mode::LOCAL_TOKEN.sync_scope(mode_token, || {
        appearance.validate()?;
        let db = app.state::<DbState>();
        let conn = db_lock(&db)?;
        let encoded = serde_json::to_string(&appearance).map_err(|_| ChatError::new("INVALID_CONFIG", "显示设置无法保存"))?;
        identity::set_value_in(&conn, KEY, &json!(encoded))?;
        Ok(appearance)
    })
}

fn valid_image(value: &str) -> bool {
    if value.is_empty() { return true; }
    if value.len() > 8_000_000 || value.chars().any(|c| c.is_control()) { return false; }
    if value.starts_with('/') && !value.starts_with("//") && !value.contains(['?', '#', '\\']) && !value.split('/').any(|part| matches!(part, "." | "..")) {
        return ["/companion/", "/rooms/", "/avatars/", "/remote/"].iter().any(|prefix| value.starts_with(*prefix));
    }
    if ["data:image/png;base64,", "data:image/jpeg;base64,", "data:image/webp;base64,"].iter().any(|prefix| value.starts_with(*prefix)) {
        return value.split_once(',').is_some_and(|(_, content)| !content.is_empty() && content.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'+' | b'/' | b'=')));
    }
    reqwest::Url::parse(value).ok().is_some_and(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some() && url.username().is_empty() && url.password().is_none())
}

// 角色包携带自己的立绘、完整 CG 和房间资源，界面组件不按角色名字绑定素材。
pub fn normalize_assets(fields: &mut Map<String, Value>, id: &str) -> ChatResult<()> {
    let (image, room, layout) = match id {
        "pero" => ("/companion/pero-standing.svg", "/companion/sunlit-room.png", "standing"),
        "nana" => ("/companion/nana-standing.png", "/rooms/nana.svg", "standing"),
        _ => ("", "/companion/sunlit-room.png", "standing"),
    };
    // 仅迁移旧版内置素材组合，自定义图片和显式空值保持原样。
    if id == "nana" && fields.get("standing_image").and_then(Value::as_str) == Some("/companion/nana-cg.png") &&
        fields.get("illustration_layout").and_then(Value::as_str).is_none_or(|value| value == "card") {
        fields.insert("standing_image".into(), json!(image));
        fields.insert("illustration_layout".into(), json!(layout));
    }
    for (key, fallback) in [("standing_image", image), ("room_background", room)] {
        if !fields.contains_key(key) { fields.insert(key.into(), json!(fallback)); }
        let value = fields.get(key).and_then(Value::as_str).ok_or_else(|| ChatError::new("INVALID_CONFIG", "角色立绘和房间地址必须为文本"))?;
        if !valid_image(value) { return Err(ChatError::new("INVALID_CONFIG", "角色图片地址无效，请使用应用内素材、PNG/JPEG/WebP 图片或 HTTP(S) 地址")); }
    }
    if !fields.contains_key("illustration_layout") { fields.insert("illustration_layout".into(), json!(layout)); }
    if !fields.get("illustration_layout").and_then(Value::as_str).is_some_and(|value| matches!(value, "standing" | "card")) {
        return Err(ChatError::new("INVALID_CONFIG", "立绘排版应为透明立绘或完整 CG"));
    }
    Ok(())
}

pub fn presentation(config: &Value, agent_id: &str) -> ChatResult<Value> {
    let agents = identity::agents_from(config)?;
    let agent = agents.get(agent_id).ok_or_else(|| ChatError::new("AGENT_NOT_FOUND", "角色不存在"))?;
    Ok(json!({
        "illustration": agent["standing_image"],
        "layout": agent["illustration_layout"],
        "background": agent["room_background"].as_str().filter(|value| !value.is_empty()).unwrap_or("/companion/sunlit-room.png"),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_legacy_nana_card_to_cutout() {
        let mut fields = json!({ "standing_image": "/companion/nana-cg.png", "illustration_layout": "card" }).as_object().unwrap().clone();
        normalize_assets(&mut fields, "nana").unwrap();
        assert_eq!(fields["standing_image"], "/companion/nana-standing.png");
        assert_eq!(fields["illustration_layout"], "standing");
    }

    #[test]
    fn preserve_custom_artwork_and_explicit_empty_value() {
        for image in ["https://example.com/nana.png", ""] {
            let mut fields = json!({ "standing_image": image, "illustration_layout": "card" }).as_object().unwrap().clone();
            normalize_assets(&mut fields, "nana").unwrap();
            assert_eq!(fields["standing_image"], image);
            assert_eq!(fields["illustration_layout"], "card");
        }
        let mut fields = json!({ "standing_image": "/companion/nana-cg.png", "illustration_layout": "standing" }).as_object().unwrap().clone();
        normalize_assets(&mut fields, "nana").unwrap();
        assert_eq!(fields["standing_image"], "/companion/nana-cg.png");
    }
}
