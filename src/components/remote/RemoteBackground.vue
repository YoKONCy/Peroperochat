<template>
  <section class="remote-background" aria-label="后台通知">
    <div class="remote-background-heading"><span><RoomIcon name="bell" /><strong>后台接收</strong></span><small>{{ label }}</small></div>
    <p v-if="runtime?.supported">关闭界面后，由轻量原生服务接收当前角色的 infOS 持久通知。无需 Google 服务；切回独立模式会停止连接。</p>
    <p v-else-if="runtime">{{ runtime.platform === 'ios' ? 'iOS 后台推送尚未接入，当前仅支持应用在线接收。' : '此平台当前仅支持应用在线接收。' }}</p>
    <template v-if="runtime?.supported">
      <p v-if="!runtime.eligible" class="remote-background-hint">请先使用「保持登录」重新授权，后台接收不会延长设备授权期限。</p>
      <p v-if="!runtime.notifications" class="remote-background-hint">系统通知未开启，消息可能无法显示。</p>
      <p v-if="!runtime.batteryUnrestricted" class="remote-background-hint">建议允许后台运行并取消电池限制；部分手机还需要手动允许自启动。</p>
      <p v-if="runtime.blocked" class="remote-background-hint">连接已暂停，请核对授权或服务端版本后重试。</p>
      <div class="remote-background-actions"><button class="room-secondary-button" :disabled="busy || !runtime.eligible && !runtime.enabled" @click="configure(!runtime.enabled)">{{ busy ? '正在处理…' : runtime.enabled ? '关闭后台接收' : '开启后台接收' }}</button><button v-if="runtime.enabled && (runtime.blocked || runtime.state === 'stopped')" class="room-text-button" :disabled="busy" @click="configure(true)">重新连接</button><button class="room-text-button" :disabled="busy" @click="settings('notifications')">通知设置</button><button class="room-text-button" :disabled="busy" @click="settings('battery')">后台设置</button></div>
      <p class="remote-background-note">运行时会保留连接通知。系统强行停止或限制后台时，恢复连接后补收服务端保留的通知。</p>
    </template>
    <p v-if="error" class="room-error" role="alert">{{ error }}</p><button v-if="error && !runtime" class="room-text-button" @click="refresh">重新读取</button>
  </section>
</template>
<script setup>
import { computed, onMounted, onBeforeUnmount, ref } from 'vue'
import { isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification'
import { remoteClient } from '../../services/remote'
import { currentToken, assertToken } from '../../stores/mode'
import RoomIcon from './RemoteIcon.vue'
const token = currentToken(), runtime = ref(null), busy = ref(false), error = ref('')
let disposed = false, timer = null, refreshing = false
const label = computed(() => {
  if (!runtime.value) return '正在读取…'
  if (!runtime.value.supported) return '暂未支持'
  if (!runtime.value.enabled) return '未开启'
  if (runtime.value.blocked) return '需要处理'
  return { online: '已连接', connecting: '正在连接', reconnecting: '等待重连', stopped: '等待启动' }[runtime.value.state] || '等待连接'
})
async function refresh() {
  if (disposed || refreshing || document.hidden) return
  refreshing = true
  try { const result = await remoteClient.backgroundRuntime(token); assertToken(token, 'remote'); if (!disposed) { runtime.value = result.data; error.value = '' } }
  catch (failure) { if (!disposed) error.value = failure.message }
  finally { refreshing = false }
}
async function configure(enabled) {
  if (busy.value) return
  busy.value = true; error.value = ''
  try {
    if (enabled && !await isPermissionGranted() && await requestPermission() !== 'granted') throw new Error('请先允许系统通知，再开启后台接收')
    assertToken(token, 'remote')
    const result = await remoteClient.configureBackground(enabled, token)
    if (!disposed) runtime.value = result.data
  } catch (failure) { if (!disposed) error.value = failure.message }
  finally { if (!disposed) busy.value = false }
}
async function settings(kind) { try { await remoteClient.backgroundSettings(kind, token) } catch (failure) { if (!disposed) error.value = failure.message } }
onMounted(() => { refresh(); timer = setInterval(refresh, 5000); window.addEventListener('focus', refresh) })
onBeforeUnmount(() => { disposed = true; clearInterval(timer); window.removeEventListener('focus', refresh) })
</script>
<style scoped>
.remote-background { margin: 0 0 18px; padding: 15px; border: 1px solid var(--room-line); border-radius: 14px; background: var(--room-subtle); }.remote-background-heading, .remote-background-heading > span { display: flex; align-items: center; gap: 7px; }.remote-background-heading { justify-content: space-between; }.remote-background-heading svg { width: 15px; height: 15px; color: var(--room-accent); }.remote-background-heading strong { font-size: 12px; font-weight: 600; }.remote-background-heading small { font-size: 10px; color: var(--room-accent); }.remote-background p { font-size: 11px; color: var(--room-muted); line-height: 1.8; margin: 10px 0 0; }.remote-background .remote-background-hint { color: var(--room-ink); }.remote-background-actions { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; margin-top: 13px; }.remote-background .remote-background-note { font-size: 10px; }
</style>
