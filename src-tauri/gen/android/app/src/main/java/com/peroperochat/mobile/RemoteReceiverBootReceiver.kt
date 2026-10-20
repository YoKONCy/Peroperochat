package com.peroperochat.mobile

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log

/** 只恢复持久选择的远程模式，不初始化或切换运行目标。 */
class RemoteReceiverBootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action !in setOf(Intent.ACTION_BOOT_COMPLETED, Intent.ACTION_MY_PACKAGE_REPLACED, Intent.ACTION_USER_UNLOCKED)) return
        val pending = goAsync()
        RemoteReceiverController.executor.execute {
            try { RemoteReceiverController.synchronize(context.applicationContext) }
            catch (_: Exception) { Log.w("远程通知", "系统未允许恢复后台接收，等待用户打开应用") }
            finally { pending.finish() }
        }
    }
}
