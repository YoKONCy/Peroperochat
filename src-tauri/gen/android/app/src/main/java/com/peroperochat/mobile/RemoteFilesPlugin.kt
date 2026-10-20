package com.peroperochat.mobile

import android.app.Activity
import android.content.Intent
import androidx.activity.result.ActivityResult
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean
import java.io.File
import android.provider.DocumentsContract

@InvokeArg
class RemoteExportArgs { lateinit var name: String; lateinit var content: String }
@InvokeArg
class RemoteSaveArgs {
    lateinit var id: String; lateinit var path: String; lateinit var name: String
    lateinit var mimeType: String; lateinit var controlPath: String; lateinit var modeToken: String
}
@InvokeArg
class RemoteCancelArgs { lateinit var id: String }

/** 使用系统文档选择器，不申请整盘存储权限，也不暂存含密钥的配置。 */
@TauriPlugin
class RemoteFilesPlugin(private val activity: Activity): Plugin(activity) {
    private val executor = Executors.newSingleThreadExecutor()
    private var pending = false
    private class SaveJob(val args: RemoteSaveArgs, val invoke: Invoke) {
        val cancelled = AtomicBoolean(false)
        @Volatile var copying = false
    }
    @Volatile private var fileJob: SaveJob? = null
    private val cancelledIds = linkedSetOf<String>()
    private external fun nativeGuard(controlPath: String, token: String, action: Runnable): Boolean

    @Command
    fun saveFile(invoke: Invoke) {
        activity.runOnUiThread {
            if (pending) { invoke.reject("请先完成当前文件保存"); return@runOnUiThread }
            try {
                val args = invoke.parseArgs(RemoteSaveArgs::class.java)
                if (cancelledIds.remove(args.id)) {
                    val response = JSObject(); response.put("saved", false); response.put("cancelled", true); invoke.resolve(response)
                    return@runOnUiThread
                }
                val root = File(activity.cacheDir, "remote-transfers").canonicalFile
                val file = File(args.path).canonicalFile
                check(file.parentFile == root && file.extension == "part" && file.isFile)
                java.util.UUID.fromString(file.nameWithoutExtension)
                check(nativeGuard(args.controlPath, args.modeToken, Runnable {}))
                val job = SaveJob(args, invoke); fileJob = job; pending = true
                val mime = args.mimeType.substringBefore(';').trim()
                    .takeIf { it.matches(Regex("[a-zA-Z0-9.+-]+/[a-zA-Z0-9.+-]+")) } ?: "application/octet-stream"
                startActivityForResult(invoke, Intent(Intent.ACTION_CREATE_DOCUMENT)
                    .addCategory(Intent.CATEGORY_OPENABLE).setType(mime)
                    .putExtra(Intent.EXTRA_TITLE, args.name), "fileSaved")
            } catch (_: Exception) { fileJob = null; pending = false; invoke.reject("无法打开系统文件保存入口") }
        }
    }

    @Command
    fun cancelSave(invoke: Invoke) {
        val id = invoke.parseArgs(RemoteCancelArgs::class.java).id
        activity.runOnUiThread {
            cancelledIds.add(id)
            if (cancelledIds.size > 64) cancelledIds.remove(cancelledIds.first())
            fileJob?.takeIf { it.args.id == id }?.let { job ->
                job.cancelled.set(true)
                if (!job.copying) {
                    fileJob = null; pending = false
                    val response = JSObject(); response.put("saved", false); response.put("cancelled", true)
                    job.invoke.resolve(response)
                }
            }
            invoke.resolve()
        }
    }

    @ActivityCallback
    private fun fileSaved(invoke: Invoke, result: ActivityResult) {
        val args = invoke.parseArgs(RemoteSaveArgs::class.java)
        val job = fileJob?.takeIf { it.args.id == args.id }
        val uri = result.data?.data
        fun removePartial() { if (uri != null) try { DocumentsContract.deleteDocument(activity.contentResolver, uri) } catch (_: Exception) { } }
        if (job == null) { removePartial(); return }
        if (result.resultCode != Activity.RESULT_OK || uri == null) {
            fileJob = null; pending = false
            val response = JSObject(); response.put("saved", false); response.put("cancelled", true); invoke.resolve(response)
            return
        }
        job.copying = true
        executor.execute {
            try {
                val output = activity.contentResolver.openOutputStream(uri, "wt") ?: error("目标文件不可写")
                output.use { target ->
                    File(args.path).inputStream().use { source ->
                        val buffer = ByteArray(256 * 1024)
                        while (true) {
                            check(!job.cancelled.get())
                            val count = source.read(buffer); if (count < 0) break
                            check(nativeGuard(args.controlPath, args.modeToken, Runnable { target.write(buffer, 0, count) }))
                        }
                    }
                    check(!job.cancelled.get())
                    check(nativeGuard(args.controlPath, args.modeToken, Runnable { target.flush() }))
                }
                check(!job.cancelled.get())
                check(nativeGuard(args.controlPath, args.modeToken, Runnable {}))
                val response = JSObject(); response.put("saved", true); invoke.resolve(response)
            } catch (_: Exception) {
                removePartial(); invoke.reject(if (job.cancelled.get()) "已取消保存" else "保存失败或运行模式已切换，已清理不完整文件")
            } finally { activity.runOnUiThread { if (fileJob === job) { fileJob = null; pending = false } } }
        }
    }

    @Command
    fun saveText(invoke: Invoke) {
        activity.runOnUiThread {
            if (pending) { invoke.reject("请先完成当前文件保存"); return@runOnUiThread }
            try {
                val args = invoke.parseArgs(RemoteExportArgs::class.java)
                check(args.content.toByteArray(Charsets.UTF_8).size <= 8 * 1024 * 1024)
                pending = true
                val intent = Intent(Intent.ACTION_CREATE_DOCUMENT).addCategory(Intent.CATEGORY_OPENABLE)
                    .setType("application/json").putExtra(Intent.EXTRA_TITLE, args.name)
                startActivityForResult(invoke, intent, "saved")
            } catch (_: Exception) { pending = false; invoke.reject("无法打开系统文件保存入口") }
        }
    }

    @ActivityCallback
    private fun saved(invoke: Invoke, result: ActivityResult) {
        val uri = result.data?.data
        if (result.resultCode != Activity.RESULT_OK || uri == null) {
            pending = false
            val response = JSObject(); response.put("saved", false); response.put("cancelled", true); invoke.resolve(response)
            return
        }
        executor.execute {
            try {
                val args = invoke.parseArgs(RemoteExportArgs::class.java)
                val stream = activity.contentResolver.openOutputStream(uri, "wt") ?: error("目标文件不可写")
                stream.use { it.write(args.content.toByteArray(Charsets.UTF_8)) }
                val response = JSObject(); response.put("saved", true); invoke.resolve(response)
            } catch (_: Exception) { invoke.reject("文件保存失败，请检查目标位置") }
            finally { activity.runOnUiThread { pending = false } }
        }
    }
}
