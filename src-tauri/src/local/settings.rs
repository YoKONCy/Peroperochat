use rusqlite::Connection;
use serde_json::{json,Value};
use tauri::{AppHandle,Manager};
use crate::db::DbState;
use super::{db_lock,identity,types::*};

fn invalid()->ChatError { ChatError::new("INVALID_CONFIG","设置格式无效或超出允许范围") }

fn text<'a>(values:&'a Value,key:&str,max:usize)->ChatResult<&'a str> {
    values[key].as_str().filter(|s|s.len()<=max).ok_or_else(invalid)
}


pub fn save_in(conn:&mut Connection,section:&str,values:&Value)->ChatResult<()> {
    if !values.is_object() { return Err(invalid()); }
    let tx=conn.transaction()?;
    let mut config=identity::snapshot_in(&tx)?;
    let writes:Vec<(&str,Value)>=match section {
        "api"=> {
            let base=text(values,"base",4000)?.trim();let model=text(values,"model",1000)?.trim();let key=text(values,"key",16000)?.trim();
            let mut connection=config["ppc.apiConfig"].as_str().and_then(|s|serde_json::from_str::<Value>(s).ok()).filter(Value::is_object).unwrap_or(json!({}));
            for (name,value) in values.as_object().unwrap() {
                if ["base","model","key"].contains(&name.as_str()) {continue;}
                if !["apiFormat","reasoningDialect","headers","extraBody"].contains(&name.as_str()) { return Err(invalid()); }
                connection[name]=value.clone();
            }
            vec![("ppc.apiBase",json!(base)),("ppc.modelName",json!(model)),("api_key",json!(key)),("ppc.apiConfig",json!(connection.to_string()))]
        }
        "model"=> {
            let mut model=config["ppc.modelSettings"].as_str().and_then(|s|serde_json::from_str::<Value>(s).ok()).filter(Value::is_object).unwrap_or(json!({}));
            for (key,value) in values.as_object().unwrap() {
                if !["temperature","topP","frequencyPenalty","presencePenalty","reasoningEffort","maxTokens","contextWindow","memoryRounds","stream","toolsEnabled","maxSteps","turnTimeout","disabledParams","extraBody"].contains(&key.as_str()) { return Err(invalid()); }
                model[key]=value.clone();
            }
            vec![("ppc.modelSettings",json!(model.to_string()))]
        }
        "user"=>vec![("ppc.userName",json!(text(values,"name",200)?)),("ppc.userPersonaText",json!(text(values,"persona",100_000)?))],
        _=>return Err(invalid()),
    };
    for (key,value) in &writes { config[*key]=value.clone(); }
    super::model_config::from_config(&config)?;
    for (key,value) in writes { identity::set_value_in(&tx,key,&value)?; }
    tx.commit()?;
    Ok(())
}

#[tauri::command]
pub fn local_save_settings(app:AppHandle,section:String,values:Value)->ChatResult<()> {
    let db=app.state::<DbState>();
    let mut conn=db_lock(&db)?;
    save_in(&mut conn,&section,&values)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_commit_together_and_keep_unedited_model_options() {
        let mut conn=super::super::tests::database();
        super::super::repository::migrate(&mut conn,&[],&json!({"pero":{"name":"佩洛"}})).unwrap();
        save_in(&mut conn,"api",&json!({"base":"https://example.com/v1","model":"模型","key":" 密钥 "})).unwrap();
        let previous=identity::snapshot_in(&conn).unwrap();
        assert_eq!(previous["api_key"],"密钥");
        assert!(save_in(&mut conn,"api",&json!({"base":"无效地址","model":"另一个模型","key":"另一个密钥"})).is_err());
        assert_eq!(identity::snapshot_in(&conn).unwrap(),previous);
        identity::set_value_in(&conn,"ppc.modelSettings",&json!(json!({"presencePenalty":0.3}).to_string())).unwrap();
        save_in(&mut conn,"model",&json!({"temperature":0.7,"topP":0.8,"memoryRounds":20,"stream":true})).unwrap();
        let config=identity::snapshot_in(&conn).unwrap();
        let settings=identity::model_settings(&config).unwrap();
        assert_eq!(settings.presence_penalty,Some(0.3));
        assert!(settings.stream);
        assert!(save_in(&mut conn,"model",&json!({"temperature":0.7,"topP":0.8,"memoryRounds":0,"stream":true})).is_err());
        assert_eq!(identity::snapshot_in(&conn).unwrap(),config);
    }
}
