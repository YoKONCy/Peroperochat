package com.peroperochat.mobile

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec
import android.util.AtomicFile
import java.io.File
import java.util.UUID

/** 界面与无界面服务共用 Keystore；凭据不经 Intent、日志或 WebView 传递。 */
object RemoteCredentials {
    private const val ALIAS = "peroperochat.remote.v1"
    private fun prefs(context: Context) = context.getSharedPreferences("ppc.remote.credentials", Context.MODE_PRIVATE)
    private fun file(context: Context, id: String): AtomicFile {
        UUID.fromString(id)
        val directory = File(context.noBackupFilesDir, "remote-credentials")
        check(directory.isDirectory || directory.mkdirs()) { "系统凭据目录不可用" }
        return AtomicFile(File(directory, id))
    }
    private fun write(context: Context, id: String, bytes: ByteArray) {
        val target = file(context, id)
        val stream = target.startWrite()
        try { stream.write(bytes); target.finishWrite(stream) }
        catch (error: Exception) { target.failWrite(stream); throw error }
    }
    private fun key(create: Boolean): SecretKey {
        val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        (store.getKey(ALIAS, null) as? SecretKey)?.let { return it }
        check(create) { "远程凭据密钥不存在，请重新授权" }
        return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore").apply {
            init(KeyGenParameterSpec.Builder(ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM).setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setRandomizedEncryptionRequired(true).build())
        }.generateKey()
    }
    fun save(context: Context, id: String, secret: String) {
        val cipher = Cipher.getInstance("AES/GCM/NoPadding").apply { init(Cipher.ENCRYPT_MODE, key(true)) }
        val bytes = cipher.iv + cipher.doFinal(secret.toByteArray(Charsets.UTF_8))
        write(context, id, bytes)
        prefs(context).edit().remove(id).commit()
    }
    fun load(context: Context, id: String): String {
        // 原子文件支持跨进程读取；旧版本密文在首次成功解密后迁移，失败保留原记录。
        val target = file(context, id)
        val legacy = !target.baseFile.isFile
        val bytes = if (!legacy) target.readFully() else Base64.decode(checkNotNull(prefs(context).getString(id, null)), Base64.NO_WRAP)
        check(bytes.size >= 28)
        val cipher = Cipher.getInstance("AES/GCM/NoPadding").apply {
            init(Cipher.DECRYPT_MODE, key(false), GCMParameterSpec(128, bytes.copyOfRange(0, 12)))
        }
        val secret = String(cipher.doFinal(bytes.copyOfRange(12, bytes.size)), Charsets.UTF_8)
        if (legacy) { write(context, id, bytes); prefs(context).edit().remove(id).commit() }
        return secret
    }
    fun delete(context: Context, id: String) { file(context, id).delete(); check(prefs(context).edit().remove(id).commit()) }
}
