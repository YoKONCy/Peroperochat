<template>
  <div class="room-character" :class="{ 'is-quiet': quiet, 'is-reacting': reacting, 'is-static': renderer === 'illustration', 'is-cg': presentation.layout === 'card' }" @pointerdown="startTouch" @pointerup="finishTouch" @pointercancel="touch = null">
    <button v-if="renderer === 'illustration' && presentation.illustration && !imageFailed" :key="presentation.illustration" type="button" class="room-static-character" :class="{ 'room-static-character--card': presentation.layout === 'card' }" :aria-label="`${agent.name} 的立绘，打开聊天`" @click="$emit('open-chat')"><img :src="presentation.illustration" :alt="`${agent.name} 的角色立绘`" draggable="false" @error="imageFailed = true" /></button>
    <div v-else-if="renderer === 'illustration'" class="character-fallback"><CharacterAvatar class="character-fallback-avatar" :src="agent.avatar" :label="agent.name" /><span>{{ imageFailed ? '角色图片暂时无法显示' : '还没有设置角色立绘' }}</span><button type="button" @click="$emit('open-display')">选择显示方式</button></div>
    <div v-show="renderer === 'live2d'" id="l2d-panel" class="room-character-canvas" :class="{ 'has-error': modelError, 'is-loading': loading }" :aria-label="`${agent.name} 的 Live2D 角色`">
      <div v-if="modelError" class="character-fallback"><CharacterAvatar class="character-fallback-avatar" :src="agent.avatar" :label="agent.name" /><span>{{ modelError }}</span><button type="button" @click="initializeModel">重新加载角色</button><button type="button" @click="$emit('open-display')">改用静态立绘</button></div>
    </div>
    <Transition name="room-speech"><button v-if="speech && !quiet && (renderer === 'illustration' || !modelError)" type="button" class="room-character-speech" :aria-label="`${agent.name}：${speech}。打开聊天`" @click="$emit('open-chat')"><span class="speech-name">{{ agent.name }}</span><span class="speech-text">{{ speech }}</span></button></Transition>
    <span v-if="renderer === 'live2d' && loading && !quiet" class="character-loading" role="status">{{ agent.name }} 正在过来…</span>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { localInvoke } from '../services/localChat'
import { activeAgentId, AGENTS, getActiveAgentId } from '../stores/agents'
import { visibleMessage } from '../utils/messageContent'
import { INTERACTION_LINES } from '../constants/interaction'
import CharacterAvatar from './companion/CharacterAvatar.vue'

const props = defineProps({ quiet: Boolean, interactive: { type: Boolean, default: true }, active: { type: Boolean, default: true }, renderer: { type: String, default: 'live2d' }, presentation: { type: Object, default: () => ({}) } })
const emit = defineEmits(['projection', 'open-chat', 'open-display'])
const agent = computed(() => AGENTS[activeAgentId.value] || { name: '伙伴' })
const speech = ref(''), loading = ref(false), modelError = ref(''), reacting = ref(false), imageFailed = ref(false)
let speechTimer = null, reactionTimer = null, touch = null, observer = null, unlistenEnvironment = null
let projectionVersion = 0, initializationVersion = 0, disposed = false, mounted = false, currentProjection = null

function syncActivity() {
  window.PPC_LIVE2D_ENABLED = mounted && !disposed && props.renderer === 'live2d'
  window.PPC_LIVE2D_ACTIVE = window.PPC_LIVE2D_ENABLED && props.active && !document.hidden && !modelError.value
  window.dispatchEvent(new Event('ppc:live2d-visibility'))
}
function startTouch(event) {
  if (!props.interactive || props.renderer !== 'live2d' || event.button !== 0 || event.target.id !== 'live2d') return
  touch = { x: event.clientX, y: event.clientY, id: event.pointerId, at: Date.now() }
}
function finishTouch(event) {
  const start = touch; touch = null
  if (!start || !props.interactive || event.pointerId !== start.id || Date.now() - start.at > 600 || Math.hypot(event.clientX - start.x, event.clientY - start.y) > 12) return
  reacting.value = true; clearTimeout(reactionTimer)
  reactionTimer = setTimeout(() => { reacting.value = false }, 650)
}
function loadAutoload() {
  if (window.__PPC_LIVE2D_READY) return window.__PPC_LIVE2D_READY
  if (window.__PPC_LIVE2D_SCRIPT) return window.__PPC_LIVE2D_SCRIPT
  window.__PPC_LIVE2D_SCRIPT = new Promise((resolve, reject) => {
    const script = document.createElement('script')
    script.src = '/live2d-widget/autoload.js'
    script.onload = () => Promise.resolve(window.__PPC_LIVE2D_READY).then(resolve, reject)
    script.onerror = () => { script.remove(); reject(new Error('角色资源加载失败')) }
    document.head.appendChild(script)
  }).catch(error => { delete window.__PPC_LIVE2D_SCRIPT; delete window.__PPC_LIVE2D_READY; throw error })
  return window.__PPC_LIVE2D_SCRIPT
}
function moveWidgetIntoPanel() {
  if (disposed || props.renderer !== 'live2d') return
  const panel = document.getElementById('l2d-panel'), waifu = document.getElementById('waifu')
  if (!panel || !waifu) return
  document.getElementById('waifu-toggle')?.remove(); document.getElementById('waifu-tool')?.remove()
  if (!panel.contains(waifu)) panel.appendChild(waifu)
  waifu.style.display = ''
  // 完成归位后不再观察整个页面，流式消息和抽屉更新无需反复查找角色节点。
  observer?.disconnect(); observer = null
}
function showSpeech(value) {
  clearTimeout(speechTimer); speech.value = visibleMessage(value)
  if (speech.value) speechTimer = setTimeout(() => { speech.value = '' }, 12000)
}
async function applyProjection(state, reloadModel) {
  if (disposed || state.agentId !== getActiveAgentId()) return
  if ((currentProjection?.environment?.fetchedAt || 0) > (state.environment?.fetchedAt || 0)) state = { ...state, environment: currentProjection.environment }
  currentProjection = state
  window.WAIFU_TEXTS = state.texts
  window.PPC_LIVE2D_CONFIG = { modelId: state.modelId, modelTexturesId: state.modelTexturesId, activeAgent: state.agentId, waifuTexts: state.texts }
  emit('projection', state)
  if (reloadModel) {
    const welcome = state.texts?.welcome
    showSpeech(typeof welcome === 'string' ? welcome : welcome?.message || agent.value.welcome_message || '')
    if (props.renderer === 'live2d' && window.WaifuWidget) await window.WaifuWidget.loadModel(Number(state.modelId) || 0, Number(state.modelTexturesId) || 0, undefined, false)
  }
}
async function loadProjection(reloadModel = false) {
  const agentId = getActiveAgentId(), version = ++projectionVersion
  try {
    const state = await localInvoke('local_character_projection', { agentId })
    if (!disposed && version === projectionVersion && agentId === getActiveAgentId()) { await applyProjection(state, reloadModel); return true }
  } catch (error) { if (!disposed && version === projectionVersion) console.warn('加载角色展示数据失败：', error) }
  return false
}
async function initializeModel() {
  if (disposed || props.renderer !== 'live2d') return
  const version = ++initializationVersion
  modelError.value = ''; loading.value = true; syncActivity()
  await loadProjection()
  if (disposed || version !== initializationVersion || props.renderer !== 'live2d') return
  try {
    await loadAutoload()
    if (disposed || version !== initializationVersion || props.renderer !== 'live2d') return
    if (!observer) { observer = new MutationObserver(moveWidgetIntoPanel); observer.observe(document.body, { childList: true, subtree: true }) }
    moveWidgetIntoPanel()
    const canvas = document.getElementById('live2d')
    if (!canvas) throw new Error('角色画布暂时不可用')
    canvas.addEventListener('webglcontextlost', onContextLost); canvas.addEventListener('webglcontextrestored', onContextRestored)
    if (!await loadProjection(true)) throw new Error('角色展示数据暂时不可用')
    if (!disposed && version === initializationVersion) { loading.value = false; syncActivity() }
  } catch (error) {
    if (disposed || version !== initializationVersion) return
    loading.value = false; modelError.value = '角色暂时没加载出来'; syncActivity()
    console.warn('加载 Live2D 角色失败：', error)
  }
}
async function updateRenderer() {
  if (!mounted || disposed) return
  initializationVersion++; imageFailed.value = false; modelError.value = ''; speech.value = ''; touch = null; reacting.value = false
  clearTimeout(speechTimer); clearTimeout(reactionTimer)
  syncActivity()
  if (props.renderer === 'live2d') await initializeModel()
  else {
    observer?.disconnect(); observer = null; loading.value = false
    await loadProjection(true)
  }
}
function onContextLost(event) { event.preventDefault(); modelError.value = '角色画面暂时中断了'; syncActivity() }
function onContextRestored() { if (props.renderer === 'live2d') initializeModel() }
async function onAgentChanged() { currentProjection = null; projectionVersion++; emit('projection', { agentId: getActiveAgentId(), mood: '', vibe: '', mind: '' }); await updateRenderer() }
function onChat(event) { if (!props.quiet) showSpeech(event.detail) }
function onWaifuMessage(event) { if (props.renderer === 'live2d' && props.active) showSpeech(event.detail?.text || '') }
function onStatusUpdated(event) { if (event.detail?.agentId === getActiveAgentId()) applyProjection({ ...currentProjection, ...event.detail }, false) }
function onTextsUpdated(event) { if (event.detail) window.WAIFU_TEXTS = event.detail }
async function saveModelPreferences(modelId, textureId, agentId = getActiveAgentId()) {
  if (disposed || props.renderer !== 'live2d') throw new Error('角色显示方式已经切换')
  await localInvoke('local_save_model_preferences', { agentId, modelId, textureId })
  if (!disposed && agentId === getActiveAgentId()) await loadProjection()
}
watch(() => props.renderer, updateRenderer)
watch(() => props.active, syncActivity)
watch(() => props.presentation.illustration, () => { imageFailed.value = false })
onMounted(async () => {
  mounted = true
  window.WAIFU_INTERACTION_LINES = INTERACTION_LINES; window.WAIFU_GET_AGENT_ID = getActiveAgentId; window.PPC_SAVE_MODEL_SETTINGS = saveModelPreferences
  syncActivity()
  window.addEventListener('ppc:agent-switched', onAgentChanged); window.addEventListener('ppc:chat', onChat); window.addEventListener('waifu-message', onWaifuMessage)
  window.addEventListener('ppc:pero-status-updated', onStatusUpdated); window.addEventListener('ppc:waifu-texts-updated', onTextsUpdated)
  document.addEventListener('visibilitychange', syncActivity)
  try {
    const unlisten = await listen('local-environment-event', ({ payload }) => {
      if (!disposed && currentProjection && (payload?.fetchedAt || 0) >= (currentProjection.environment?.fetchedAt || 0)) { currentProjection = { ...currentProjection, environment: payload }; emit('projection', currentProjection) }
    })
    if (disposed) unlisten(); else unlistenEnvironment = unlisten
  } catch (error) { console.warn('环境展示监听不可用：', error) }
  if (!disposed) await updateRenderer()
})
onBeforeUnmount(() => {
  disposed = true; initializationVersion++; projectionVersion++; syncActivity()
  clearTimeout(speechTimer); clearTimeout(reactionTimer); observer?.disconnect(); unlistenEnvironment?.()
  window.removeEventListener('ppc:agent-switched', onAgentChanged); window.removeEventListener('ppc:chat', onChat); window.removeEventListener('waifu-message', onWaifuMessage)
  window.removeEventListener('ppc:pero-status-updated', onStatusUpdated); window.removeEventListener('ppc:waifu-texts-updated', onTextsUpdated); document.removeEventListener('visibilitychange', syncActivity)
  const canvas = document.getElementById('live2d')
  canvas?.removeEventListener('webglcontextlost', onContextLost); canvas?.removeEventListener('webglcontextrestored', onContextRestored)
  if (window.PPC_SAVE_MODEL_SETTINGS === saveModelPreferences) delete window.PPC_SAVE_MODEL_SETTINGS
  if (window.WAIFU_GET_AGENT_ID === getActiveAgentId) { delete window.WAIFU_GET_AGENT_ID; window.WAIFU_TEXTS = {}; window.PPC_LIVE2D_CONFIG = {} }
  // 保留一个原生实例供下次重新挂载，但暂停渲染，静态显示与其他页面不承担其绘制开销。
  const waifu = document.getElementById('waifu')
  if (waifu) { waifu.style.display = 'none'; document.body.appendChild(waifu) }
})
</script>
<style>
.room-character { position: relative; width: 100%; height: 100%; }
.room-character-canvas { position: absolute; inset: 0; isolation: isolate; transform-origin: 50% 65%; }
.is-reacting .room-character-canvas { animation: character-react .65s cubic-bezier(.2,.7,.2,1); }
.room-character-canvas #waifu { position: absolute !important; inset: 0 !important; width: 100%; height: 100%; margin: 0 !important; padding: 0; line-height: 0; }
.room-character-canvas #live2d { display: block !important; width: 100% !important; height: 100% !important; touch-action: manipulation; filter: drop-shadow(0 14px 12px #414d861c); }
.room-character-canvas #waifu-tips, .room-character-canvas #waifu-tool { display: none !important; }
.room-character-canvas.has-error #waifu { visibility: hidden; }
.room-character-speech { position: absolute; top: min(calc(var(--actor-size, 100%) * .7), calc(var(--room-height, 100dvh) - var(--dock-height) - var(--actor-top, 0px) - 136px - var(--reminder-space, 0px))); left: 50%; width: max-content; min-width: min(40vw, 160px); max-width: min(78vw, 320px); transform: translateX(-50%); transform-origin: 62% 0; padding: 22px 19px 17px; text-align: left; color: var(--room-ink); border: 2px solid var(--room-accent-bright); border-radius: 23px 23px 23px 9px; background: var(--room-surface); box-shadow: 0 4px 0 color-mix(in srgb, var(--room-accent-bright) 50%, transparent), 0 10px 24px #28354d1f; line-height: 1.65; isolation: isolate; }
/* 气泡尾巴朝向人物，白底覆盖交接处的边框，保持轮廓连续。 */
.room-character-speech::before { content: ''; position: absolute; top: -10px; left: calc(62% - 9px); width: 18px; height: 18px; transform: rotate(45deg); border-top: 2px solid var(--room-accent-bright); border-left: 2px solid var(--room-accent-bright); border-radius: 5px 0 0; background: var(--room-surface); }
.room-character-speech:active:not(:disabled) { transform: translateX(-50%) scale(.97); }
.speech-name { position: absolute; top: -13px; left: 16px; z-index: 1; max-width: min(calc(100% - 32px), 180px); overflow: hidden; white-space: nowrap; text-overflow: ellipsis; padding: 2px 11px; border: 2px solid var(--room-surface); border-radius: 12px; background: var(--room-accent); color: white; font-size: 11px; line-height: 1.5; font-weight: 600; }
.speech-text { position: relative; font-size: 14px; display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden; overflow-wrap: anywhere; }
.character-loading { position: absolute; top: 60%; left: 50%; transform: translateX(-50%); white-space: nowrap; color: var(--room-accent); font-size: 12px; background: var(--room-glass); padding: 10px 16px; border: 2px solid white; border-radius: 20px; box-shadow: var(--room-shadow); }
.character-fallback { height: 100%; display: flex; flex-direction: column; justify-content: center; align-items: center; gap: 12px; font-size: 12px; color: var(--room-muted); }
.character-fallback-avatar { width: min(34%, 180px); height: auto; aspect-ratio: 4 / 5; border: 4px solid #ffffffcc; border-radius: 3px 12px 3px 3px; }
.character-fallback button { padding: 8px 14px; border: 2px solid white; border-radius: 20px; background: var(--room-surface); }
.room-speech-enter-active, .room-speech-leave-active { transition: opacity .2s, transform .2s; }
.room-speech-enter-from, .room-speech-leave-to { opacity: 0; transform: translateX(-50%) translateY(8px) scale(.96); }
@keyframes character-react { 0%, 100% { transform: translateY(0); } 35% { transform: translateY(-3px); } }
@media (max-height: 480px) { .room-character-speech { display: none; } }
</style>
