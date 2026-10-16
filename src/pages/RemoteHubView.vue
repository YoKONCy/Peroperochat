<template>
  <div class="remote-hub" :style="viewportStyle">

    <!-- 顶部状态栏 -->
    <CompanionHeader title="远程连接" :subtitle="appMode.remoteConfig.connected ? '已连接到 PeroCore' : '未连接'" back-label="返回房间" @back="handleExitRemote" />

    <!-- 连接面板 (未连接时显示) -->
    <div v-if="!appMode.remoteConfig.connected" class="connection-panel">
      <div class="pixel-card">
        <h2 class="pixel-title">远程连接设置</h2>
        <div class="pixel-form">
          <div class="input-group">
            <label>服务器地址</label>
            <input v-model="config.url" placeholder="http://192.168.x.x:9120" class="pixel-input" />
          </div>
          <div class="input-group">
            <label>访问令牌</label>
            <input
              v-model="config.token"
              type="password"
              placeholder="pero_default_token"
              class="pixel-input"
            />
          </div>
          <div class="btn-row">
            <button class="pixel-btn connect-btn" @click="handleConnect" :disabled="isConnecting">
              {{ isConnecting ? '连接中...' : '连接到本体' }}
            </button>
            <button class="pixel-btn scan-btn" @click="startScan" title="扫码连接">
              <RoomIcon name="scan" />
            </button>
          </div>
        </div>
        <p class="hint">请在 PC 版 PeroCore 的设置中查看连接信息</p>
      </div>
    </div>

    <!-- 功能导航 (已连接时显示) -->
    <div v-else class="remote-main">
      <div class="hub-grid">
        <!-- 单聊模式入口 -->
        <button class="hub-card chat-card pixel-border" @click="activeView = 'chat'">
          <div class="card-icon">
            <RoomIcon name="chat" />
          </div>
          <div class="card-title">远程对话</div>
          <div class="card-desc">与当前活跃角色聊天</div>
        </button>

        <!-- 据点模式入口 -->
        <button class="hub-card stronghold-card pixel-border" @click="activeView = 'stronghold'">
          <div class="card-icon">
            <RoomIcon name="group" />
          </div>
          <div class="card-title">远程据点</div>
          <div class="card-desc">管理据点成员与群聊</div>
        </button>
      </div>

      <!-- 底部控制面板 -->
      <div class="remote-footer pixel-border-sm">
        <div class="current-agent">
          <img
            v-if="activeAgent?.avatar"
            :src="getFullUrl(activeAgent.avatar)"
            class="agent-thumb pixel-border-sm"
          />
          <div class="agent-info">
            <div class="agent-name">{{ activeAgent?.name || '同步中...' }}</div>
            <div class="agent-status">在线</div>
          </div>
        </div>
        <button class="pixel-btn-icon" aria-label="刷新远程状态" @click="fetchRemoteStatus">
          <RoomIcon name="refresh" :class="{ 'is-spinning': isFetching }" />
        </button>
      </div>
    </div>

    <!-- 全屏视图：对话/据点 (覆盖层) -->
    <Transition name="slide-up">
      <div v-if="activeView" class="remote-overlay">
        <CompanionHeader :title="activeView === 'chat' ? '远程对话' : '远程据点'" back-label="返回远程连接" @back="activeView = null" />

        <!-- 这里挂载具体的远程视图 -->
        <div class="overlay-content">
          <RemoteChatView
            v-if="activeView === 'chat'"
            :pc-url="appMode.remoteConfig.url"
            :token="appMode.remoteConfig.token"
          />
          <RemoteStrongholdView
            v-if="activeView === 'stronghold'"
            :pc-url="appMode.remoteConfig.url"
            :token="appMode.remoteConfig.token"
          />
        </div>
      </div>
    </Transition>

    <!-- 扫码弹窗 -->
    <Transition name="fade">
      <div v-if="showScanner" class="scan-overlay">
        <div class="scan-container pixel-border">
          <div class="scan-header">
            <span>正在扫码...</span>
            <button class="close-btn" aria-label="关闭扫码" @click="stopScan">
              <RoomIcon name="close" />
            </button>
          </div>
          <div id="qr-reader" class="qr-reader"></div>
          <p class="scan-hint">请对准 PC 端的连接二维码</p>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, onUnmounted } from 'vue'
import { appMode, setAppMode, saveRemoteConfig } from '../api'
import RemoteChatView from '../components/remote/RemoteChatView.vue'
import RemoteStrongholdView from '../components/remote/RemoteStrongholdView.vue'
import { Html5Qrcode } from 'html5-qrcode'
import CompanionHeader from '../components/companion/CompanionHeader.vue'
import RoomIcon from '../components/companion/RoomIcon.vue'
import { useRoomViewport } from '../composables/useRoomViewport'
const { viewportStyle } = useRoomViewport()

const config = reactive({
  url: appMode.remoteConfig.url || '',
  token: appMode.remoteConfig.token || ''
})

const isConnecting = ref(false)
const isFetching = ref(false)
const activeView = ref(null) // 'chat' | 'stronghold'
const activeAgent = ref(null)

// 扫码相关
const showScanner = ref(false)
let html5QrCode = null

const startScan = async () => {
  showScanner.value = true
  // 等待 DOM 渲染
  setTimeout(async () => {
    try {
      html5QrCode = new Html5Qrcode('qr-reader')
      const config = { fps: 10, qrbox: { width: 250, height: 250 } }

      await html5QrCode.start(
        { facingMode: 'environment' },
        config,
        (decodedText) => {
          handleScanSuccess(decodedText)
        },
        () => {
          // 忽略扫描过程中的错误
        }
      )
    } catch (err) {
      console.error('扫码初始化失败:', err)
      alert('无法打开摄像头，请检查权限')
      showScanner.value = false
    }
  }, 300)
}

const stopScan = async () => {
  if (html5QrCode && html5QrCode.isScanning) {
    await html5QrCode.stop()
    await html5QrCode.clear()
  }
  showScanner.value = false
}

const handleScanSuccess = async (text) => {
  // 协议解析：perolink://{ip}:{port}#{token}
  if (text.startsWith('perolink://')) {
    try {
      const mainPart = text.replace('perolink://', '')
      const [addrPart, tokenPart] = mainPart.split('#')

      config.url = `http://${addrPart}`
      config.token = tokenPart

      await stopScan()
      handleConnect()
    } catch {
      alert('二维码格式错误')
    }
  } else {
    alert('识别到非法的连接二维码')
  }
}

onUnmounted(() => {
  stopScan()
})

const getFullUrl = (path) => {
  if (!path) return ''
  if (path.startsWith('http')) return path
  return `${appMode.remoteConfig.url.replace('/api', '')}${path}`
}

const handleConnect = async () => {
  if (!config.url) return
  isConnecting.value = true
  try {
    const res = await fetch(`${config.url}/api/agents/active`, {
      headers: { Authorization: `Bearer ${config.token}` }
    })
    if (res.ok) {
      saveRemoteConfig({ url: config.url, token: config.token, connected: true })
      fetchRemoteStatus()
    } else {
      alert('连接失败：请检查地址和令牌是否正确')
    }
  } catch {
    alert('连接失败：无法访问服务器，请确保 PC 版 PeroCore 已开启并处于同一网络')
  } finally {
    isConnecting.value = false
  }
}

const handleExitRemote = () => {
  setAppMode('standalone')
  window.location.reload()
}

const fetchRemoteStatus = async () => {
  if (!appMode.remoteConfig.connected) return
  isFetching.value = true
  try {
    const res = await fetch(`${appMode.remoteConfig.url}/api/agents`, {
      headers: { Authorization: `Bearer ${appMode.remoteConfig.token}` }
    })
    if (res.ok) {
      const agents = await res.json()
      activeAgent.value = agents.find((a) => a.is_active)
    }
  } catch {
    saveRemoteConfig({ connected: false })
  } finally {
    isFetching.value = false
  }
}

onMounted(() => {
  if (appMode.remoteConfig.connected) {
    fetchRemoteStatus()
  }
})
</script>

<style scoped>
.remote-hub { position: fixed; inset: var(--room-top, 0px) 0 auto; height: var(--room-height, 100dvh); display: flex; flex-direction: column; background: var(--room-page); overflow-y: auto; }
.connection-panel { width: min(calc(100% - 32px), 520px); margin: 40px auto; }
.pixel-card { padding: 28px; border: 1px solid var(--room-line); border-radius: 18px; background: var(--room-surface); }
.pixel-title { font-size: 18px; font-weight: 600; margin: 0 0 26px; color: var(--room-ink); }
.input-group { display: flex; flex-direction: column; gap: 10px; margin-bottom: 22px; }
.input-group label { color: var(--room-ink); font-size: 13px; }
.pixel-input { width: 100%; min-height: 44px; padding: 11px 13px; border: 1px solid var(--room-line); border-radius: 10px; background: var(--room-surface); color: var(--room-ink); font-size: 14px; outline: none; }
.pixel-input:focus { border-color: var(--room-accent); box-shadow: 0 0 0 3px var(--room-accent-soft); }
.btn-row { display: flex; gap: 10px; }
.pixel-btn { display: inline-flex; align-items: center; justify-content: center; gap: 8px; min-height: 42px; padding: 11px 16px; border: 1px solid var(--room-line); border-radius: 10px; background: var(--room-surface); color: var(--room-ink); }
.connect-btn { flex: 1; background: var(--room-accent); border-color: transparent; color: white !important; }
.scan-btn { width: 44px; padding: 10px; }
.hint, .scan-hint { font-size: 12px; color: var(--room-muted); line-height: 1.7; margin: 18px 0 0; }
.remote-main { width: min(calc(100% - 32px), 800px); margin: 32px auto; }
.hub-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
.hub-card { padding: 30px 24px; border: 1px solid var(--room-line); border-radius: 16px; background: var(--room-surface); text-align: left; }
.hub-card:hover { border-color: var(--room-accent); }
.card-icon { display: grid; place-items: center; width: 42px; height: 42px; border-radius: 12px; background: var(--room-accent-soft); color: var(--room-accent); margin-bottom: 24px; }
.card-title { font-size: 17px; font-weight: 600; }
.card-desc { margin-top: 9px; font-size: 12px; color: var(--room-muted); line-height: 1.7; }
.remote-footer { display: flex; align-items: center; justify-content: space-between; padding: 18px; border: 1px solid var(--room-line); border-radius: 15px; background: var(--room-surface); margin-top: 18px; }
.current-agent { display: flex; align-items: center; gap: 12px; }
.agent-thumb { width: 40px; height: 44px; border-radius: 12px; object-fit: cover; }
.agent-name { font-size: 13px; font-weight: 500; }
.agent-status { margin-top: 5px; font-size: 11px; color: var(--room-muted); }
.pixel-btn-icon, .close-btn { display: grid; place-items: center; width: 38px; height: 38px; border: 0; border-radius: 10px; background: var(--room-subtle); color: var(--room-muted); }
.remote-overlay { position: absolute; inset: 0; z-index: 100; display: flex; flex-direction: column; background: var(--room-page); }
.overlay-content { flex: 1; min-height: 0; overflow: hidden; }
.scan-overlay { position: absolute; inset: 0; z-index: 200; display: flex; align-items: center; justify-content: center; padding: 20px; background: #202c4255; }
.scan-container { width: min(100%, 400px); max-height: 100%; overflow-y: auto; padding: 22px; border: 1px solid var(--room-line); border-radius: 18px; background: var(--room-surface); }
.scan-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; font-size: 15px; font-weight: 500; }
.qr-reader { width: 100%; border-radius: 12px; overflow: hidden; }
.slide-up-enter-active, .slide-up-leave-active, .fade-enter-active, .fade-leave-active { transition: opacity .2s; }
.slide-up-enter-from, .slide-up-leave-to, .fade-enter-from, .fade-leave-to { opacity: 0; }
.is-spinning { animation: remote-spin 1s linear infinite; }
@keyframes remote-spin { to { transform: rotate(360deg); } }
@media (max-width: 480px) { .hub-grid { grid-template-columns: 1fr; } .pixel-card { padding: 22px; } .pixel-input { font-size: 16px; } }
</style>
