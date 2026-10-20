//! 工作台只转交服务端返回的路径、文件指纹和终端标识，不在手机执行文件业务。
use serde::{Deserialize,Serialize};
use crate::local::types::*;

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct WorkspaceContext {
    #[serde(default,skip_serializing_if="Option::is_none")] pub file_path:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")] pub file_hash:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")] pub file_dirty:Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")] pub terminal_id:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")] pub selection:Option<Selection>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Selection {text:String,start_line:u32,end_line:u32}
impl WorkspaceContext {
    pub fn validate(&self)->ChatResult<()> {
        let invalid=self.file_path.as_ref().is_some_and(|v|v.is_empty() || v.len()>2048 || v.chars().any(char::is_control))
            || self.terminal_id.as_ref().is_some_and(|v|v.is_empty() || v.len()>256 || v.chars().any(char::is_control))
            || self.file_hash.as_ref().is_some_and(|v|v.len()!=64 || !v.bytes().all(|v|v.is_ascii_hexdigit()))
            || (self.file_hash.is_some() || self.file_dirty.is_some() || self.selection.is_some()) && self.file_path.is_none()
            || self.file_path.is_none() && self.terminal_id.is_none()
            || self.selection.as_ref().is_some_and(|v|v.text.trim().is_empty() || v.text.chars().count()>4000 || v.start_line==0 || v.end_line<v.start_line);
        if invalid {return Err(ChatError::new("INVALID_INPUT","工作台引用的范围或文件指纹无效"));}Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn context_is_bounded_and_requires_a_scoped_resource() {
        let valid:WorkspaceContext=serde_json::from_value(serde_json::json!({"filePath":"notes.md","fileHash":"a".repeat(64),"fileDirty":false})).unwrap();assert!(valid.validate().is_ok());
        for value in [serde_json::json!({}),serde_json::json!({"fileHash":"a".repeat(64)}),serde_json::json!({"filePath":"notes.md","selection":{"text":"内容","startLine":4,"endLine":2}}),serde_json::json!({"terminalId":"x".repeat(257)})] {
            let context:WorkspaceContext=serde_json::from_value(value).unwrap();assert!(context.validate().is_err());
        }
        assert!(serde_json::from_value::<WorkspaceContext>(serde_json::json!({"url":"https://example.invalid"})).is_err());
    }
}
