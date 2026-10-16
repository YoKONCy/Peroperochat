use std::collections::BTreeMap;
use serde_json::{json,Value};
use super::{types::*,wire};

#[derive(Default)]
pub struct Accumulator {
    pub text:String,pub reasoning:String,pub usage:Value,pub terminal:bool,
    finish:Option<String>,items:BTreeMap<usize,Value>,arguments:BTreeMap<usize,String>,final_response:Option<Value>,
    details:BTreeMap<usize,Value>,
}
impl Accumulator {
    pub fn event(&mut self,format:&str,data:&str,mut emit:impl FnMut(&str,&str)->ChatResult<()>)->ChatResult<bool> {
        if data.trim()=="[DONE]" {
            if format=="chat_completions" { self.terminal=true;return Ok(true); }
            return Ok(self.terminal);
        }
        let value:Value=serde_json::from_str(data).map_err(|_|ChatError::new("INVALID_RESPONSE","模型流事件不是有效 JSON"))?;
        if value.get("error").is_some() || value["type"]=="error" || value["type"]=="response.failed" {
            return Err(ChatError::new("UPSTREAM_ERROR",value.pointer("/error/message").and_then(Value::as_str).unwrap_or("模型服务返回流式错误")));
        }
        let mut text=String::new();let mut reasoning=String::new();
        match format {
            "chat_completions"=> {
                if let Some(delta)=value.pointer("/choices/0/delta") {
                    text=delta["content"].as_str().unwrap_or("").into();
                    reasoning=delta["reasoning_content"].as_str().or_else(||delta["reasoning"].as_str()).unwrap_or("").into();
                    for detail in delta["reasoning_details"].as_array().into_iter().flatten() {
                        let index=detail["index"].as_u64().unwrap_or(0) as usize;
                        if index>=100 { return Err(ChatError::new("RESPONSE_TOO_LARGE","思考内容项过多")); }
                        let saved=self.details.entry(index).or_insert(json!({}));
                        if let Some(fields)=detail.as_object() { for (key,value) in fields {
                            if ["text","data","signature","summary"].contains(&key.as_str()) && value.is_string() {
                                saved[key]=json!(format!("{}{}",saved[key].as_str().unwrap_or(""),value.as_str().unwrap()));
                            } else { saved[key]=value.clone(); }
                        } }
                    }
                    for item in delta["tool_calls"].as_array().into_iter().flatten() {
                        let index=item["index"].as_u64().unwrap_or(0) as usize;
                        if index>=24 { return Err(ChatError::new("TOOL_LIMIT","模型返回过多工具调用")); }
                        let saved=self.items.entry(index).or_insert(json!({"id":"","type":"function","function":{"name":"","arguments":""}}));
                        if let Some(id)=item["id"].as_str() { saved["id"]=json!(id); }
                        if let Some(name)=item.pointer("/function/name").and_then(Value::as_str) { saved["function"]["name"]=json!(format!("{}{name}",saved["function"]["name"].as_str().unwrap_or(""))); }
                        if let Some(args)=item.pointer("/function/arguments").and_then(Value::as_str) { self.arguments.entry(index).or_default().push_str(args); }
                    }
                }
                if let Some(reason)=value.pointer("/choices/0/finish_reason").and_then(Value::as_str) {
                    wire::check_end(Some(reason))?;self.finish=Some(reason.into());self.terminal=true;
                }
                if !value["usage"].is_null() { self.usage=value["usage"].clone(); }
            }
            "responses"=>match value["type"].as_str() {
                Some("response.output_text.delta")=>text=value["delta"].as_str().unwrap_or("").into(),
                Some("response.reasoning_summary_text.delta"|"response.reasoning_text.delta")=>reasoning=value["delta"].as_str().unwrap_or("").into(),
                Some("response.output_item.added"|"response.output_item.done")=> {
                    let index=value["output_index"].as_u64().unwrap_or(0) as usize;
                    if index>=100 { return Err(ChatError::new("RESPONSE_TOO_LARGE","Responses 输出项过多")); }
                    self.items.insert(index,value["item"].clone());
                }
                Some("response.function_call_arguments.delta")=> {
                    let index=value["output_index"].as_u64().unwrap_or(0) as usize;
                    if index>=100 { return Err(ChatError::new("RESPONSE_TOO_LARGE","Responses 输出项过多")); }
                    self.arguments.entry(index).or_default().push_str(value["delta"].as_str().unwrap_or(""));
                }
                Some("response.completed")=> { self.final_response=Some(value["response"].clone());self.terminal=true;return Ok(true); }
                Some("response.incomplete"|"response.cancelled")=>return Err(ChatError::new("OUTPUT_LIMIT","Responses 输出未完整结束")),_=>{}
            },
            "anthropic"=>match value["type"].as_str() {
                Some("message_start")=>self.usage=value["message"]["usage"].clone(),
                Some("content_block_start")=> {
                    let index=value["index"].as_u64().unwrap_or(0) as usize;
                    if index>=100 { return Err(ChatError::new("RESPONSE_TOO_LARGE","Claude 内容块过多")); }
                    let block=value["content_block"].clone();
                    text=block["text"].as_str().unwrap_or("").into();reasoning=block["thinking"].as_str().unwrap_or("").into();
                    self.items.insert(index,block);
                }
                Some("content_block_delta")=> {
                    let index=value["index"].as_u64().unwrap_or(0) as usize;
                    let delta=&value["delta"];
                    if let Some(block)=self.items.get_mut(&index) { match delta["type"].as_str() {
                        Some("text_delta")=> { text=delta["text"].as_str().unwrap_or("").into();block["text"]=json!(format!("{}{}",block["text"].as_str().unwrap_or(""),text)); }
                        Some("thinking_delta")=> { reasoning=delta["thinking"].as_str().unwrap_or("").into();block["thinking"]=json!(format!("{}{}",block["thinking"].as_str().unwrap_or(""),reasoning)); }
                        Some("signature_delta")=>block["signature"]=json!(format!("{}{}",block["signature"].as_str().unwrap_or(""),delta["signature"].as_str().unwrap_or(""))),
                        Some("input_json_delta")=>self.arguments.entry(index).or_default().push_str(delta["partial_json"].as_str().unwrap_or("")),_=>{}
                    } }
                }
                Some("message_delta")=> {
                    self.finish=value["delta"]["stop_reason"].as_str().map(str::to_owned);wire::check_end(self.finish.as_deref())?;
                    if let Some(usage)=value["usage"].as_object() { if !self.usage.is_object() { self.usage=json!({}); }self.usage.as_object_mut().unwrap().extend(usage.clone()); }
                }
                Some("message_stop")=> { self.terminal=true;return Ok(true); },_=>{}
            },
            "gemini"=> {
                for part in value.pointer("/candidates/0/content/parts").and_then(Value::as_array).into_iter().flatten() {
                    if let Some(t)=part["text"].as_str() { if part["thought"]==true { reasoning.push_str(t); } else { text.push_str(t); } }
                    self.items.insert(self.items.len(),part.clone());
                }
                if let Some(reason)=value.pointer("/candidates/0/finishReason").and_then(Value::as_str) {
                    wire::check_end(Some(reason))?;self.finish=Some(reason.into());self.terminal=true;
                }
                if !value["usageMetadata"].is_null() { self.usage=value["usageMetadata"].clone(); }
            }
            _=>return Err(ChatError::new("INVALID_CONFIG","流式协议无效")),
        }
        self.text.push_str(&text);self.reasoning.push_str(&reasoning);
        if self.text.len()>4*1024*1024 || self.reasoning.len()>2*1024*1024 || self.arguments.values().any(|a|a.len()>64_000) || self.items.values().chain(self.details.values()).map(|v|v.to_string().len()).sum::<usize>()>4*1024*1024 {
            return Err(ChatError::new("RESPONSE_TOO_LARGE","模型流式输出过大"));
        }
        if !text.is_empty() { emit("delta",&text)?; }if !reasoning.is_empty() { emit("reasoning",&reasoning)?; }
        Ok(false)
    }

    pub fn finish(mut self,format:&str)->ChatResult<Completion> {
        if !self.terminal { return Err(ChatError::new("STREAM_INTERRUPTED","连接提前结束，回复未完整接收")); }
        if let Some(response)=self.final_response { return wire::parse(format,&response); }
        for (index,args) in self.arguments { if let Some(item)=self.items.get_mut(&index) {
            if format=="anthropic" { item["input"]=serde_json::from_str(&args).map_err(|_|ChatError::new("INVALID_RESPONSE","工具参数未完整接收"))?; }
            else if format=="chat_completions" { item["function"]["arguments"]=json!(args); }
            else { item["arguments"]=json!(args); }
        } }
        let parts=self.items.into_values().collect::<Vec<_>>();
        let value=match format {
            "chat_completions"=>json!({"choices":[{"message":{"content":self.text,"reasoning_content":self.reasoning,"reasoning_details":self.details.into_values().collect::<Vec<_>>(),"tool_calls":parts},"finish_reason":self.finish}],"usage":self.usage}),
            "responses"=>json!({"status":"completed","output":parts,"usage":self.usage}),
            "anthropic"=>json!({"content":parts,"stop_reason":self.finish,"usage":self.usage}),
            "gemini"=>json!({"candidates":[{"content":{"parts":parts},"finishReason":self.finish}],"usageMetadata":self.usage}),
            _=>return Err(ChatError::new("INVALID_CONFIG","模型协议无效")),
        };
        wire::parse(format,&value)
    }
}
