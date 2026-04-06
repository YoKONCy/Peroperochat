package com.peroperochat.mobile

import android.os.Bundle
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)

    // 注册闹铃插件，使 Rust 侧可以通过 invoke 调用 scheduleAlarm / cancelAlarm
    registerPlugin(AlarmPlugin::class.java)
  }
}
