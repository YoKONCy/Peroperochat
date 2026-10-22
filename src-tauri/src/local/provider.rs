use std::time::Duration;
use futures::StreamExt;
use reqwest::{Client, Url};
use serde_json::{json,Value};
use super::types::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all="camelCase", deny_unknown_fields)]
pub struct ModelSettings {
    pub base:String, pub api_key:String, pub model:String,
    pub api_format:String, pub reasoning_dialect:String, pub reasoning_effort:Option<String>,
    pub temperature:Option<f64>, pub top_p:Option<f64>,
    pub frequency_penalty:Option<f64>, pub presence_penalty:Option<f64>,
    pub max_tokens:usize, pub context_window:usize, pub stream:bool, pub memory_rounds:usize,
    pub tools_enabled:bool, pub max_steps:usize, pub turn_timeout:u64,
    pub headers:serde_json::Map<String,Value>, pub extra_body:serde_json::Map<String,Value>,
    pub disabled_params:Vec<String>,
}
impl Default for ModelSettings {
    fn default()->Self {
        Self { base:"https://api.openai.com/v1".into(),api_key:String::new(),model:String::new(),
            api_format:"chat_completions".into(),reasoning_dialect:"auto".into(),reasoning_effort:None,
            temperature:Some(0.7),top_p:Some(1.0),frequency_penalty:None,presence_penalty:None,
            max_tokens:4096,context_window:32768,stream:true,memory_rounds:40,tools_enabled:true,max_steps:8,turn_timeout:300,
            headers:Default::default(),extra_body:Default::default(),disabled_params:Vec::new() }
    }
}

pub fn endpoint(base: &str, resource: &str) -> ChatResult<Url> {
    let mut url = Url::parse(base.trim()).map_err(|_| ChatError::new("INVALID_CONFIG", "API 地址格式无效"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() {
        return Err(ChatError::new("INVALID_CONFIG", "API 地址必须是有效的 HTTP 或 HTTPS 地址"));
    }
    let mut path = url.path().trim_end_matches('/').to_owned();
    for suffix in ["/chat/completions", "/models", "/responses", "/messages"] {
        if path.ends_with(suffix) { path.truncate(path.len() - suffix.len()); }
    }
    // 根地址补标准版本；厂商已有的 /api/v3、/api/paas/v4 等路径原样保留。
    if path.is_empty() { path = "/v1".into(); }
    url.set_path(&format!("{path}/{resource}"));
    url.set_fragment(None);
    Ok(url)
}

pub fn client() -> ChatResult<Client> {
    Client::builder().connect_timeout(Duration::from_secs(15)).redirect(reqwest::redirect::Policy::none()).build()
        .map_err(|_| ChatError::new("NETWORK_ERROR", "无法初始化网络连接"))
}

fn network_error(error: reqwest::Error) -> ChatError {
    ChatError::new("NETWORK_ERROR", if error.is_timeout() { "模型请求超时，请重试" } else { "无法连接模型服务，请检查网络和 API 地址" })
}

#[derive(Default)]
pub struct SseDecoder { bytes: Vec<u8>, data: Vec<String> }

impl SseDecoder {
    pub fn push(&mut self, bytes: &[u8]) -> ChatResult<Vec<String>> {
        self.bytes.extend_from_slice(bytes);
        if self.bytes.len() > 2 * 1024 * 1024 { return Err(ChatError::new("INVALID_RESPONSE", "模型流式事件过大")); }
        let mut events = Vec::new();
        while let Some(end) = self.bytes.iter().position(|b| *b == b'\n') {
            let mut line = self.bytes.drain(..=end).collect::<Vec<_>>();
            line.pop();
            if line.last() == Some(&b'\r') { line.pop(); }
            let line = std::str::from_utf8(&line).map_err(|_| ChatError::new("INVALID_RESPONSE", "模型响应包含无效字符编码"))?;
            if line.is_empty() {
                if !self.data.is_empty() { events.push(self.data.join("\n")); self.data.clear(); }
            } else if let Some(data) = line.strip_prefix("data:") {
                self.data.push(data.strip_prefix(' ').unwrap_or(data).to_owned());
            }
        }
        if self.data.iter().map(String::len).sum::<usize>() > 2 * 1024 * 1024 {
            return Err(ChatError::new("INVALID_RESPONSE", "模型流式事件过大"));
        }
        Ok(events)
    }

    pub fn finish(&mut self) -> ChatResult<Vec<String>> {
        // 允许最后一个事件缺少空行，但内容必须完整且成功结束标记仍须存在。
        self.push(b"\n\n")
    }
}

#[cfg(test)]
fn consume_event(data: &str, output: &mut String, on_delta: &mut impl FnMut(&str) -> ChatResult<()>) -> ChatResult<bool> {
    if data.trim() == "[DONE]" { return Ok(true); }
    let value: Value = serde_json::from_str(data).map_err(|_| ChatError::new("INVALID_RESPONSE", "模型流式响应格式无效"))?;
    if let Some(error) = value.get("error") {
        let text = error.get("message").and_then(Value::as_str).unwrap_or("模型服务报告错误");
        return Err(ChatError::new("UPSTREAM_ERROR", text.chars().take(300).collect::<String>()));
    }
    if let Some(content) = value.pointer("/choices/0/delta/content").and_then(Value::as_str) {
        output.push_str(content);
        if output.len() > 4 * 1024 * 1024 { return Err(ChatError::new("RESPONSE_TOO_LARGE", "回复过长，已停止接收")); }
        on_delta(content)?;
    }
    // finish_reason 明确结束也可兼容省略 [DONE] 的服务；单纯 EOF 永远不是成功。
    match value.pointer("/choices/0/finish_reason").and_then(Value::as_str) {
        Some("stop") => Ok(true),
        Some("length") => Err(ChatError::new("OUTPUT_LIMIT", "模型输出达到长度上限，回复未完整结束")),
        Some("content_filter") => Err(ChatError::new("UPSTREAM_ERROR", "模型服务未完成这次回复")),
        Some(_) => Err(ChatError::new("INVALID_RESPONSE", "当前单机聊天不支持模型返回工具调用")),
        None => Ok(false),
    }
}

pub fn request_url(settings:&ModelSettings)->ChatResult<Url> {
    if settings.api_format!="gemini" { return endpoint(&settings.base,match settings.api_format.as_str() { "responses"=>"responses","anthropic"=>"messages",_=>"chat/completions" }); }
    let mut url=Url::parse(&settings.base).map_err(|_|ChatError::new("INVALID_CONFIG","API 地址格式无效"))?;
    let base=url.path().split("/models/").next().unwrap_or("").trim_end_matches('/').to_owned();
    url.set_path(&if base.is_empty() { "/v1beta".to_owned() } else { base });
    let action=if settings.stream { "streamGenerateContent" } else { "generateContent" };
    url.path_segments_mut().map_err(|_|ChatError::new("INVALID_CONFIG","Gemini 地址无效"))?.push("models").push(&format!("{}:{action}",settings.model.trim_start_matches("models/")));
    if settings.stream { url.query_pairs_mut().append_pair("alt","sse"); }
    Ok(url)
}

async fn bytes(response:reqwest::Response)->ChatResult<Vec<u8>> {
    let mut stream=response.bytes_stream();let mut output=Vec::new();
    while let Some(chunk)=tokio::time::timeout(Duration::from_secs(60),stream.next()).await.map_err(|_|ChatError::new("TIMEOUT","模型响应读取超时"))? {
        output.extend_from_slice(&chunk.map_err(network_error)?);
        if output.len()>8*1024*1024 { return Err(ChatError::new("RESPONSE_TOO_LARGE","模型响应过大")); }
    }
    Ok(output)
}

fn authorize(mut request:reqwest::RequestBuilder,settings:&ModelSettings)->reqwest::RequestBuilder {
    if !settings.api_key.is_empty() { request=match settings.api_format.as_str() {
        "anthropic"=>request.header("x-api-key",&settings.api_key),"gemini"=>request.header("x-goog-api-key",&settings.api_key),_=>request.bearer_auth(&settings.api_key),
    }; }
    if settings.api_format=="anthropic" { request=request.header("anthropic-version","2023-06-01"); }
    for (key,value) in &settings.headers { request=request.header(key,value.as_str().unwrap_or("")); }
    request
}

async fn send(request:reqwest::RequestBuilder,settings:&ModelSettings)->ChatResult<reqwest::Response> {
    let response=tokio::time::timeout(Duration::from_secs(30),request.send()).await.map_err(|_|ChatError::new("TIMEOUT","连接模型服务超时"))?.map_err(network_error)?;
    if response.status().is_success() { return Ok(response); }
    let status=response.status().as_u16();
    let data=bytes(response).await.unwrap_or_default();
    let error=serde_json::from_slice::<Value>(&data).ok().and_then(|v|v.pointer("/error/message").or_else(||v.get("message")).and_then(Value::as_str).map(str::to_owned)).unwrap_or_default();
    Err(ChatError::new("UPSTREAM_ERROR",format!("模型服务返回 HTTP {status}：{}",super::model_config::redact(settings,&error))))
}

pub async fn list_models(settings:&ModelSettings)->ChatResult<Value> {
    super::model_config::validate(settings)?;
    let mut url=if settings.api_format=="gemini" {
        let mut url=request_url(settings)?;let path=url.path().split("/models/").next().unwrap_or("/v1beta").to_owned();
        url.set_path(&format!("{path}/models"));url.set_query(None);url
    } else {endpoint(&settings.base,"models")?};
    let mut models=Vec::new();
    for _ in 0..10 {
        let response=send(authorize(client()?.get(url.clone()),settings),settings).await?;
        let value:Value=serde_json::from_slice(&bytes(response).await?).map_err(|_|ChatError::new("INVALID_RESPONSE","模型列表不是有效 JSON"))?;
        let entries=value[if settings.api_format=="gemini" {"models"} else {"data"}].as_array()
            .ok_or_else(||ChatError::new("INVALID_RESPONSE","模型列表缺少数据"))?;
        for item in entries {
            let id=item[if settings.api_format=="gemini" {"name"} else {"id"}].as_str().ok_or_else(||ChatError::new("INVALID_RESPONSE","模型列表缺少身份"))?;
            let mut item=item.clone();item["id"]=serde_json::json!(id.trim_start_matches("models/"));models.push(item);
        }
        if models.len()>10_000 { return Err(ChatError::new("RESPONSE_TOO_LARGE","模型列表过大")); }
        let next=if settings.api_format=="gemini" {value["nextPageToken"].as_str().map(|s|("pageToken",s))}
            else if settings.api_format=="anthropic" && value["has_more"]==true {value["last_id"].as_str().map(|s|("after_id",s))} else {None};
        match next.filter(|(_,value)|!value.is_empty()) {
            Some((key,value))=> {let pairs=url.query_pairs().filter(|(k,_)|k!=key).map(|(k,v)|(k.into_owned(),v.into_owned())).collect::<Vec<_>>();url.set_query(None);url.query_pairs_mut().extend_pairs(pairs).append_pair(key,value);}
            None=>return Ok(serde_json::json!({"data":models})),
        }
    }
    Err(ChatError::new("OUTPUT_LIMIT","模型列表分页过多，请缩小服务端列表"))
}

// 仅记录已发出请求的显示摘要；不从当前设置反推历史，也不把密钥或额外请求体带入消息。
fn reasoning_presentation(format:&str,model:&str,body:&Value)->Value {
    let effort=match format {
        "responses"=>body.pointer("/reasoning/effort"),
        "anthropic"=>body.pointer("/output_config/effort"),
        "gemini"=>body.pointer("/generationConfig/thinkingConfig/thinkingLevel"),
        _=>body.get("reasoning_effort"),
    }.and_then(Value::as_str).map(str::to_ascii_lowercase);
    let value=if let Some(effort)=effort {
        if ["off","none","minimal","low","medium","high","xhigh","max"].contains(&effort.as_str()) {json!(effort)} else {json!("unknown")}
    } else if body.pointer("/thinking/type").is_some_and(|v|v=="disabled") || body.pointer("/generationConfig/thinkingConfig/thinkingBudget").is_some_and(|v|v==0) {json!("off")}
    else if body.pointer("/thinking/budget_tokens").is_some() || body.pointer("/generationConfig/thinkingConfig/thinkingBudget").is_some() || body.pointer("/thinking/type").is_some_and(|v|v=="enabled") {json!("unknown")}
    else {Value::Null};
    json!({"reasoningEffort":value,"modelId":model,"protocol":format})
}
fn with_presentation(mut completion:Completion,presentation:&Value)->Completion {
    if !completion.usage.is_object() { completion.usage=json!({"upstream":completion.usage}); }
    completion.usage["_ppcReasoning"]=presentation.clone();completion
}
pub async fn complete(settings:&ModelSettings,messages:&[ModelMessage],tools:&[Value],mut emit:impl FnMut(&str,&str)->ChatResult<()>)->ChatResult<Completion> {
    super::model_config::validate(settings)?;
    if settings.model.trim().is_empty() || settings.model.contains("请先") { return Err(ChatError::new("INVALID_CONFIG","请先在设置中选择模型")); }
    let body=super::wire::body(settings,messages,tools)?;
    let presentation=reasoning_presentation(&settings.api_format,&settings.model,&body);
    let response=send(authorize(client()?.post(request_url(settings)?).json(&body),settings),settings).await?;
    let stream_type=response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|h|h.to_str().ok()).unwrap_or("").contains("text/event-stream");
    if !stream_type {
        let value:Value=serde_json::from_slice(&bytes(response).await?).map_err(|_|ChatError::new("INVALID_RESPONSE","模型响应不是有效 JSON"))?;
        let result=super::wire::parse(&settings.api_format,&value)?;
        if !result.reasoning.is_empty() { emit("reasoning",&result.reasoning)?; }
        if !result.content.is_empty() { emit("delta",&result.content)?; }
        return Ok(with_presentation(result,&presentation));
    }
    let mut stream=response.bytes_stream();let mut decoder=SseDecoder::default();let mut accumulator=super::provider_stream::Accumulator::default();
    while let Some(chunk)=tokio::time::timeout(Duration::from_secs(60),stream.next()).await.map_err(|_|ChatError::new("TIMEOUT","模型长时间没有响应"))? {
        for data in decoder.push(&chunk.map_err(network_error)?)? {
            if accumulator.event(&settings.api_format,&data,&mut emit)? { return accumulator.finish(&settings.api_format).map(|value|with_presentation(value,&presentation)); }
        }
    }
    for data in decoder.finish()? { if accumulator.event(&settings.api_format,&data,&mut emit)? { break; } }
    accumulator.finish(&settings.api_format).map(|value|with_presentation(value,&presentation))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reasoning_presentation_uses_sent_fields_and_never_guesses_budget_levels() {
        assert_eq!(reasoning_presentation("chat_completions","model-a",&json!({"reasoning_effort":"high"}))["reasoningEffort"],"high");
        assert!(reasoning_presentation("responses","model-a",&json!({}))["reasoningEffort"].is_null());
        assert_eq!(reasoning_presentation("anthropic","model-a",&json!({"thinking":{"type":"enabled","budget_tokens":4096}}))["reasoningEffort"],"unknown");
        assert_eq!(reasoning_presentation("gemini","model-a",&json!({"generationConfig":{"thinkingConfig":{"thinkingLevel":"HIGH"}}}))["reasoningEffort"],"high");
        let value=reasoning_presentation("chat_completions","model-a",&json!({"apiKey":"测试密钥","reasoning_effort":"off"}));assert!(!value.to_string().contains("测试密钥"));
    }

    #[test]
    fn every_byte_boundary_preserves_chinese_and_crlf() {
        let source = "event: message\r\ndata: {\"choices\":[{\"delta\":{\"content\":\"你好🐱\"}}]}\r\n\r\ndata:[DONE]\n\n";
        for split in 0..=source.len() {
            let mut decoder = SseDecoder::default();
            let mut events = decoder.push(&source.as_bytes()[..split]).unwrap();
            events.extend(decoder.push(&source.as_bytes()[split..]).unwrap());
            assert_eq!(events.len(), 2, "拆包位置：{split}");
            let mut output = String::new();
            assert!(!consume_event(&events[0], &mut output, &mut |_| Ok(())).unwrap());
            assert!(consume_event(&events[1], &mut output, &mut |_| Ok(())).unwrap());
            assert_eq!(output, "你好🐱");
        }
    }

    #[test]
    fn bytewise_events_and_multiline_data_are_buffered() {
        let mut decoder = SseDecoder::default();
        let mut events = Vec::new();
        for byte in ": 心跳\ndata: {\ndata: \"choices\": []}\n\ndata: [DONE]".as_bytes() {
            events.extend(decoder.push(&[*byte]).unwrap());
        }
        events.extend(decoder.finish().unwrap());
        assert_eq!(events, ["{\n\"choices\": []}", "[DONE]"]);
    }

    #[test]
    fn provider_paths_and_completion_endings() {
        assert_eq!(endpoint("https://example.com", "models").unwrap().path(), "/v1/models");
        assert_eq!(endpoint("https://example.com/api/v3", "chat/completions").unwrap().path(), "/api/v3/chat/completions");
        assert_eq!(endpoint("https://example.com/v1/chat/completions/", "models").unwrap().path(), "/v1/models");
        assert!(endpoint("file:///tmp", "models").is_err());
        let mut output = String::new();
        assert!(consume_event("{\"choices\":[{\"finish_reason\":\"stop\"}]}", &mut output, &mut |_| Ok(())).unwrap());
        assert!(consume_event("{\"choices\":[{\"finish_reason\":\"length\"}]}", &mut output, &mut |_| Ok(())).is_err());
        assert!(consume_event("{invalid", &mut output, &mut |_| Ok(())).is_err());
    }
}
