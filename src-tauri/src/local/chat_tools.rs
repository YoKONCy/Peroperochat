use std::collections::HashMap;
use rusqlite::{params,Connection};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use crate::db::DbState;
use super::{db_lock,diary,identity,reminders,types::*};

#[derive(Debug,Clone,Default,Serialize,Deserialize)]
#[serde(default,deny_unknown_fields)]
pub struct Status { pub mood:Option<String>,pub vibe:Option<String>,pub mind:Option<String> }
#[derive(Debug,Clone,Default,Serialize,Deserialize)]
#[serde(default,deny_unknown_fields)]
pub struct Interactions {
    pub welcome:Option<String>,pub idle:Option<Vec<String>>,pub back:Option<Vec<String>>,
    pub click:Option<HashMap<String,Vec<String>>>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterPatch { #[serde(default)]pub state:Option<Status>,#[serde(default)]pub interactions:Option<Interactions> }

impl CharacterPatch {
    fn validate(&self)->ChatResult<()> {
        let mut strings=Vec::new(); let mut supplied=false;
        if let Some(s)=&self.state {
            for v in [&s.mood,&s.vibe,&s.mind].into_iter().flatten() { strings.push(v.as_str()); supplied=true; }
        }
        if let Some(i)=&self.interactions {
            if let Some(w)=&i.welcome { strings.push(w); supplied=true; }
            let mut arrays=Vec::new();
            for lines in [&i.idle,&i.back].into_iter().flatten() { arrays.push(lines); supplied=true; }
            if let Some(click)=&i.click {
                if click.keys().any(|k| !["head","chest","body","default"].contains(&k.as_str())) { return Err(ChatError::new("INVALID_TOOL_INPUT","互动部位无效")); }
                arrays.extend(click.values()); supplied=true;
            }
            for lines in arrays { if lines.len()>50 { return Err(ChatError::new("INVALID_TOOL_INPUT","互动台词数量过多")); } strings.extend(lines.iter().map(String::as_str)); }
        }
        if !supplied || strings.iter().any(|s|s.len()>2000) { return Err(ChatError::new("INVALID_TOOL_INPUT","角色更新为空或文本过长")); }
        Ok(())
    }

    pub fn apply(&self,conn:&Connection,agent_id:&str)->ChatResult<()> {
        self.validate()?;
        if let Some(state)=&self.state {
            for (key,value) in [("mood",&state.mood),("vibe",&state.vibe),("mind",&state.mind)] {
                if let Some(value)=value { identity::set_value_in(conn,&format!("ppc.{agent_id}.{key}"),&json!(value))?; }
            }
        }
        if let Some(interactions)=&self.interactions {
            let key=format!("ppc.{agent_id}.waifu.texts");
            let config=identity::snapshot_in(conn)?;
            let mut texts=config[&key].as_str().and_then(|s|serde_json::from_str::<Value>(s).ok()).filter(Value::is_object).unwrap_or(json!({}));
            if let Some(welcome)=&interactions.welcome {
                texts["welcome"]=json!(welcome);
                for time in ["morningEarly","morning","noon","afternoon","eveningSunset","night","midnight"] { texts[format!("welcome_timeRanges_{time}")]=json!(welcome); }
                texts["lateNight_01"]=json!(welcome);texts["lateNight_02"]=json!(welcome);
            }
            if let Some(lines)=&interactions.back { texts["visibilityBack"]=json!(lines); }
            let mut arrays=Vec::new();
            if let Some(lines)=&interactions.idle { texts["idleMessages"]=json!(lines);arrays.push(("idleMessages".to_owned(),lines)); }
            if let Some(click)=&interactions.click { for (part,lines) in click {
                arrays.push((if part=="default" { "click_messages".into() } else { format!("click_{part}") },lines));
            } }
            for (prefix,lines) in arrays {
                texts.as_object_mut().unwrap().retain(|key,_| !key.starts_with(&format!("{prefix}_")));
                for (index,line) in lines.iter().enumerate() { texts[format!("{prefix}_{:02}",index+1)]=json!(line); }
            }
            identity::set_value_in(conn,&key,&json!(texts.to_string()))?;
        }
        Ok(())
    }
}

pub fn definitions()->Vec<Value> {
    vec![
        json!({"name":"write_diary","description":"记录值得长期保留的真实互动，以第一人称写简短日记，不记录猜测。草稿只在本轮回复成功保存后提交。提供 id 与 expectedRevision 可修订自己的既有日记。",
            "parameters":{"type":"object","additionalProperties":false,"required":["content"],"properties":{
                "content":{"type":"string"},"date":{"type":"string","description":"YYYY-MM-DD，省略时使用今天"},"tags":{"type":"array","items":{"type":"string"}},
                "id":{"type":"integer"},"expectedRevision":{"type":"integer"}}}}),
        json!({"name":"query_diary","description":"查询当前角色已经提交的日记，支持关键词、日期范围或最近记录，不使用向量模型。",
            "parameters":{"type":"object","additionalProperties":false,"properties":{
                "keyword":{"type":"string"},"from":{"type":"string"},"to":{"type":"string"},"limit":{"type":"integer","minimum":1,"maximum":20}}}}),
        json!({"name":"update_character","description":"在一个工具中更新角色心情、状态、内心独白和互动台词；只传需要修改的字段。此更新在本轮回复成功保存后提交。",
            "parameters":{"type":"object","additionalProperties":false,"properties":{
                "state":{"type":"object","additionalProperties":false,"properties":{"mood":{"type":"string"},"vibe":{"type":"string"},"mind":{"type":"string"}}},
                "interactions":{"type":"object","additionalProperties":false,"properties":{
                    "welcome":{"type":"string"},"idle":{"type":"array","items":{"type":"string"}},"back":{"type":"array","items":{"type":"string"}},
                    "click":{"type":"object","additionalProperties":false,"properties":{
                        "head":{"type":"array","items":{"type":"string"}},"chest":{"type":"array","items":{"type":"string"}},
                        "body":{"type":"array","items":{"type":"string"}},"default":{"type":"array","items":{"type":"string"}}}}}}}}}),
        json!({"name":"query_reminders","description":"查询当前角色已保存的提醒，返回身份、修订号、设备当前本地时间和 UTC 偏移。设置相对时间提醒前先查询；不包含本轮待提交草稿。",
            "parameters":{"type":"object","additionalProperties":false,"properties":{"keyword":{"type":"string"},"includeDelivered":{"type":"boolean"},"limit":{"type":"integer","minimum":1,"maximum":20}}}}),
        json!({"name":"manage_reminder","description":"按用户意愿创建、修改或取消当前角色的提醒。action 为 create/update/delete；创建需 task/time，时间使用设备本地 YYYY-MM-DD HH:mm:ss；不明确的时间先询问用户。修改/删除前查询身份与 expectedRevision。recur 为 none/daily/weekly，recurUntil 是重复截止时间，空串表示清除。写入仅在最终回复成功后提交。提醒独立保存，不随聊天删除。",
            "parameters":{"type":"object","additionalProperties":false,"required":["action"],"properties":{
                "action":{"type":"string","enum":["create","update","delete"]},"id":{"type":"integer"},"expectedRevision":{"type":"integer"},
                "task":{"type":"string"},"time":{"type":"string"},"recur":{"type":"string","enum":["none","daily","weekly"]},"recurUntil":{"type":"string"}}}}),
    ]
}

#[derive(Debug,Clone)]
enum Effect { Diary(diary::Draft),Character(CharacterPatch),Reminder(reminders::Draft) }
#[derive(Debug,Clone,Serialize)]
pub struct Trace { pub call:ToolCall,pub result:Value }
#[derive(Default)]
pub struct ToolSession { effects:Vec<Effect>,pub traces:Vec<Trace>,cache:HashMap<String,(String,Value)>,pub excluded_sources:Vec<i64> }
impl ToolSession {
    pub fn execute(&mut self,db:&DbState,agent_id:&str,call:&ToolCall)->ChatResult<Value> {
        if call.id.is_empty() || call.id.len()>200 || call.arguments.to_string().len()>64_000 { return Err(ChatError::new("INVALID_RESPONSE","工具调用身份或参数过长")); }
        let fingerprint=json!([call.name,call.arguments]).to_string();
        if let Some((previous,result))=self.cache.get(&call.id) {
            if previous!=&fingerprint { return Err(ChatError::new("INVALID_RESPONSE","模型重复使用工具调用身份并修改参数")); }
            return Ok(result.clone());
        }
        if self.traces.len()>=24 { return Err(ChatError::new("TOOL_LIMIT","本轮工具调用次数达到上限")); }
        let result=self.dispatch(db,agent_id,call).unwrap_or_else(|e|json!({"ok":false,"error":e}));
        self.cache.insert(call.id.clone(),(fingerprint,result.clone()));
        self.traces.push(Trace { call:call.clone(),result:result.clone() });
        Ok(result)
    }

    fn dispatch(&mut self,db:&DbState,agent_id:&str,call:&ToolCall)->ChatResult<Value> {
        let invalid=||ChatError::new("INVALID_TOOL_INPUT","工具参数格式无效");
        match call.name.as_str() {
            "write_diary"=> {
                let draft:diary::Draft=serde_json::from_value(call.arguments.clone()).map_err(|_|invalid())?;
                diary::validate(&draft)?;
                if let Some(id)=draft.id {
                    if self.effects.iter().any(|effect|matches!(effect,Effect::Diary(previous) if previous.id==Some(id))) {
                        return Err(ChatError::new("DIARY_ALREADY_STAGED","该日记本轮已有待提交修订，请在下一轮继续修改"));
                    }
                    let conn=db_lock(db)?;
                    let current=conn.query_row("SELECT revision FROM memories WHERE id=?1 AND agent_id=?2",params![id,agent_id],|r|r.get::<_,i64>(0)).optional()?;
                    if current!=draft.expected_revision { return Err(ChatError::new("STALE_DIARY","日记不存在或已被修改，请重新查询")); }
                    let (raw,source):(String,Option<i64>)=conn.query_row("SELECT source_message_ids,source_message_id FROM memories WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?)))?;
                    let mut sources:Vec<i64>=serde_json::from_str(&raw).map_err(|_|invalid())?;
                    sources.extend(source);
                    if sources.iter().any(|id|self.excluded_sources.contains(id)) { return Err(ChatError::new("STALE_DIARY","该日记来自正在重新生成的回复，将随旧回复一同替换")); }
                }
                self.effects.push(Effect::Diary(draft));
                Ok(json!({"ok":true,"commit":"after_assistant_persisted"}))
            }
            "query_diary"=> {
                let query:diary::Query=serde_json::from_value(call.arguments.clone()).map_err(|_|invalid())?;
                if query.limit>20 || query.offset!=0 { return Err(invalid()); }
                let entries=db.diary.query(&*db_lock(db)?,agent_id,&query)?.into_iter().filter(|e|!e.source_message_ids.iter().any(|id|self.excluded_sources.contains(id))).collect::<Vec<_>>();
                let mut bytes=0;let count=entries.len();
                let entries=entries.into_iter().take_while(|e| {bytes+=e.content.len()+e.tags.join(" ").len();bytes<=32_000}).collect::<Vec<_>>();
                Ok(json!({"ok":true,"truncated":entries.len()<count,"entries":entries}))
            }
            "update_character"=> {
                let patch:CharacterPatch=serde_json::from_value(call.arguments.clone()).map_err(|_|invalid())?;
                patch.validate()?;
                self.effects.push(Effect::Character(patch));
                Ok(json!({"ok":true,"commit":"after_assistant_persisted"}))
            }
            "query_reminders"=> {
                let input:reminders::Query=serde_json::from_value(call.arguments.clone()).map_err(|_|invalid())?;
                if input.limit.unwrap_or(20)>20 {return Err(invalid());}
                let result=reminders::query(&*db_lock(db)?,agent_id,&reminders::Query {limit:Some(input.limit.unwrap_or(20)),..input})?;
                Ok(json!({"ok":true,"reminders":result}))
            }
            "manage_reminder"=> {
                let change:reminders::Change=serde_json::from_value(call.arguments.clone()).map_err(|_|invalid())?;
                if change.id.is_some() && self.effects.iter().any(|effect|matches!(effect,Effect::Reminder(previous) if previous.change.id==change.id)) {
                    return Err(ChatError::new("REMINDER_ALREADY_STAGED","该提醒本轮已有待提交操作，请合并修改或在下一轮处理"));
                }
                let draft=reminders::prepare(&*db_lock(db)?,agent_id,change,chrono::Local::now().naive_local())?;
                self.effects.push(Effect::Reminder(draft));
                Ok(json!({"ok":true,"commit":"after_assistant_persisted"}))
            }
            _=>Err(ChatError::new("UNKNOWN_TOOL","该工具不可用，仅支持日记、角色和提醒管理")),
        }
    }

    pub fn commit(&self,conn:&Connection,agent_id:&str,source_ids:&[i64],timestamp:i64)->ChatResult<()> {
        for effect in &self.effects { match effect {
            Effect::Diary(draft)=> { diary::apply(conn,agent_id,draft,source_ids,timestamp)?; }
            Effect::Character(patch)=>patch.apply(conn,agent_id)?,
            Effect::Reminder(draft)=>reminders::commit(conn,agent_id,draft,chrono::Local::now().naive_local())?,
        } } Ok(())
    }

    pub fn diary_write_count(&self)->usize {self.effects.iter().filter(|effect|matches!(effect,Effect::Diary(_))).count()}
    pub fn reminder_change_count(&self)->usize {self.effects.iter().filter(|effect|matches!(effect,Effect::Reminder(_))).count()}
}

use rusqlite::OptionalExtension;
