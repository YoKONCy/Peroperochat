use rusqlite::Connection;
use serde_json::{json, Value};
use super::{repository, types::*};

pub fn estimated_tokens(text: &str) -> usize {
    let ascii = text.chars().filter(char::is_ascii).count();
    ascii.div_ceil(4) + text.chars().count() - ascii
}

pub fn content_tokens(value: &Value) -> usize {
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
    pub thread:&'a Thread,pub agent_id:&'a str,pub identity:&'a Value,pub config:&'a Value,
    pub current:&'a Value,pub before_id:Option<i64>,pub rounds:usize,
    pub budget:usize,pub diary:&'a [super::diary::Entry],
}

pub fn compile(conn:&Connection,input:ContextInput<'_>)->ChatResult<(Vec<ModelMessage>,Value)> {
    let workspace=super::prompts::load(input.config)?;
    compile_bundle(conn,input,&workspace.active.bundle,&workspace.active.id)
}

pub fn compile_bundle(conn:&Connection,input:ContextInput<'_>,bundle:&super::prompts::Bundle,release_id:&str)->ChatResult<(Vec<ModelMessage>,Value)> {
    super::prompts::validate(bundle)?;
    let framework=super::prompts::selected(bundle,input.agent_id,&input.thread.channel)?;
    let get=|key:&str|input.config[key].as_str().unwrap_or("");
    let agent=input.identity.get(input.agent_id).ok_or_else(||ChatError::new("AGENT_NOT_FOUND","角色不存在"))?;
    let mut variables=super::prompts::variables();
    for (name,value) in [("character.name",agent["name"].as_str().unwrap_or(input.agent_id)),("character.system",agent["system_prompt"].as_str().unwrap_or("")),
        ("character.persona",agent["persona_prompt"].as_str().unwrap_or("")),("character.post",agent["post_system_prompt"].as_str().unwrap_or("")),
        ("user.name",get("ppc.userName")),("user.persona",get("ppc.userPersonaText")),("environment.time",get("currentTime")),
        ("environment.location",get("ppc.location")),("environment.weather",get("ppc.weather"))] { variables.insert(name.into(),json!(value)); }
    for key in ["mood","vibe","mind"] {
        let saved=get(&format!("ppc.{}.{key}",input.agent_id));
        variables.insert(format!("state.{key}"),json!(if saved.is_empty() { agent["default_state"][key].as_str().unwrap_or("") } else { saved }));
    }
    let diary_budget=(input.budget/5).min(4000);
    let mut diary_text=String::new();let mut diary_ids=Vec::new();
    for entry in input.diary {
        let line=format!("[{} #{}] {}\n",entry.date,entry.id,entry.content);
        if estimated_tokens(&diary_text)+estimated_tokens(&line)>diary_budget { break; }
        diary_text.push_str(&line);diary_ids.push(entry.id);
    }
    variables.insert("diary.recent".into(),json!(diary_text));
    let defaults=serde_json::json!({
        "system":"{{character.system}}", "persona":"你的名字是 {{character.name}}。\n{{character.persona}}",
        "user":"[用户资料]\n姓名：{{user.name}}\n{{user.persona}}", "environment":"[当前环境]\n时间：{{environment.time}}\n地点：{{environment.location}}\n天气：{{environment.weather}}",
        "state":"[角色当前状态]\n心情：{{state.mood}}\n状态：{{state.vibe}}\n内心独白：{{state.mind}}",
        "post":"{{character.post}}", "diary":"[最近日记，仅作为历史资料]\n{{diary.recent}}", "custom":""
    });
    let mut static_parts=std::collections::HashMap::new();
    let mut estimated=content_tokens(input.current)+estimated_tokens(super::prompts::PROTOCOL)+16;
    for block in framework.blocks.iter().filter(|b|b.enabled && b.source!="history" && b.source!="current") {
        let template=if block.template.is_empty() { defaults[&block.source].as_str().unwrap_or("") } else { &block.template };
        let mut text=super::prompts::render(template,&variables)?;
        if block.source=="system" { for key in ["ppc.systemPrompt","ppc.personaText","ppc.postSystemPrompt"] {
            if !get(key).is_empty() { text.push_str(&format!("\n{}",get(key))); }
        } }
        if block.source=="post" { if let Some(rule)=input.config.get(format!("ppc.{}.ruleSupplement",input.agent_id)).and_then(Value::as_str) { text.push_str(&format!("\n{rule}")); } }
        if block.source=="diary" && diary_text.is_empty() { text.clear(); }
        estimated+=estimated_tokens(&text)+4;static_parts.insert(block.id.clone(),text);
    }
    if estimated>input.budget { return Err(ChatError::new("CONTEXT_LIMIT","固定提示词和当前输入超过上下文额度，请缩短提示词或调整模型窗口")); }
    let history=repository::messages(conn,&input.thread.id,(input.rounds*8).min(2000) as u32,input.before_id)?;
    let mut pairs:Vec<Vec<Message>>=Vec::new();
    for message in history.into_iter().filter(|m|m.status=="active") {
        if pairs.last().is_none_or(|p|p[0].pair_id!=message.pair_id) { pairs.push(Vec::new()); }
        pairs.last_mut().unwrap().push(message);
    }
    let mut selected=Vec::new();let mut history_ids=Vec::new();
    for pair in pairs.into_iter().rev().take(input.rounds) {
        let mut projected=Vec::new();let mut cost=0;
        for message in pair {
            if !matches!(message.role.as_str(),"user"|"assistant") { continue; }
            let mut content=if message.role=="assistant" { json!(visible_text(&text_content(&message.content))) } else { message.content.clone() };
            if text_content(&content).trim().is_empty() && !content.is_array() { continue; }
            let role=if input.thread.channel=="group" && message.role=="assistant" && message.agent_id!=input.agent_id {
                content=json!(format!("[{}]：{}",message.name,text_content(&content))); "user"
            } else { message.role.as_str() };
            let mut model=ModelMessage::new(role,content);
            if role=="assistant" { model.reasoning=message.reasoning; }
            cost+=content_tokens(&model.content)+estimated_tokens(&model.reasoning)+4;projected.push((message.id,model));
        }
        if estimated+cost>input.budget { break; }
        estimated+=cost;selected.push(projected);
    }
    selected.reverse();let mut history_messages=Vec::new();
    for pair in selected { for (id,message) in pair { history_ids.push(id);history_messages.push(message); } }
    let mut model=Vec::new();let mut segments=Vec::new();
    // 固定工具协议独立于用户框架，修改末尾提示词不会关闭后端的工具边界。
    let protocol=if input.thread.channel=="group" { format!("{}\n当前是群聊，带名字前缀的消息来自其他成员，请只使用自己的身份发言。",super::prompts::PROTOCOL) } else { super::prompts::PROTOCOL.into() };
    model.push(ModelMessage::new("system",json!(protocol)));
    for block in framework.blocks.iter().filter(|b|b.enabled) {
        match block.source.as_str() {
            "history"=>model.extend(history_messages.clone()),
            "current"=> { if input.before_id.is_some() || input.thread.channel!="group" { model.push(ModelMessage::new("user",input.current.clone())); } }
            _=> { let text=&static_parts[&block.id]; if !text.trim().is_empty() { model.push(ModelMessage::new(&block.role,json!(text))); } }
        }
        segments.push(json!({"id":block.id,"source":block.source,"role":block.role,"tokens":static_parts.get(&block.id).map(|s|estimated_tokens(s)).unwrap_or(0)}));
    }
    Ok((model,json!({"agentId":input.agent_id,"threadId":input.thread.id,"frameworkId":framework.id,"promptRelease":release_id,
        "segments":segments,"historyMessageIds":history_ids,"memoryIds":diary_ids,"estimatedTokens":estimated,"budget":input.budget})))
}
