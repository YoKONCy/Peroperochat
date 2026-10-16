use serde_json::{json,Value};
use super::{provider::ModelSettings,types::*};

fn parts(content:&Value,format:&str,assistant:bool)->ChatResult<Vec<Value>> {
    if let Some(text)=content.as_str() { return Ok(vec![if format=="responses" { json!({"type":if assistant {"output_text"} else {"input_text"},"text":text}) } else if format=="gemini" { json!({"text":text}) } else { json!({"type":"text","text":text}) }]); }
    let mut result=Vec::new();
    for part in content.as_array().into_iter().flatten() {
        match part["type"].as_str() {
            Some("text")=>result.extend(parts(&part["text"],format,assistant)?),
            Some("image_url")=> {
                let url=part.pointer("/image_url/url").and_then(Value::as_str).ok_or_else(||ChatError::new("INVALID_INPUT","图片地址无效"))?;
                if format=="responses" { result.push(json!({"type":"input_image","image_url":url})); }
                else if format=="anthropic" {
                    let source=if let Some(data)=url.strip_prefix("data:") {
                        let (mime,bytes)=data.split_once(";base64,").ok_or_else(||ChatError::new("INVALID_INPUT","图片附件编码无效"))?;
                        json!({"type":"base64","media_type":mime,"data":bytes})
                    } else { json!({"type":"url","url":url}) };
                    result.push(json!({"type":"image","source":source}));
                } else {
                    let data=url.strip_prefix("data:").ok_or_else(||ChatError::new("INVALID_INPUT","Gemini 原生接口请使用上传的图片附件"))?;
                    let (mime,bytes)=data.split_once(";base64,").ok_or_else(||ChatError::new("INVALID_INPUT","图片附件编码无效"))?;
                    result.push(json!({"inlineData":{"mimeType":mime,"data":bytes}}));
                }
            }
            _=>return Err(ChatError::new("INVALID_INPUT","模型协议不支持该消息内容")),
        }
    }
    if format=="gemini" { for part in &mut result { if part["type"]=="text" { *part=json!({"text":part["text"]}); } } }
    Ok(result)
}

fn system(messages:&[ModelMessage])->String {
    messages.iter().filter(|m|m.role=="system").map(|m|text_content(&m.content)).collect::<Vec<_>>().join("\n\n")
}

pub fn body(settings:&ModelSettings,messages:&[ModelMessage],tools:&[Value])->ChatResult<Value> {
    let enabled=settings.tools_enabled && !tools.is_empty();
    let effort=settings.reasoning_effort.as_deref();
    let reasoning=effort.is_some_and(|s|s!="off" && s!="none");
    let model=settings.model.to_ascii_lowercase();
    let dialect=if settings.reasoning_dialect=="auto" {
        if model.contains("deepseek") || settings.base.contains("deepseek") {"deepseek"}
        else if settings.base.contains("openrouter.ai") {"openrouter"}
        else if settings.base.contains("api.openai.com") {"openai"} else {"generic"}
    } else {settings.reasoning_dialect.as_str()};
    let native_reasoning_model=model.starts_with("o1") || model.starts_with("o3") || model.starts_with("o4") || model.starts_with("gpt-5") || model.starts_with("gpt-6");
    let native_reasoning=native_reasoning_model && effort!=Some("off") && effort!=Some("none");
    let sampling=!reasoning && !native_reasoning;
    let effort=effort.map(|e|if e=="off" { "none" } else { e });
    let mut value=match settings.api_format.as_str() {
        "chat_completions"=> {
            let items=messages.iter().map(|m| {
                let mut item=json!({"role":m.role,"content":m.content});
                if !m.calls.is_empty() { item["tool_calls"]=json!(m.calls.iter().map(|c|json!({"id":c.id,"type":"function","function":{"name":c.name,"arguments":c.arguments.to_string()}})).collect::<Vec<_>>()); }
                if let Some(id)=&m.call_id { item["tool_call_id"]=json!(id); }
                if !m.reasoning.is_empty() && dialect!="openai" && !native_reasoning { item["reasoning_content"]=json!(m.reasoning); }
                if let Some(details)=m.native.get("reasoning_details").filter(|v|v.as_array().is_some_and(|a|!a.is_empty())) { item["reasoning_details"]=details.clone(); }
                item
            }).collect::<Vec<_>>();
            let mut value=json!({"model":settings.model,"messages":items,"stream":settings.stream});
            value[if native_reasoning_model || (dialect=="openai" && reasoning) { "max_completion_tokens" } else { "max_tokens" }]=json!(settings.max_tokens);
            if enabled { value["tools"]=json!(tools.iter().map(|t|json!({"type":"function","function":t})).collect::<Vec<_>>()); }
            if let Some(effort)=effort {
                value["reasoning_effort"]=json!(effort);
                if dialect=="deepseek" { value["thinking"]=json!({"type":if reasoning {"enabled"} else {"disabled"}}); }
                if dialect=="openrouter" { value["reasoning"]=json!({"effort":effort});value.as_object_mut().unwrap().remove("reasoning_effort"); }
            }
            if sampling {
                if let Some(v)=settings.frequency_penalty { value["frequency_penalty"]=json!(v); }
                if let Some(v)=settings.presence_penalty { value["presence_penalty"]=json!(v); }
            }
            value
        }
        "responses"=> {
            let mut items=Vec::new();
            for m in messages {
                if let Some(native)=m.native.as_array() { items.extend(native.iter().cloned());continue; }
                if m.role=="tool" { items.push(json!({"type":"function_call_output","call_id":m.call_id,"output":text_content(&m.content)})); }
                else if m.role=="assistant" {items.push(json!({"role":"assistant","content":text_content(&m.content)}));}
                else { items.push(json!({"role":m.role,"content":parts(&m.content,"responses",false)?})); }
            }
            let mut value=json!({"model":settings.model,"input":items,"stream":settings.stream,"store":false,"include":["reasoning.encrypted_content"],"max_output_tokens":settings.max_tokens});
            if let Some(effort)=effort { value["reasoning"]=json!({"effort":effort,"summary":"auto"}); }
            if enabled { value["tools"]=json!(tools.iter().map(|t| { let mut t=t.clone();t["type"]=json!("function");t["strict"]=json!(false);t }).collect::<Vec<_>>()); }
            value
        }
        "anthropic"=> {
            let mut items:Vec<Value>=Vec::new();
            for m in messages.iter().filter(|m|m.role!="system") {
                let content=if let Some(native)=m.native.as_array() { native.clone() }
                    else if m.role=="tool" { vec![json!({"type":"tool_result","tool_use_id":m.call_id,"content":text_content(&m.content)})] }
                    else { parts(&m.content,"anthropic",m.role=="assistant")? };
                let role=if m.role=="assistant" { "assistant" } else { "user" };
                if let Some(last)=items.last_mut().filter(|v|v["role"]==role) { last["content"].as_array_mut().unwrap().extend(content); }
                else { items.push(json!({"role":role,"content":content})); }
            }
            let mut value=json!({"model":settings.model,"system":system(messages),"messages":items,"stream":settings.stream,"max_tokens":settings.max_tokens});
            if reasoning {
                if model.contains("4-6") || model.contains("4.6") || model.contains("4-7") || model.contains("4.7") || model.contains("4-8") || model.contains("4.8")
                    || model.starts_with("claude-5") || ["claude-opus-5","claude-sonnet-5","claude-haiku-5"].iter().any(|family|model.contains(family)) {
                    value["thinking"]=json!({"type":"adaptive"});value["output_config"]=json!({"effort":match effort {Some("minimal")=>"low",Some("xhigh")=>"high",Some(e)=>e,None=>"high"}});
                } else {
                    if settings.max_tokens<=1024 { return Err(ChatError::new("INVALID_CONFIG","Claude 预算思考模式的总输出额度需要大于 1024")); }
                    let budget=(match effort { Some("minimal"|"low")=>1024,Some("medium")=>4096,_=>8192 }).min((settings.max_tokens/2).max(1024));
                    value["thinking"]=json!({"type":"enabled","budget_tokens":budget});
                }
            }
            if enabled { value["tools"]=json!(tools.iter().map(|t|json!({"name":t["name"],"description":t["description"],"input_schema":t["parameters"]})).collect::<Vec<_>>()); }
            value
        }
        "gemini"=> {
            let mut items:Vec<Value>=Vec::new();
            for m in messages.iter().filter(|m|m.role!="system") {
                let content=if let Some(native)=m.native.as_array() { native.clone() }
                    else if m.role=="tool" { vec![json!({"functionResponse":{"id":m.call_id,"name":m.name,"response":serde_json::from_str::<Value>(&text_content(&m.content)).unwrap_or(json!({"result":text_content(&m.content)}))}})] }
                    else { parts(&m.content,"gemini",m.role=="assistant")? };
                let role=if m.role=="assistant" { "model" } else { "user" };
                if let Some(last)=items.last_mut().filter(|v|v["role"]==role) { last["parts"].as_array_mut().unwrap().extend(content); }
                else { items.push(json!({"role":role,"parts":content})); }
            }
            let mut value=json!({"systemInstruction":{"parts":[{"text":system(messages)}]},"contents":items,"generationConfig":{"maxOutputTokens":settings.max_tokens}});
            if let Some(effort)=effort {
                let thinking=if model.starts_with("gemini-3") { json!({"thinkingLevel":match effort {"none"|"minimal"=>"minimal","low"=>"low","medium"=>"medium",_=>"high"}}) }
                    else { json!({"thinkingBudget":match effort {"none"=>0,"minimal"|"low"=>1024,"medium"=>4096,_=>8192}}) };
                value["generationConfig"]["thinkingConfig"]=thinking;
            }
            if enabled { value["tools"]=json!([{"functionDeclarations":tools.iter().map(|t|json!({"name":t["name"],"description":t["description"],"parameters":gemini_schema(&t["parameters"])})).collect::<Vec<_>>()}]); }
            value
        }
        _=>return Err(ChatError::new("INVALID_CONFIG","模型协议无效")),
    };
    if sampling {
        let target=if settings.api_format=="gemini" { &mut value["generationConfig"] } else { &mut value };
        if let Some(v)=settings.temperature { target["temperature"]=json!(v); }
        if let Some(v)=settings.top_p { target[if settings.api_format=="gemini" {"topP"} else {"top_p"}]=json!(v); }
    }
    for (key,extra) in &settings.extra_body {
        if let (Some(target),Some(extra))=(value.get_mut(key).and_then(Value::as_object_mut),extra.as_object()) { target.extend(extra.clone()); }
        else { value[key]=extra.clone(); }
    }
    for key in &settings.disabled_params {
        value.as_object_mut().unwrap().remove(key);
        if let Some(config)=value.get_mut("generationConfig").and_then(Value::as_object_mut) { config.remove(key); }
    }
    Ok(value)
}

fn gemini_schema(schema:&Value)->Value {
    let mut value=schema.clone();
    if let Some(object)=value.as_object_mut() {
        object.remove("additionalProperties");
        if let Some(kind)=object.get_mut("type") { if let Some(text)=kind.as_str() {*kind=json!(text.to_ascii_uppercase());} }
        if let Some(properties)=object.get_mut("properties").and_then(Value::as_object_mut) { for child in properties.values_mut() {*child=gemini_schema(child);} }
        if let Some(items)=object.get_mut("items") {*items=gemini_schema(items);}
    }
    value
}

fn call(id:Option<&str>,name:Option<&str>,args:Value)->ChatResult<ToolCall> {
    let name=name.filter(|s|!s.is_empty() && s.len()<=100).ok_or_else(||ChatError::new("INVALID_RESPONSE","模型工具名称无效"))?;
    let arguments=if let Some(raw)=args.as_str() { serde_json::from_str(raw).map_err(|_|ChatError::new("INVALID_RESPONSE","模型工具参数不是完整 JSON"))? } else { args };
    if !arguments.is_object() { return Err(ChatError::new("INVALID_RESPONSE","工具参数必须是对象")); }
    Ok(ToolCall { id:id.filter(|s|!s.is_empty()).map(str::to_owned).unwrap_or_else(||uuid::Uuid::new_v4().to_string()),name:name.into(),arguments })
}

pub fn parse(format:&str,value:&Value)->ChatResult<Completion> {
    if let Some(error)=value.get("error") { return Err(ChatError::new("UPSTREAM_ERROR",error["message"].as_str().unwrap_or("模型服务返回错误"))); }
    let mut result=Completion::default();
    match format {
        "chat_completions"=> {
            let message=value.pointer("/choices/0/message").ok_or_else(||ChatError::new("INVALID_RESPONSE","模型响应缺少消息"))?;
            result.content=text_content(&message["content"]);
            result.reasoning=message["reasoning_content"].as_str().or_else(||message["reasoning"].as_str()).unwrap_or("").into();
            result.native=message.clone(); result.usage=value["usage"].clone();
            for t in message["tool_calls"].as_array().into_iter().flatten() { result.calls.push(call(t["id"].as_str(),t.pointer("/function/name").and_then(Value::as_str),t["function"]["arguments"].clone())?); }
            check_end(value.pointer("/choices/0/finish_reason").and_then(Value::as_str))?;
        }
        "responses"=> {
            if value["status"]=="incomplete" || value["status"]=="failed" || value["status"]=="cancelled" { return Err(ChatError::new("OUTPUT_LIMIT","模型响应未完整结束")); }
            let output=value["output"].as_array().ok_or_else(||ChatError::new("INVALID_RESPONSE","Responses 响应缺少输出"))?;
            result.native=json!(output);result.usage=value["usage"].clone();
            for item in output { match item["type"].as_str() {
                Some("message")=>for part in item["content"].as_array().into_iter().flatten() { if part["type"]=="output_text" { result.content.push_str(part["text"].as_str().unwrap_or("")); } },
                Some("function_call")=>result.calls.push(call(item["call_id"].as_str(),item["name"].as_str(),item["arguments"].clone())?),
                Some("reasoning")=>for part in item["summary"].as_array().into_iter().flatten() { result.reasoning.push_str(part["text"].as_str().unwrap_or("")); },_=>{}
            } }
        }
        "anthropic"=> {
            let content=value["content"].as_array().ok_or_else(||ChatError::new("INVALID_RESPONSE","Claude 响应缺少正文"))?;
            result.native=json!(content);result.usage=value["usage"].clone();
            for part in content { match part["type"].as_str() {
                Some("text")=>result.content.push_str(part["text"].as_str().unwrap_or("")),
                Some("thinking")=>result.reasoning.push_str(part["thinking"].as_str().unwrap_or("")),
                Some("tool_use")=>result.calls.push(call(part["id"].as_str(),part["name"].as_str(),part["input"].clone())?),_=>{}
            } }
            check_end(value["stop_reason"].as_str())?;
        }
        "gemini"=> {
            let candidate=value.pointer("/candidates/0").ok_or_else(||ChatError::new("INVALID_RESPONSE","Gemini 没有返回候选回复"))?;
            let parts=candidate.pointer("/content/parts").and_then(Value::as_array).ok_or_else(||ChatError::new("INVALID_RESPONSE","Gemini 响应缺少正文"))?;
            result.native=json!(parts);result.usage=value["usageMetadata"].clone();
            for part in parts {
                if let Some(text)=part["text"].as_str() { if part["thought"]==true { result.reasoning.push_str(text); } else { result.content.push_str(text); } }
                if let Some(function)=part.get("functionCall") { result.calls.push(call(function["id"].as_str(),function["name"].as_str(),function["args"].clone())?); }
            }
            check_end(candidate["finishReason"].as_str())?;
        }
        _=>return Err(ChatError::new("INVALID_CONFIG","模型协议无效")),
    }
    if result.content.len()>4*1024*1024 || result.reasoning.len()>2*1024*1024 || result.calls.len()>24 { return Err(ChatError::new("RESPONSE_TOO_LARGE","模型响应过大")); }
    Ok(result)
}

pub fn check_end(reason:Option<&str>)->ChatResult<()> {
    match reason {
        Some("length"|"max_tokens"|"MAX_TOKENS")=>Err(ChatError::new("OUTPUT_LIMIT","模型输出达到长度上限，本轮尚未完整结束")),
        Some("content_filter"|"SAFETY"|"RECITATION"|"refusal")=>Err(ChatError::new("UPSTREAM_ERROR","模型服务未完成此次回复")),
        Some("stop"|"tool_calls"|"end_turn"|"tool_use"|"stop_sequence"|"STOP")|None=>Ok(()),
        _=>Err(ChatError::new("INVALID_RESPONSE","模型返回不支持的结束原因")),
    }
}
