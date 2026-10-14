<template>
  <div class="top-soul-container">
    <div class="digital-clock">
      <div class="time-main">{{ currentTime }}</div>
      <div class="date-sub">{{ currentDate }}</div>
    </div>
    
    <div class="status-tags-container">
      <div class="status-tag location" v-if="locationText">
        <i class="fas fa-location-dot"></i>
        <span>{{ locationText }}</span>
      </div>
      <div class="status-tag weather" v-if="weatherText">
        <i class="fas fa-cloud-sun"></i>
        <span>{{ weatherText }}</span>
      </div>
      <div class="status-tag mood">
        <i class="fas fa-heart"></i>
        <span>{{ moodText }}</span>
      </div>
      <div class="status-tag vibe">
        <i class="fas fa-sparkles"></i>
        <span>{{ vibeText }}</span>
      </div>
    </div>

    <div class="mind-monologue" v-if="mindText">
      <div class="monologue-text">{{ mindText }}</div>
    </div>

    <div class="memory-bubbles">
      <div 
        v-for="mem in recentMemories" 
        :key="mem.id" 
        class="memory-bubble"
        :style="mem.style"
      >
        {{ mem.tag }}
      </div>
    </div>
  </div>

  <div class="pero-container-bottom">
    <!-- 原本的卡片已拆散到顶部，这里保留 Live2D 的挂载容器 -->
  </div>

  <div id="l2d-panel" class="l2d-panel" v-bind="$attrs">
    <div v-if="chatVisible" class="chat-bubble-container">
      <div class="chat-bubble markdown-body" v-html="renderMarkdown(chatText)">
      </div>
    </div>
    <div class="l2d-fabs">
      <el-tooltip content="更换模型" placement="left" :offset="8" popper-class="cute-tip">
        <button class="fab fab-switch" @click.stop="onSwitchModel" aria-label="更换模型">
          <i class="fa-solid fa-shuffle"></i>
        </button>
      </el-tooltip>
      <el-tooltip content="换装" placement="left" :offset="8" popper-class="cute-tip">
        <button class="fab fab-dress" @click.stop="openOutfitDrawer" aria-label="换装">
          <i class="fa-solid fa-shirt"></i>
        </button>
      </el-tooltip>
    </div>
  </div>

  <el-drawer
    v-model="outfitDrawerVisible"
    title="衣柜"
    direction="rtl"
    size="320px"
    :with-header="true"
    custom-class="outfit-drawer"
    :append-to-body="true"
  >
    <div class="outfit-list">
      <div 
        v-for="outfit in outfitList" 
        :key="outfit.id" 
        class="outfit-item"
        @click="changeOutfit(outfit)"
      >
        <div class="outfit-name">{{ outfit.name }}</div>
        <div class="outfit-id">{{ outfit.id }}</div>
      </div>
    </div>
  </el-drawer>
</template>

<script setup>
import { visibleMessage } from '../utils/messageContent'
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { localInvoke } from '../services/localChat'
import { marked } from 'marked'
import DOMPurify from 'dompurify'
import { getActiveAgentId, AGENTS } from '../api'
import { INTERACTION_LINES } from '../constants/interaction'

// Markdown 渲染
function renderMarkdown(text) {
  if (!text) return ''
  const clean = cleanMessageContent(text)
  const html = marked.parse(clean)
  return DOMPurify.sanitize(html)
}

// 清理消息中的隐藏标签
function cleanMessageContent(content) { return visibleMessage(content) }

function ensureFontAwesome() {
  const exists = Array.from(document.querySelectorAll('link[rel="stylesheet"]')).some(l => String(l.href || '').includes('fontawesome'))
  if (!exists) {
    const link = document.createElement('link')
    link.rel = 'stylesheet'
    link.href = 'https://fastly.jsdelivr.net/npm/@fortawesome/fontawesome-free@6/css/all.min.css'
    document.head.appendChild(link)
  }
}

function loadAutoload() {
  return new Promise((resolve, reject) => {
    if (window.__live2d_autoload_loaded) { resolve() ; return }
    const s = document.createElement('script')
    s.src = '/live2d-widget/autoload.js'
    s.onload = () => { window.__live2d_autoload_loaded = true; resolve() }
    s.onerror = () => reject(new Error('角色组件加载失败'))
    document.body.appendChild(s)
  })
}

function moveWidgetIntoPanel() {
  const panel = document.getElementById('l2d-panel')
  if (!panel) return // 如果面板不存在，直接返回，避免报错

  const waifu = document.getElementById('waifu')
  const toggle = document.getElementById('waifu-toggle')
  if (toggle) toggle.remove()
  const tool = document.getElementById('waifu-tool')
  if (tool) tool.remove()
  
  const canvas = document.getElementById('live2d')
  
  if (waifu && !panel.contains(waifu)) {
    try {
      panel.appendChild(waifu)
      waifu.style.bottom = '0'
      waifu.style.left = '0'
      waifu.style.position = 'relative'
      waifu.style.display = ''
    } catch (e) { console.warn('挂载角色面板失败：', e) }
  }
  
  if (!waifu && canvas && !panel.contains(canvas)) {
    try {
      panel.appendChild(canvas)
    } catch (e) { console.warn('挂载角色画布失败：', e) }
  }
  
  if (canvas) {
    try {
      canvas.style.width = '300px'
      canvas.style.height = '300px'
      canvas.style.display = 'block'
      canvas.style.willChange = 'transform, opacity'
      canvas.style.transform = 'translateZ(0)' // 强制开启 GPU 加速
    } catch (e) { console.warn('设置角色画布样式失败：', e) }
  }
}

let observer = null
let tick = null
let unlistenEnvironment = null
let environmentTimestamp = 0
const currentTime = ref('')
const currentDate = ref('')
const locationText = ref('')
const weatherText = ref('')
const moodText = ref('软萌中')
const mindText = ref(`"${AGENTS[getActiveAgentId()]?.name || 'Pero'}要永远跟主人在一起！"`)
const vibeText = ref('--')
const chatText = ref('')
const chatVisible = ref(false)
const recentMemories = ref([])
let chatTimer = null

// 换装相关
const outfitDrawerVisible = ref(false)
const outfitList = ref([])

async function loadOutfits() {
  try {
    const res = await fetch('/live2d-widget/model/potion-Maker-Pio/outfits.json')
    if (res.ok) {
      outfitList.value = await res.json()
    }
  } catch (e) {
    console.error('加载服装列表失败：', e)
  }
}

function openOutfitDrawer() {
  outfitDrawerVisible.value = true
  if (outfitList.value.length === 0) {
    loadOutfits()
  }
}

async function changeOutfit(outfit) {
  const match = outfit.id.match(/index_(\d+)\.json/)
  if (match) {
    const textureId = parseInt(match[1])
    try {
      if (window.WaifuWidget && typeof window.WaifuWidget.loadModel === 'function') {
        await window.WaifuWidget.loadModel(0, textureId, `换上新衣服啦：${outfit.name}`)
        outfitDrawerVisible.value = false
      }
    } catch (e) {
      console.error('切换服装失败：', e)
    }
  }
}

function applyEnvironment(value) {
  if (disposed || !value || value.fetchedAt < environmentTimestamp) return
  environmentTimestamp = value.fetchedAt
  locationText.value = value.location
  weatherText.value = value.weather
}

async function loadRecentMemories(event) {
  try {
    const agentId = getActiveAgentId()
    const state = await localInvoke('local_character_projection', { agentId })
    if (disposed || agentId !== getActiveAgentId()) return
    moodText.value = state.mood || '软萌中'
    mindText.value = state.mind || `"${AGENTS[agentId]?.name || 'Pero'}要永远跟主人在一起！"`
    vibeText.value = state.vibe || '--'
    // 角色变化时整体替换台词投影，避免较短的新台词列表残留旧条目。
    window.WAIFU_TEXTS = state.texts
    window.PPC_LIVE2D_CONFIG = { modelId: state.modelId, modelTexturesId: state.modelTexturesId, activeAgent: state.agentId, waifuTexts: state.texts }
    if (event?.type === 'ppc:agent-switched') window.dispatchEvent(new CustomEvent('ppc:character-projected', { detail: state }))
    applyEnvironment(state.environment)
    recentMemories.value = state.memoryTags.map(tag => ({
      id: tag, tag,
      style: {
        left: `${Math.random() * 80 + 5}%`, top: `${Math.random() * 60 + 10}%`,
        animationDelay: `${Math.random() * 5}s`, animationDuration: `${10 + Math.random() * 10}s`,
        opacity: 0.1 + Math.random() * 0.15,
      },
    }))
  } catch (error) { console.error('加载角色展示数据失败：', error) }
}

function showChatBubble(text) {
  if (!text) return
  const agentId = getActiveAgentId()
  const agentName = AGENTS[agentId]?.name || 'Pero'
  const displayText = text === '__loading__' ? `${agentName}正在思考中...` : text
  
  // 先隐藏再显示，确保动画重置
  chatVisible.value = false
  setTimeout(() => {
    chatText.value = displayText
    chatVisible.value = true
    if (chatTimer) clearTimeout(chatTimer)
    chatTimer = setTimeout(() => {
      chatVisible.value = false
    }, 15000)
  }, 0)
}

function updateTime() {
  const now = new Date()
  currentTime.value = now.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', hour12: false })
  currentDate.value = now.toLocaleDateString('zh-CN', { month: 'long', day: 'numeric', weekday: 'long' })
}
async function onSwitchModel() {
  try { await window.WaifuWidget?.loadOtherModel() }
  catch (error) { console.warn('切换角色模型失败：', error) }
}

const onMood = e => { try { moodText.value = String(e.detail || '').trim() || '软萌中' } catch { /* ignore */ } }
const onMind = e => { try { mindText.value = String(e.detail || '').trim() || `"${AGENTS[getActiveAgentId()]?.name || 'Pero'}要永远跟主人在一起！"` } catch { /* ignore */ } }
const onVibe = e => { try { vibeText.value = String(e.detail || '').trim() || '--' } catch { /* ignore */ } }
const onChat = e => { try { showChatBubble(String(e.detail || '').trim()) } catch { /* ignore */ } }
const onWaifuMessage = e => { try { showChatBubble(String(e.detail.text || '').trim()) } catch { /* ignore */ } }

let disposed = false
let widgetMountTimer = null
function onTextsUpdated(e) {
    try {
      const newTexts = e.detail
      if (newTexts) {
        window.WAIFU_TEXTS = newTexts
        console.log('Live2D 台词已更新')
      }
    } catch (err) {
      console.warn('Live2D 台词更新失败：', err)
    }
}
function onStatusUpdated(e) {
    const status = e.detail
    moodText.value = status.mood || '--'
    vibeText.value = status.vibe || '--'
    mindText.value = status.mind || ''
    loadRecentMemories() // 对话更新时同步刷新记忆气泡
}
onMounted(async () => {
  ensureFontAwesome()
  // 暴露配置给全局 waifu-tips.js 使用
  window.WAIFU_INTERACTION_LINES = INTERACTION_LINES
  window.WAIFU_GET_AGENT_ID = getActiveAgentId

  // 暴露模型保存接口
  window.PPC_SAVE_MODEL_SETTINGS = async (modelId, modelTexturesId, agentId = getActiveAgentId()) => {
    try {
      await localInvoke('local_save_model_preferences', {
        agentId, modelId, textureId: modelTexturesId,
      })
    } catch (error) { console.warn('保存模型显示偏好失败：', error); throw error }
  }

  try {
    unlistenEnvironment = await listen('local-environment-event', ({ payload }) => applyEnvironment(payload))
    if (disposed) { unlistenEnvironment(); unlistenEnvironment = null; return }
    const agentId = getActiveAgentId()
    const state = await localInvoke('local_character_projection', { agentId })
    if (disposed || agentId !== getActiveAgentId()) return
    window.PPC_LIVE2D_CONFIG = {
      modelId: state.modelId, modelTexturesId: state.modelTexturesId,
      activeAgent: state.agentId, waifuTexts: state.texts,
    }
    window.WAIFU_TEXTS = state.texts
    applyEnvironment(state.environment)
  } catch (error) { console.warn('预加载角色展示配置失败：', error) }

  if (disposed) return
  try { await loadAutoload() } catch { /* ignore */ }
  if (disposed) return
  widgetMountTimer = setTimeout(() => {
    if (disposed) return
    if (!document.getElementById('waifu') && typeof window.initWidget === 'function') {
      try { window.initWidget({ waifuPath: '/live2d-widget/waifu-texts.json', cdnPath: '/live2d-widget/' }) } catch { /* ignore */ }
    }
    moveWidgetIntoPanel()
  }, 50)
  observer = new MutationObserver(() => moveWidgetIntoPanel())
  observer.observe(document.body, { childList: true, subtree: true })
  updateTime()
  if (disposed) return
  tick = setInterval(updateTime, 1000)

  window.addEventListener('ppc:mood', onMood)
  window.addEventListener('ppc:mind', onMind)
  window.addEventListener('ppc:vibe', onVibe)
  window.addEventListener('ppc:chat', onChat)
  window.addEventListener('waifu-message', onWaifuMessage)
  
  window.addEventListener('ppc:waifu-texts-updated', onTextsUpdated)

  window.addEventListener('ppc:pero-status-updated', onStatusUpdated)

  window.addEventListener('ppc:agent-switched', loadRecentMemories)
  loadRecentMemories()
})

onBeforeUnmount(() => {
  disposed = true
  clearTimeout(widgetMountTimer)
  window.removeEventListener('ppc:waifu-texts-updated', onTextsUpdated)
  window.removeEventListener('ppc:pero-status-updated', onStatusUpdated)
  try { if (observer) observer.disconnect() } catch { /* ignore */ }
  try { if (tick) clearInterval(tick) } catch { /* ignore */ }
  unlistenEnvironment?.()
  window.removeEventListener('ppc:agent-switched', loadRecentMemories)
  try { if (chatTimer) clearTimeout(chatTimer) } catch { /* ignore */ }
  window.removeEventListener('ppc:mood', onMood)
  window.removeEventListener('ppc:mind', onMind)
  window.removeEventListener('ppc:vibe', onVibe)
  window.removeEventListener('ppc:chat', onChat)
  window.removeEventListener('waifu-message', onWaifuMessage)

  try {
    const waifu = document.getElementById('waifu')
    if (waifu) waifu.style.display = 'none'
  } catch { /* ignore */ }
})
</script>

<style>
.top-soul-container {
  position: fixed;
  top: 40px;
  left: 20px;
  right: 20px;
  height: 300px;
  pointer-events: none;
  z-index: 5;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.status-tags-container {
  display: flex;
  gap: 10px;
  margin-top: -5px;
  animation: tags-fade 1.2s ease-out;
}

.status-tag {
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  padding: 6px 12px;
  border-radius: 12px;
  font-size: 11px;
  color: #fff;
  display: flex;
  align-items: center;
  gap: 6px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.05);
  transition: all 0.3s ease;
}

.status-tag i {
  font-size: 10px;
}

.status-tag.mood { 
  background: rgba(236, 72, 153, 0.15); 
  border-color: rgba(236, 72, 153, 0.2);
}
.status-tag.mood i { color: #ec4899; }

.status-tag.energy { 
  background: rgba(245, 158, 11, 0.15); 
  border-color: rgba(245, 158, 11, 0.2);
}
.status-tag.energy i { color: #f59e0b; }

.status-tag.vibe { 
  background: rgba(16, 185, 129, 0.15); 
  border-color: rgba(16, 185, 129, 0.2);
}
.status-tag.vibe i { color: #10b981; }

.status-tag.location { 
  background: rgba(59, 130, 246, 0.15); 
  border-color: rgba(59, 130, 246, 0.2);
}
.status-tag.location i { color: #3b82f6; }

.status-tag.weather { 
  background: rgba(14, 165, 233, 0.15); 
  border-color: rgba(14, 165, 233, 0.2);
}
.status-tag.weather i { color: #0ea5e9; }

@keyframes tags-fade {
  from { opacity: 0; transform: translateY(5px); }
  to { opacity: 1; transform: translateY(0); }
}

.digital-clock {
  text-align: center;
  color: #fff;
  text-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  animation: clock-fade 1s ease-out;
}

.time-main {
  font-size: 64px;
  font-weight: 400;
  line-height: 1;
  letter-spacing: -1px;
  font-family: 'Quicksand', 'ZCOOL KuaiLe', sans-serif;
}

.date-sub {
  font-size: 14px;
  opacity: 0.8;
  margin-top: 4px;
  font-weight: 400;
}

.mind-monologue {
  background: rgba(255, 255, 255, 0.12);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border-radius: 20px;
  padding: 14px 24px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  max-width: 90%;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.08);
  animation: monologue-fade 1.2s ease-out;
  position: relative;
  pointer-events: auto;
}

.monologue-text {
  font-size: 15px;
  color: #fff;
  text-shadow: 0 1px 4px rgba(0, 0, 0, 0.2);
  font-family: 'ZCOOL KuaiLe', cursive;
  text-align: center;
  line-height: 1.6;
  letter-spacing: 1px;
}

@keyframes clock-fade {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}

.memory-bubbles {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  z-index: -1;
}

.memory-bubble {
  position: absolute;
  font-size: 12px;
  color: #fff;
  background: rgba(255, 255, 255, 0.1);
  padding: 4px 10px;
  border-radius: 20px;
  white-space: nowrap;
  animation: bubble-float linear infinite;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  pointer-events: none;
}

@keyframes monologue-fade {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes bubble-float {
  0% { transform: translateY(0) translateX(0); }
  25% { transform: translateY(-10px) translateX(5px); }
  50% { transform: translateY(-20px) translateX(0); }
  75% { transform: translateY(-10px) translateX(-5px); }
  100% { transform: translateY(0) translateX(0); }
}

.l2d-panel { position: relative; height: 350px; width: 100%; display: flex; justify-content: center; align-items: flex-end; padding: 0; overflow: visible }
#live2d { display: none }
.l2d-panel #waifu { position: relative !important; bottom: 0 !important; left: 0 !important; transform-origin: bottom center !important; margin: 0 !important; display: block !important; line-height: 0; pointer-events: auto }
.l2d-panel #live2d { width: 320px !important; height: 320px !important; display: block }
.l2d-panel #waifu-tips { 
  position: absolute !important; 
  bottom: -10px !important; 
  left: 50% !important; 
  transform: translateX(-50%) !important; 
  width: 240px !important; 
  padding: 0 !important; 
  border: none !important; 
  background: transparent !important; 
  backdrop-filter: none !important; 
  box-shadow: none !important; 
  color: #ec4899 !important; /* 改回可爱的粉色 */
   font-size: 18px !important; 
   font-weight: 400 !important; 
   line-height: 1.4 !important; 
   margin: 0 !important; 
   text-align: center; 
   pointer-events: none; 
   text-shadow: 0 0 8px #fff, 0 0 15px #fff, 0 2px 6px rgba(236, 72, 153, 0.2) !important; 
  opacity: 0; 
  transition: opacity 0.5s !important; 
  font-family: 'ZCOOL KuaiLe', cursive !important; 
  z-index: 9999 !important; 
}
 .l2d-panel #waifu-tips.waifu-tips-active { opacity: 1; animation: cuteComplainingBelow 3s ease-out !important }
 .l2d-panel #waifu-tips::after { display: none }
 .l2d-panel #waifu-tips .fa-lg { display: none }
 
 @keyframes cuteComplainingBelow {
   0% { opacity: 0; transform: translate(-50%, -10px) scale(0.9); }
   15% { opacity: 1; transform: translate(-50%, 0) scale(1); }
   70% { opacity: 1; transform: translate(-50%, 5px) scale(1); }
   100% { opacity: 0; transform: translate(-50%, 15px) scale(0.95); }
 }
.l2d-panel #waifu-tool { display: none !important }

.chat-bubble-container {
  position: absolute;
  left: 20px;
  top: 10px;
  width: 200px;
  z-index: 100;
  pointer-events: none; /* 容器穿透 */
  filter: drop-shadow(0 10px 20px rgba(0, 0, 0, 0.1));
}

.chat-bubble {
  position: relative;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1.5px solid rgba(255, 255, 255, 0.6);
  border-radius: 20px 20px 20px 4px;
  padding: 14px 18px;
  color: #db2777; /* 使用 Pero 的粉色色调 */
  font-family: 'ZCOOL KuaiLe', 'Quicksand', cursive;
  font-size: 16px;
  font-weight: 400;
  line-height: 1.6;
  box-shadow: 
    inset 0 0 0 1px rgba(255, 255, 255, 0.4),
    0 8px 32px rgba(219, 39, 119, 0.1);
  word-break: break-word;
  max-height: 180px;
  overflow-y: auto;
  pointer-events: auto;
  scrollbar-width: thin;
  scrollbar-color: rgba(219, 39, 119, 0.2) transparent;
  animation: bubblePop 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.outfit-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px;
}

.outfit-item {
  padding: 12px;
  background: rgba(255, 255, 255, 0.5);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid transparent;
}

.outfit-item:hover {
  background: rgba(236, 72, 153, 0.1);
  border-color: rgba(236, 72, 153, 0.3);
}

.outfit-name {
  font-weight: 500;
  color: #333;
}

.outfit-id {
  font-size: 12px;
  color: #999;
  margin-top: 2px;
}

.chat-bubble::-webkit-scrollbar {
  width: 4px;
}

.chat-bubble::-webkit-scrollbar-track {
  background: transparent;
}

.chat-bubble::-webkit-scrollbar-thumb {
  background-color: rgba(0, 0, 0, 0.1);
  border-radius: 10px;
}

.chat-bubble::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  right: 0;
  bottom: 0;
  border-radius: 20px 20px 20px 4px;
  background: linear-gradient(135deg, rgba(255,255,255,0.4) 0%, rgba(255,255,255,0) 100%);
  pointer-events: none;
}

@keyframes bubblePop {
  0% { transform: scale(0.8) translateY(10px); opacity: 0; }
  100% { transform: scale(1) translateY(0); opacity: 1; }
}

@keyframes bubbleFloat {
  0%, 100% { transform: translateY(0) rotate(0deg); }
  33% { transform: translateY(-6px) rotate(1deg); }
  66% { transform: translateY(-3px) rotate(-1deg); }
}

.chat-bubble-container {
  animation: bubbleFloat 5s ease-in-out infinite;
}

.l2d-fabs { position: absolute; right: 16px; bottom: 20px; display: flex; flex-direction: column; gap: 12px; z-index: 10 }
.fab { width: 36px; height: 36px; border-radius: 50%; display: grid; place-items: center; color: #555; background: rgba(255, 255, 255, 0.7); backdrop-filter: blur(4px); border: 1px solid rgba(255, 255, 255, 0.5); box-shadow: 0 4px 12px rgba(0,0,0,0.08); cursor: pointer; transition: all .2s }
.fab:active { transform: scale(0.9) }
.fab i { font-size: 14px }

.pero-container-bottom {
  z-index: 10;
}
</style>
