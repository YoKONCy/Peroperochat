package com.peroperochat.mobile

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.util.UUID

@InvokeArg
class RemoteSecretArgs { lateinit var id: String; var secret: String = "" }

/** 凭据密钥留在 Android Keystore，密文通过原子文件供原生接收进程读取。 */
@TauriPlugin
class RemoteVaultPlugin(private val activity: Activity): Plugin(activity) {
    @Command
    fun save(invoke: Invoke) {
        try {
            val input = invoke.parseArgs(RemoteSecretArgs::class.java)
            UUID.fromString(input.id); check(input.secret.length <= 4096)
            RemoteCredentials.save(activity, input.id, input.secret); invoke.resolve()
        } catch (_: Exception) { invoke.reject("系统凭据保存失败") }
    }
    @Command
    fun load(invoke: Invoke) {
        try {
            val input = invoke.parseArgs(RemoteSecretArgs::class.java)
            UUID.fromString(input.id)
            val result = JSObject(); result.put("secret", RemoteCredentials.load(activity, input.id)); invoke.resolve(result)
        } catch (_: Exception) { invoke.reject("系统凭据不可读，请重新保存连接") }
    }
    @Command
    fun delete(invoke: Invoke) {
        try {
            val input = invoke.parseArgs(RemoteSecretArgs::class.java)
            UUID.fromString(input.id); RemoteCredentials.delete(activity, input.id); invoke.resolve()
        } catch (_: Exception) { invoke.reject("系统凭据删除失败") }
    }
}
