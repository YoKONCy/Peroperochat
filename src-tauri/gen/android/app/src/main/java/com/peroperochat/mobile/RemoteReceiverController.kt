package com.peroperochat.mobile

import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.UserManager
import androidx.core.content.ContextCompat
import java.util.concurrent.Executors
import org.json.JSONObject

object RemoteReceiverController {
    const val SERVICE_CHANNEL = "ppc_remote_connection"
    const val MESSAGE_CHANNEL = "ppc_remote_messages"
    const val OPEN = "com.peroperochat.mobile.OPEN_REMOTE_NOTICE"
    val executor = Executors.newSingleThreadExecutor()

    fun channels(context: Context) {
        if (Build.VERSION.SDK_INT >= 26) {
            val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            manager.createNotificationChannel(NotificationChannel(SERVICE_CHANNEL, "infOS 后台连接", NotificationManager.IMPORTANCE_LOW))
            manager.createNotificationChannel(NotificationChannel(MESSAGE_CHANNEL, "infOS 消息", NotificationManager.IMPORTANCE_DEFAULT))
        }
    }
    fun synchronize(context: Context) {
        if (!(context.getSystemService(Context.USER_SERVICE) as UserManager).isUserUnlocked) return
        val plan = RemoteReceiverBridge.execute(context, "plan")
        if (!plan.optBoolean("active")) { stop(context); return }
        channels(context)
        // 仅传递不可执行的代次身份；服务自行核对数据库及读取 Keystore 凭据。
        ContextCompat.startForegroundService(context, Intent(context, RemoteNotificationService::class.java)
            .putExtra("modeToken", plan.getJSONObject("modeToken").toString()))
    }
    fun stop(context: Context) {
        context.stopService(Intent(context, RemoteNotificationService::class.java))
        val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        manager.activeNotifications.filter { it.tag?.startsWith("ppc.remote.") == true }.forEach { manager.cancel(it.tag, it.id) }
    }
    fun visible(context: Context, visible: Boolean) {
        executor.execute { try { RemoteReceiverBridge.execute(context, "visible", JSONObject().put("visible", visible)) } catch (_: Exception) { /* 闭门期间不恢复任何连接。 */ } }
    }
    fun open(context: Context, intent: Intent?) {
        if (intent?.action != OPEN) return
        try {
            val input = JSONObject(intent.getStringExtra("notice") ?: return)
            RemoteReceiverBridge.execute(context, "open", input)
        } catch (_: Exception) { /* 过期通知不能切换目标或打开旧实例。 */ }
        intent.removeExtra("notice")
    }
}
