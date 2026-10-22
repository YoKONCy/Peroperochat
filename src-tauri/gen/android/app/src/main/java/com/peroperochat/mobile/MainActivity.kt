package com.peroperochat.mobile

import android.os.Bundle
import android.content.Intent
import android.os.Handler
import android.os.Looper
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
    override fun onWebViewCreate(webView: WebView) {
        super.onWebViewCreate(webView)
        // 禁止系统缩放整个应用界面，内容预览仍由页面内控件缩放。
        webView.settings.setSupportZoom(false)
        webView.settings.builtInZoomControls = false
        webView.settings.displayZoomControls = false
    }
    private val visibilityHandler = Handler(Looper.getMainLooper())
    private val visibilityHeartbeat = object : Runnable {
        override fun run() {
            RemoteReceiverController.visible(applicationContext, true)
            visibilityHandler.postDelayed(this, 15_000)
        }
    }
    override fun onCreate(savedInstanceState: Bundle?) {
        // 在 Tauri 更新运行代次前验证通知点击，随后由新界面消费一次。
        RemoteReceiverController.open(applicationContext, intent)
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
    }
    override fun onNewIntent(intent: Intent) {
        RemoteReceiverController.open(applicationContext, intent)
        super.onNewIntent(intent)
        setIntent(intent)
    }
    override fun onResume() {
        super.onResume()
        visibilityHandler.removeCallbacks(visibilityHeartbeat)
        visibilityHandler.post(visibilityHeartbeat)
        RemoteReceiverController.executor.execute {
            try { RemoteReceiverController.synchronize(applicationContext) }
            catch (_: Exception) { /* 系统暂未允许启动时，连接设置保留重试入口。 */ }
        }
    }
    override fun onPause() {
        visibilityHandler.removeCallbacks(visibilityHeartbeat)
        RemoteReceiverController.visible(applicationContext, false)
        super.onPause()
    }
}
