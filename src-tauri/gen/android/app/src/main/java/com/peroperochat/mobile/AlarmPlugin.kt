package com.peroperochat.mobile

import android.app.AlarmManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Build
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

/**
 * Tauri 原生插件：安卓系统级闹铃管理
 * 
 * 功能：
 * 1. scheduleAlarm  — 注册一个精确闹铃，到时后触发 AlarmReceiver 发送系统通知
 * 2. cancelAlarm    — 取消指定的闹铃
 * 
 * 使用 AlarmManager.setExactAndAllowWhileIdle()，
 * 即使设备处于 Doze 省电模式，也能准时触发。
 */
@TauriPlugin
class AlarmPlugin(private val activity: android.app.Activity) : Plugin(activity) {

    companion object {
        private const val TAG = "PPC_AlarmPlugin"
    }

    /**
     * 注册系统级精确闹铃
     * 参数 (通过 Invoke 传入):
     *   reminderId: Int     — 提醒的唯一 ID（用于后续取消）
     *   triggerAtMs: Long   — 触发时间的毫秒时间戳 (System.currentTimeMillis 格式)
     *   task: String        — 提醒任务的文本内容
     *   agentName: String   — 角色名 (如 "Pero")
     */
    @Command
    fun scheduleAlarm(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ScheduleAlarmArgs::class.java)
            
            val context = activity.applicationContext
            val alarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager

            // 构建闹铃触发时要发送的 Intent
            val intent = Intent(context, AlarmReceiver::class.java).apply {
                putExtra(AlarmReceiver.EXTRA_TASK, args.task)
                putExtra(AlarmReceiver.EXTRA_AGENT, args.agentName)
                putExtra(AlarmReceiver.EXTRA_REMINDER_ID, args.reminderId)
            }

            val pendingIntent = PendingIntent.getBroadcast(
                context,
                args.reminderId,
                intent,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
            )

            // 使用 setExactAndAllowWhileIdle —— 即使 Doze 模式也能触发
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                // Android 12+ 需要检查是否有精确闹铃权限
                if (alarmManager.canScheduleExactAlarms()) {
                    alarmManager.setExactAndAllowWhileIdle(
                        AlarmManager.RTC_WAKEUP,
                        args.triggerAtMs,
                        pendingIntent
                    )
                } else {
                    // 降级为不精确闹铃
                    alarmManager.setAndAllowWhileIdle(
                        AlarmManager.RTC_WAKEUP,
                        args.triggerAtMs,
                        pendingIntent
                    )
                    Log.w(TAG, "没有精确闹铃权限，已降级为不精确闹铃")
                }
            } else {
                alarmManager.setExactAndAllowWhileIdle(
                    AlarmManager.RTC_WAKEUP,
                    args.triggerAtMs,
                    pendingIntent
                )
            }

            Log.i(TAG, "闹铃已注册: id=${args.reminderId}, time=${args.triggerAtMs}, task=${args.task}")
            invoke.resolve()
        } catch (e: Exception) {
            Log.e(TAG, "注册闹铃失败", e)
            invoke.reject(e.message ?: "注册闹铃失败")
        }
    }

    /**
     * 取消已注册的闹铃
     * 参数:
     *   reminderId: Int — 要取消的提醒 ID
     */
    @Command
    fun cancelAlarm(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(CancelAlarmArgs::class.java)

            val context = activity.applicationContext
            val alarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager

            val intent = Intent(context, AlarmReceiver::class.java)
            val pendingIntent = PendingIntent.getBroadcast(
                context,
                args.reminderId,
                intent,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
            )

            alarmManager.cancel(pendingIntent)
            pendingIntent.cancel()

            Log.i(TAG, "闹铃已取消: id=${args.reminderId}")
            invoke.resolve()
        } catch (e: Exception) {
            Log.e(TAG, "取消闹铃失败", e)
            invoke.reject(e.message ?: "取消闹铃失败")
        }
    }
}

// 参数数据类
data class ScheduleAlarmArgs(
    val reminderId: Int,
    val triggerAtMs: Long,
    val task: String,
    val agentName: String
)

data class CancelAlarmArgs(
    val reminderId: Int
)
