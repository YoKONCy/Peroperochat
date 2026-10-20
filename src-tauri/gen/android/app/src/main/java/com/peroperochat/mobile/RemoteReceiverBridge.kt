package com.peroperochat.mobile

import android.content.Context
import java.io.File
import org.json.JSONObject

/** 冷启动只装载 JNI 核心，不创建 Tauri Activity 或 WebView。 */
object RemoteReceiverBridge {
    init { System.loadLibrary("app_lib") }
    @JvmStatic external fun nativeExecute(dataPath: String, command: String, input: String): String
    @JvmStatic external fun nativeRun(dataPath: String, token: String, secret: String, callback: RemoteReceiverCallback): String
    fun data(context: Context): File = File(context.applicationInfo.dataDir, "data").canonicalFile
    fun execute(context: Context, command: String, input: JSONObject = JSONObject()): JSONObject {
        val response = JSONObject(nativeExecute(data(context).absolutePath, command, input.toString()))
        check(response.optBoolean("ok")) { response.optJSONObject("error")?.optString("message") ?: "后台通知控制失败" }
        return response.optJSONObject("value") ?: JSONObject()
    }
}

interface RemoteReceiverCallback {
    fun isStopped(): Boolean
    fun show(payload: String): Boolean
}
