use serde_json::{json,Value};
use super::{provider::ModelSettings,types::*};

pub fn validate(settings:&ModelSettings)->ChatResult<()> {
    let invalid=||ChatError::new("INVALID_CONFIG","模型协议、参数或请求额度无效");
    if !["chat_completions","responses","anthropic","gemini"].contains(&settings.api_format.as_str())
        || !["auto","openai","deepseek","openrouter","generic"].contains(&settings.reasoning_dialect.as_str())
        || settings.reasoning_effort.as_ref().is_some_and(|s| !["off","none","minimal","low","medium","high","xhigh","max"].contains(&s.as_str()))
        || settings.max_tokens<128 || settings.max_tokens>65536 || settings.context_window<settings.max_tokens+1024 || settings.context_window>2_000_000
        || !(1..=12).contains(&settings.max_steps) || !(30..=900).contains(&settings.turn_timeout) || !(1..=200).contains(&settings.memory_rounds)
        || settings.model.len()>1000 || settings.base.len()>4000 || settings.api_key.len()>16000 { return Err(invalid()); }
    for (value,min,max) in [(settings.temperature,0.0,2.0),(settings.top_p,0.0,1.0),(settings.frequency_penalty,-2.0,2.0),(settings.presence_penalty,-2.0,2.0)] {
        if value.is_some_and(|v|!v.is_finite() || v<min || v>max) { return Err(invalid()); }
    }
    super::provider::endpoint(&settings.base,"chat/completions")?;
    if settings.headers.len()>20 || Value::Object(settings.extra_body.clone()).to_string().len()>32000 || settings.disabled_params.len()>20 { return Err(invalid()); }
    for (key,value) in &settings.headers {
        reqwest::header::HeaderName::from_bytes(key.as_bytes()).map_err(|_|invalid())?;
        reqwest::header::HeaderValue::from_str(value.as_str().filter(|s|s.len()<=16000).ok_or_else(invalid)?).map_err(|_|invalid())?;
        if ["host","content-length","transfer-encoding","connection","cookie"].contains(&key.to_ascii_lowercase().as_str()) { return Err(invalid()); }
    }
    for key in settings.extra_body.keys() {
        if ["model","messages","input","contents","system","systemInstruction","system_instruction","tools","tool_choice","toolChoice","toolConfig","tool_config","stream","n","candidateCount","function_call","previous_response_id","conversation","background","store","max_tool_calls"].contains(&key.as_str()) {
            return Err(ChatError::new("INVALID_CONFIG",format!("扩展参数不能覆盖受管理字段：{key}")));
        }
    }
    for key in ["max_tokens","max_completion_tokens","max_output_tokens"] {
        if let Some(value)=settings.extra_body.get(key) { if !value.as_u64().is_some_and(|v|v>=128 && v<=settings.max_tokens as u64) { return Err(invalid()); } }
    }
    if let Some(config)=settings.extra_body.get("generationConfig") {
        let config=config.as_object().ok_or_else(invalid)?;
        if config.contains_key("candidateCount") || config.get("maxOutputTokens").is_some_and(|v|!v.as_u64().is_some_and(|n|n>=128 && n<=settings.max_tokens as u64)) { return Err(invalid()); }
    }
    if settings.extra_body.contains_key("generation_config") { return Err(invalid()); }
    for key in &settings.disabled_params {
        if !["temperature","top_p","frequency_penalty","presence_penalty","max_tokens","max_completion_tokens","max_output_tokens","reasoning_effort","reasoning","thinking","topP","maxOutputTokens"].contains(&key.as_str()) { return Err(invalid()); }
    }
    Ok(())
}

pub fn from_config(config:&Value)->ChatResult<ModelSettings> {
    let get=|key:&str|config[key].as_str().unwrap_or("");
    let mut values:Value=if get("ppc.modelSettings").trim().is_empty() { json!({}) } else {
        serde_json::from_str(get("ppc.modelSettings")).map_err(|_|ChatError::new("INVALID_CONFIG","模型参数配置损坏"))?
    };
    if !values.is_object() { return Err(ChatError::new("INVALID_CONFIG","模型参数必须是对象")); }
    if !get("ppc.apiConfig").is_empty() {
        let connection:serde_json::Map<String,Value>=serde_json::from_str(get("ppc.apiConfig")).map_err(|_|ChatError::new("INVALID_CONFIG","API 配置损坏"))?;
        let extra=values.get("extraBody").cloned();
        values.as_object_mut().unwrap().extend(connection);
        if let Some(extra)=extra.and_then(|v|v.as_object().cloned()) {
            if !values["extraBody"].is_object() { values["extraBody"]=json!({}); }
            values["extraBody"].as_object_mut().unwrap().extend(extra);
        }
    }
    values["base"]=json!(if get("ppc.apiBase").trim().is_empty() { match values["apiFormat"].as_str().unwrap_or("chat_completions") {
        "anthropic"=>"https://api.anthropic.com/v1","gemini"=>"https://generativelanguage.googleapis.com/v1beta",_=>"https://api.openai.com/v1"
    } } else { get("ppc.apiBase").trim() });
    values["apiKey"]=json!(get("api_key").trim());values["model"]=json!(get("ppc.modelName"));
    let settings:ModelSettings=serde_json::from_value(values).map_err(|_|ChatError::new("INVALID_CONFIG","模型参数类型或字段无效"))?;
    validate(&settings)?; Ok(settings)
}

pub fn redact(settings:&ModelSettings,text:&str)->String {
    let mut result=text.to_owned();
    for secret in std::iter::once(settings.api_key.as_str()).chain(settings.headers.values().filter_map(Value::as_str)) {
        if !secret.is_empty() { result=result.replace(secret,"[已隐藏]"); }
    }
    result.chars().take(1000).collect()
}
