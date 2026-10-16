import { createApp } from 'vue'
import ElementPlus, { ElMessage } from 'element-plus'
import 'element-plus/dist/index.css'
import App from './App.vue'
import router from './router/index.js'
import { loadAgents } from './stores/agents'

// 初始化角色后再创建页面，避免首页先读取默认角色的历史。
async function bootstrap() {
  try {
    await loadAgents()
  } catch (error) {
    console.error('单机数据初始化失败：', error)
    ElMessage.error({ message: error.message || '初始化失败，请通过 Tauri 启动应用', duration: 0 })
  }
  const app = createApp(App)
  app.use(ElementPlus)
  app.use(router)
  app.mount('#app')
}
bootstrap()

if (!('__TAURI_INTERNALS__' in window) && 'serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js').catch(() => {})
  })
}
