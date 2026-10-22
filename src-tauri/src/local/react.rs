use serde_json::json;
use crate::db::DbState;
use super::{chat_tools::{self,ToolSession},context,provider::{self,ModelSettings},types::*};

pub async fn run(db:&DbState,settings:&ModelSettings,agent_id:&str,mut messages:Vec<ModelMessage>,session:&mut ToolSession,
    output:&mut String,mut emit:impl FnMut(&str,&str)->ChatResult<()>)->ChatResult<Completion> {
    let definitions=if settings.tools_enabled { chat_tools::definitions() } else { Vec::new() };
    let mut reasoning=String::new();let mut usage=Vec::new();
    for step in 0..settings.max_steps {
        let tokens=messages.iter().map(|m|8+context::content_tokens(&m.content)+context::estimated_tokens(&m.reasoning)
            +context::estimated_tokens(&json!(m.calls).to_string())+if m.native.is_null() {0} else {context::estimated_tokens(&m.native.to_string())}).sum::<usize>()
            +context::estimated_tokens(&json!(definitions).to_string());
        if tokens+settings.max_tokens+256>settings.context_window { return Err(ChatError::new("CONTEXT_LIMIT","本轮工具结果超过模型上下文窗口")); }
        output.clear();if step>0 { emit("step","")?; }
        let mut completion=provider::complete(settings,&messages,&definitions,|kind,delta| {
            if kind=="delta" { output.push_str(delta); } emit(kind,delta)
        }).await?;
        *output=completion.content.clone();
        if !completion.reasoning.is_empty() { if !reasoning.is_empty() { reasoning.push_str("\n\n"); }reasoning.push_str(&completion.reasoning); }
        if reasoning.len()>2*1024*1024 { return Err(ChatError::new("RESPONSE_TOO_LARGE","本轮思考摘要过长")); }
        usage.push(json!({"step":step+1,"protocol":settings.api_format,"usage":completion.usage}));
        if completion.calls.is_empty() {
            completion.content=context::visible_text(&completion.content);
            if completion.content.trim().is_empty() { return Err(ChatError::new("EMPTY_RESPONSE","模型没有返回可显示的回复")); }
            let presentation=completion.usage["_ppcReasoning"].clone();
            completion.reasoning=reasoning;completion.usage=json!({"steps":usage,"presentation":presentation});return Ok(completion);
        }
        if !settings.tools_enabled || step+1==settings.max_steps { return Err(ChatError::new("TOOL_LIMIT","模型工具调用达到本轮上限，请重新发送")); }
        messages.push(ModelMessage { role:"assistant".into(),content:json!(completion.content),calls:completion.calls.clone(),
            reasoning:completion.reasoning,native:completion.native,..Default::default() });
        for call in completion.calls {
            emit("tool",&json!(super::tool_timeline::describe(&call,&serde_json::Value::Null,None)).to_string())?;
            let result=session.execute(db,agent_id,&call)?;
            emit("tool",&json!(super::tool_timeline::describe(&call,&result,None)).to_string())?;
            messages.push(ModelMessage { role:"tool".into(),content:json!(result.to_string()),call_id:Some(call.id),name:Some(call.name),..Default::default() });
        }
    }
    Err(ChatError::new("TOOL_LIMIT","本轮请求达到工具调用上限"))
}
