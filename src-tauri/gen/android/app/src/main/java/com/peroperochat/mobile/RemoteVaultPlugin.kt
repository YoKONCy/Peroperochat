package com.peroperochat.mobile

import android.app.Activity
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.security.KeyStore
import java.util.UUID
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

@InvokeArg
class RemoteSecretArgs { lateinit var id: String; var secret: String = "" }

/** 凭据密钥留在 Android Keystore，配置与备份中只保存不透明连接身份。 */
@TauriPlugin
class RemoteVaultPlugin(private val activity: Activity): Plugin(activity) {
    private val alias = "peroperochat.remote.v1"
    private val prefs get() = activity.getSharedPreferences("ppc.remote.credentials", Activity.MODE_PRIVATE)
    private fun key(): SecretKey {
        val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        (store.getKey(alias, null) as? SecretKey)?.let { return it }
        return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore").apply {
            init(KeyGenParameterSpec.Builder(alias, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM).setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setRandomizedEncryptionRequired(true).build())
        }.generateKey()
    }
    @Command
    fun save(invoke: Invoke) {
        try {
            val input = invoke.parseArgs(RemoteSecretArgs::class.java)
            UUID.fromString(input.id)
            check(input.secret.length <= 4096)
            val cipher = Cipher.getInstance("AES/GCM/NoPadding").apply { init(Cipher.ENCRYPT_MODE, key()) }
            val bytes = cipher.iv + cipher.doFinal(input.secret.toByteArray(Charsets.UTF_8))
            check(prefs.edit().putString(input.id, Base64.encodeToString(bytes, Base64.NO_WRAP)).commit())
            invoke.resolve()
        } catch (_: Exception) { invoke.reject("系统凭据保存失败") }
    }
    @Command
    fun load(invoke: Invoke) {
        try {
            val input = invoke.parseArgs(RemoteSecretArgs::class.java)
            UUID.fromString(input.id)
            val bytes = Base64.decode(checkNotNull(prefs.getString(input.id, null)), Base64.NO_WRAP)
            check(bytes.size >= 28)
            val cipher = Cipher.getInstance("AES/GCM/NoPadding").apply {
                init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(128, bytes.copyOfRange(0, 12)))
            }
            val result = JSObject()
            result.put("secret", String(cipher.doFinal(bytes.copyOfRange(12, bytes.size)), Charsets.UTF_8))
            invoke.resolve(result)
        } catch (_: Exception) { invoke.reject("系统凭据不可读，请重新保存连接") }
    }
    @Command
    fun delete(invoke: Invoke) {
        try {
            val input = invoke.parseArgs(RemoteSecretArgs::class.java)
            UUID.fromString(input.id)
            check(prefs.edit().remove(input.id).commit())
            invoke.resolve()
        } catch (_: Exception) { invoke.reject("系统凭据删除失败") }
    }
}
