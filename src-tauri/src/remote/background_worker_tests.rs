use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
use super::*;
use tokio::io::{AsyncReadExt,AsyncWriteExt};

#[tokio::test]
async fn native_receiver_replays_notifications_without_claiming_input_or_read_ack() {
    receiver_reconnects(false).await;
}
#[tokio::test]
async fn native_receiver_recovers_after_docker_recreation_with_original_device() {
    receiver_reconnects(true).await;
}
async fn receiver_reconnects(recreated:bool) {
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();
    let root=std::env::temp_dir().join(format!("ppc-notifier-wire-{}",uuid::Uuid::new_v4())).join("data");
    let mode=crate::mode::Coordinator::initialize(root.join("control/runtime.sqlite")).unwrap();let profile=uuid::Uuid::new_v4().to_string();
    let device_id=uuid::Uuid::new_v4().to_string();
    let device=if recreated {json!({"id":device_id,"expiresAt":chrono::Utc::now().timestamp_millis()+86_400_000,"admin":false})} else {Value::Null};
    crate::mode::open_existing(mode.path()).unwrap().execute("INSERT INTO remote_profiles VALUES(?1,?2)",rusqlite::params![profile,json!({"id":profile,"serverId":"server","endpoint":format!("http://{address}/infos"),"displayName":"测试","appVersion":"1","device":device}).to_string()]).unwrap();
    mode.begin_switch(crate::mode::Target::Remote {profile_id:profile.clone()}).unwrap();mode.activate().unwrap();let token=mode.token().unwrap();
    store::execute(&root,"configure",&json!({"modeToken":token,"enabled":true})).unwrap();
    store::connection(&root,&token).unwrap().execute("INSERT INTO preferences VALUES('principal','custom-role')",[]).unwrap();
    let expected_server=if recreated {"server-after-recreation"} else {"server"};
    let server=tokio::spawn(async move {
        let (mut http,_)=listener.accept().await.unwrap();let mut bytes=Vec::new();
        loop {let byte=http.read_u8().await.unwrap();bytes.push(byte);assert!(bytes.len()<4096);if bytes.ends_with(b"\r\n\r\n") {break;}}
        let request=String::from_utf8(bytes).unwrap().to_lowercase();assert!(request.starts_with("get /infos/api/distributed/identity "));assert!(request.contains("authorization: bearer test-secret"));
        let identity=json!({"code":"OK","data":{"serverId":expected_server,"displayName":"测试","appVersion":"1"}});
        let body=identity.to_string();
        http.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();drop(http);
        if recreated {
            for (path,body) in [("client-access/status",json!({"code":"OK","data":{"authenticated":true,"admin":false,"deviceId":device_id}})),("distributed/identity",identity)] {
                let (mut http,_)=listener.accept().await.unwrap();let mut bytes=Vec::new();
                while !bytes.ends_with(b"\r\n\r\n") {bytes.push(http.read_u8().await.unwrap());assert!(bytes.len()<4096);}
                let request=String::from_utf8(bytes).unwrap().to_lowercase();assert!(request.starts_with(&format!("get /infos/api/{path} ")));assert!(request.contains("authorization: bearer test-secret"));
                let body=body.to_string();http.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
            }
        }
        let (tcp,_)=listener.accept().await.unwrap();
        let mut socket=tokio_tungstenite::accept_hdr_async(tcp,|request:&tokio_tungstenite::tungstenite::handshake::server::Request,response| {
            assert_eq!(request.uri().path(),"/infos/ws/gateway");assert_eq!(request.headers()["Authorization"],"Bearer test-secret");Ok(response)
        }).await.unwrap();
        let hello=socket.next().await.unwrap().unwrap();let hello:Value=serde_json::from_str(hello.to_text().unwrap()).unwrap();
        assert_eq!(hello["type"],"hello");assert_eq!(hello["payload"]["principalId"],"custom-role");assert_eq!(hello["payload"]["platform"]["runtime"],"native");assert!(hello["payload"]["nodeId"].as_str().unwrap().starts_with("ppc-notifier-"));
        socket.send(Message::Text(json!({"protocolVersion":1,"type":"hello_ack","payload":{"agreedVersion":1,"features":["audience-v1"]}}).to_string().into())).await.unwrap();
        let push=json!({"protocolVersion":1,"type":"push","id":"e","payload":{"action":"durable_notification","principalId":"custom-role","notificationId":"notice","revision":1,"body":"离线补收","replayed":true}});
        for _ in 0..2 {socket.send(Message::Text(push.to_string().into())).await.unwrap();}
        while let Some(message)=socket.next().await {
            let Ok(message)=message else {break};if !message.is_text() {continue;}
            let value:Value=serde_json::from_str(message.to_text().unwrap()).unwrap();assert_eq!(value["type"],"heartbeat","接收器不能申请输入席位或发送已读回执");
        }
    });
    let delivered=Arc::new(AtomicBool::new(false));let cancel=delivered.clone();let mut count=0;
    let result=tokio::time::timeout(Duration::from_secs(12),run(&root,&token,"test-secret",&mut ||cancel.load(Ordering::SeqCst),&mut |notice| {
        assert_eq!(notice["principalId"],"custom-role");assert_eq!(notice["modeToken"],json!(token));count+=1;delivered.store(true,Ordering::SeqCst);Ok(true)
    })).await.unwrap();
    assert_eq!(result.unwrap_err().code,"MODE_INACTIVE");assert_eq!(count,1);server.await.unwrap();
    assert_eq!(super::super::profiles::get(&mode,&profile).unwrap().server_id,expected_server);
}
