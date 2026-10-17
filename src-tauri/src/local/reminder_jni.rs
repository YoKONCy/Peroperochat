use jni::{objects::{JClass,JString,JObject},sys::{jstring,jboolean},JNIEnv};
use rusqlite::{Connection,OpenFlags};
use serde_json::{json,Value};
use super::{reminder_dispatch,types::*};

fn execute(path:&str,command:&str,payload:&str)->ChatResult<Value> {
    let path=std::path::Path::new(path);
    if !path.is_absolute() || path.file_name().and_then(|value|value.to_str())!=Some("peropero.db") || !path.is_file() || payload.len()>2*1024*1024 {
        return Err(ChatError::new("INVALID_INPUT","原生提醒数据库路径或参数无效"));
    }
    let control=path.parent().ok_or_else(||ChatError::new("INVALID_INPUT","数据库路径无效"))?.join("control/runtime.sqlite");
    let payload:Value=serde_json::from_str(payload).map_err(|_|ChatError::new("INVALID_INPUT","原生提醒参数不是有效 JSON"))?;
    if command=="mode" {
        return Ok(match crate::mode::lock_existing(&control,None,true) {
            Ok(gate)=>json!({"active":true,"modeToken":gate.snapshot.token()?}),
            Err(_)=>json!({"active":false}),
        });
    }
    let expected=payload.get("modeToken").map(|value|serde_json::from_value::<crate::mode::Token>(value.clone()))
        .transpose().map_err(|_|ChatError::new("MODE_INACTIVE","提醒运行代次无效"))?;
    if command!="plan" && expected.is_none() {return Err(ChatError::new("MODE_INACTIVE","旧提醒缺少运行代次，已失效"));}
    let gate=crate::mode::lock_existing(&control,expected.as_ref(),true)?;
    let mut conn=Connection::open_with_flags(path,OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    conn.busy_timeout(std::time::Duration::from_millis(1500))?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    super::reminders::initialize(&conn)?;
    let mut result=reminder_dispatch::execute(&mut conn,command,payload.get("input").cloned().unwrap_or(Value::Null),chrono::Utc::now().timestamp_millis())?;
    if command=="plan" {result["modeToken"]=json!(gate.snapshot.token()?);}
    Ok(result)
}

/// 展示或系统登记与模式切换共用同一控制事务；回调只能执行原生短操作，不能重入 JNI。
#[no_mangle]
pub extern "system" fn Java_com_peroperochat_mobile_ReminderBridge_nativeGuard(mut env:JNIEnv<'_>,_class:JClass<'_>,path:JString<'_>,token:JString<'_>,action:JObject<'_>)->jboolean {
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||->ChatResult<()> {
        let path:String=env.get_string(&path).map_err(|_|ChatError::new("INVALID_INPUT","路径编码无效"))?.into();
        let token:String=env.get_string(&token).map_err(|_|ChatError::new("INVALID_INPUT","代次编码无效"))?.into();
        let token:crate::mode::Token=serde_json::from_str(&token).map_err(|_|ChatError::new("MODE_INACTIVE","代次无效"))?;
        let path=std::path::Path::new(&path);
        if !path.is_absolute() || path.file_name().and_then(|v|v.to_str())!=Some("peropero.db") {return Err(ChatError::new("INVALID_INPUT","数据库路径无效"));}
        let _gate=crate::mode::lock_existing(&path.parent().unwrap().join("control/runtime.sqlite"),Some(&token),true)?;
        env.call_method(action,"run","()V",&[]).map_err(|_|ChatError::new("NATIVE_ERROR","系统操作失败"))?;
        Ok(())
    }));
    if matches!(result,Ok(Ok(()))) {1} else {0}
}

/// 旧回调只能撤销已经休眠的独立登记，不能取消重新激活后的新一代闹铃。
#[no_mangle]
pub extern "system" fn Java_com_peroperochat_mobile_ReminderBridge_nativeInactiveGuard(mut env:JNIEnv<'_>,_class:JClass<'_>,path:JString<'_>,action:JObject<'_>)->jboolean {
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||->ChatResult<()> {
        let path:String=env.get_string(&path).map_err(|_|ChatError::new("INVALID_INPUT","路径编码无效"))?.into();
        let path=std::path::Path::new(&path);
        if !path.is_absolute() || path.file_name().and_then(|v|v.to_str())!=Some("peropero.db") {return Err(ChatError::new("INVALID_INPUT","数据库路径无效"));}
        let _gate=crate::mode::inactive_local_gate(&path.parent().unwrap().join("control/runtime.sqlite"))?;
        env.call_method(action,"run","()V",&[]).map_err(|_|ChatError::new("NATIVE_ERROR","系统撤销失败"))?;
        Ok(())
    }));
    if matches!(result,Ok(Ok(()))) {1} else {0}
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cold_reminder_bridge_requires_native_control_and_rejects_old_epoch() {
        let data=std::env::temp_dir().join(format!("ppc-cold-{}",uuid::Uuid::new_v4()));std::fs::create_dir_all(&data).unwrap();
        let path=data.join("peropero.db");let conn=Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE reminders(id INTEGER PRIMARY KEY,task TEXT,time TEXT,timestamp INTEGER,agent_id TEXT,delivered INTEGER DEFAULT 0,recur TEXT,recur_until TEXT);
            CREATE TABLE local_configs(key TEXT PRIMARY KEY,value TEXT);").unwrap();
        conn.execute("INSERT INTO local_configs VALUES('ppc.agents',?1)",[json!(crate::local::tests::agents().to_string()).to_string()]).unwrap();drop(conn);
        let path=path.to_str().unwrap();
        assert!(execute(path,"plan","{}").is_err());
        let mode=crate::mode::Coordinator::initialize(data.join("control/runtime.sqlite")).unwrap();mode.activate().unwrap();
        let plan=execute(path,"plan","{}").unwrap();let token=plan["modeToken"].clone();
        assert_eq!(execute(path,"mode","{}").unwrap()["active"],true);
        mode.begin_switch(crate::mode::Target::Remote {profile_id:"remote".into()}).unwrap();mode.activate().unwrap();
        assert_eq!(execute(path,"mode","{}").unwrap()["active"],false);
        assert!(execute(path,"plan","{}").is_err());
        mode.begin_switch(crate::mode::Target::Standalone).unwrap();mode.activate().unwrap();
        assert!(execute(path,"registered",&json!({"modeToken":token,"input":[]}).to_string()).is_err());
        assert!(execute(path,"registered",&json!({"input":[]}).to_string()).is_err());
    }
}
