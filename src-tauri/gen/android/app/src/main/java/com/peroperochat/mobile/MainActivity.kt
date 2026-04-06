package com.peroperochat.mobile

import android.app.AlarmManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.util.Log
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {

    companion object {
        private const val TAG = "PPC_MainActivity"
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
    }

    /**
     * 注册系统级精确闹铃
     * 由 WebView JS 层通过 window.__TAURI__.event.listen 接收 Rust emit 的事件后，
     * 再通过 Android JavascriptInterface 或直接在原生层处理。
     * 
     * 这里提供静态工具方法，供 AlarmHelper 调用。
     */
    object AlarmHelper {
        private const val TAG = "PPC_AlarmHelper"

        fun scheduleAlarm(context: Context, reminderId: Int, triggerAtMs: Long, task: String, agentName: String) {
            val alarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager

            val intent = Intent(context, AlarmReceiver::class.java).apply {
                putExtra(AlarmReceiver.EXTRA_TASK, task)
                putExtra(AlarmReceiver.EXTRA_AGENT, agentName)
                putExtra(AlarmReceiver.EXTRA_REMINDER_ID, reminderId)
            }

            val pendingIntent = PendingIntent.getBroadcast(
                context,
                reminderId,
                intent,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
            )

            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                if (alarmManager.canScheduleExactAlarms()) {
                    alarmManager.setExactAndAllowWhileIdle(
                        AlarmManager.RTC_WAKEUP, triggerAtMs, pendingIntent
                    )
                } else {
                    alarmManager.setAndAllowWhileIdle(
                        AlarmManager.RTC_WAKEUP, triggerAtMs, pendingIntent
                    )
                    Log.w(TAG, "没有精确闹铃权限，已降级为不精确闹铃")
                }
            } else {
                alarmManager.setExactAndAllowWhileIdle(
                    AlarmManager.RTC_WAKEUP, triggerAtMs, pendingIntent
                )
            }

            Log.i(TAG, "闹铃已注册: id=$reminderId, time=$triggerAtMs, task=$task")
        }

        fun cancelAlarm(context: Context, reminderId: Int) {
            val alarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
            val intent = Intent(context, AlarmReceiver::class.java)
            val pendingIntent = PendingIntent.getBroadcast(
                context, reminderId, intent,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
            )
            alarmManager.cancel(pendingIntent)
            pendingIntent.cancel()
            Log.i(TAG, "闹铃已取消: id=$reminderId")
        }
    }
}
