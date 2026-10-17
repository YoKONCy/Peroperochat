package com.peroperochat.mobile

import android.content.Context
import java.io.File
import org.json.JSONObject

/** 只把系统事件送给 Rust 核心，接收器不启动 Tauri 界面。 */
object ReminderBridge {
    init { System.loadLibrary("app_lib") }

    @JvmStatic external fun nativeExecute(databasePath: String, command: String, payload: String): String
    @JvmStatic external fun nativeGuard(databasePath: String, token: String, action: Runnable): Boolean
    @JvmStatic external fun nativeInactiveGuard(databasePath: String, action: Runnable): Boolean

    fun database(context: Context): File = File(context.applicationInfo.dataDir, "data/peropero.db").canonicalFile

    fun active(context: Context): Boolean = try {
        (execute(context, "mode") as? JSONObject)?.optBoolean("active") == true
    } catch (_: Exception) { false }

    fun guard(context: Context, token: JSONObject, action: () -> Unit): Boolean =
        nativeGuard(database(context).absolutePath, token.toString(), Runnable { action() })
    fun inactiveGuard(context: Context, action: () -> Unit): Boolean =
        nativeInactiveGuard(database(context).absolutePath, Runnable { action() })

    fun execute(context: Context, command: String, payload: Any = JSONObject(), modeToken: JSONObject? = null): Any? {
        val file = database(context)
        check(file.isFile) { "提醒数据库尚未初始化，请先打开应用" }
        val input = JSONObject().put("input", payload)
        if (modeToken != null) input.put("modeToken", modeToken)
        val response = JSONObject(nativeExecute(file.absolutePath, command, input.toString()))
        check(response.optBoolean("ok")) { response.optJSONObject("error")?.optString("message") ?: "原生提醒处理失败" }
        return response.opt("value").takeUnless { it == JSONObject.NULL }
    }
}
