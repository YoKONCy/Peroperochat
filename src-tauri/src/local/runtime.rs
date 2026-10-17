use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use tokio::sync::watch;
use super::types::{ChatError, ChatResult};

#[derive(Default)]
struct RuntimeInner {
    threads: HashMap<String, (String, watch::Sender<bool>)>,
    early_cancellations: VecDeque<String>,
}

#[derive(Default)]
pub struct ChatRuntime { inner: Mutex<RuntimeInner>, mode:Option<crate::mode::Coordinator> }

pub struct TurnLease<'a> {
    runtime: &'a ChatRuntime,
    thread_id: String,
    pub cancel: watch::Receiver<bool>,
}

impl ChatRuntime {
    pub fn with_mode(mode:crate::mode::Coordinator)->Self {Self {inner:Mutex::new(RuntimeInner::default()),mode:Some(mode)}}
    pub fn cancel_all(&self)->ChatResult<()> {
        let inner=self.inner.lock().map_err(|_|ChatError::new("RUNTIME_ERROR","聊天运行状态不可用"))?;
        for (_,sender) in inner.threads.values() {let _=sender.send(true);}Ok(())
    }
    pub fn is_idle(&self)->ChatResult<bool> {Ok(self.inner.lock().map_err(|_|ChatError::new("RUNTIME_ERROR","聊天运行状态不可用"))?.threads.is_empty())}
    pub fn reserve(&self, thread_id: &str, request_id: &str) -> ChatResult<TurnLease<'_>> {
        let _gate=self.mode.as_ref().map(|mode|mode.local_gate()).transpose()?;
        let mut inner = self.inner.lock().map_err(|_| ChatError::new("RUNTIME_ERROR", "聊天运行状态不可用"))?;
        if inner.threads.contains_key("__maintenance__") || inner.threads.contains_key(thread_id) || inner.threads.values().any(|(id, _)| id == request_id) {
            return Err(ChatError::new("RESOURCE_BUSY", "该会话正在回复，请等待完成或先停止"));
        }
        let cancelled = inner.early_cancellations.iter().any(|id| id == request_id);
        inner.early_cancellations.retain(|id| id != request_id);
        let (sender, receiver) = watch::channel(cancelled);
        inner.threads.insert(thread_id.into(), (request_id.into(), sender));
        Ok(TurnLease { runtime: self, thread_id: thread_id.into(), cancel: receiver })
    }

    pub fn cancel(&self, request_id: &str) -> ChatResult<()> {
        let _gate=self.mode.as_ref().map(|mode|mode.local_gate()).transpose()?;
        let mut inner = self.inner.lock().map_err(|_| ChatError::new("RUNTIME_ERROR", "聊天运行状态不可用"))?;
        if let Some((_, sender)) = inner.threads.values().find(|(id, _)| id == request_id) {
            let _ = sender.send(true);
        } else {
            // 取消可能先于 IPC 请求到达，只缓存有界的请求身份。
            if inner.early_cancellations.len() >= 128 { inner.early_cancellations.pop_front(); }
            inner.early_cancellations.push_back(request_id.into());
        }
        Ok(())
    }

    pub fn maintenance(&self) -> ChatResult<TurnLease<'_>> {
        let _gate=self.mode.as_ref().map(|mode|mode.local_gate()).transpose()?;
        let mut inner = self.inner.lock().map_err(|_| ChatError::new("RUNTIME_ERROR", "聊天运行状态不可用"))?;
        if !inner.threads.is_empty() { return Err(ChatError::new("RESOURCE_BUSY", "请先停止回复，再修改角色或导入数据")); }
        let (sender, receiver) = watch::channel(false);
        inner.threads.insert("__maintenance__".into(), ("maintenance".into(), sender));
        Ok(TurnLease { runtime: self, thread_id: "__maintenance__".into(), cancel: receiver })
    }
}

impl Drop for TurnLease<'_> {
    fn drop(&mut self) {
        if let Ok(mut inner) = self.runtime.inner.lock() { inner.threads.remove(&self.thread_id); }
    }
}
