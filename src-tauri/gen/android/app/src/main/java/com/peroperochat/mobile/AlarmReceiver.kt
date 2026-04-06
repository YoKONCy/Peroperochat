package com.peroperochat.mobile

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.core.app.NotificationCompat

/**
 * 系统级闹铃广播接收器
 * 当 AlarmManager 触发时，此 Receiver 会被系统唤醒（即使 App 已被杀死），
 * 然后在通知栏展示提醒通知。
 */
class AlarmReceiver : BroadcastReceiver() {

    companion object {
        const val CHANNEL_ID = "ppc_reminder_channel"
        const val EXTRA_TASK = "ppc_task"
        const val EXTRA_AGENT = "ppc_agent"
        const val EXTRA_REMINDER_ID = "ppc_reminder_id"
    }

    override fun onReceive(context: Context, intent: Intent) {
        val task = intent.getStringExtra(EXTRA_TASK) ?: "你有一个待办事项"
        val agentName = intent.getStringExtra(EXTRA_AGENT) ?: "Pero"
        val reminderId = intent.getIntExtra(EXTRA_REMINDER_ID, 0)

        // 确保通知通道已创建 (Android 8.0+)
        createNotificationChannel(context)

        // 构建点击通知后打开 App 的 Intent
        val openAppIntent = Intent(context, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP
        }
        val pendingIntent = PendingIntent.getActivity(
            context,
            reminderId,
            openAppIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        // 构建通知
        val notification = NotificationCompat.Builder(context, CHANNEL_ID)
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setContentTitle("${agentName}的提醒")
            .setContentText(task)
            .setStyle(NotificationCompat.BigTextStyle().bigText(task))
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setAutoCancel(true)
            .setContentIntent(pendingIntent)
            .setDefaults(NotificationCompat.DEFAULT_ALL)
            .build()

        // 发送通知
        val notificationManager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        notificationManager.notify(reminderId, notification)
    }

    private fun createNotificationChannel(context: Context) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val name = "Pero的任务提醒"
            val descriptionText = "来自PeroperoChat角色的定时任务提醒通知"
            val importance = NotificationManager.IMPORTANCE_HIGH
            val channel = NotificationChannel(CHANNEL_ID, name, importance).apply {
                description = descriptionText
                enableVibration(true)
            }
            val notificationManager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            notificationManager.createNotificationChannel(channel)
        }
    }
}
