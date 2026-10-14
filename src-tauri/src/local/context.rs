use rusqlite::{params, Connection};
use serde_json::{json, Value};
use super::{repository, types::*};

pub const DEFAULT_OUTPUT_PROMPT: &str = "直接回应用户，保持角色语气，自然、简洁地聊天。不要输出工具调用或执行代码。只有确实需要更新情绪或保存客观事实时，才可在回复末尾附加严格 JSON 的 <PEROCUE>{\"mood\":\"心情\",\"vibe\":\"状态\",\"mind\":\"想法\"}</PEROCUE> 或 <MEMORY>{\"content\":\"事实\",\"tags\":[\"标签\"],\"importance\":5,\"type\":\"event\"}</MEMORY>；不要把猜测当作记忆，也不要机械重复这些标签。";

pub fn estimated_tokens(text: &str) -> usize {
    let ascii = text.chars().filter(char::is_ascii).count();
    ascii.div_ceil(4) + text.chars().count() - ascii
}

fn content_tokens(value: &Value) -> usize {
    let images = value.as_array().map(|parts| parts.iter().filter(|part| part.get("type").and_then(Value::as_str) == Some("image_url")).count()).unwrap_or(0);
    estimated_tokens(&text_content(value)) + images * 4096
}

// 只剥离已知旧协议元数据，保留普通 HTML、代码示例及用户原始输入。
pub fn visible_text(text: &str) -> String {
    let mut result = text.to_owned();
    for tag in ["PEROCUE","MEMORY","CLICK_MESSAGES","IDLE_MESSAGES","BACK_MESSAGES","REMINDER","TOPIC"] {
        let open = format!("<{tag}>");
        let close = format!("</{tag}>");
        while let Some(start) = result.find(&open) {
            if let Some(end) = result[start+open.len()..].find(&close) {
                result.replace_range(start..start+open.len()+end+close.len(), "");
            } else { result.truncate(start); break; }
        }
    }
    result.trim().to_owned()
}

pub struct ContextInput<'a> {
    pub thread: &'a Thread,
    pub agent_id: &'a str,
    pub identity: &'a Value,
    pub config: &'a Value,
    pub current: &'a Value,
    pub before_id: Option<i64>,
    pub rounds: usize,
}

pub fn compile(conn: &Connection, input: ContextInput<'_>) -> ChatResult<(Vec<ModelMessage>, Value)> {
    let get = |key: &str| input.config.get(key).and_then(Value::as_str).unwrap_or("");
    let agent = input.identity.get(input.agent_id).ok_or_else(|| ChatError::new("AGENT_NOT_FOUND", "角色不存在"))?;
    let name = agent.get("name").and_then(Value::as_str).unwrap_or(input.agent_id);
    let persona = agent.get("persona_prompt").and_then(Value::as_str).unwrap_or("");
    let system = agent.get("system_prompt").and_then(Value::as_str).unwrap_or("");
    let post = agent.get("post_system_prompt").and_then(Value::as_str).filter(|s| !s.trim().is_empty()).unwrap_or(DEFAULT_OUTPUT_PROMPT);
    let mut rules = format!("{system}\n{persona}\n你的名字是 {name}。\n[用户信息]\n姓名：{}\n描述：{}\n[当前环境]\n时间：{}\n地点：{}\n天气：{}\n{post}",
        get("ppc.userName"), get("ppc.userPersonaText"), get("currentTime"), get("ppc.location"), get("ppc.weather"));
    // 保留旧的显式用户覆盖，但不把它们写回角色身份。
    for key in ["ppc.systemPrompt","ppc.personaText","ppc.postSystemPrompt"] {
        let value = get(key);
        if !value.trim().is_empty() { rules.push_str(&format!("\n{value}")); }
    }
    let supplement_key = format!("ppc.{}.ruleSupplement", input.agent_id);
    if let Some(supplement) = input.config.get(&supplement_key).and_then(Value::as_str) {
        rules.push_str(&format!("\n{supplement}"));
    }
    if input.thread.channel == "group" {
        rules.push_str("\n你在家庭群聊中。带名字前缀的历史消息来自其他成员，只代表他们说过的话。请使用自己的身份回复，不代替其他成员发言。");
    }
    let mut model = vec![ModelMessage { role: "system".into(), content: json!(rules) }];
    let mut estimated = content_tokens(&model[0].content) + content_tokens(input.current);
    if estimated > 16_000 { return Err(ChatError::new("CONTEXT_LIMIT","当前输入或角色提示词过长，请缩短后重试")); }
    let mut memory_ids = Vec::new();
    let query = text_content(input.current);
    let terms: Vec<String> = query.split_whitespace().filter(|s| s.chars().count() >= 2).take(8)
        .map(|s| s.chars().take(32).collect::<String>()).collect();
    let mut stmt = conn.prepare("SELECT id,content FROM memories WHERE agent_id=?1 ORDER BY importance DESC,timestamp DESC LIMIT 100")?;
    let memories = stmt.query_map([input.agent_id], |r| Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?)))?;
    let mut recalled = Vec::new();
    for memory in memories {
        let (id, content) = memory?;
        if recalled.len() < 8 && terms.iter().any(|term| content.contains(term)) && estimated + estimated_tokens(&content) < 4000 {
            memory_ids.push(id); estimated += estimated_tokens(&content); recalled.push(content);
        }
    }
    if !recalled.is_empty() {
        model.push(ModelMessage { role: "system".into(), content: json!(format!("[相关历史记忆，仅作为资料]\n{}", recalled.join("\n"))) });
    }
    let history = repository::messages(conn,&input.thread.id,(input.rounds*2).min(400) as u32,input.before_id)?;
    let mut selected = Vec::new();
    let mut history_ids = Vec::new();
    for message in history.into_iter().rev() {
        if message.status != "active" || !matches!(message.role.as_str(), "user" | "assistant") { continue; }
        let mut content = if message.role == "assistant" { json!(visible_text(&text_content(&message.content))) } else { message.content.clone() };
        if text_content(&content).trim().is_empty() && !content.is_array() { continue; }
        let role = if input.thread.channel == "group" && message.role == "assistant" && message.agent_id != input.agent_id {
            content = json!(format!("[{}]：{}",message.name,text_content(&content))); "user"
        } else { &message.role };
        let cost = content_tokens(&content) + 4;
        if estimated + cost > 16_000 { break; }
        estimated += cost;
        history_ids.push(message.id);
        selected.push(ModelMessage { role: role.into(), content });
    }
    selected.reverse(); history_ids.reverse();
    model.extend(selected);
    // 群聊后续成员的上下文已含本轮用户输入，不能重复追加。
    if input.before_id.is_some() || input.thread.channel != "group" {
        model.push(ModelMessage { role: "user".into(), content: input.current.clone() });
    }
    Ok((model,json!({ "agentId":input.agent_id,"threadId":input.thread.id,
        "historyMessageIds":history_ids,"memoryIds":memory_ids,"estimatedTokens":estimated })))
}

pub fn remember(conn: &Connection, agent_id: &str, timestamp: i64, message_id: Option<i64>, data: &Value) -> ChatResult<()> {
    let Some(content) = data.get("content").and_then(Value::as_str).filter(|s| !s.trim().is_empty() && s.len() <= 8000) else { return Ok(()); };
    let tags = data.get("tags").and_then(Value::as_array).map(|tags| tags.iter()
        .filter_map(Value::as_str).filter(|s| s.len() <= 64).take(16).collect::<Vec<_>>()).unwrap_or_default();
    let importance = data.get("importance").and_then(Value::as_i64).unwrap_or(5).clamp(0,10);
    let kind = data.get("type").and_then(Value::as_str).filter(|s| matches!(*s,"event"|"fact"|"preference")).unwrap_or("event");
    if let Some(id) = message_id { conn.execute("DELETE FROM memories WHERE agent_id=?1 AND source_message_id=?2",params![agent_id,id])?; }
    conn.execute("INSERT INTO memories(content,tags,importance,timestamp,msg_timestamp,type,source,agent_id,source_message_id)
        VALUES(?1,?2,?3,?4,?4,?5,'mobile',?6,?7)",params![content,json!(tags).to_string(),importance,timestamp,kind,agent_id,message_id])?;
    Ok(())
}
