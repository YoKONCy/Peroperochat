use std::time::Duration;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use crate::db::DbState;
use super::{db_lock, identity, types::*};

const CACHE_MS:i64 = 30*60*1000;

pub fn projection(config:&Value)->Value {
    json!({"location":config["ppc.location"].as_str().unwrap_or(""),
        "weather":config["ppc.weather"].as_str().unwrap_or(""),
        "fetchedAt":timestamp(config)})
}

fn timestamp(config:&Value)->i64 { config["ppc.env.last_fetch"].as_str().and_then(|s|s.parse().ok()).unwrap_or(0) }

fn fresh(config:&Value, now:i64)->bool {
    let age=now.saturating_sub(timestamp(config));
    timestamp(config)>0 && (0..CACHE_MS).contains(&age)
        && !config["ppc.location"].as_str().unwrap_or("").is_empty()
        && !config["ppc.weather"].as_str().unwrap_or("").is_empty()
}

fn location(value:&Value)->ChatResult<(String,f64,f64)> {
    let city=value["city"].as_str().filter(|s| !s.trim().is_empty()).ok_or_else(||ChatError::new("ENVIRONMENT_ERROR","地理位置响应缺少城市"))?;
    let lat=value["latitude"].as_f64().filter(|v|v.is_finite() && (-90.0..=90.0).contains(v));
    let lon=value["longitude"].as_f64().filter(|v|v.is_finite() && (-180.0..=180.0).contains(v));
    match (lat,lon) { (Some(lat),Some(lon)) if value["success"]!=false && value["error"]!=true => Ok((city.into(),lat,lon)),
        _ => Err(ChatError::new("ENVIRONMENT_ERROR","地理位置坐标无效")) }
}

fn weather(value:&Value)->ChatResult<String> {
    let current=&value["current_weather"];
    let temp=current["temperature"].as_f64().filter(|v|v.is_finite()).ok_or_else(||ChatError::new("ENVIRONMENT_ERROR","天气响应缺少温度"))?;
    let code=current["weathercode"].as_u64().ok_or_else(||ChatError::new("ENVIRONMENT_ERROR","天气响应缺少天气代码"))?;
    let desc=match code {
        0=>"晴朗",1=>"大部晴朗",2=>"多云",3=>"阴天",45|48=>"雾",51|53|55=>"毛毛雨",56|57=>"冻毛毛雨",
        61=>"小雨",63=>"中雨",65=>"大雨",66|67=>"冻雨",71=>"小雪",73=>"中雪",75=>"大雪",77=>"雪粒",
        80..=82=>"阵雨",85|86=>"阵雪",95|96|99=>"雷阵雨",_=>"天气未知",
    };
    Ok(format!("{desc} {}°C",temp.round()))
}

async fn get(client:&reqwest::Client, url:&str)->ChatResult<Value> {
    client.get(url).send().await.and_then(reqwest::Response::error_for_status)
        .map_err(|_|ChatError::new("ENVIRONMENT_ERROR","环境信息服务暂时不可用"))?
        .json().await.map_err(|_|ChatError::new("ENVIRONMENT_ERROR","环境信息响应格式无效"))
}

async fn refresh(app:&AppHandle, client:&reqwest::Client)->ChatResult<()> {
    let config=identity::snapshot(app)?;
    if fresh(&config,chrono::Utc::now().timestamp_millis()) { return Ok(()); }
    let first=get(client,"https://ipapi.co/json/").await.and_then(|v|location(&v));
    let (city,lat,lon)=match first {
        Ok(value)=>value,
        Err(_)=>location(&get(client,"https://ipwho.is/").await?)?,
    };
    let value=get(client,&format!("https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current_weather=true&timezone=auto")).await?;
    let weather=weather(&value)?;
    let result={
        let db=app.state::<DbState>();
        let mut conn=db_lock(&db)?;
        let tx=conn.transaction()?;
        let current=identity::snapshot_in(&tx)?;
        // 请求期间发生备份恢复或缓存更新时，丢弃旧请求，避免覆盖新环境。
        if projection(&current)!=projection(&config) { return Ok(()); }
        identity::set_value_in(&tx,"ppc.location",&json!(city))?;
        identity::set_value_in(&tx,"ppc.weather",&json!(weather))?;
        identity::set_value_in(&tx,"ppc.env.last_fetch",&json!(chrono::Utc::now().timestamp_millis().to_string()))?;
        let result=projection(&identity::snapshot_in(&tx)?);
        tx.commit()?;
        result
    };
    let _=app.emit("local-environment-event",result);
    Ok(())
}

// 应用只启动一个刷新循环；网络访问、失败重试和缓存寿命不依赖页面挂载。
pub fn start(app:AppHandle) {
    tauri::async_runtime::spawn(async move {
        let client=match reqwest::Client::builder().timeout(Duration::from_secs(15)).connect_timeout(Duration::from_secs(8)).build() {
            Ok(client)=>client,Err(_)=> { eprintln!("环境信息客户端初始化失败"); return; }
        };
        loop {
            if let Err(error)=refresh(&app,&client).await { eprintln!("环境信息刷新失败，保留已有缓存：{}",error.message); }
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cache_requires_complete_data_and_valid_clock() {
        let mut config=json!({"ppc.env.last_fetch":"1000","ppc.location":"北京","ppc.weather":"晴朗 20°C"});
        assert!(fresh(&config,1001));
        assert!(!fresh(&config,999));
        assert!(!fresh(&config,1000+CACHE_MS));
        config["ppc.weather"]=json!("");
        assert!(!fresh(&config,1001));
    }
    #[test]
    fn environment_responses_are_validated_before_cache_write() {
        assert!(location(&json!({"city":"北京","latitude":91,"longitude":116})).is_err());
        assert!(location(&json!({"success":false,"city":"北京","latitude":39,"longitude":116})).is_err());
        assert_eq!(location(&json!({"city":"北京","latitude":39,"longitude":116})).unwrap().0,"北京");
        assert_eq!(weather(&json!({"current_weather":{"temperature":-1.6,"weathercode":71}})).unwrap(),"小雪 -2°C");
        assert!(weather(&json!({"current_weather":{"weathercode":0}})).is_err());
        assert_eq!(weather(&json!({"current_weather":{"temperature":20,"weathercode":123}})).unwrap(),"天气未知 20°C");
    }
}
