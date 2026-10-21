import { createApp } from 'vue'
import ElementPlus from 'element-plus'
import { appMessage as ElMessage } from './services/appFeedback'
import 'element-plus/dist/index.css'
import App from './App.vue'
import router from './router/index.js'
import { loadAgents } from './stores/agents'
import { loadAppearance } from './stores/appearance'
import { watch } from 'vue'
import { initializeMode, modeState, refreshMode } from './stores/mode'
import { initializeRemote } from './stores/remote'

let restoredKey = null
let restoration = null

// 先读取原生运行权威，再恢复对应模式；远程断线不会触发独立数据初始化。
async function bootstrap() {
  try {
    await initializeMode()
  } catch (error) {
    console.error('运行模式初始化失败：', error)
    ElMessage.error({ message: error.message || '初始化失败，请通过 Tauri 启动应用', duration: 0 })
  }
  watch(() => [modeState.snapshot, modeState.restoreAttempt], () => {
    const requested = modeState.snapshot, attempt = modeState.restoreAttempt
    restoreTarget().catch(error => {
      if (modeState.snapshot?.installationId !== requested?.installationId || modeState.snapshot?.revision !== requested?.revision || modeState.restoreAttempt !== attempt || modeState.snapshot?.phase !== 'stable') return
      console.error('模式数据恢复失败：', error)
      if (modeState.snapshot.activeTarget?.mode === 'standalone') modeState.restoreError = error.message || '独立数据尚未恢复'
      else ElMessage.error(error.message || '当前模式尚未就绪')
    })
  }, { immediate: true })
  const app = createApp(App)
  app.use(ElementPlus)
  app.use(router)
  app.mount('#app')
}
async function restoreTarget() {
  const snapshot = modeState.snapshot
  if (snapshot?.phase !== 'stable') return
  const key = `${snapshot.installationId}:${snapshot.revision}:${modeState.restoreAttempt}`
  if (restoredKey === key) return
  if (restoration?.key === key) return restoration.promise
  const local = snapshot.activeTarget?.mode === 'standalone'
  modeState.restoring = local
  modeState.restoreError = null
  const promise = (async () => {
    if (snapshot.activeTarget?.mode === 'standalone') {
      await loadAgents()
      // 显示偏好读取失败不阻断已有对话；页面提供重试入口。
      await loadAppearance().catch(error => { console.warn('显示偏好暂未恢复：', error) })
    }
    else if (snapshot.activeTarget?.mode === 'remote') await initializeRemote()
    if (modeState.snapshot?.installationId === snapshot.installationId && modeState.snapshot?.revision === snapshot.revision) restoredKey = key
  })()
  restoration = { key, promise }
  try { await promise } finally {
    if (restoration?.key === key) { restoration = null; modeState.restoring = false }
  }
}
bootstrap()

// 重新显示窗口时核对原生所有权；另一个应用进程接管后旧窗口不能继续提交业务。
window.addEventListener('focus', () => { refreshMode().catch(() => {}) })
document.addEventListener('visibilitychange', () => { if (!document.hidden) refreshMode().catch(() => {}) })

if (!('__TAURI_INTERNALS__' in window) && 'serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js').catch(() => {})
  })
}
