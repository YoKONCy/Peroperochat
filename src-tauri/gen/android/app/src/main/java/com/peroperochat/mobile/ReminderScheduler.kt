package com.peroperochat.mobile

import android.Manifest
import android.app.AlarmManager
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.util.Log
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat
import java.util.concurrent.Executors
import org.json.JSONArray
import org.json.JSONObject

/** 平台适配层只注册系统任务、发送通知；时间和状态规则始终由 Rust 计算。 */
object ReminderScheduler {
    const val CHANNEL = "ppc_reminder_channel"
    const val RETRY_ACTION = "com.peroperochat.mobile.REMINDER_RETRY"
    val executor = Executors.newSingleThreadExecutor()
    private const val PREFS = "ppc.reminder.dispatch"
    private const val TAG = "伙伴提醒"

    fun exact(context: Context): Boolean = Build.VERSION.SDK_INT < Build.VERSION_CODES.S ||
        (context.getSystemService(Context.ALARM_SERVICE) as AlarmManager).canScheduleExactAlarms()

    fun notifications(context: Context): Boolean {
        if (Build.VERSION.SDK_INT >= 33 && ContextCompat.checkSelfPermission(context, Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) return false
        if (!NotificationManagerCompat.from(context).areNotificationsEnabled()) return false
        if (Build.VERSION.SDK_INT >= 26) {
            val channel = (context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager).getNotificationChannel(CHANNEL)
            if (channel?.importance == NotificationManager.IMPORTANCE_NONE) return false
        }
        return true
    }

    fun status(context: Context): JSONObject = JSONObject().put("platform", "android").put("background", true)
        .put("exact", exact(context)).put("notifications", notifications(context))
        .put("syncError", context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).getString("error", null))

    private fun intent(context: Context, id: Long, token: String): Intent = Intent(context, AlarmReceiver::class.java)
        .setData(Uri.parse("peroperochat://reminder/$id")).putExtra("id", id).putExtra("token", token)

    private fun operation(context: Context, id: Long, token: String, create: Boolean = true): PendingIntent? = PendingIntent.getBroadcast(
        context, 0, intent(context, id, token), (if (create) PendingIntent.FLAG_UPDATE_CURRENT else PendingIntent.FLAG_NO_CREATE) or PendingIntent.FLAG_IMMUTABLE)

    private fun cancel(context: Context, id: Long, dismissNotification: Boolean) {
        operation(context, id, "", false)?.let { pending ->
            (context.getSystemService(Context.ALARM_SERVICE) as AlarmManager).cancel(pending)
            pending.cancel()
        }
        if (dismissNotification) dismiss(context, id)
    }

    private fun dismiss(context: Context, id: Long) {
        (context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager).cancel("ppc.reminder.$id", 0)
    }

    private fun register(context: Context, id: Long, token: String, at: Long, precise: Boolean, modeToken: JSONObject) {
        val manager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
        val pending = checkNotNull(PendingIntent.getBroadcast(context, 0, intent(context, id, token)
            .putExtra("modeToken", modeToken.toString()), PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE))
        if (precise) manager.setExactAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, at, pending)
        else manager.setAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, at, pending)
    }

    private fun receipt(id: Long, token: String, mode: String, error: String? = null): JSONObject =
        JSONObject().put("id", id).put("token", token).put("mode", mode).put("error", error ?: JSONObject.NULL)

    // 所有调用在同一工作线程执行；SQLite 修订检查仍防护并行的 Rust 编辑。
    fun synchronize(context: Context, force: Boolean = false): JSONObject {
        if (!ReminderBridge.active(context)) { suspend(context); return status(context).put("background", false) }
        val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        val saved = try { JSONObject(prefs.getString("jobs", "{}") ?: "{}") } catch (_: Exception) { JSONObject() }
        val plan = ReminderBridge.execute(context, "plan") as JSONObject
        val modeToken = plan.getJSONObject("modeToken")
        val jobs = plan.getJSONArray("jobs")
        val cancellations = plan.getJSONArray("cancellations")
        val desired = JSONObject()
        val results = JSONArray()
        val precise = exact(context)
        val allowed = notifications(context)
        val mode = if (precise) "exact" else "inexact"
        var retryNeeded = false
        for (index in 0 until cancellations.length()) {
            val item = cancellations.getJSONObject(index)
            val id = item.getLong("id")
            if (force || saved.has(id.toString()) || !item.optBoolean("registered")) cancel(context, id, item.optBoolean("dismiss"))
            results.put(receipt(id, item.getString("token"), "cancelled"))
        }
        for (index in 0 until jobs.length()) {
            val job = jobs.getJSONObject(index)
            val id = job.getLong("id")
            val key = id.toString()
            val token = job.getString("token")
            val old = saved.optJSONObject(key)
            val at = job.getLong("triggerAtMs")
            val signature = JSONObject().put("token", token).put("at", at).put("mode", mode)
            try {
                if (!allowed) {
                    cancel(context, id, false)
                    results.put(receipt(id, token, "blocked", "系统通知权限或提醒通知通道未开启"))
                    continue
                }
                val changed = old?.optString("token") != token
                check(ReminderBridge.guard(context, modeToken) {
                    if (changed && job.optBoolean("dismissPrevious")) dismiss(context, id)
                    if (force || changed || old?.optString("mode") != mode || old?.optLong("at") != at || old?.optString("epoch") != modeToken.toString()) {
                        register(context, id, token, at, precise, modeToken)
                    }
                }) { "提醒运行代次已失效" }
                signature.put("epoch", modeToken.toString())
                desired.put(key, signature)
                results.put(receipt(id, token, mode))
            } catch (error: Exception) {
                retryNeeded = true
                Log.e(TAG, "系统闹铃注册失败", error)
                results.put(receipt(id, token, "failed", "系统闹铃注册失败，等待恢复"))
            }
        }
        // 旧缓存用于清理系统登记，数据库始终是任务的权威来源。
        val keys = saved.keys()
        while (keys.hasNext()) {
            val key = keys.next()
            if (!desired.has(key)) cancel(context, key.toLong(), false)
        }
        check(prefs.edit().putString("jobs", desired.toString()).remove("error").commit()) { "系统调度缓存保存失败" }
        ReminderBridge.execute(context, "registered", results, modeToken)
        if (retryNeeded) retry(context)
        return status(context)
    }

    fun fire(context: Context, id: Long, token: String, modeToken: JSONObject) {
        val input = JSONObject().put("id", id).put("token", token)
        val claim = ReminderBridge.execute(context, "claim", input, modeToken) as? JSONObject
        if (claim == null) { synchronize(context); return }
        var failure: String? = null
        try {
            check(notifications(context)) { "系统通知权限或提醒通道未开启" }
            val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            if (Build.VERSION.SDK_INT >= 26) manager.createNotificationChannel(NotificationChannel(CHANNEL, "伙伴提醒", NotificationManager.IMPORTANCE_HIGH))
            val open = Intent(context, MainActivity::class.java).setAction("com.peroperochat.mobile.OPEN_REMINDER")
                .setData(Uri.parse("peroperochat://reminder/$id")).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP)
            val pending = PendingIntent.getActivity(context, 0, open, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
            val notification = NotificationCompat.Builder(context, CHANNEL).setSmallIcon(R.drawable.ic_stat_reminder)
                .setContentTitle(claim.getString("title")).setContentText(claim.getString("body"))
                .setStyle(NotificationCompat.BigTextStyle().bigText(claim.getString("body"))).setContentIntent(pending)
                .setAutoCancel(true).setPriority(NotificationCompat.PRIORITY_HIGH).build()
            check(ReminderBridge.guard(context, modeToken) { manager.notify("ppc.reminder.$id", 0, notification) }) { "提醒运行代次已失效" }
        } catch (error: Exception) {
            failure = if (!notifications(context)) "系统通知权限或提醒通道未开启" else "系统通知发送失败，将稍后重试"
            Log.e(TAG, failure, error)
        }
        input.put("lease", claim.getString("lease")).put("error", failure ?: JSONObject.NULL)
        val accepted = ReminderBridge.execute(context, "finish", input, modeToken) == true
        if (!accepted && failure == null) dismiss(context, id)
        synchronize(context)
    }

    fun retry(context: Context) {
        val modeToken = try { (ReminderBridge.execute(context, "mode") as? JSONObject)?.optJSONObject("modeToken") } catch (_: Exception) { null } ?: return
        val operation = PendingIntent.getBroadcast(context, 0, Intent(context, BootReceiver::class.java)
            .setAction(RETRY_ACTION).setData(Uri.parse("peroperochat://reminder-recovery")), PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        ReminderBridge.guard(context, modeToken) {
            (context.getSystemService(Context.ALARM_SERVICE) as AlarmManager).setAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, System.currentTimeMillis() + 60_000, operation)
        }
    }

    fun failed(context: Context, error: Throwable) {
        if (!ReminderBridge.active(context)) { suspend(context); return }
        Log.e(TAG, "原生提醒等待恢复", error)
        context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit().putString("error", "系统调度暂不可用，等待恢复").commit()
        try { retry(context) } catch (retryError: Exception) { Log.e(TAG, "提醒恢复任务注册失败", retryError) }
    }

    /** 仅撤销本机独立提醒，不修改提醒业务记录。门关闭时允许执行。 */
    fun suspend(context: Context): Boolean = ReminderBridge.inactiveGuard(context) { suspendInactive(context) }

    private fun suspendInactive(context: Context) {
        val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        val saved = try { JSONObject(prefs.getString("jobs", "{}") ?: "{}") } catch (_: Exception) { JSONObject() }
        saved.keys().forEach { key -> key.toLongOrNull()?.let { cancel(context, it, true) } }
        val retry = PendingIntent.getBroadcast(context, 0, Intent(context, BootReceiver::class.java)
            .setAction(RETRY_ACTION).setData(Uri.parse("peroperochat://reminder-recovery")), PendingIntent.FLAG_NO_CREATE or PendingIntent.FLAG_IMMUTABLE)
        retry?.let { (context.getSystemService(Context.ALARM_SERVICE) as AlarmManager).cancel(it); it.cancel() }
        val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        if (Build.VERSION.SDK_INT >= 23) manager.activeNotifications.filter { it.tag?.startsWith("ppc.reminder.") == true }
            .forEach { manager.cancel(it.tag, it.id) }
        check(prefs.edit().remove("jobs").remove("error").commit()) { "旧提醒登记缓存撤销失败" }
    }
}
