//! 仅为用户确认的连接保存叶证书指纹；探测只做 TLS 握手，不发送配对码或令牌。
use std::sync::{Arc, Mutex, OnceLock};
use rustls::{client::{danger::{ServerCertVerifier, ServerCertVerified, HandshakeSignatureValid}, WebPkiServerVerifier}, pki_types::{CertificateDer, ServerName, UnixTime}, DigitallySignedStruct, SignatureScheme};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};
use crate::{local::types::*, mode::{Coordinator, Token}};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct TrustedCertificate { pub origin:String, pub fingerprint:String, pub der:Vec<u8> }
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all="camelCase")]
pub struct CertificateInfo { pub fingerprint:String, pub subject:String, pub issuer:String, pub valid_from:i64, pub valid_to:i64 }
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Inspection { pub origin:String, pub certificate:Option<CertificateInfo>, pub previous_fingerprint:Option<String>, pub requires_confirmation:bool }

pub fn is_certificate_error(code:&str)->bool { code.starts_with("REMOTE_CERTIFICATE_") }
fn invalid()->ChatError { ChatError::new("REMOTE_CERTIFICATE_INVALID","无法读取此服务器的证书，请检查服务配置") }
fn changed()->ChatError { ChatError::new("REMOTE_CERTIFICATE_CHANGED","服务器证书有变化，请到连接选项中核对") }
fn untrusted()->ChatError { ChatError::new("REMOTE_CERTIFICATE_UNTRUSTED","请先在连接选项中核对服务器证书") }
fn expired()->ChatError { ChatError::new("REMOTE_CERTIFICATE_EXPIRED","服务器证书已过期或尚未生效，请检查证书和手机时间") }
pub fn error(error:&(dyn std::error::Error + 'static))->Option<ChatError> {
    let mut current=Some(error);
    for _ in 0..12 {
        let Some(value)=current else {break};
        let text=value.to_string();
        if text.contains("PPC_CERTIFICATE_CHANGED") {return Some(changed());}
        if text.contains("PPC_CERTIFICATE_UNTRUSTED") {return Some(untrusted());}
        if text.contains("PPC_CERTIFICATE_EXPIRED") {return Some(expired());}
        if text.contains("PPC_CERTIFICATE_INVALID") {return Some(invalid());}
        if let Some(rustls::Error::InvalidCertificate(reason))=value.downcast_ref::<rustls::Error>() {
            return Some(if matches!(reason,rustls::CertificateError::Expired|rustls::CertificateError::NotValidYet|rustls::CertificateError::ExpiredContext {..}|rustls::CertificateError::NotValidYetContext {..}) {expired()} else {untrusted()});
        }
        current=value.source();
    }
    None
}
fn fingerprint(der:&[u8])->String { Sha256::digest(der).iter().map(|byte|format!("{byte:02X}")).collect::<Vec<_>>().join(":") }
fn info(der:&[u8],now:u64)->ChatResult<CertificateInfo> {
    if der.len()>64*1024 {return Err(invalid());}
    let (rest,cert)=x509_parser::parse_x509_certificate(der).map_err(|_|invalid())?;
    if !rest.is_empty() {return Err(invalid());}
    let start=cert.validity().not_before.timestamp();let end=cert.validity().not_after.timestamp();
    if (now as i64)<start || (now as i64)>end {return Err(expired());}
    Ok(CertificateInfo {fingerprint:fingerprint(der),subject:cert.subject().to_string(),issuer:cert.issuer().to_string(),valid_from:start*1000,valid_to:end*1000})
}
fn roots()->Arc<WebPkiServerVerifier> {
    static ROOTS:OnceLock<Arc<WebPkiServerVerifier>>=OnceLock::new();
    ROOTS.get_or_init(|| {
        let roots=rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        WebPkiServerVerifier::builder_with_provider(Arc::new(roots),Arc::new(rustls::crypto::ring::default_provider())).build().expect("内置根证书集合不能为空")
    }).clone()
}
fn config(verifier:Arc<dyn ServerCertVerifier>)->Arc<rustls::ClientConfig> {
    let mut config=rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider())).with_safe_default_protocol_versions().expect("TLS 协议配置有效").dangerous().with_custom_certificate_verifier(verifier).with_no_client_auth();
    // 重连必须重新核对实际证书，不能让会话恢复跳过指纹检查。
    config.resumption=rustls::client::Resumption::disabled();
    Arc::new(config)
}
#[derive(Debug)]
struct Verifier { standard:Arc<WebPkiServerVerifier>, pin:Option<TrustedCertificate>, host:String, capture:Option<Arc<Mutex<Option<(Vec<u8>,bool)>>>> }
impl ServerCertVerifier for Verifier {
    fn verify_server_cert(&self,cert:&CertificateDer<'_>,chain:&[CertificateDer<'_>],name:&ServerName<'_>,ocsp:&[u8],now:UnixTime)->Result<ServerCertVerified,rustls::Error> {
        if let Some(capture)=&self.capture {
            let standard=self.standard.verify_server_cert(cert,chain,name,ocsp,now);
            info(cert.as_ref(),now.as_secs()).map_err(|error|rustls::Error::General(if error.code=="REMOTE_CERTIFICATE_EXPIRED" {"PPC_CERTIFICATE_EXPIRED"} else {"PPC_CERTIFICATE_INVALID"}.into()))?;
            *capture.lock().map_err(|_|rustls::Error::General("PPC_CERTIFICATE_INVALID".into()))?=Some((cert.as_ref().to_vec(),standard.is_ok()));
            return Ok(ServerCertVerified::assertion());
        }
        if let Some(pin)=&self.pin {
            if name.to_str()!=self.host || fingerprint(cert.as_ref())!=pin.fingerprint {return Err(rustls::Error::General("PPC_CERTIFICATE_CHANGED".into()));}
            info(cert.as_ref(),now.as_secs()).map_err(|_|rustls::Error::General("PPC_CERTIFICATE_EXPIRED".into()))?;
            return Ok(ServerCertVerified::assertion());
        }
        self.standard.verify_server_cert(cert,chain,name,ocsp,now).map_err(|error| {
            // HTTP 连接器可能擦除底层错误类型，在验证边界保留稳定的错误标记。
            match error {
                rustls::Error::InvalidCertificate(reason)=>rustls::Error::General(if matches!(reason,rustls::CertificateError::Expired|rustls::CertificateError::NotValidYet|rustls::CertificateError::ExpiredContext {..}|rustls::CertificateError::NotValidYetContext {..}) {"PPC_CERTIFICATE_EXPIRED"} else {"PPC_CERTIFICATE_UNTRUSTED"}.into()),
                other=>other,
            }
        })
    }
    // 即使用户确认了自签证书，也必须验证握手签名，证明对端持有对应私钥。
    fn verify_tls12_signature(&self,message:&[u8],cert:&CertificateDer<'_>,signature:&DigitallySignedStruct)->Result<HandshakeSignatureValid,rustls::Error> {self.standard.verify_tls12_signature(message,cert,signature)}
    fn verify_tls13_signature(&self,message:&[u8],cert:&CertificateDer<'_>,signature:&DigitallySignedStruct)->Result<HandshakeSignatureValid,rustls::Error> {self.standard.verify_tls13_signature(message,cert,signature)}
    fn supported_verify_schemes(&self)->Vec<SignatureScheme> {self.standard.supported_verify_schemes()}
}
fn address(endpoint:&str)->ChatResult<reqwest::Url> {reqwest::Url::parse(endpoint).map_err(|_|invalid())}
fn host(url:&reqwest::Url)->ChatResult<String> {Ok(url.host_str().ok_or_else(invalid)?.trim_matches(['[',']']).to_owned())}
pub fn tls_config(endpoint:&str,pin:Option<&TrustedCertificate>)->ChatResult<Arc<rustls::ClientConfig>> {
    let url=address(endpoint)?;
    if let Some(pin)=pin {
        if url.scheme()!="https" || url.origin().ascii_serialization()!=pin.origin || fingerprint(&pin.der)!=pin.fingerprint {return Err(invalid());}
        info(&pin.der,UnixTime::now().as_secs())?;
    }
    Ok(config(Arc::new(Verifier {standard:roots(),pin:pin.cloned(),host:host(&url)?,capture:None})))
}
async fn inspect(endpoint:&str)->ChatResult<Option<(TrustedCertificate,CertificateInfo,bool)>> {
    let url=address(endpoint)?;
    if url.scheme()=="http" {return Ok(None);}
    if url.scheme()!="https" {return Err(invalid());}
    let hostname=host(&url)?;
    let capture=Arc::new(Mutex::new(None));
    let config=config(Arc::new(Verifier {standard:roots(),pin:None,host:hostname.clone(),capture:Some(capture.clone())}));
    let name=ServerName::try_from(hostname.clone()).map_err(|_|invalid())?;
    tokio::time::timeout(std::time::Duration::from_secs(10),async {
        let tcp=tokio::net::TcpStream::connect((hostname.as_str(),url.port_or_known_default().unwrap_or(443))).await.map_err(|_|super::transport::network())?;
        // 连接结束即丢弃，不写入任何应用层内容。
        let _stream=tokio_rustls::TlsConnector::from(config).connect(name,tcp).await.map_err(|failure|error(&failure).unwrap_or_else(super::transport::network))?;
        Ok::<_,ChatError>(())
    }).await.map_err(|_|super::transport::network())??;
    let (der,authorized)=capture.lock().map_err(|_|invalid())?.take().ok_or_else(invalid)?;
    let info=info(&der,UnixTime::now().as_secs())?;
    Ok(Some((TrustedCertificate {origin:url.origin().ascii_serialization(),fingerprint:info.fingerprint.clone(),der},info,authorized)))
}
fn needs_confirmation(next:&TrustedCertificate,authorized:bool,known:Option<&TrustedCertificate>)->bool {known.map_or(!authorized,|old|old.origin!=next.origin || old.fingerprint!=next.fingerprint)}
pub async fn prepare(endpoint:&str,known:Option<&TrustedCertificate>,accepted:Option<&str>)->ChatResult<Option<TrustedCertificate>> {
    let Some((next,_,authorized))=inspect(endpoint).await? else {return Ok(None)};
    if needs_confirmation(&next,authorized,known) && accepted!=Some(next.fingerprint.as_str()) {return Err(if known.is_some() {changed()} else {untrusted()});}
    // 公共 CA 正常验证的首次连接无须额外固定叶证书，允许正常续期。
    Ok(if known.is_some() || !authorized {Some(next)} else {None})
}
#[tauri::command]
pub async fn remote_inspect_certificate(app:AppHandle,endpoint:String,profile_id:Option<String>,allow_insecure:bool,mode_token:Token)->ChatResult<Inspection> {
    let mode=app.state::<Coordinator>();{let _gate=mode.gate(Some(&mode_token),false)?;}
    let endpoint=super::profiles::normalize_endpoint(&endpoint,allow_insecure)?;
    let previous=profile_id.as_deref().map(|id|super::profiles::get(&mode,id)).transpose()?;
    if previous.as_ref().is_some_and(|profile|profile.endpoint!=endpoint) {return Err(invalid());}
    let inspected=inspect(&endpoint).await?;
    let _gate=mode.gate(Some(&mode_token),false)?;
    let known=previous.as_ref().and_then(|profile|profile.certificate.as_ref());
    Ok(Inspection {origin:address(&endpoint)?.origin().ascii_serialization(),previous_fingerprint:known.map(|pin|pin.fingerprint.clone()),requires_confirmation:inspected.as_ref().is_some_and(|(next,_,authorized)|needs_confirmation(next,*authorized,known)),certificate:inspected.map(|(_,info,_)|info)})
}

#[tauri::command]
pub async fn remote_trust_certificate(app:AppHandle,profile_id:String,fingerprint:String,mode_token:Token)->ChatResult<super::profiles::Profile> {
    use tauri::Emitter;
    use crate::mode::Target;
    let mode=app.state::<Coordinator>();{let _gate=mode.gate(Some(&mode_token),false)?;}
    let mut profile=super::profiles::get(&mode,&profile_id)?;
    // 再次握手核对用户刚确认的指纹，不能让前端提交任意证书内容。
    let certificate=prepare(&profile.endpoint,profile.certificate.as_ref(),Some(&fingerprint)).await?;
    if certificate.as_ref().map(|pin|pin.fingerprint.as_str())!=Some(fingerprint.as_str()) {return Err(changed());}
    {let _gate=mode.gate(Some(&mode_token),false)?;}
    let client=super::transport::client_for(&profile.endpoint,certificate.as_ref())?;
    let secret=super::vault::load(&app,profile.credential_key())?;
    let identity=super::transport::probe_with(&client,&profile.endpoint,&secret).await?;
    if identity.server_id!=profile.server_id {return Err(ChatError::new("REMOTE_IDENTITY_CHANGED","这个地址的 infOS 已更换，请重新添加连接"));}
    let _switch=mode.switch_lock.lock().await;
    {let _gate=mode.gate(Some(&mode_token),false)?;}
    let current=super::profiles::get(&mode,&profile_id)?;
    if current.credential_id!=profile.credential_id || current.certificate.as_ref().map(|pin|&pin.fingerprint)!=profile.certificate.as_ref().map(|pin|&pin.fingerprint) {return Err(ChatError::new("REMOTE_CONFLICT","连接刚刚有变化，请重新核对证书"));}
    let active=mode_token.target==(Target::Remote {profile_id:profile_id.clone()});
    if active {
        let state=mode.begin_switch(mode_token.target.clone())?;let _=app.emit("mode-event",state);super::stop_all(&app);
        if let Err(error)=super::background::suspend_async(app.clone()).await {if let Ok(state)=mode.fail() {let _=app.emit("mode-event",state);}return Err(error);}
    }
    profile.certificate=certificate;
    let saved=(||->ChatResult<()> {
        let conn=crate::mode::open_existing(mode.path())?;
        conn.execute("UPDATE remote_profiles SET value=?1 WHERE id=?2",rusqlite::params![serde_json::to_string(&profile).map_err(|_|invalid())?,profile_id])?;
        Ok(())
    })();
    if active {
        if saved.is_err() {if let Ok(state)=mode.fail() {let _=app.emit("mode-event",state);}}
        else {
            match mode.activate() {
                Ok(state)=> {let _=app.emit("mode-event",state);},
                Err(error)=> {if let Ok(state)=mode.fail() {let _=app.emit("mode-event",state);}return Err(error);},
            }
            if let Err(error)=super::background::reauthorized(&app) {eprintln!("后台接收证书状态等待恢复：{}",error.message);}
            super::background::resume_async(app.clone()).await;
        }
    }
    saved?;Ok(profile)
}

#[cfg(test)]
mod tests;
