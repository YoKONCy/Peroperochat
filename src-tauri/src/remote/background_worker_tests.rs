use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
use super::*;
use tokio::io::{AsyncReadExt,AsyncWriteExt};

#[tokio::test]
async fn native_receiver_replays_notifications_without_claiming_input_or_read_ack() {
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();
    let root=std::env::temp_dir().join(format!("ppc-notifier-wire-{}",uuid::Uuid::new_v4())).join("data");
    let mode=crate::mode::Coordinator::initialize(root.join("control/runtime.sqlite")).unwrap();let profile=uuid::Uuid::new_v4().to_string();
    crate::mode::open_existing(mode.path()).unwrap().execute("INSERT INTO remote_profiles VALUES(?1,?2)",rusqlite::params![profile,json!({"id":profile,"serverId":"server","endpoint":format!("http://{address}/infos"),"displayName":"测试","appVersion":"1"}).to_string()]).unwrap();
    mode.begin_switch(crate::mode::Target::Remote {profile_id:profile}).unwrap();mode.activate().unwrap();let token=mode.token().unwrap();
    store::execute(&root,"configure",&json!({"modeToken":token,"enabled":true})).unwrap();
    store::connection(&root,&token).unwrap().execute("INSERT INTO preferences VALUES('principal','custom-role')",[]).unwrap();
    let server=tokio::spawn(async move {
        let (mut http,_)=listener.accept().await.unwrap();let mut bytes=Vec::new();
        loop {let byte=http.read_u8().await.unwrap();bytes.push(byte);assert!(bytes.len()<4096);if bytes.ends_with(b"\r\n\r\n") {break;}}
        let request=String::from_utf8(bytes).unwrap().to_lowercase();assert!(request.starts_with("get /infos/api/distributed/identity "));assert!(request.contains("authorization: bearer test-secret"));
        let body=json!({"code":"OK","data":{"serverId":"server","displayName":"测试","appVersion":"1"}}).to_string();
        http.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();drop(http);
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
}
