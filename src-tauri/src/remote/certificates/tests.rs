use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct TestServer { endpoint:String, der:Vec<u8>, task:tokio::task::JoinHandle<()>, received:Arc<Mutex<Vec<Vec<u8>>>> }
impl Drop for TestServer {fn drop(&mut self) {self.task.abort();}}
async fn server(expired:bool)->TestServer {
    let key=rcgen::KeyPair::generate().unwrap();
    let mut params=rcgen::CertificateParams::new(vec!["localhost".into()]).unwrap();
    if expired {params.not_before=rcgen::date_time_ymd(2020,1,1);params.not_after=rcgen::date_time_ymd(2021,1,1);}
    let cert=params.self_signed(&key).unwrap();let der=cert.der().as_ref().to_vec();
    let tls=rustls::ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider())).with_safe_default_protocol_versions().unwrap().with_no_client_auth().with_single_cert(vec![cert.der().clone()],rustls::pki_types::PrivatePkcs8KeyDer::from(key.serialize_der()).into()).unwrap();
    let acceptor=tokio_rustls::TlsAcceptor::from(Arc::new(tls));
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let port=listener.local_addr().unwrap().port();
    let received=Arc::new(Mutex::new(Vec::new()));let requests=received.clone();
    let task=tokio::spawn(async move {
        loop {
            let Ok((tcp,_))=listener.accept().await else {return};let acceptor=acceptor.clone();let requests=requests.clone();
            tokio::spawn(async move {
                let Ok(mut stream)=acceptor.accept(tcp).await else {return};
                let mut input=vec![0u8;8192];let n=stream.read(&mut input).await.unwrap_or(0);input.truncate(n);requests.lock().unwrap().push(input.clone());
                if n==0 {return;}
                let body=r#"{"code":"OK","data":{"serverId":"test-server","displayName":"测试实例","appVersion":"1"}}"#;
                let output=format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
                let _=stream.write_all(output.as_bytes()).await;let _=stream.shutdown().await;
            });
        }
    });
    TestServer {endpoint:format!("https://127.0.0.1:{port}"),der,task,received}
}
#[tokio::test]
async fn self_signed_probe_sends_no_credentials_and_only_confirmed_pin_allows_api() {
    let server=server(false).await;
    assert_eq!(prepare(&server.endpoint,None,None).await.unwrap_err().code,"REMOTE_CERTIFICATE_UNTRUSTED");
    assert_eq!(prepare(&server.endpoint,None,Some("错误指纹")).await.unwrap_err().code,"REMOTE_CERTIFICATE_UNTRUSTED");
    let pin=prepare(&server.endpoint,None,Some(&fingerprint(&server.der))).await.unwrap().unwrap();
    // 探测阶段的三个连接均不能带有 HTTP 请求或任何应用层内容。
    tokio::time::timeout(std::time::Duration::from_secs(2),async {
        while server.received.lock().unwrap().len()<3 {tokio::task::yield_now().await;}
    }).await.unwrap();
    assert!(server.received.lock().unwrap().iter().all(Vec::is_empty));
    let client=super::super::transport::client_for(&server.endpoint,Some(&pin)).unwrap();
    let identity=super::super::transport::probe_with(&client,&server.endpoint,"test-secret").await.unwrap();
    assert_eq!(identity.server_id,"test-server");
    assert!(server.received.lock().unwrap().iter().any(|bytes|String::from_utf8_lossy(bytes).to_lowercase().contains("authorization: bearer test-secret")));
    assert!(prepare(&server.endpoint,Some(&pin),None).await.unwrap().is_some());
    assert!(tls_config("https://other.invalid",Some(&pin)).is_err());
    assert!(tls_config("http://127.0.0.1",Some(&pin)).is_err());
    assert!(super::super::transport::client_for("http://127.0.0.1",Some(&pin)).is_err());
    let failure=super::super::transport::probe_with(&super::super::transport::client_for(&server.endpoint,None).unwrap(),&server.endpoint,"not-trusted").await.unwrap_err();
    assert_eq!(failure.code,"REMOTE_CERTIFICATE_UNTRUSTED");
}
#[tokio::test]
async fn websocket_uses_the_same_confirmed_certificate_after_reload() {
    use futures::{SinkExt,StreamExt};
    use tokio_tungstenite::{Connector,connect_async_tls_with_config,tungstenite::Message};
    let key=rcgen::KeyPair::generate().unwrap();
    let cert=rcgen::CertificateParams::new(vec!["localhost".into()]).unwrap().self_signed(&key).unwrap();
    let tls=rustls::ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider())).with_safe_default_protocol_versions().unwrap().with_no_client_auth().with_single_cert(vec![cert.der().clone()],rustls::pki_types::PrivatePkcs8KeyDer::from(key.serialize_der()).into()).unwrap();
    let acceptor=tokio_rustls::TlsAcceptor::from(Arc::new(tls));
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let port=listener.local_addr().unwrap().port();
    let endpoint=format!("https://127.0.0.1:{port}");
    let pin=TrustedCertificate {origin:endpoint.clone(),fingerprint:fingerprint(cert.der()),der:cert.der().as_ref().to_vec()};
    let pin:TrustedCertificate=serde_json::from_str(&serde_json::to_string(&pin).unwrap()).unwrap();
    let task=tokio::spawn(async move {
        let (tcp,_)=listener.accept().await.unwrap();let tls=acceptor.accept(tcp).await.unwrap();
        let mut socket=tokio_tungstenite::accept_async(tls).await.unwrap();
        assert_eq!(socket.next().await.unwrap().unwrap().into_text().unwrap(),"前台与后台共用信任");
        socket.send(Message::Text("已接收".into())).await.unwrap();
    });
    let (mut socket,_)=connect_async_tls_with_config(endpoint.replace("https:","wss:"),None,false,Some(Connector::Rustls(tls_config(&endpoint,Some(&pin)).unwrap()))).await.unwrap();
    socket.send(Message::Text("前台与后台共用信任".into())).await.unwrap();
    assert_eq!(socket.next().await.unwrap().unwrap().into_text().unwrap(),"已接收");
    task.await.unwrap();
}
#[tokio::test]
async fn persisted_pin_blocks_rotation_for_http_and_websocket_and_requires_exact_confirmation() {
    use tokio_tungstenite::{Connector,connect_async_tls_with_config};
    let old=server(false).await;let next=server(false).await;
    let mut pin=TrustedCertificate {origin:address(&next.endpoint).unwrap().origin().ascii_serialization(),fingerprint:fingerprint(&old.der),der:old.der.clone()};
    let client=super::super::transport::client_for(&next.endpoint,Some(&pin)).unwrap();
    let error=super::super::transport::probe_with(&client,&next.endpoint,"secret-must-not-leak").await.unwrap_err();
    assert_eq!(error.code,"REMOTE_CERTIFICATE_CHANGED");assert_eq!(error.outcome.as_deref(),Some("not_sent"));
    let websocket=next.endpoint.replace("https:","wss:");
    let failure=connect_async_tls_with_config(websocket,None,false,Some(Connector::Rustls(tls_config(&next.endpoint,Some(&pin)).unwrap()))).await.unwrap_err();
    assert_eq!(error_from_ws(&failure).code,"REMOTE_CERTIFICATE_CHANGED");
    assert_eq!(prepare(&next.endpoint,Some(&pin),None).await.unwrap_err().code,"REMOTE_CERTIFICATE_CHANGED");
    pin=prepare(&next.endpoint,Some(&pin),Some(&fingerprint(&next.der))).await.unwrap().unwrap();
    let restored:TrustedCertificate=serde_json::from_str(&serde_json::to_string(&pin).unwrap()).unwrap();
    let client=super::super::transport::client_for(&next.endpoint,Some(&restored)).unwrap();
    assert_eq!(super::super::transport::probe_with(&client,&next.endpoint,"new-secret").await.unwrap().server_id,"test-server");
    assert!(!next.received.lock().unwrap().iter().any(|bytes|String::from_utf8_lossy(bytes).contains("secret-must-not-leak")));
}
fn error_from_ws(failure:&tokio_tungstenite::tungstenite::Error)->ChatError {error(failure).unwrap()}
#[tokio::test]
async fn expired_certificates_cannot_be_confirmed_and_http_remains_available() {
    let server=server(true).await;
    assert_eq!(prepare(&server.endpoint,None,Some(&fingerprint(&server.der))).await.unwrap_err().code,"REMOTE_CERTIFICATE_EXPIRED");
    assert!(prepare("http://192.168.1.2:9122",None,None).await.unwrap().is_none());
    assert!(super::super::profiles::normalize_endpoint("http://192.168.1.2:9122",true).is_ok());
    assert!(super::super::profiles::normalize_endpoint("http://192.168.1.2:9122",false).is_err());
}
#[test]
fn public_ca_renewal_is_automatic_unless_connection_was_explicitly_pinned() {
    let pin=TrustedCertificate {origin:"https://example.com".into(),fingerprint:"new".into(),der:vec![]};
    assert!(!needs_confirmation(&pin,true,None));
    assert!(needs_confirmation(&pin,false,None));
    let mut old=pin.clone();old.fingerprint="old".into();
    assert!(needs_confirmation(&pin,true,Some(&old)));
}
