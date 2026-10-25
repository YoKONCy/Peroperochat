<template>
  <section ref="panelElement" class="room-chat" :class="[`room-chat--${mode}`, { 'room-chat--keyboard': keyboardOpen }]" aria-label="聊天面板">
    <div class="chat-handle-area" @pointerdown="startDrag" @pointerup="finishDrag" @pointercancel="drag = null">
      <button class="chat-handle" :aria-label="mode === 'rest' ? '展开对话' : '调整对话面板高度'" @click="cycleMode"><span /><RoomAccent class="chat-handle-mark" kind="orbit" /></button>
    </div>
    <header v-show="mode !== 'rest'" class="chat-heading">
      <div class="chat-heading-name"><CharacterAvatar class="chat-role-avatar" :src="agent.avatar" :label="agent.name" /><strong>{{ agent.name }}</strong><span v-if="busy" class="heading-busy">正在回应</span></div>
      <div class="chat-heading-actions">
        <button class="room-icon-button" :aria-label="mode === 'full' ? '缩小对话' : '展开完整对话'" @click="$emit('update:mode', mode === 'full' ? 'peek' : 'full')"><RoomIcon :name="mode === 'full' ? 'collapse' : 'expand'" /></button>
        <button class="room-icon-button" aria-label="收起对话，回到房间" @click="collapse"><RoomIcon name="down" /></button>
      </div>
    </header>

    <Transition name="room-soft"><button v-if="mode === 'rest' && (lastReply || busy)" class="last-reply" @click="$emit('update:mode', 'peek')">
      <span class="last-reply-symbol"><RoomIcon name="chat" /></span><span>{{ busy ? activity || '正在想怎么回答你…' : lastReply }}</span><ElapsedTime v-if="busy" :started-at="startedAt" prefix="已等待" /><RoomIcon name="up" />
    </button></Transition>

    <div v-if="historyMounted" v-show="mode !== 'rest'" ref="list" class="chat-messages" @scroll.passive="onScroll">
      <button v-if="messages.length > visibleCount" class="earlier-button" @click="showEarlier">查看更早的对话</button>
      <div v-if="!messages.length" class="chat-intro">
        <CharacterAvatar class="intro-avatar" :src="agent.avatar" :label="agent.name" /><p>想和 {{ agent.name }} 聊点什么？</p>
      </div>
      <article v-for="{ message, index } in visibleMessages" :id="`room-message-${message.id}`" :key="message.id" v-memo="[index, message.content, message.reasoning, message.usage, message.toolTimeline, message.status, message.error, selectedId === message.id, jumpedId === message.id, freshId === message.id, agent.name, agent.avatar, busy]" class="room-message" :class="[message.role, { 'is-selected': selectedId === message.id, 'is-jumped': jumpedId === message.id, 'is-new': freshId === message.id }]">
        <div class="message-byline"><CharacterAvatar v-if="message.role !== 'user'" class="message-avatar" :src="agent.avatar" :label="agent.name" /><span class="message-speaker">{{ message.role === 'user' ? '你' : agent.name }}</span><time>{{ formatTime(message.timestamp) }}</time></div>
        <div class="message-body" @contextmenu.prevent="selectedId = message.id" @pointerdown="startPress($event, message.id)" @pointermove="movePress" @pointerup="clearPress" @pointercancel="clearPress">
          <ToolTimeline v-if="message.role === 'assistant' && message.toolTimeline?.length" :items="message.toolTimeline" @resize="imageLoaded" />
          <MessageReasoning v-if="message.reasoning" :source="message.reasoning" :running="message.status === 'pending'" :generation="message.usage?.presentation" />
          <div v-if="message.content === '__loading__'" class="typing-indicator" aria-label="正在思考"><i /><i /><i /></div>
          <template v-else-if="Array.isArray(message.content)">
            <template v-for="(part, partIndex) in message.content" :key="partIndex">
              <div v-if="part.type === 'text'" class="markdown-body" v-html="renderMarkdown(part.text)" />
              <button v-else-if="part.type === 'image_url'" class="message-attachment" aria-label="查看对话图片" @click="$emit('preview-image', safeImage(part.image_url?.url))"><img :src="safeImage(part.image_url?.url)" alt="对话附件" loading="lazy" @load="imageLoaded" /></button>
            </template>
          </template>
          <div v-else class="markdown-body" v-html="renderMarkdown(message.content)" />
          <p v-if="message.error && visibleMessage(message.content) !== message.error" class="message-error">{{ message.error }}</p>
          <span v-if="['cancelled', 'interrupted'].includes(message.status)" class="message-status">{{ message.status === 'cancelled' ? '已停止' : '上次回复中断了' }}</span>
        </div>
        <button class="message-more room-icon-button" :aria-label="`操作${message.role === 'user' ? '你的消息' : agent.name + '的回复'}`" :aria-expanded="selectedId === message.id" @click="selectMessage(message.id)"><RoomIcon name="more" /></button>
        <Transition name="room-soft"><div v-if="selectedId === message.id" class="message-actions">
          <button @click="action('copy', message)"><RoomIcon name="copy" />复制</button>
          <button v-if="message.role === 'assistant'" :disabled="busy" @click="action('regenerate', index)"><RoomIcon name="refresh" />{{ message.status === 'failed' ? '重试' : '重新生成' }}</button>
          <button :disabled="busy" class="delete-action" @click="action('delete', index)"><RoomIcon name="trash" />删除</button>
        </div></Transition>
      </article>
    </div>

    <button v-if="!following && mode !== 'rest'" class="latest-button" @click="scrollToLatest(true)"><RoomIcon name="down" />回到最新消息</button>
    <Transition name="room-soft"><div v-if="busy && mode !== 'rest'" class="chat-activity" role="status"><span class="activity-dot" />{{ activity || `${agent.name} 正在回应你…` }}<ElapsedTime :started-at="startedAt" prefix="已等待" /></div></Transition>
    <Transition name="room-soft"><div v-if="image" class="composer-attachment"><img :src="image" alt="待发送图片" /><span>图片已准备好</span><button class="room-icon-button" aria-label="移除待发送图片" @click="$emit('remove-image')"><RoomIcon name="close" /></button></div></Transition>

    <form class="room-composer" @submit.prevent="submit">
      <button type="button" class="room-icon-button attachment-button" aria-label="添加图片" @click="$emit('attach')"><RoomIcon name="image" /></button>
      <textarea ref="input" :value="modelValue" rows="1" maxlength="100000" :placeholder="`和 ${agent.name} 聊聊…`" :aria-label="`给 ${agent.name} 的消息`" enterkeyhint="send" @input="updateInput" @focus="$emit('focus')" @keydown="handleEnter" />
      <button v-if="busy" type="button" class="composer-send is-stop" aria-label="停止回复" title="停止回复" @click="$emit('stop')"><RoomIcon name="stop" /></button>
      <button v-else type="submit" class="composer-send" :disabled="!modelValue.trim() && !image" aria-label="发送消息"><RoomIcon name="send" /></button>
    </form>
    <div v-if="mode !== 'rest'" class="composer-footnote"><span>Shift + Enter 换行</span><button type="button" @click="collapse">回到房间 <RoomIcon name="down" /></button></div>
  </section>
</template>

<script setup>
import { computed, ref, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import RoomIcon from './RoomIcon.vue'
import ElapsedTime from './ElapsedTime.vue'
import MessageReasoning from './MessageReasoning.vue'
import RoomAccent from './RoomAccent.vue'
import ToolTimeline from './ToolTimeline.vue'
import CharacterAvatar from './CharacterAvatar.vue'
import { roomMarkdown } from '../../utils/roomMarkdown'
import { createTextRenderCache } from '../../utils/textRenderCache'
import { visibleMessage } from '../../utils/messageContent'

const props = defineProps({
  agent: { type: Object, required: true }, messages: { type: Array, default: () => [] }, modelValue: { type: String, default: '' },
  mode: { type: String, default: 'rest' }, busy: Boolean, keyboardOpen: Boolean, image: { type: String, default: null },
  activity: { type: String, default: '' }, startedAt: [Number, String], jumpId: { type: Number, default: null },
})
const emit = defineEmits(['update:modelValue', 'update:mode', 'send', 'stop', 'focus', 'attach', 'remove-image', 'copy', 'delete', 'regenerate', 'preview-image', 'dock-size'])
const panelElement = ref(null)
const input = ref(null)
const list = ref(null)
const selectedId = ref(null)
const visibleCount = ref(24)
const historyMounted = ref(false)
const renderMarkdown = createTextRenderCache(roomMarkdown)
const following = ref(true)
const jumpedId = ref(null)
const freshId = ref(null)
const visibleMessages = computed(() => {
  const start = Math.max(0, props.messages.length - visibleCount.value)
  return props.messages.slice(start).map((message, index) => ({ message, index: start + index }))
})
const lastReply = computed(() => {
  for (let index = props.messages.length - 1; index >= 0; index--) {
    const message = props.messages[index]
    if (message.role === 'assistant' && message.status === 'active') return visibleMessage(message.content)
  }
  return ''
})
let drag = null
let pressTimer = null
let pressOrigin = null
let jumpTimer = null
let freshTimer = null
let suppressClickUntil = 0
let disposed = false
let dockObserver = null
let scrollFrame = null
let inputFrame = null
let forceScroll = false
let lastDockSize = 0

// 房间模式按实际输入栏、最近回复与底部安全区占位定位人物，避免固定高度估算造成悬空。
function reportDockSize() {
  if (disposed || props.mode !== 'rest' || !panelElement.value) return
  const bottom = Number.parseFloat(getComputedStyle(panelElement.value).bottom) || 0
  const size = Math.ceil(panelElement.value.getBoundingClientRect().height + bottom)
  if (size !== lastDockSize) { lastDockSize = size; emit('dock-size', size) }
}

function cycleMode() {
  if (Date.now() < suppressClickUntil) return
  emit('update:mode', props.mode === 'rest' ? 'peek' : props.mode === 'peek' ? 'full' : 'peek')
}
function startDrag(event) {
  if (event.button !== 0) return
  drag = { y: event.clientY, id: event.pointerId }
  event.currentTarget.setPointerCapture(event.pointerId)
}
function finishDrag(event) {
  if (!drag) return
  const difference = event.clientY - drag.y
  drag = null
  if (Math.abs(difference) < 35) return
  suppressClickUntil = Date.now() + 400
  if (difference < 0) emit('update:mode', props.mode === 'rest' ? 'peek' : 'full')
  else if (props.mode === 'full') emit('update:mode', 'peek')
  else collapse()
}
function collapse() { input.value?.blur(); selectedId.value = null; emit('update:mode', 'rest') }
function updateInput(event) { emit('update:modelValue', event.target.value) }
function scheduleInputResize() {
  if (inputFrame !== null) return
  inputFrame = requestAnimationFrame(() => { inputFrame = null; if (!disposed) resizeInput() })
}
function resizeInput() {
  if (!input.value) return
  input.value.style.height = 'auto'
  input.value.style.height = `${Math.min(input.value.scrollHeight, 112)}px`
}
function handleEnter(event) {
  if (event.key !== 'Enter' || event.shiftKey || event.isComposing || event.keyCode === 229) return
  event.preventDefault(); submit()
}
function submit() { if (!props.busy && (props.modelValue.trim() || props.image)) { following.value = true; emit('send') } }
function selectMessage(id) { selectedId.value = selectedId.value === id ? null : id }
function startPress(event, id) {
  if (event.pointerType !== 'touch' || event.target.closest('a, button, summary')) return
  clearPress(); pressOrigin = { x: event.clientX, y: event.clientY }
  pressTimer = setTimeout(() => { selectedId.value = id; pressTimer = null }, 500)
}
function movePress(event) { if (pressOrigin && Math.hypot(event.clientX - pressOrigin.x, event.clientY - pressOrigin.y) > 8) clearPress() }
function clearPress() { clearTimeout(pressTimer); pressTimer = null; pressOrigin = null }
function action(name, value) { selectedId.value = null; emit(name, value) }
function onScroll() {
  if (!list.value || props.mode === 'rest') return
  following.value = list.value.scrollHeight - list.value.scrollTop - list.value.clientHeight < 72
}
function scrollToLatest(force = false) {
  if (props.mode === 'rest' || !force && !following.value) return
  forceScroll ||= force
  if (scrollFrame !== null) return
  scrollFrame = requestAnimationFrame(async () => {
    scrollFrame = null
    const requested = forceScroll; forceScroll = false
    await nextTick()
    if (disposed || props.mode === 'rest' || !requested && !following.value) return
    if (list.value) list.value.scrollTop = list.value.scrollHeight
    if (requested) following.value = true
  })
}
async function showEarlier() {
  const height = list.value.scrollHeight
  const position = list.value.scrollTop
  visibleCount.value += 24
  await nextTick()
  if (list.value) list.value.scrollTop = position + list.value.scrollHeight - height
}
function imageLoaded() { scrollToLatest() }
function formatTime(value) { return new Date(value).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }) }
function safeImage(value) { return typeof value === 'string' && /^(https?:\/\/|data:image\/)/i.test(value) ? value : '' }
async function jumpTo(id) {
  if (!id) return
  const index = props.messages.findIndex(message => message.id === id)
  if (index < 0) return
  visibleCount.value = Math.max(visibleCount.value, props.messages.length - index)
  following.value = false
  await nextTick()
  if (disposed) return
  document.getElementById(`room-message-${id}`)?.scrollIntoView({ block: 'center' })
  jumpedId.value = id
  clearTimeout(jumpTimer)
  jumpTimer = setTimeout(() => { jumpedId.value = null }, 2200)
}
// 只让刚追加的消息轻轻出现；历史加载、翻页和流式文本更新不重复播放。
watch(() => props.messages.at(-1)?.id, (id, previousId) => {
  clearTimeout(freshTimer); freshId.value = null
  if (previousId == null || !props.messages.slice(-3, -1).some(message => message.id === previousId)) return
  freshId.value = id
  freshTimer = setTimeout(() => { freshId.value = null }, 320)
})
watch(() => [props.messages.at(-1)?.id, props.messages.at(-1)?.content, props.messages.at(-1)?.reasoning, props.busy], () => scrollToLatest())
watch(() => props.mode, mode => { if (mode !== 'rest') { historyMounted.value = true; scrollToLatest() } else nextTick(reportDockSize) }, { immediate: true })
watch(() => props.jumpId, jumpTo)
watch(() => props.modelValue, scheduleInputResize)
onMounted(() => {
  resizeInput(); reportDockSize()
  if (typeof ResizeObserver !== 'undefined') { dockObserver = new ResizeObserver(reportDockSize); dockObserver.observe(panelElement.value) }
  window.addEventListener('resize', reportDockSize)
  if (props.jumpId) jumpTo(props.jumpId); else scrollToLatest(true)
})
onBeforeUnmount(() => { disposed = true; dockObserver?.disconnect(); cancelAnimationFrame(scrollFrame); cancelAnimationFrame(inputFrame); window.removeEventListener('resize', reportDockSize); clearPress(); clearTimeout(jumpTimer); clearTimeout(freshTimer) })
defineExpose({ focus: () => input.value?.focus({ preventScroll: true }), blur: () => input.value?.blur() })
</script>

<style scoped>
.room-chat { position: absolute; z-index: 20; inset: auto 0 0; width: 100%; height: var(--dock-height); display: flex; flex-direction: column; background: var(--room-subtle); border: 1px solid var(--room-line); border-bottom: 0; border-radius: 5px 18px 0 0; box-shadow: 0 -3px 16px #263b510a; padding-bottom: env(safe-area-inset-bottom); }
.room-chat:not(.room-chat--rest) { animation: chat-panel-open .2s var(--room-spring); }
@keyframes chat-panel-open { from { opacity: .65; transform: translateY(18px); } to { opacity: 1; transform: translateY(0); } }
@media (prefers-reduced-motion: reduce) { .room-chat:not(.room-chat--rest) { animation: none; } }
.room-chat--rest { inset: auto 12px calc(12px + env(safe-area-inset-bottom)); width: auto; height: auto; padding: 0; border: 0; background: transparent; box-shadow: none; }
.room-chat--rest .chat-handle-area { display: none; }
.chat-handle-area { flex: none; height: 22px; display: flex; justify-content: center; touch-action: none; }
.chat-handle { display: grid; place-items: center; width: 100%; border: 0; background: transparent; border-radius: 26px 26px 0 0; }
.chat-handle span { width: 32px; height: 4px; background: var(--room-line); border-radius: 10px; }
.chat-heading { padding: 0 16px 10px 24px; display: flex; align-items: center; justify-content: space-between; flex: none; border-bottom: 1px solid var(--room-line); }
.chat-heading-name { display: flex; gap: 9px; align-items: center; font-size: 13px; min-width: 0; }
.chat-heading-name strong { font-weight: 600; min-width: 0; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
.chat-role-avatar { width: 24px; height: 28px; border-radius: 2px 6px 2px 2px; }
.presence-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--room-accent); }
.heading-busy { font-size: 11px; color: var(--room-muted); white-space: nowrap; }
.chat-heading-actions { display: flex; flex: none; }
.chat-heading-actions .room-icon-button { width: 36px; height: 36px; }
.chat-heading-actions svg { width: 17px; height: 17px; }
.last-reply { display: flex; align-items: center; gap: 9px; min-height: 36px; margin: 0 8px 8px; padding: 9px 12px; border: 1px solid #ffffff96; border-radius: 3px 9px 3px 3px; background: var(--room-glass); box-shadow: none; backdrop-filter: blur(12px); font-size: 12px; color: var(--room-ink); text-align: left; }
.last-reply > svg:first-child { color: var(--room-accent); }
.last-reply > span { flex: 1; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
.last-reply > svg { width: 15px; height: 15px; }
.chat-messages { flex: 1; overflow-y: auto; min-height: 0; padding: 18px 20px; overscroll-behavior: contain; scrollbar-width: thin; scrollbar-color: var(--room-line) transparent; }
.chat-intro { min-height: 150px; display: flex; flex-direction: column; justify-content: center; align-items: center; gap: 10px; color: var(--room-muted); font-size: 13px; line-height: 1.7; text-align: center; }
.chat-intro p { margin: 0; }
.intro-avatar { width: 50px; height: 60px; display: grid; place-items: center; background: var(--room-accent-soft); border-radius: 3px 9px 3px 3px; overflow: hidden; color: var(--room-accent); }
.intro-avatar img { width: 100%; height: 100%; object-fit: cover; }
.room-message { position: relative; margin: 0 20px 18px 0; padding-right: 10px; }
.message-byline { display: flex; align-items: center; gap: 9px; font-size: 11px; color: var(--room-muted); margin-bottom: 7px; }
.message-byline time { opacity: .65; font-size: 10px; }
.message-body { padding: 12px 15px; border: 1px solid var(--room-line); border-radius: 3px 10px 10px; width: fit-content; min-width: 40px; max-width: 100%; background: var(--room-bubble); }
.is-new .message-body { transform-origin: bottom left; animation: message-arrive .3s var(--room-spring); }
.is-new.user .message-body { transform-origin: bottom right; }
@keyframes message-arrive { from { opacity: .35; transform: translateY(7px) scale(.97); } to { opacity: 1; transform: translateY(0) scale(1); } }
.room-message.user { margin-left: 30px; margin-right: 0; }
.user .message-byline { justify-content: flex-end; }
.user .message-body { background: var(--room-user-bubble); border-color: transparent; margin-left: auto; border-radius: 10px 3px 10px 10px; }
.message-more { position: absolute; right: -19px; top: 20px; width: 30px; height: 36px; opacity: .5; }
.user .message-more { right: auto; left: -30px; }
.message-more svg { width: 18px; height: 18px; stroke-width: 2.5; }
.room-message:hover .message-more, .room-message:focus-within .message-more, .is-selected .message-more { opacity: 1; }
.message-actions { display: flex; flex-wrap: wrap; gap: 4px; margin-top: 9px; padding: 5px; max-width: 100%; width: fit-content; border: 1px solid var(--room-line); border-radius: 14px; box-shadow: var(--room-shadow); background: var(--room-surface); }
.user .message-actions { margin-left: auto; }
.message-actions button { border: 0; display: flex; align-items: center; gap: 5px; background: transparent; padding: 9px; border-radius: 7px; font-size: 12px; }
.message-actions button:hover { background: var(--room-accent-soft); }
.message-actions svg { width: 14px; height: 14px; }
.message-actions .delete-action { color: #a35d56; }
.message-error { color: #a14848; font-size: 12px; line-height: 1.7; margin: 8px 0 0; }
.message-status { display: block; color: var(--room-muted); font-size: 11px; margin-top: 8px; }
.message-reasoning { color: var(--room-muted); font-size: 12px; margin-bottom: 10px; }
.message-reasoning summary { cursor: pointer; padding: 5px 0; }
.message-reasoning > div { max-height: 220px; overflow: auto; white-space: pre-wrap; line-height: 1.8; padding: 10px 0; }
.message-attachment { display: block; border: 0; background: transparent; padding: 0; max-width: 100%; }
.message-attachment img { max-width: 100%; max-height: 240px; border-radius: 12px; object-fit: contain; }
.earlier-button { display: block; margin: 0 auto 20px; background: transparent; border: 0; padding: 10px; color: var(--room-muted); font-size: 12px; }
.latest-button { position: absolute; bottom: 112px; left: 50%; transform: translateX(-50%); display: flex; align-items: center; gap: 6px; padding: 9px 14px; border: 1px solid var(--room-line); border-radius: 20px; background: var(--room-surface); box-shadow: var(--room-shadow); font-size: 12px; }
.latest-button:active:not(:disabled) { transform: translateX(-50%) scale(.96); }
.latest-button svg { width: 14px; height: 14px; }
.chat-activity { font-size: 11px; color: var(--room-muted); padding: 5px 25px; display: flex; flex-wrap: wrap; align-items: center; gap: 8px; }
.activity-dot { width: 5px; height: 5px; border-radius: 50%; background: var(--room-accent); animation: room-breathe 1.6s infinite; }
.room-composer { display: flex; align-items: flex-end; gap: 6px; padding: 6px; margin: 6px 12px 0; border: 1px solid var(--room-line); border-radius: 4px 12px 4px 4px; background: var(--room-surface); flex: none; }
.room-chat--rest .room-composer { margin: 0; border-color: #ffffffcc; border-bottom: 2px solid var(--room-accent-bright); background: #fffffff2; box-shadow: 0 4px 16px #263b510d; }
.room-composer { transition: border-color .2s, box-shadow .2s; }
.room-composer:focus-within { border-color: var(--room-accent-bright); box-shadow: 0 0 0 3px var(--room-accent-soft); }
.room-chat--rest .room-composer:focus-within { border-bottom-color: var(--room-accent); }
.room-composer textarea { flex: 1; resize: none; min-width: 0; max-height: 112px; overflow-y: auto; padding: 10px 2px; color: var(--room-ink); background: transparent; border: 0; outline: 0; font-size: 14px; line-height: 22px; }
.room-chat--rest textarea { max-height: 44px; }
.room-composer textarea::placeholder { color: var(--room-muted); opacity: .8; }
.room-composer textarea:focus-visible { box-shadow: 0 2px 0 var(--room-accent); }
.attachment-button { color: var(--room-accent); width: 40px; height: 42px; }
.composer-send { flex: none; height: 42px; width: 42px; display: grid; place-items: center; border-radius: 3px 8px 3px 3px; border: 0; background: var(--room-action); color: white !important; }
.composer-send:disabled { background: var(--room-accent-soft); color: var(--room-accent) !important; opacity: .7; box-shadow: none; }
.composer-send.is-stop { background: var(--room-accent-soft); color: var(--room-accent) !important; box-shadow: none; }
.composer-send > svg { transition: transform .24s var(--room-spring); }
.composer-send:active:not(:disabled):not(.is-stop) > svg { transform: translate(2px, -2px) scale(.92); }
@media (hover: hover) { .composer-send:hover:not(:disabled):not(.is-stop) > svg { transform: translate(1px, -1px) rotate(-6deg); } }
.composer-footnote { display: flex; align-items: center; justify-content: space-between; gap: 10px; padding: 2px 24px 12px; color: var(--room-muted); font-size: 10px; flex: none; min-height: 28px; }
.composer-footnote > button { display: flex; align-items: center; gap: 4px; padding: 0; border: 0; background: transparent; font-size: 11px; }
.composer-footnote svg { width: 12px; height: 12px; }
.composer-attachment { display: flex; align-items: center; gap: 10px; padding: 6px 22px; font-size: 12px; color: var(--room-muted); }
.composer-attachment img { width: 40px; height: 40px; object-fit: cover; border-radius: 9px; }
.composer-attachment > span { flex: 1; }
.typing-indicator { display: flex; gap: 4px; height: 20px; align-items: center; }
.typing-indicator i { width: 5px; height: 5px; border-radius: 50%; background: var(--room-accent); animation: room-breathe 1.2s infinite; }
.typing-indicator i:nth-child(2) { animation-delay: .15s; }
.typing-indicator i:nth-child(3) { animation-delay: .3s; }
.is-jumped { background: var(--room-accent-soft); border-radius: 12px; outline: 8px solid var(--room-accent-soft); }
@keyframes room-breathe { 0%, 100% { opacity: .3; } 50% { opacity: 1; } }
@media (min-width: 960px) { .room-chat { left: auto; right: 28px; bottom: 24px; width: 420px; border-radius: 5px 16px 5px 5px; border-bottom: 1px solid var(--room-line); padding-bottom: 0; } .room-chat--rest { height: auto; border: 0; } }
@media (max-height: 480px) { .composer-footnote { display: none; } .chat-heading { padding-bottom: 4px; } .chat-messages { padding-top: 10px; padding-bottom: 10px; } }
@supports (-webkit-touch-callout: none) { .room-composer textarea { font-size: 16px; } }
</style>
