import { createApp } from 'vue'
import App from './App.vue'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import router from './router/index.js'
import { initLocalDB } from './db'
import { listen } from '@tauri-apps/api/event'
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'

// Initialize Local IndexedDB
initLocalDB().catch(console.error)

const app = createApp(App)
app.use(ElementPlus)
app.use(router)
app.mount('#app')

if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js').catch(() => {})
  })
}

// 后台提醒事件监听：当 Rust 端检测到提醒到期时，发送系统通知
;(async () => {
  try {
    let granted = await isPermissionGranted()
    if (!granted) {
      const p = await requestPermission()
      granted = p === 'granted'
    }
    await listen('ppc:reminder-due', (event) => {
      const payload = event?.payload || {}
      const title = '提醒'
      const body = payload.task ? `${payload.task}（${payload.time || ''}）` : '有新的提醒到期'
      if (granted) {
        sendNotification({ title, body })
      }
    })
  } catch (e) {
    // 忽略通知权限或监听失败
    console.warn('通知权限或监听初始化失败:', e)
  }
})()
