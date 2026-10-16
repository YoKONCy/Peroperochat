package com.peroperochat.mobile

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.PowerManager

/** 系统唤醒后直接核对 Rust 任务并发送通知，不依赖界面和前端事件。 */
class AlarmReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val id = intent.getLongExtra("id", 0)
        val token = intent.getStringExtra("token") ?: return
        if (id <= 0 || token.length > 200) return
        val result = goAsync()
        val app = context.applicationContext
        val lock = (app.getSystemService(Context.POWER_SERVICE) as PowerManager)
                .newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "PeroperoChat:提醒处理")
        lock.acquire(30_000)
        ReminderScheduler.executor.execute {
            try { ReminderScheduler.fire(app, id, token) }
            catch (error: Throwable) { ReminderScheduler.failed(app, error) }
            finally { try { if (lock.isHeld) lock.release() } finally { result.finish() } }
        }
    }
}
