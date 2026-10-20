//! 远程凭据只进入系统保护存储，前端接口不会返回凭据。
use tauri::AppHandle;
use crate::local::types::*;

#[cfg(target_os="android")]
use tauri::Manager;
#[cfg(target_os="android")]
struct Native(tauri::plugin::PluginHandle<tauri::Wry>);

pub fn init()->tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("remote-vault").setup(|app,api| {
        #[cfg(target_os="android")]
        app.manage(Native(api.register_android_plugin("com.peroperochat.mobile","RemoteVaultPlugin")?));
        #[cfg(not(target_os="android"))]
        let _=(app,api);
        Ok(())
    }).build()
}

pub fn save(app:&AppHandle,id:&str,secret:&str)->ChatResult<()> {
    if uuid::Uuid::parse_str(id).is_err() || secret.is_empty() || secret.len()>4096 {return Err(error());}
    #[cfg(target_os="android")]
    {app.state::<Native>().0.run_mobile_plugin::<serde_json::Value>("save",serde_json::json!({"id":id,"secret":secret}))
        .map_err(|_|error())?;Ok(())}
    #[cfg(windows)]
    {
        let file=path(app,id)?;let bytes=protect(secret.as_bytes(),false)?;
        let temporary=file.with_extension("pending");
        std::fs::write(&temporary,bytes).map_err(|_|error())?;
        std::fs::rename(&temporary,&file).map_err(|_|error())?;Ok(())
    }
    #[cfg(target_os="ios")]
    {let _=app;security_framework::passwords::set_generic_password_options(secret.as_bytes(),ios_options(id)).map_err(|_|error())}
    #[cfg(not(any(windows,target_os="android",target_os="ios")))]
    {let _=(app,id,secret);Err(error())}
}
pub fn load(app:&AppHandle,id:&str)->ChatResult<String> {
    if uuid::Uuid::parse_str(id).is_err() {return Err(error());}
    #[cfg(target_os="android")]
    {let result:serde_json::Value=app.state::<Native>().0.run_mobile_plugin("load",serde_json::json!({"id":id})).map_err(|_|error())?;
        result["secret"].as_str().map(str::to_owned).ok_or_else(error)}
    #[cfg(windows)]
    {let bytes=std::fs::read(path(app,id)?).map_err(|_|error())?;String::from_utf8(protect(&bytes,true)?).map_err(|_|error())}
    #[cfg(target_os="ios")]
    {let _=app;let bytes=security_framework::passwords::generic_password(ios_options(id)).map_err(|_|error())?;String::from_utf8(bytes).map_err(|_|error())}
    #[cfg(not(any(windows,target_os="android",target_os="ios")))]
    {let _=(app,id);Err(error())}
}
pub fn delete(app:&AppHandle,id:&str)->ChatResult<()> {
    if uuid::Uuid::parse_str(id).is_err() {return Err(error());}
    #[cfg(target_os="android")]
    {app.state::<Native>().0.run_mobile_plugin::<serde_json::Value>("delete",serde_json::json!({"id":id})).map_err(|_|error())?;Ok(())}
    #[cfg(windows)]
    {let file=path(app,id)?;if file.exists() {std::fs::remove_file(file).map_err(|_|error())?;}Ok(())}
    #[cfg(target_os="ios")]
    {
        let _=app;
        match security_framework::passwords::delete_generic_password_options(ios_options(id)) {
            Ok(())=>Ok(()),Err(value) if value.code()==-25300=>Ok(()),Err(_)=>Err(error()),
        }
    }
    #[cfg(not(any(windows,target_os="android",target_os="ios")))]
    {let _=(app,id);Err(error())}
}
fn error()->ChatError {ChatError::new("CREDENTIAL_UNAVAILABLE","系统凭据存储不可用，请重新保存连接；此平台尚未支持时不会降级为明文存储")}

#[cfg(target_os="ios")]
fn ios_options(id:&str)->security_framework::passwords::PasswordOptions {
    let mut options=security_framework::passwords::PasswordOptions::new_generic_password("com.peroperochat.mobile.remote.v1",id);
    // 使用本设备的数据保护钥匙串，不把远程设备授权同步到 iCloud。
    options.set_access_synchronized(Some(false));
    options.use_protected_keychain();
    options
}

#[cfg(windows)]
fn path(app:&AppHandle,id:&str)->ChatResult<std::path::PathBuf> {
    use tauri::Manager;
    if uuid::Uuid::parse_str(id).is_err() {return Err(error());}
    let dir=app.path().app_data_dir().map_err(|_|error())?.join("data/control/credentials");
    std::fs::create_dir_all(&dir).map_err(|_|error())?;Ok(dir.join(format!("{id}.bin")))
}

#[cfg(windows)]
fn protect(bytes:&[u8],decrypt:bool)->ChatResult<Vec<u8>> {
    #[repr(C)]
    struct Blob {length:u32,data:*mut u8}
    #[link(name="crypt32")]
    extern "system" {
        fn CryptProtectData(input:*const Blob,description:*const u16,entropy:*const Blob,reserved:*mut std::ffi::c_void,prompt:*mut std::ffi::c_void,flags:u32,output:*mut Blob)->i32;
        fn CryptUnprotectData(input:*const Blob,description:*mut *mut u16,entropy:*const Blob,reserved:*mut std::ffi::c_void,prompt:*mut std::ffi::c_void,flags:u32,output:*mut Blob)->i32;
    }
    #[link(name="kernel32")]
    extern "system" {fn LocalFree(pointer:*mut std::ffi::c_void)->*mut std::ffi::c_void;}
    if bytes.len()>65536 {return Err(error());}
    let input=Blob {length:bytes.len() as u32,data:bytes.as_ptr() as *mut u8};
    let mut output=Blob {length:0,data:std::ptr::null_mut()};
    // DPAPI 绑定当前 Windows 用户，禁止凭据接口弹出系统对话框。
    let success=unsafe {if decrypt {CryptUnprotectData(&input,std::ptr::null_mut(),std::ptr::null(),std::ptr::null_mut(),std::ptr::null_mut(),1,&mut output)}
        else {CryptProtectData(&input,std::ptr::null(),std::ptr::null(),std::ptr::null_mut(),std::ptr::null_mut(),1,&mut output)}};
    if success==0 || output.data.is_null() {return Err(error());}
    let result=unsafe {std::slice::from_raw_parts(output.data,output.length as usize).to_vec()};
    unsafe {LocalFree(output.data.cast());}Ok(result)
}
