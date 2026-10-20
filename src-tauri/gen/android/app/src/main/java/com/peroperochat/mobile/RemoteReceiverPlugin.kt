package com.peroperochat.mobile

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.PowerManager
import android.provider.Settings
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import androidx.core.app.NotificationManagerCompat
import org.json.JSONObject

@InvokeArg
class RemoteReceiverArgs { var modeToken: String = ""; var kind: String = "" }

@TauriPlugin
class RemoteReceiverPlugin(private val activity: Activity): Plugin(activity) {
    private fun resolve(invoke: Invoke, value: JSONObject) {
        val result = JSObject(); value.keys().forEach { result.put(it, value.get(it)) }; invoke.resolve(result)
    }
    @Command
    fun synchronize(invoke: Invoke) {
        RemoteReceiverController.executor.execute {
            try {
                if (activity.hasWindowFocus()) RemoteReceiverBridge.execute(activity, "visible", JSONObject().put("visible", true))
                RemoteReceiverController.synchronize(activity.applicationContext); invoke.resolve()
            }
            catch (_: Exception) { invoke.reject("后台连接未启动，请核对后台权限后重试") }
        }
    }
    @Command
    fun stop(invoke: Invoke) {
        try { RemoteReceiverController.stop(activity.applicationContext); invoke.resolve() }
        catch (_: Exception) { invoke.reject("旧后台连接停止失败") }
    }
    @Command
    fun status(invoke: Invoke) {
        val args = invoke.parseArgs(RemoteReceiverArgs::class.java)
        RemoteReceiverController.executor.execute {
            try {
                val status = RemoteReceiverBridge.execute(activity, "status", JSONObject().put("modeToken", JSONObject(args.modeToken)))
                val power = activity.getSystemService(Context.POWER_SERVICE) as PowerManager
                val manager = activity.getSystemService(Context.NOTIFICATION_SERVICE) as android.app.NotificationManager
                val channelAllowed = android.os.Build.VERSION.SDK_INT < 26 || manager.getNotificationChannel(RemoteReceiverController.MESSAGE_CHANNEL)?.importance != android.app.NotificationManager.IMPORTANCE_NONE
                status.put("platform", "android").put("supported", true)
                    .put("notifications", NotificationManagerCompat.from(activity).areNotificationsEnabled() && channelAllowed)
                    .put("batteryUnrestricted", power.isIgnoringBatteryOptimizations(activity.packageName))
                resolve(invoke, status)
            } catch (_: Exception) { invoke.reject("后台接收状态暂不可读") }
        }
    }
    @Command
    fun openSettings(invoke: Invoke) {
        val args = invoke.parseArgs(RemoteReceiverArgs::class.java)
        activity.runOnUiThread {
            try {
                val intent = when (args.kind) {
                    "battery" -> Intent(Settings.ACTION_IGNORE_BATTERY_OPTIMIZATION_SETTINGS)
                    "notifications" -> if (android.os.Build.VERSION.SDK_INT >= 26) Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS).putExtra(Settings.EXTRA_APP_PACKAGE, activity.packageName)
                        else Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS, Uri.parse("package:${activity.packageName}"))
                    else -> throw IllegalArgumentException("未知系统设置")
                }
                activity.startActivity(intent); invoke.resolve()
            } catch (_: Exception) { invoke.reject("无法打开系统设置，请在应用信息中调整") }
        }
    }
}
