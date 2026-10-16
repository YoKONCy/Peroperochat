package com.peroperochat.mobile

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.PowerManager
import android.os.UserManager
import java.io.File

/** 开机解锁、更新、校时和授权后，直接从数据库恢复系统登记。 */
class BootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val allowed = setOf(Intent.ACTION_BOOT_COMPLETED, Intent.ACTION_MY_PACKAGE_REPLACED,
            Intent.ACTION_TIME_CHANGED, Intent.ACTION_TIMEZONE_CHANGED, Intent.ACTION_USER_UNLOCKED,
            "android.app.action.SCHEDULE_EXACT_ALARM_PERMISSION_STATE_CHANGED", ReminderScheduler.RETRY_ACTION)
        if (intent.action !in allowed) return
        if (!(context.getSystemService(Context.USER_SERVICE) as UserManager).isUserUnlocked) return
        if (!File(context.applicationInfo.dataDir, "data/peropero.db").isFile) return
        val result = goAsync()
        val app = context.applicationContext
        val lock = (app.getSystemService(Context.POWER_SERVICE) as PowerManager)
                .newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "PeroperoChat:提醒恢复")
        lock.acquire(30_000)
        ReminderScheduler.executor.execute {
            try { ReminderScheduler.synchronize(app, force = true) }
            catch (error: Throwable) { ReminderScheduler.failed(app, error) }
            finally { try { if (lock.isHeld) lock.release() } finally { result.finish() } }
        }
    }
}
