use std::cell::RefCell;
use jni::{JNIEnv,objects::{JClass,JObject,JString,JValue},sys::jstring};
use serde_json::{json,Value};
use crate::{local::types::*,mode::Token};
use super::{background_store as store,background_worker};

fn response(result:ChatResult<Value>)->Value {
    match result {Ok(value)=>json!({"ok":true,"value":value}),Err(error)=>json!({"ok":false,"error":error})}
}
#[no_mangle]
pub extern "system" fn Java_com_peroperochat_mobile_RemoteReceiverBridge_nativeExecute(mut env:JNIEnv<'_>,_:JClass<'_>,path:JString<'_>,command:JString<'_>,input:JString<'_>)->jstring {
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||->ChatResult<Value> {
        let path:String=env.get_string(&path).map_err(|_|super::inactive())?.into();
        let command:String=env.get_string(&command).map_err(|_|super::inactive())?.into();
        let input:String=env.get_string(&input).map_err(|_|super::inactive())?.into();
        if input.len()>16*1024 {return Err(super::inactive());}
        let input:Value=serde_json::from_str(&input).map_err(|_|super::inactive())?;
        store::execute(store::root(&path)?,&command,&input)
    })).unwrap_or_else(|_|Err(ChatError::new("NATIVE_ERROR","后台通知处理异常")));
    env.new_string(response(result).to_string()).map(|s|s.into_raw()).unwrap_or(std::ptr::null_mut())
}

/// 当前线程只启动通知传输运行时；Java 回调也始终留在该 JNI 线程。
#[no_mangle]
pub extern "system" fn Java_com_peroperochat_mobile_RemoteReceiverBridge_nativeRun(mut env:JNIEnv<'_>,_:JClass<'_>,path:JString<'_>,token:JString<'_>,secret:JString<'_>,callback:JObject<'_>)->jstring {
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||->ChatResult<Value> {
        let path:String=env.get_string(&path).map_err(|_|super::inactive())?.into();
        let token:String=env.get_string(&token).map_err(|_|super::inactive())?.into();
        let secret:String=env.get_string(&secret).map_err(|_|super::inactive())?.into();
        let token:Token=serde_json::from_str(&token).map_err(|_|super::inactive())?;let root=store::root(&path)?;
        let runtime=tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|_|ChatError::new("NATIVE_ERROR","后台网络运行时不可用"))?;
        let shared=RefCell::new(&mut env);
        let result=runtime.block_on(background_worker::run(root,&token,&secret,
            &mut ||shared.borrow_mut().call_method(&callback,"isStopped","()Z",&[]).and_then(|v|v.z()).unwrap_or(true),
            &mut |notice| {
                let mut env=shared.borrow_mut();
                // 一次 JNI 接收可能持续数天，每条通知都释放局部引用，避免引用表耗尽。
                env.with_local_frame(8,|env|->jni::errors::Result<bool> {
                    let raw=env.new_string(notice.to_string())?;
                    env.call_method(&callback,"show","(Ljava/lang/String;)Z",&[JValue::Object(raw.as_ref())]).and_then(|v|v.z())
                }).map_err(|_|ChatError::new("NATIVE_ERROR","系统通知回调失败"))
            }));
        drop(shared);
        if let Err(error)=&result {if error.code!="MODE_INACTIVE" {
            let blocked=matches!(error.code.as_str(),"REMOTE_AUTH_REQUIRED"|"REMOTE_IDENTITY_CHANGED"|"REMOTE_INCOMPATIBLE"|"REMOTE_PROTOCOL");
            let _=store::state(root,&token,if blocked {"blocked"} else {"reconnecting"},if blocked {&error.code} else {""});
        }}
        result.map(|_|Value::Null)
    })).unwrap_or_else(|_|Err(ChatError::new("NATIVE_ERROR","后台通知接收异常")));
    // 不把 Java 异常带回下一次 JNI 调用，也不输出凭据或服务端原文。
    if env.exception_check().unwrap_or(false) {let _=env.exception_clear();}
    env.new_string(response(result).to_string()).map(|s|s.into_raw()).unwrap_or(std::ptr::null_mut())
}
