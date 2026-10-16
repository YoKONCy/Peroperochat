use jni::{objects::{JClass,JString},sys::jstring,JNIEnv};
use rusqlite::{Connection,OpenFlags};
use serde_json::{json,Value};
use super::{reminder_dispatch,types::*};

fn execute(path:&str,command:&str,payload:&str)->ChatResult<Value> {
    let path=std::path::Path::new(path);
    if !path.is_absolute() || path.file_name().and_then(|value|value.to_str())!=Some("peropero.db") || !path.is_file() || payload.len()>2*1024*1024 {
        return Err(ChatError::new("INVALID_INPUT","原生提醒数据库路径或参数无效"));
    }
    let mut conn=Connection::open_with_flags(path,OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    conn.busy_timeout(std::time::Duration::from_millis(1500))?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    super::reminders::initialize(&conn)?;
    let payload:Value=serde_json::from_str(payload).map_err(|_|ChatError::new("INVALID_INPUT","原生提醒参数不是有效 JSON"))?;
    reminder_dispatch::execute(&mut conn,command,payload,chrono::Utc::now().timestamp_millis())
}

#[no_mangle]
pub extern "system" fn Java_com_peroperochat_mobile_ReminderBridge_nativeExecute(mut env:JNIEnv<'_>,_class:JClass<'_>,path:JString<'_>,command:JString<'_>,payload:JString<'_>)->jstring {
    // 不让数据库错误或 Rust panic 穿过 JNI 边界。
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||->ChatResult<Value> {
        let path:String=env.get_string(&path).map_err(|_|ChatError::new("INVALID_INPUT","数据库路径编码无效"))?.into();
        let command:String=env.get_string(&command).map_err(|_|ChatError::new("INVALID_INPUT","原生操作编码无效"))?.into();
        let payload:String=env.get_string(&payload).map_err(|_|ChatError::new("INVALID_INPUT","原生参数编码无效"))?.into();
        execute(&path,&command,&payload)
    }));
    let response=match result {
        Ok(Ok(value))=>json!({"ok":true,"value":value}),
        Ok(Err(error))=>json!({"ok":false,"error":error}),
        Err(_)=>json!({"ok":false,"error":{"code":"NATIVE_ERROR","message":"原生提醒处理异常，等待恢复"}}),
    };
    env.new_string(response.to_string()).map(|value|value.into_raw()).unwrap_or(std::ptr::null_mut())
}
