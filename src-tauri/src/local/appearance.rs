use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use tauri::{AppHandle, Manager};
use crate::db::DbState;
use super::{db_lock, identity, types::*};
use chrono::{DateTime,FixedOffset,Timelike};

const KEY: &str = "ppc.ui.appearance";
const PERO_ROOM: &str = "/companion/sunlit-room.png";
const NANA_ROOM: &str = "/companion/moonlit-room.png";

fn default_background(id: &str) -> &'static str {
    if id == "nana" { NANA_ROOM } else { PERO_ROOM }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum RoomPeriod { Day, Twilight, Night }
impl RoomPeriod {
    fn at(hour:u32)->Self {
        match hour {8..=16=>Self::Day,5..=7|17..=19=>Self::Twilight,_=>Self::Night}
    }
    fn name(self)->&'static str {match self {Self::Day=>"day",Self::Twilight=>"twilight",Self::Night=>"night"}}
    fn image(self,id:&str)->&'static str {
        match (id=="nana",self) {
            (false,Self::Day)=>PERO_ROOM,
            (false,Self::Twilight)=>"/companion/pero-twilight-room.png",
            (false,Self::Night)=>"/companion/pero-night-room.png",
            (true,Self::Day)=>"/companion/nana-day-room.png",
            (true,Self::Twilight)=>"/companion/nana-twilight-room.png",
            (true,Self::Night)=>NANA_ROOM,
        }
    }
}

fn background_at(agent:&Value,id:&str,now:DateTime<FixedOffset>)->Value {
    let period=RoomPeriod::at(now.hour());let fallback=period.image(id);
    let saved=agent["room_background"].as_str().unwrap_or("");
    // 内置默认路径是自动房间的旧存储形式；上传图片或其他地址始终保持固定。
    let automatic=saved.is_empty() || saved==default_background(id);
    json!({"agentId":id,"background":if automatic {fallback} else {saved},"defaultBackground":fallback,
        "backgroundMode":if automatic {"automatic"} else {"custom"},"backgroundPeriod":period.name(),
        "selectedAt":now.to_rfc3339(),"utcOffsetMinutes":now.offset().local_minus_utc()/60})
}

#[tauri::command]
pub fn local_room_background(app:AppHandle,agent_id:String,utc_offset_minutes:i32,mode_token:crate::mode::Token)->ChatResult<Value> {
    crate::mode::LOCAL_TOKEN.sync_scope(mode_token,|| {
        // 切屏时重新采集设备偏移，避免移动系统更改时区后沿用进程内时区缓存。
        if !(-840..=840).contains(&utc_offset_minutes) {return Err(ChatError::new("INVALID_INPUT","设备时区偏移无效"));}
        let offset=FixedOffset::east_opt(utc_offset_minutes*60).ok_or_else(||ChatError::new("INVALID_INPUT","设备时区偏移无效"))?;
        let agents=identity::agents_from(&identity::snapshot(&app)?)?;
        let agent=agents.get(&agent_id).ok_or_else(||ChatError::new("AGENT_NOT_FOUND","角色不存在"))?;
        Ok(background_at(agent,&agent_id,chrono::Utc::now().with_timezone(&offset)))
    })
}

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
        "pero" => ("/companion/pero-standing.svg", default_background(id), "standing"),
        "nana" => ("/companion/nana-standing.png", default_background(id), "standing"),
        _ => ("", default_background(id), "standing"),
    };
    // 旧内置房间升级为插画，用户上传图片、外链和显式空值不覆盖。
    let previous_room = fields.get("room_background").and_then(Value::as_str);
    if (id == "nana" && previous_room == Some("/rooms/nana.svg")) ||
        (id != "nana" && previous_room == Some("/rooms/pero.svg")) {
        fields.insert("room_background".into(), json!(room));
    }
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
    let mut result=background_at(agent,agent_id,chrono::Local::now().fixed_offset());
    result["illustration"]=agent["standing_image"].clone();result["layout"]=agent["illustration_layout"].clone();
    Ok(result)
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

    #[test]
    fn room_defaults_migrate_legacy_svg_and_keep_custom_uploads() {
        for (id, old, expected) in [("nana", "/rooms/nana.svg", NANA_ROOM), ("pero", "/rooms/pero.svg", PERO_ROOM), ("custom", "/rooms/pero.svg", PERO_ROOM)] {
            let mut fields = json!({"room_background":old}).as_object().unwrap().clone();
            normalize_assets(&mut fields, id).unwrap();assert_eq!(fields["room_background"], expected);
        }
        for (id, expected) in [("nana", NANA_ROOM), ("pero", PERO_ROOM), ("custom", PERO_ROOM)] {
            let mut fields = Map::new();normalize_assets(&mut fields, id).unwrap();assert_eq!(fields["room_background"], expected);
            for custom in ["data:image/png;base64,aGVsbG8=", "https://example.com/my-room.webp", ""] {
                fields.insert("room_background".into(), json!(custom));normalize_assets(&mut fields, id).unwrap();
                assert_eq!(fields["room_background"], custom);
            }
        }
    }

    #[test]
    fn empty_room_uses_character_default_in_rust_projection() {
        let agents=json!({"nana":{"name":"Nana","room_background":""},"custom":{"name":"自定义","room_background":""}});
        let config=json!({"ppc.agents":agents.to_string()});
        let now=DateTime::parse_from_rfc3339("2026-10-21T22:00:00+08:00").unwrap();
        let agents=identity::agents_from(&config).unwrap();
        assert_eq!(background_at(&agents["nana"],"nana",now)["background"],NANA_ROOM);
        assert_eq!(background_at(&agents["custom"],"custom",now)["background"],"/companion/pero-night-room.png");
    }

    #[test]
    fn room_period_boundaries_cover_day_and_shared_dawn_dusk() {
        for (hour,period) in [(0,RoomPeriod::Night),(4,RoomPeriod::Night),(5,RoomPeriod::Twilight),(7,RoomPeriod::Twilight),(8,RoomPeriod::Day),(16,RoomPeriod::Day),(17,RoomPeriod::Twilight),(19,RoomPeriod::Twilight),(20,RoomPeriod::Night),(23,RoomPeriod::Night)] {
            assert_eq!(RoomPeriod::at(hour),period);
        }
    }

    #[test]
    fn room_selection_uses_timezone_offset_and_keeps_custom_artwork_fixed() {
        let utc=DateTime::parse_from_rfc3339("2026-10-21T00:30:00Z").unwrap();
        let east=utc.with_timezone(&FixedOffset::east_opt(8*3600).unwrap());
        let west=utc.with_timezone(&FixedOffset::west_opt(7*3600).unwrap());
        let quarter=utc.with_timezone(&FixedOffset::east_opt(5*3600+45*60).unwrap());
        assert_eq!(background_at(&json!({}),"nana",east)["background"],"/companion/nana-day-room.png");
        assert_eq!(background_at(&json!({}),"nana",west)["backgroundPeriod"],"twilight");
        assert_eq!(background_at(&json!({}),"pero",quarter)["backgroundPeriod"],"twilight");
        assert_eq!(background_at(&json!({}),"pero",quarter)["utcOffsetMinutes"],345);
        for id in ["pero","nana","custom"] {
            let default=json!({"room_background":default_background(id)});
            assert_eq!(background_at(&default,id,east)["backgroundMode"],"automatic");
            for image in ["https://example.com/room.webp","data:image/png;base64,aGVsbG8="] {
                let custom=json!({"room_background":image});
                assert_eq!(background_at(&custom,id,east)["background"],image);
                assert_eq!(background_at(&custom,id,west)["background"],image);
                assert_eq!(background_at(&custom,id,west)["backgroundMode"],"custom");
            }
        }
    }

    #[test]
    fn all_room_variants_are_packaged_and_custom_defaults_follow_pero() {
        let public=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../public");
        for period in [RoomPeriod::Day,RoomPeriod::Twilight,RoomPeriod::Night] {
            assert_eq!(period.image("custom"),period.image("pero"));
            assert_ne!(period.image("nana"),period.image("pero"));
            for id in ["pero","nana"] {assert!(public.join(period.image(id).trim_start_matches('/')).is_file(),"房间时段素材缺失");}
        }
    }
}
