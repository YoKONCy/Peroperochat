package com.peroperochat.mobile

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import java.io.File

@InvokeArg
class ReminderSyncArgs { lateinit var databasePath: String }

@TauriPlugin
class ReminderPlugin(private val activity: Activity): Plugin(activity) {
    @Command
    fun suspend(invoke: Invoke) {
        ReminderScheduler.executor.execute {
            try { check(ReminderScheduler.suspend(activity.applicationContext)); invoke.resolve() }
            catch (_: Throwable) { invoke.reject("旧系统提醒撤销失败") }
        }
    }
    @Command
    fun synchronize(invoke: Invoke) {
        val args = invoke.parseArgs(ReminderSyncArgs::class.java)
        val context = activity.applicationContext
        ReminderScheduler.executor.execute {
            try {
                check(File(args.databasePath).canonicalFile == ReminderBridge.database(context)) { "系统调度数据库路径不匹配" }
                val status = ReminderScheduler.synchronize(context)
                val result = app.tauri.plugin.JSObject()
                status.keys().forEach { key -> result.put(key, status.get(key)) }
                invoke.resolve(result)
            } catch (error: Throwable) { ReminderScheduler.failed(context, error); invoke.reject("提醒已保存，系统调度等待恢复") }
        }
    }

    @Command
    fun status(invoke: Invoke) {
        val status = ReminderScheduler.status(activity)
        val result = app.tauri.plugin.JSObject()
        status.keys().forEach { key -> result.put(key, status.get(key)) }
        invoke.resolve(result)
    }

    @Command
    fun openNotificationSettings(invoke: Invoke) {
        activity.runOnUiThread {
            try {
                val intent = if (Build.VERSION.SDK_INT >= 26) {
                    val manager = activity.getSystemService(android.content.Context.NOTIFICATION_SERVICE) as android.app.NotificationManager
                    if (manager.getNotificationChannel(ReminderScheduler.CHANNEL) != null) Intent(Settings.ACTION_CHANNEL_NOTIFICATION_SETTINGS)
                        .putExtra(Settings.EXTRA_APP_PACKAGE, activity.packageName).putExtra(Settings.EXTRA_CHANNEL_ID, ReminderScheduler.CHANNEL)
                    else Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS).putExtra(Settings.EXTRA_APP_PACKAGE, activity.packageName)
                } else Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS, Uri.parse("package:${activity.packageName}"))
                activity.startActivity(intent)
                invoke.resolve()
            } catch (error: Exception) { invoke.reject("无法打开系统通知设置") }
        }
    }

    @Command
    fun requestPrecision(invoke: Invoke) {
        activity.runOnUiThread {
            try {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S && !ReminderScheduler.exact(activity)) {
                    activity.startActivity(Intent(Settings.ACTION_REQUEST_SCHEDULE_EXACT_ALARM, Uri.parse("package:${activity.packageName}")))
                }
                invoke.resolve()
            } catch (error: Exception) { invoke.reject("无法打开系统闹钟与提醒设置") }
        }
    }

    override fun onResume() {
        super.onResume()
        val context = activity.applicationContext
        ReminderScheduler.executor.execute {
            try { if (ReminderBridge.database(context).isFile) ReminderScheduler.synchronize(context) }
            catch (error: Throwable) { ReminderScheduler.failed(context, error) }
        }
    }
}
