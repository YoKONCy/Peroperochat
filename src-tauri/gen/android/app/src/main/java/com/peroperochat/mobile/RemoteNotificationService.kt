package com.peroperochat.mobile

import android.app.Service
import android.app.ActivityManager
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Intent
import android.content.pm.ServiceInfo
import android.net.Uri
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import org.json.JSONObject
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.random.Random

/** 独立原生进程只维持通知连接；没有 Vue、Live2D 或本地 Agent。 */
class RemoteNotificationService : Service() {
    private val stopped = AtomicBoolean(false)
    @Volatile private var runningToken: String? = null
    @Volatile private var worker: Thread? = null
    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        try {
            // 先满足系统前台服务时限，再做可能等待 SQLite 锁的运行门核验。
            RemoteReceiverController.channels(this)
            val open = PendingIntent.getActivity(this, 0, Intent(this, MainActivity::class.java), PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
            val notice = NotificationCompat.Builder(this, RemoteReceiverController.SERVICE_CHANNEL)
                .setSmallIcon(R.drawable.ic_stat_reminder).setContentTitle("infOS 后台接收已开启")
                .setContentText("保持连接，接收远程伙伴的通知").setContentIntent(open)
                .setOngoing(true).setCategory(NotificationCompat.CATEGORY_SERVICE).build()
            if (Build.VERSION.SDK_INT >= 34) startForeground(7411, notice, ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE)
            else startForeground(7411, notice)
            val requested = intent?.getStringExtra("modeToken")
            val plan = RemoteReceiverBridge.execute(this, "plan", if (requested == null) JSONObject() else JSONObject().put("modeToken", JSONObject(requested)))
            if (!plan.optBoolean("active")) { stopSelf(); return START_NOT_STICKY }
            val token = plan.getJSONObject("modeToken").toString()
            if (worker?.isAlive == true) {
                // 新代次由旧工作线程结束后接手，不同时维持两条后台连接。
                runningToken = token
                return START_STICKY
            }
            stopped.set(false); runningToken = token
            worker = Thread({ receive() }, "infOS通知接收").apply { start() }
            return START_STICKY
        } catch (_: Exception) {
            Log.w("远程通知", "后台连接启动受限，请打开应用核对通知与后台权限")
            stopSelf(); return START_NOT_STICKY
        }
    }

    private fun receive() {
        var delay = 2_000L
        try {
            while (!stopped.get()) {
                val plan = RemoteReceiverBridge.execute(this, "plan")
                if (!plan.optBoolean("active")) break
                val token = plan.getJSONObject("modeToken").toString()
                runningToken = token
                val secret = try { RemoteCredentials.load(this, plan.getString("credentialId")) }
                catch (_: Exception) {
                    RemoteReceiverBridge.execute(this, "block", JSONObject().put("modeToken", plan.getJSONObject("modeToken")))
                    break
                }
                val began = System.currentTimeMillis()
                val result = JSONObject(RemoteReceiverBridge.nativeRun(RemoteReceiverBridge.data(this).absolutePath, token, secret, object : RemoteReceiverCallback {
                    override fun isStopped(): Boolean = stopped.get() || runningToken != token
                    override fun show(payload: String): Boolean = !stopped.get() && showNotice(JSONObject(payload))
                }))
                val code = result.optJSONObject("error")?.optString("code")
                if (code?.startsWith("REMOTE_CERTIFICATE_") == true || code in setOf("REMOTE_AUTH_REQUIRED", "REMOTE_IDENTITY_CHANGED", "REMOTE_INCOMPATIBLE", "REMOTE_PROTOCOL")) break
                if (stopped.get()) break
                if (System.currentTimeMillis() - began > 60_000) delay = 2_000
                val wait = delay + Random.nextLong(0, 1_000)
                var elapsed = 0L
                while (!stopped.get() && elapsed < wait) { Thread.sleep(250); elapsed += 250 }
                delay = (delay * 2).coerceAtMost(60_000)
            }
        } catch (_: InterruptedException) { /* 服务停止时结束重连等待。 */ }
        catch (_: Exception) { Log.w("远程通知", "后台接收中断，等待打开应用后恢复") }
        finally { stopSelf() }
    }

    private fun showNotice(notice: JSONObject): Boolean {
        // 可见租约必须与系统进程状态同时成立，主进程突然退出时不能吞掉新通知。
        val main = (getSystemService(ACTIVITY_SERVICE) as ActivityManager).runningAppProcesses?.firstOrNull { it.processName == packageName }
        if (notice.optBoolean("foregroundHint") && main?.importance == ActivityManager.RunningAppProcessInfo.IMPORTANCE_FOREGROUND) return true
        if (!NotificationManagerCompat.from(this).areNotificationsEnabled()) return false
        val manager = getSystemService(NOTIFICATION_SERVICE) as NotificationManager
        if (Build.VERSION.SDK_INT >= 26 && manager.getNotificationChannel(RemoteReceiverController.MESSAGE_CHANNEL)?.importance == NotificationManager.IMPORTANCE_NONE) return false
        val token = notice.getJSONObject("modeToken")
        val profile = token.getJSONObject("target").getString("profileId")
        val principal = notice.getString("principalId"); val id = notice.getString("notificationId")
        val uri = Uri.Builder().scheme("peroperochat").authority("remote-notice").appendPath(profile).appendPath(principal).appendPath(id).build()
        val open = Intent(this, MainActivity::class.java).setAction(RemoteReceiverController.OPEN).setData(uri)
            .putExtra("notice", JSONObject().put("modeToken", token).put("principalId", principal).put("notificationId", id).toString())
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP)
        val pending = PendingIntent.getActivity(this, 0, open, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        val notification = NotificationCompat.Builder(this, RemoteReceiverController.MESSAGE_CHANNEL)
            .setSmallIcon(R.drawable.ic_stat_reminder).setContentTitle(notice.optString("title").ifBlank { "infOS 通知" })
            .setContentText(notice.optString("body")).setStyle(NotificationCompat.BigTextStyle().bigText(notice.optString("body")))
            .setContentIntent(pending).setAutoCancel(true).setOnlyAlertOnce(true).setSilent(notice.optBoolean("replayed"))
            .setGroup("ppc.remote.$profile").setCategory(NotificationCompat.CATEGORY_MESSAGE).build()
        return try { manager.notify("ppc.remote.$uri", 0, notification); true }
        catch (_: SecurityException) { false }
    }
    override fun onDestroy() {
        stopped.set(true); worker?.interrupt()
        super.onDestroy()
    }
}
