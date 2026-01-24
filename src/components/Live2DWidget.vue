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
        <button class="fab fab-dress" @click.stop="onRandTextures" aria-label="换装">
          <i class="fa-solid fa-shirt"></i>
        </button>
      </el-tooltip>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import axios from 'axios'
import { db } from '../db'
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
function cleanMessageContent(text) {
  if (!text) return ''
  if (text === '__loading__') return '正在思考...'
  
  // 移除所有 XML 标签及其内容
  return text
    .replace(/<(nit(?:-[0-9a-fA-F]{4})?)>[\s\S]*?<\/\1>/gi, '')
    .replace(/<([A-Z_]+)>[\s\S]*?<\/\1>/g, '')
    .replace(/\[\[\[NIT_CALL\]\]\][\s\S]*?\[\[\[NIT_END\]\]\]/g, '')
    .trim()
}

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
    s.onerror = () => reject(new Error('autoload.js failed'))
    document.body.appendChild(s)
  })
}

const handleCanvasClick = (e) => {
  e.stopPropagation()
  handleLive2DClick(e)
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
    } catch (e) { console.warn('Failed to append waifu to panel:', e) }
  }
  
  if (!waifu && canvas && !panel.contains(canvas)) {
    try {
      panel.appendChild(canvas)
    } catch (e) { console.warn('Failed to append canvas to panel:', e) }
  }
  
  if (canvas) {
    try {
      canvas.style.width = '300px'
      canvas.style.height = '300px'
      canvas.style.display = 'block'
    } catch (e) { console.warn('Failed to style canvas:', e) }
  }
}

let observer = null
let tick = null
let envRefreshTick = null
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

async function loadRecentMemories() {
  try {
    const memories = await db.memories.orderBy('timestamp').reverse().limit(10).toArray()
    const tags = []
    const seenTags = new Set()
    
    memories.forEach(m => {
      (m.tags || []).forEach(t => {
        if (!seenTags.has(t) && tags.length < 8) {
          seenTags.add(t)
          tags.push({
            id: Math.random().toString(36),
            tag: t,
            style: {
              left: `${Math.random() * 80 + 5}%`,
              top: `${Math.random() * 60 + 10}%`,
              animationDelay: `${Math.random() * 5}s`,
              animationDuration: `${10 + Math.random() * 10}s`,
              opacity: 0.1 + Math.random() * 0.15
            }
          })
        }
      })
    })
    recentMemories.value = tags
  } catch (e) {
    console.error('Failed to load recent memories:', e)
  }
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
function onSwitchModel() {
  try { if (window.WaifuWidget && typeof window.WaifuWidget.loadOtherModel === 'function') window.WaifuWidget.loadOtherModel() } catch {}
}
function onRandTextures() {
  try { if (window.WaifuWidget && typeof window.WaifuWidget.loadRandModel === 'function') window.WaifuWidget.loadRandModel() } catch {}
}

const onMood = e => { try { moodText.value = String(e.detail || '').trim() || '软萌中' } catch {} }
const onMind = e => { try { mindText.value = String(e.detail || '').trim() || `"${AGENTS[getActiveAgentId()]?.name || 'Pero'}要永远跟主人在一起！"` } catch {} }
const onVibe = e => { try { vibeText.value = String(e.detail || '').trim() || '--' } catch {} }
const onChat = e => { try { showChatBubble(String(e.detail || '').trim()) } catch {} }
const onWaifuMessage = e => { try { showChatBubble(String(e.detail.text || '').trim()) } catch {} }

async function fetchLocationAndWeather(force = false) {
  try {
    const CACHE_KEY = 'ppc.env.last_fetch'
    const CACHE_EXPIRE = 30 * 60 * 1000 // 30 分钟缓存
    const lastFetch = Number(localStorage.getItem(CACHE_KEY) || 0)
    const nowTimestamp = Date.now()

    // 如果不是强制更新，且在缓存时间内，则直接跳过
    if (!force && (nowTimestamp - lastFetch < CACHE_EXPIRE)) {
      console.log('Using cached location and weather data')
      return
    }

    // 1. 获取地理位置
    let city = ''
    let lat = null
    let lon = null
    
    try {
      // 尝试 ipapi.co
      const locRes = await axios.get('https://ipapi.co/json/')
      if (locRes.data && locRes.data.city) {
        city = locRes.data.city
        lat = locRes.data.latitude
        lon = locRes.data.longitude
      }
    } catch (err1) {
      try {
        // 备用方案: ipwho.is
        const locRes2 = await axios.get('https://ipwho.is/')
        if (locRes2.data && locRes2.data.city) {
          city = locRes2.data.city
          lat = locRes2.data.latitude
          lon = locRes2.data.longitude
        }
      } catch (err2) {
        console.warn('All location APIs failed')
      }
    }

    if (city) {
      locationText.value = city
      localStorage.setItem('ppc.location', city)
    }

    // 2. 获取天气 (使用更稳定的 Open-Meteo API)
    if (lat !== null && lon !== null) {
      try {
        const weatherRes = await axios.get(`https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}&current_weather=true&timezone=auto`)
        if (weatherRes.data && weatherRes.data.current_weather) {
          const cw = weatherRes.data.current_weather
          const temp = Math.round(cw.temperature)
          const code = cw.weathercode
          
          // 简单的天气代码映射
          const weatherMap = {
            0: '晴朗',
            1: '大部晴朗', 2: '多云', 3: '阴天',
            45: '雾', 48: '雾',
            51: '毛毛雨', 53: '毛毛雨', 55: '毛毛雨',
            61: '小雨', 63: '中雨', 65: '大雨',
            71: '小雪', 73: '中雪', 75: '大雪',
            80: '阵雨', 81: '阵雨', 82: '阵雨',
            95: '雷阵雨', 96: '雷阵雨', 99: '雷阵雨'
          }
          
          const desc = weatherMap[code] || '天气晴'
          const weatherString = `${desc} ${temp}°C`
          weatherText.value = weatherString
          localStorage.setItem('ppc.weather', weatherString)
          
          // 更新最后抓取时间戳
          localStorage.setItem(CACHE_KEY, String(Date.now()))
        }
      } catch (weaErr) {
        console.warn('Weather API failed:', weaErr)
      }
    }
  } catch (e) {
    console.warn('fetchLocationAndWeather global error:', e)
  }
}

onMounted(async () => {
  ensureFontAwesome()
  try { await loadAutoload() } catch {}
  setTimeout(() => {
    if (!document.getElementById('waifu') && typeof window.initWidget === 'function') {
      try { window.initWidget({ waifuPath: '/live2d-widget/waifu-texts.json', cdnPath: '/live2d-widget/' }) } catch {}
    }
    moveWidgetIntoPanel()
  }, 50)
  observer = new MutationObserver(() => moveWidgetIntoPanel())
  observer.observe(document.body, { childList: true, subtree: true })
  updateTime()
  fetchLocationAndWeather()
  try {
    const agentId = getActiveAgentId()
    const loc = localStorage.getItem('ppc.location')
    if (loc) locationText.value = loc
    const wet = localStorage.getItem('ppc.weather')
    if (wet) weatherText.value = wet
    const m = String(localStorage.getItem(`ppc.${agentId}.mood`) || '').trim()
    if (m) moodText.value = m
    const md = String(localStorage.getItem(`ppc.${agentId}.mind`) || '').trim()
    if (md) mindText.value = md
    const vb = String(localStorage.getItem(`ppc.${agentId}.vibe`) || '').trim()
    if (vb) vibeText.value = vb
  } catch {}
  tick = setInterval(() => { updateTime() }, 1000)
  
  // 每 30 分钟尝试刷新一次环境信息 (fetchLocationAndWeather 内部有缓存逻辑)
  envRefreshTick = setInterval(() => { fetchLocationAndWeather() }, 30 * 60 * 1000)
  
  window.addEventListener('ppc:mood', onMood)
  window.addEventListener('ppc:mind', onMind)
  window.addEventListener('ppc:vibe', onVibe)
  window.addEventListener('ppc:chat', onChat)
  window.addEventListener('waifu-message', onWaifuMessage)
  
  // 监听并应用台词更新
  window.addEventListener('ppc:waifu-texts-updated', (e) => {
    try {
      const newTexts = e.detail
      if (newTexts) {
        window.WAIFU_TEXTS = { ...(window.WAIFU_TEXTS || {}), ...newTexts }
        console.log('[Live2D] Texts updated dynamically')
      }
    } catch (err) {
      console.warn('Failed to update waifu texts:', err)
    }
  })

  // 暴露配置给全局 waifu-tips.js 使用
  window.WAIFU_INTERACTION_LINES = INTERACTION_LINES
  window.WAIFU_GET_AGENT_ID = getActiveAgentId

  // 初始加载本地存储的自定义台词
  try {
    const agentId = getActiveAgentId()
    const savedTexts = localStorage.getItem(`ppc.${agentId}.waifu.texts`)
    if (savedTexts) {
      const parsed = JSON.parse(savedTexts)
      window.WAIFU_TEXTS = { ...(window.WAIFU_TEXTS || {}), ...parsed }
    }
  } catch (e) {}

  window.addEventListener('ppc:pero-status-updated', (e) => {
    const status = e.detail
    moodText.value = status.mood || '--'
    vibeText.value = status.vibe || '--'
    mindText.value = status.mind || ''
    loadRecentMemories() // 对话更新时同步刷新记忆气泡
  })

  loadRecentMemories()
})

onBeforeUnmount(() => {
  try { if (observer) observer.disconnect() } catch {}
  try { if (tick) clearInterval(tick) } catch {}
  try { if (envRefreshTick) clearInterval(envRefreshTick) } catch {}
  try { if (chatTimer) clearTimeout(chatTimer) } catch {}
  window.removeEventListener('ppc:mood', onMood)
  window.removeEventListener('ppc:mind', onMind)
  window.removeEventListener('ppc:vibe', onVibe)
  window.removeEventListener('ppc:chat', onChat)
  window.removeEventListener('waifu-message', onWaifuMessage)
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
