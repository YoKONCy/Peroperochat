use std::time::Duration;
use futures::StreamExt;
use reqwest::{Client, Url};
use serde_json::{json, Value};
use super::types::*;

#[derive(Debug, Clone)]
pub struct ModelSettings {
    pub base: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f64,
    pub top_p: f64,
    pub frequency_penalty: f64,
    pub presence_penalty: f64,
    pub stream: bool,
    pub memory_rounds: usize,
}

pub fn endpoint(base: &str, resource: &str) -> ChatResult<Url> {
    let mut url = Url::parse(base.trim()).map_err(|_| ChatError::new("INVALID_CONFIG", "API 地址格式无效"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() {
        return Err(ChatError::new("INVALID_CONFIG", "API 地址必须是有效的 HTTP 或 HTTPS 地址"));
    }
    let mut path = url.path().trim_end_matches('/').to_owned();
    for suffix in ["/chat/completions", "/models"] {
        if path.ends_with(suffix) { path.truncate(path.len() - suffix.len()); }
    }
    // 根地址补标准版本；厂商已有的 /api/v3、/api/paas/v4 等路径原样保留。
    if path.is_empty() { path = "/v1".into(); }
    url.set_path(&format!("{path}/{resource}"));
    url.set_fragment(None);
    Ok(url)
}

pub fn client() -> ChatResult<Client> {
    Client::builder().connect_timeout(Duration::from_secs(15)).build()
        .map_err(|_| ChatError::new("NETWORK_ERROR", "无法初始化网络连接"))
}

fn network_error(error: reqwest::Error) -> ChatError {
    ChatError::new("NETWORK_ERROR", if error.is_timeout() { "模型请求超时，请重试" } else { "无法连接模型服务，请检查网络和 API 地址" })
}

async fn checked_response(response: reqwest::Response, api_key: &str) -> ChatResult<reqwest::Response> {
    if response.status().is_success() { return Ok(response); }
    let status = response.status().as_u16();
    let data = tokio::time::timeout(Duration::from_secs(10), response.text()).await.ok().and_then(Result::ok).unwrap_or_default();
    let detail = serde_json::from_str::<Value>(&data).ok()
        .and_then(|value| value.pointer("/error/message").or_else(|| value.get("message")).and_then(Value::as_str).map(str::to_owned))
        .unwrap_or_default();
    let detail = if api_key.is_empty() { detail } else { detail.replace(api_key, "[已隐藏]") };
    let detail: String = detail.chars().take(300).collect();
    Err(ChatError::new("UPSTREAM_ERROR", format!("模型服务返回 HTTP {status}{}", if detail.is_empty() { String::new() } else { format!("：{detail}") })))
}

pub async fn list_models(base: &str, api_key: &str) -> ChatResult<Value> {
    let request = client()?.get(endpoint(base, "models")?).bearer_auth(api_key);
    let response = tokio::time::timeout(Duration::from_secs(30), request.send()).await
        .map_err(|_| ChatError::new("TIMEOUT", "获取模型列表超时"))?.map_err(network_error)?;
    checked_response(response, api_key).await?.json().await
        .map_err(|_| ChatError::new("INVALID_RESPONSE", "模型列表响应不是有效 JSON"))
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

pub async fn complete(settings: &ModelSettings, messages: &[ModelMessage], output: &mut String,
    mut on_delta: impl FnMut(&str) -> ChatResult<()>) -> ChatResult<()> {
    if settings.model.trim().is_empty() || settings.model.contains("请先") {
        return Err(ChatError::new("INVALID_CONFIG", "请先在设置中选择模型"));
    }
    let body = json!({ "model": settings.model, "messages": messages,
        "temperature": settings.temperature, "top_p": settings.top_p,
        "frequency_penalty": settings.frequency_penalty, "presence_penalty": settings.presence_penalty,
        "stream": settings.stream });
    let response = tokio::time::timeout(Duration::from_secs(30),client()?.post(endpoint(&settings.base, "chat/completions")?)
        .bearer_auth(&settings.api_key).json(&body).send()).await
        .map_err(|_| ChatError::new("TIMEOUT","连接模型服务超时，请重试"))?.map_err(network_error)?;
    let response = checked_response(response, &settings.api_key).await?;
    let content_type = response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|h| h.to_str().ok()).unwrap_or("");
    if !settings.stream || content_type.contains("application/json") {
        let value: Value = response.json().await.map_err(|_| ChatError::new("INVALID_RESPONSE", "模型响应不是有效 JSON"))?;
        if value.pointer("/choices/0/message/tool_calls").and_then(Value::as_array).is_some_and(|calls| !calls.is_empty()) {
            return Err(ChatError::new("INVALID_RESPONSE", "当前单机聊天不支持工具调用回复"));
        }
        let content = value.pointer("/choices/0/message/content").and_then(Value::as_str)
            .ok_or_else(|| ChatError::new("INVALID_RESPONSE", "模型没有返回可显示的回复"))?;
        output.push_str(content);
        on_delta(content)?;
        if value.pointer("/choices/0/finish_reason").and_then(Value::as_str) == Some("length") {
            return Err(ChatError::new("OUTPUT_LIMIT","模型输出达到长度上限，回复未完整结束"));
        }
        return Ok(());
    }
    let mut stream = response.bytes_stream();
    let mut decoder = SseDecoder::default();
    while let Some(item) = tokio::time::timeout(Duration::from_secs(60), stream.next()).await
        .map_err(|_| ChatError::new("TIMEOUT", "模型长时间没有响应，请重试"))? {
        for data in decoder.push(&item.map_err(network_error)?)? {
            if consume_event(&data, output, &mut on_delta)? { return Ok(()); }
        }
    }
    for data in decoder.finish()? {
        if consume_event(&data, output, &mut on_delta)? { return Ok(()); }
    }
    Err(ChatError::new("STREAM_INTERRUPTED", "连接提前结束，回复未完整接收，请重新生成"))
}

#[cfg(test)]
mod tests {
    use super::*;

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
