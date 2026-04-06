<template>
  <div class="remote-hub pixel-font">
    <!-- 背景装饰：流动的像素点 -->
    <div class="pixel-bg"></div>

    <!-- 顶部状态栏 -->
    <div class="remote-header pixel-border-sm">
      <div class="status-indicator">
        <div :class="['status-dot', appMode.remoteConfig.connected ? 'online' : 'offline']"></div>
        <span>{{
          appMode.remoteConfig.connected ? '已连接到 PeroCore' : '远程模式 (未连接)'
        }}</span>
      </div>
      <button class="pixel-btn exit-btn" @click="handleExitRemote">
        <i class="fas fa-power-off"></i> 退出
      </button>
    </div>

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
              <i class="fas fa-qrcode"></i>
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
        <div class="hub-card chat-card pixel-border" @click="activeView = 'chat'">
          <div class="card-icon">
            <i class="fas fa-comment-dots"></i>
          </div>
          <div class="card-title">远程对话</div>
          <div class="card-desc">与当前活跃角色聊天</div>
        </div>

        <!-- 据点模式入口 -->
        <div class="hub-card stronghold-card pixel-border" @click="activeView = 'stronghold'">
          <div class="card-icon">
            <i class="fas fa-fort-awesome"></i>
          </div>
          <div class="card-title">远程据点</div>
          <div class="card-desc">管理据点成员与群聊</div>
        </div>
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
        <button class="pixel-btn-icon" @click="fetchRemoteStatus">
          <i class="fas fa-sync-alt" :class="{ 'fa-spin': isFetching }"></i>
        </button>
      </div>
    </div>

    <!-- 全屏视图：对话/据点 (覆盖层) -->
    <Transition name="slide-up">
      <div v-if="activeView" class="remote-overlay">
        <div class="overlay-header">
          <button class="back-btn" @click="activeView = null">
            <i class="fas fa-chevron-left"></i> 返回枢纽
          </button>
          <div class="overlay-title">{{ activeView === 'chat' ? '远程对话' : '远程据点' }}</div>
        </div>

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
            <button class="close-btn" @click="stopScan">
              <i class="fas fa-times"></i>
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
.remote-hub {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: #1a1a1a;
  color: #fff;
  z-index: 1000;
  display: flex;
  flex-direction: column;
}

.pixel-bg {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-image: radial-gradient(#333 1px, transparent 0);
  background-size: 20px 20px;
  z-index: -1;
  opacity: 0.5;
}

/* 像素风格组件 */
.pixel-border {
  border: 4px solid #fff;
  box-shadow:
    0 4px 0 rgba(0, 0, 0, 0.5),
    4px 0 0 rgba(0, 0, 0, 0.5),
    4px 4px 0 rgba(0, 0, 0, 0.5);
  background: #222;
}

.pixel-border-sm {
  border: 2px solid #fff;
  background: #333;
}

.pixel-btn {
  padding: 10px 20px;
  border: 4px solid #fff;
  background: #555;
  color: #fff;
  cursor: pointer;
  transition: all 0.1s;
  box-shadow: 2px 2px 0 #000;
}

.pixel-btn:active {
  transform: translate(2px, 2px);
  box-shadow: 0 0 0 #000;
}

.pixel-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pixel-input {
  width: 100%;
  padding: 10px;
  border: 4px solid #fff;
  background: #000;
  color: #0f0;
  outline: none;
}

.remote-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 15px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
}

.status-dot {
  width: 12px;
  height: 12px;
  border: 2px solid #fff;
}

.status-dot.online {
  background: #0f0;
}
.status-dot.offline {
  background: #f00;
}

.connection-panel {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.pixel-card {
  width: 100%;
  max-width: 400px;
  padding: 20px;
  border: 4px solid #fff;
  background: #333;
}

.pixel-title {
  text-align: center;
  margin-bottom: 20px;
  color: #0ff;
  text-transform: uppercase;
}

.input-group {
  margin-bottom: 15px;
}

.input-group label {
  display: block;
  margin-bottom: 5px;
  font-size: 12px;
}

.connect-btn {
  flex: 1;
  background: #0088ff;
  font-weight: bold;
}

.scan-btn {
  width: 50px;
  background: #f59e0b;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
}

.btn-row {
  display: flex;
  gap: 10px;
}

.hint {
  font-size: 10px;
  color: #aaa;
  margin-top: 15px;
  text-align: center;
}

.remote-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 20px;
}

.hub-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  flex: 1;
}

.hub-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 20px;
  cursor: pointer;
  transition: transform 0.2s;
}

.hub-card:active {
  transform: scale(0.95);
}

.card-icon {
  font-size: 40px;
  margin-bottom: 15px;
  color: #0ff;
}

.card-title {
  font-size: 18px;
  font-weight: bold;
  margin-bottom: 5px;
}

.card-desc {
  font-size: 10px;
  color: #aaa;
}

.remote-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px;
  margin-top: 20px;
}

.current-agent {
  display: flex;
  align-items: center;
  gap: 15px;
}

.agent-thumb {
  width: 40px;
  height: 40px;
  object-fit: cover;
}

.agent-name {
  font-weight: bold;
}
.agent-status {
  font-size: 10px;
  color: #0f0;
}

/* 全屏覆盖层 */
.remote-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: #111;
  z-index: 1100;
  display: flex;
  flex-direction: column;
}

.overlay-header {
  padding: 15px;
  background: #222;
  border-bottom: 2px solid #fff;
  display: flex;
  align-items: center;
  gap: 20px;
}

.back-btn {
  background: none;
  border: none;
  color: #fff;
  font-family: inherit;
  cursor: pointer;
}

.overlay-title {
  font-weight: bold;
}

.overlay-content {
  flex: 1;
  overflow: hidden;
}

/* 扫码相关样式 */
.scan-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.9);
  z-index: 2000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.scan-container {
  width: 100%;
  max-width: 400px;
  background: #222;
  display: flex;
  flex-direction: column;
}

.scan-header {
  padding: 15px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 2px solid #fff;
  font-weight: bold;
}

.close-btn {
  background: none;
  border: none;
  color: #fff;
  font-size: 20px;
  cursor: pointer;
}

.qr-reader {
  width: 100%;
  background: #000;
  overflow: hidden;
}

.scan-hint {
  padding: 15px;
  text-align: center;
  font-size: 12px;
  color: #aaa;
}

/* 动画 */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: transform 0.3s ease;
}
.slide-up-enter-from,
.slide-up-leave-to {
  transform: translateY(100%);
}
</style>
