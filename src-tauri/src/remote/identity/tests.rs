use super::*;
use serde_json::{json,Value};
use std::sync::{Arc,Mutex};
use tokio::io::{AsyncReadExt,AsyncWriteExt};

struct Server {endpoint:String,requests:Arc<Mutex<Vec<String>>>,task:tokio::task::JoinHandle<()>}
impl Drop for Server {fn drop(&mut self) {self.task.abort();}}
async fn server(responses:Vec<(&'static str,u16,Value)>)->Server {
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint=format!("http://{}/infos",listener.local_addr().unwrap());
    let requests=Arc::new(Mutex::new(Vec::new()));let received=requests.clone();
    let task=tokio::spawn(async move {
        for (path,status,data) in responses {
            let (mut socket,_)=listener.accept().await.unwrap();let mut bytes=Vec::new();
            while !bytes.ends_with(b"\r\n\r\n") {bytes.push(socket.read_u8().await.unwrap());assert!(bytes.len()<8192);}
            let request=String::from_utf8(bytes).unwrap().to_lowercase();
            assert!(request.starts_with(&format!("get /infos/api/{path} ")));
            assert!(request.contains("authorization: bearer original-device-secret"));
            received.lock().unwrap().push(request);
            let body=json!({"code":"OK","data":data}).to_string();
            socket.write_all(format!("HTTP/1.1 {status} Result\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
        }
    });
    Server {endpoint,requests,task}
}
fn identity(id:&str,version:&str)->Value {json!({"serverId":id,"displayName":"我的 infOS","appVersion":version})}
fn device_status(id:&str)->Value {json!({"authenticated":true,"admin":false,"deviceId":id})}
fn profile()->Profile {
    Profile {id:uuid::Uuid::new_v4().to_string(),server_id:"infos-node-before".into(),endpoint:String::new(),display_name:"原来的 infOS".into(),app_version:"1".into(),
        credential_id:Some(uuid::Uuid::new_v4().to_string()),device:Some(access::Device {id:uuid::Uuid::new_v4().to_string(),expires_at:chrono::Utc::now().timestamp_millis()+86_400_000,admin:false}),session_owner:None,certificate:None}
}
#[test]
fn connection_verification_requires_same_credentials_profile_and_runtime() {
    let original = profile();
    let token = Token { installation_id: "测试安装".into(), epoch: 1,
        target: mode::Target::Remote { profile_id: original.id.clone() } };
    let now = std::time::Instant::now();
    let verified = super::super::VerifiedConnection {
        token: token.clone(), profile: original.clone(),
        client: transport::client().unwrap(), checked_at: now,
    };
    assert!(verified.reusable(&token, &original, now));
    let mut switched = token.clone(); switched.epoch += 1;
    assert!(!verified.reusable(&switched, &original, now));
    let mut changed = original.clone(); changed.credential_id = Some(uuid::Uuid::new_v4().to_string());
    assert!(!verified.reusable(&token, &changed, now));
    changed = original.clone(); changed.endpoint = "https://other.invalid".into();
    assert!(!verified.reusable(&token, &changed, now));
    assert!(!verified.reusable(&token, &original, now + std::time::Duration::from_secs(10)));
}
fn fixture(profile:&Profile)->(mode::Coordinator,Token) {
    let root=std::env::temp_dir().join(format!("ppc-identity-{}",uuid::Uuid::new_v4())).join("data");
    let coordinator=mode::Coordinator::initialize(root.join("control/runtime.sqlite")).unwrap();
    save(&coordinator,profile);
    coordinator.begin_switch(mode::Target::Remote {profile_id:profile.id.clone()}).unwrap();coordinator.activate().unwrap();
    let token=coordinator.token().unwrap();(coordinator,token)
}
fn save(coordinator:&mode::Coordinator,profile:&Profile) {
    mode::open_existing(coordinator.path()).unwrap().execute("INSERT INTO remote_profiles VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET value=excluded.value",rusqlite::params![profile.id,serde_json::to_string(profile).unwrap()]).unwrap();
}

#[tokio::test]
async fn docker_recreation_keeps_profile_credentials_drafts_and_request_history() {
    let mut old=profile();let device=&old.device.as_ref().unwrap().id;
    let server=server(vec![
        ("distributed/identity",200,identity("infos-node-after","2")),
        ("client-access/status",200,device_status(device)),
        ("distributed/identity",200,identity("infos-node-after","2")),
        ("distributed/identity",200,identity("infos-node-after","2")),
    ]).await;
    old.endpoint=server.endpoint.clone();let (coordinator,token)=fixture(&old);
    let root=coordinator.path().parent().unwrap().parent().unwrap();
    let cache=super::super::cache::connection_at(root,&token).unwrap();
    cache.execute("INSERT INTO preferences VALUES('draft','尚未发送的消息')",[]).unwrap();
    cache.execute("INSERT INTO requests(id,fingerprint,state,created) VALUES('original-request','fingerprint','unknown',1)",[]).unwrap();
    let verified=verify(&transport::client().unwrap(),&old,"original-device-secret").await.unwrap();
    assert!(verified.recovered());
    let updated=persist(coordinator.path(),&token,&verified).unwrap();
    assert_eq!(persist(coordinator.path(),&token,&verified).unwrap(),updated);
    assert_eq!(updated.id,old.id);assert_eq!(updated.credential_id,old.credential_id);assert_eq!(updated.device,old.device);
    assert_eq!(updated.server_id,"infos-node-after");assert_eq!(updated.app_version,"2");
    assert_eq!(coordinator.token().unwrap(),token);
    assert_eq!(cache.query_row("SELECT value FROM preferences WHERE key='draft'",[],|row|row.get::<_,String>(0)).unwrap(),"尚未发送的消息");
    assert_eq!(cache.query_row("SELECT state FROM requests WHERE id='original-request'",[],|row|row.get::<_,String>(0)).unwrap(),"unknown");
    let reloaded=profiles::get(&coordinator,&old.id).unwrap();
    assert!(!verify(&transport::client().unwrap(),&reloaded,"original-device-secret").await.unwrap().recovered());
    assert_eq!(server.requests.lock().unwrap().len(),4);
}

#[tokio::test]
async fn version_only_update_does_not_require_device_reauthorization() {
    let mut old=profile();old.device=None;
    let server=server(vec![("distributed/identity",200,identity(&old.server_id,"2"))]).await;
    old.endpoint=server.endpoint.clone();let (coordinator,token)=fixture(&old);
    let verified=verify(&transport::client().unwrap(),&old,"original-device-secret").await.unwrap();
    assert!(!verified.recovered());
    let saved=persist(coordinator.path(),&token,&verified).unwrap();
    assert_eq!(saved.server_id,old.server_id);assert_eq!(saved.app_version,"2");assert_eq!(server.requests.lock().unwrap().len(),1);
}

#[tokio::test]
async fn replaced_instance_and_revoked_or_expired_authorization_cannot_reuse_partition() {
    for (status,expected) in [
        (device_status(&uuid::Uuid::new_v4().to_string()),"REMOTE_IDENTITY_CHANGED"),
        (device_status("local"),"REMOTE_IDENTITY_CHANGED"),
        (json!({"authenticated":true,"admin":true}),"REMOTE_IDENTITY_CHANGED"),
        (json!({"authenticated":false,"admin":false}),"REMOTE_AUTH_REQUIRED"),
    ] {
        let mut old=profile();
        let server=server(vec![("distributed/identity",200,identity("other-server","2")),("client-access/status",200,status)]).await;
        old.endpoint=server.endpoint.clone();let (coordinator,_token)=fixture(&old);
        assert_eq!(verify(&transport::client().unwrap(),&old,"original-device-secret").await.err().unwrap().code,expected);
        assert_eq!(profiles::get(&coordinator,&old.id).unwrap(),old);assert_eq!(server.requests.lock().unwrap().len(),2);
    }
    for expired in [false,true] {
        let mut old=profile();
        if expired {old.device.as_mut().unwrap().expires_at=1;} else {old.device=None;}
        let server=server(vec![("distributed/identity",200,identity("other-server","2"))]).await;
        old.endpoint=server.endpoint.clone();
        assert_eq!(verify(&transport::client().unwrap(),&old,"original-device-secret").await.err().unwrap().code,if expired {"REMOTE_AUTH_REQUIRED"} else {"REMOTE_IDENTITY_CHANGED"});
        assert_eq!(server.requests.lock().unwrap().len(),1);
    }
}

#[tokio::test]
async fn restart_during_authorization_check_is_retryable_and_never_persisted() {
    let mut old=profile();
    let server=server(vec![
        ("distributed/identity",200,identity("container-b","2")),
        ("client-access/status",200,device_status(&old.device.as_ref().unwrap().id)),
        ("distributed/identity",200,identity("container-c","2")),
    ]).await;
    old.endpoint=server.endpoint.clone();let (coordinator,_token)=fixture(&old);
    assert_eq!(verify(&transport::client().unwrap(),&old,"original-device-secret").await.err().unwrap().code,"REMOTE_NETWORK");
    assert_eq!(profiles::get(&coordinator,&old.id).unwrap(),old);
}

#[tokio::test]
async fn late_recovery_cannot_overwrite_new_credentials_certificate_endpoint_or_mode() {
    let mut old=profile();
    let server=server(vec![
        ("distributed/identity",200,identity("container-b","2")),
        ("client-access/status",200,device_status(&old.device.as_ref().unwrap().id)),
        ("distributed/identity",200,identity("container-b","2")),
    ]).await;
    old.endpoint=server.endpoint.clone();let (coordinator,token)=fixture(&old);
    let verified=verify(&transport::client().unwrap(),&old,"original-device-secret").await.unwrap();
    for change in 0..6 {
        let mut current=old.clone();
        match change {
            0=>current.credential_id=Some(uuid::Uuid::new_v4().to_string()),
            1=>current.endpoint="http://other.invalid".into(),
            2=>current.server_id="container-c".into(),
            3=>current.device.as_mut().unwrap().id=uuid::Uuid::new_v4().to_string(),
            4=>current.session_owner=Some(uuid::Uuid::new_v4().to_string()),
            _=>current.certificate=Some(super::super::certificates::TrustedCertificate {origin:"https://example.invalid".into(),fingerprint:"已更换".into(),der:vec![1]}),
        }
        save(&coordinator,&current);
        assert_eq!(persist(coordinator.path(),&token,&verified).unwrap_err().code,"REMOTE_CONFLICT");
        assert_eq!(profiles::get(&coordinator,&old.id).unwrap(),current);
    }
    save(&coordinator,&old);
    coordinator.begin_switch(mode::Target::Standalone).unwrap();coordinator.activate().unwrap();
    assert_eq!(persist(coordinator.path(),&token,&verified).unwrap_err().code,"MODE_INACTIVE");
    assert_eq!(profiles::get(&coordinator,&old.id).unwrap(),old);
}
