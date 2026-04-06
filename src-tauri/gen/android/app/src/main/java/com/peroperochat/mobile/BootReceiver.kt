package com.peroperochat.mobile

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log

/**
 * 开机完成广播接收器
 * 安卓系统重启后 AlarmManager 注册的闹铃会全部丢失，
 * 此 Receiver 在开机完成时被唤醒，通知 App 重新注册所有未过期的闹铃。
 * 实际的重新注册逻辑由 Rust 后端在 App 启动时完成。
 */
class BootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == Intent.ACTION_BOOT_COMPLETED) {
            Log.i("PPC_BootReceiver", "设备重启完成，等待 App 启动时重新注册闹铃")
            // 可选：在此启动一个前台 Service 来确保 App 进程被拉起
            // 目前依赖用户下次打开 App 时，Rust 侧自动重新注册
        }
    }
}
