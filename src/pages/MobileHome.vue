<template>
  <div class="home">
    <div class="hero">
      <Live2DWidget @click="handleWaifuClick" />

      <!-- 话题气泡容器 (左上角) - 聚合模式 -->
      <div class="topic-bubbles-container" v-if="topics.length > 0">
        <div
          :class="['topic-aggregate-bubble', { 'bubble-shattering': isTopicExploding }]"
          @click="handleTopicClick"
        >
          <div class="bubble-content">
            <i class="fas fa-user-secret secret-icon"></i>
            <span class="topic-text">秘密话题</span>
            <span v-if="topics.length > 4" class="badge">4+</span>
            <span v-else class="badge">{{ topics.length }}</span>
          </div>
        </div>
      </div>

      <!-- 任务提醒小气泡容器 (右上角) -->
      <div class="reminder-bubbles-container" v-if="reminders.length > 0">
        <div
          v-for="(r, idx) in reminders"
          :key="r.id || r.time + r.task"
          :class="['reminder-bubble', { 'bubble-shattering': explodingReminders.has(r.id) }]"
          :style="getBubbleStyle(idx)"
          @touchstart="handlePressStart(r)"
          @touchend="handlePressEnd(r.id)"
          @mousedown="handlePressStart(r)"
          @mouseup="handlePressEnd(r.id)"
          @mouseleave="handlePressCancel"
        >
          <div class="bubble-content">
            <span class="task-text">{{ r.task }}</span>
            <span class="time-text">{{ formatReminderTime(r.time) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 底部按钮栏 -->
    <div class="inputbar-mini" v-if="!isExpanded">
      <button class="expand-btn" :title="isLoading ? '停止回复' : '输入消息'" @click="expandInput">
        <i class="fas fa-comment-dots"></i>
        <span>{{ isLoading ? '停止' : '...' }}</span>
      </button>
      <div class="mini-tools">
        <button class="tool-btn-mini" @click="showFeatureDrawer = true" title="更多功能">
          <i class="fas fa-plus-circle"></i>
        </button>
        <button class="tool-btn-mini" @click="toSettings"><i class="fas fa-cog"></i></button>
        <button class="tool-btn-mini" @click="openHistory"><i class="fas fa-history"></i></button>
      </div>
    </div>

    <!-- 功能抽屉 (更多功能) -->
    <Transition name="fade-slide">
      <div
        class="feature-drawer-overlay"
        v-if="showFeatureDrawer"
        @click.self="showFeatureDrawer = false"
      >
        <div class="feature-drawer-card">
          <div class="drawer-header">
            <span class="drawer-title">更多功能</span>
            <button class="drawer-close" @click="showFeatureDrawer = false">
              <i class="fas fa-times"></i>
            </button>
          </div>
          <div class="feature-grid">
            <div class="feature-item" @click="toGameAction">
              <div class="feature-icon game-icon">
                <i class="fas fa-gamepad"></i>
              </div>
              <span class="feature-label">21点小游戏</span>
            </div>
            <div class="feature-item" @click="toGroupChatAction">
              <div class="feature-icon group-icon">
                <i class="fas fa-users"></i>
              </div>
              <span class="feature-label">家庭群聊</span>
            </div>
            <!-- 远程连接入口 -->
            <div class="feature-item" @click="toRemoteHub">
              <div class="feature-icon remote-icon">
                <i class="fas fa-satellite-dish"></i>
              </div>
              <span class="feature-label">远程据点</span>
            </div>
            <!-- 可以预留更多位置 -->
          </div>
        </div>
      </div>
    </Transition>

    <!-- 话题列表子页面 -->
    <Transition name="fade-slide">
      <div class="expanded-input-overlay" v-if="showTopicList" @click.self="showTopicList = false">
        <div class="expanded-input-card topic-list-card">
          <div class="card-header">
            <span class="title">{{ AGENTS[getActiveAgentId()]?.name || 'Pero' }} 的秘密话题</span>
            <button class="close-btn" @click="showTopicList = false">
              <i class="fas fa-times"></i>
            </button>
          </div>

          <div class="topic-list-content">
            <div
              v-for="t in topics"
              :key="t.id || t.time + t.topic"
              :class="['topic-item', { 'is-revealed': t.revealed }]"
              @click="toggleTopicReveal(t.id)"
            >
              <div class="topic-item-header">
                <div class="topic-info">
                  <i class="fas fa-user-secret secret-icon" v-if="!t.revealed"></i>
                  <span class="time-tag">{{ formatReminderTime(t.time) }}</span>
                </div>
                <button class="delete-btn" @click.stop="deleteTopic(t.id)">
                  <i class="fas fa-trash-alt"></i>
                </button>
              </div>

              <div class="topic-body">
                <span v-if="!t.revealed" class="blur-text">点击揭晓秘密...</span>
                <span v-else class="real-text">{{ t.topic }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 展开后的全屏/半屏输入区域 -->
    <Transition name="fade-slide">
      <div class="expanded-input-overlay" v-if="isExpanded" @click.self="collapseInput">
        <div class="expanded-input-card">
          <div class="card-header">
            <span class="title">与 {{ AGENTS[getActiveAgentId()]?.name || 'Pero' }} 对话</span>
            <button class="close-btn" @click="collapseInput"><i class="fas fa-times"></i></button>
          </div>
          <div class="image-preview-container" v-if="pendingImage">
            <img :src="pendingImage" class="image-preview" />
            <button class="remove-image-btn" @click="removeImage">
              <i class="fas fa-times"></i>
            </button>
          </div>
          <textarea
            ref="inputRef"
            v-model="text"
            class="expanded-textarea"
            placeholder="..."
            @keyup.ctrl.enter="onSend"
          ></textarea>
          <div class="card-footer">
            <div class="footer-left">
              <input
                type="file"
                ref="fileInputRef"
                style="display: none"
                accept="image/*"
                @change="handleImageUpload"
              />
              <button class="tool-btn" @click="triggerUpload" title="上传图片">
                <i class="fas fa-image"></i>
              </button>
              <span class="hint">Ctrl + Enter 发送</span>
            </div>
            <button
              class="send-btn"
              :disabled="(!text.trim() && !pendingImage) || isLoading"
              @click="onSend"
            >
              <el-icon><Promotion /></el-icon>
              <span>发送</span>
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 任务详情模态框 -->
    <Transition name="fade-slide">
      <div
        class="expanded-input-overlay"
        v-if="showReminderDetail"
        @click.self="showReminderDetail = false"
      >
        <div
          class="expanded-input-card reminder-detail-card"
          style="height: auto; max-height: 50vh"
        >
          <div class="card-header">
            <span class="title">任务详情</span>
            <button class="close-btn" @click="showReminderDetail = false">
              <i class="fas fa-times"></i>
            </button>
          </div>
          <div style="padding: 20px; display: flex; flex-direction: column; gap: 10px">
            <div
              style="color: #666; font-size: 0.9em; display: flex; align-items: center; gap: 5px"
            >
              <i class="fas fa-clock"></i>
              <span>{{ selectedReminder ? formatReminderTime(selectedReminder.time) : '' }}</span>
            </div>
            <div style="font-size: 1.1em; line-height: 1.6; color: #333; word-wrap: break-word">
              {{ selectedReminder ? selectedReminder.task : '' }}
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <HistoryOverlay
      v-if="showHistory"
      :messages="messages"
      @close="showHistory = false"
      @delete="deleteMessageAt"
      @regenerate="regenerateAndClose"
      @copy="copyMessage"
    />
  </div>
</template>

<script setup>

import { ref, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { Promotion } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { impactFeedback } from '@tauri-apps/plugin-haptics'
import { listen } from '@tauri-apps/api/event'
import Live2DWidget from '../components/Live2DWidget.vue'
import HistoryOverlay from '../components/HistoryOverlay.vue'
import { AGENTS, getActiveAgentId } from '../stores/agents'
import { getReminders, getTopics, deleteReminder as apiDeleteReminder,
  updateTopicStatus, deleteTopic as apiDeleteTopic, setAppMode } from '../api'
import { useLocalConversation } from '../stores/conversations'
import { visibleMessage } from '../utils/messageContent'

const router = useRouter()
const conversation = useLocalConversation()
const { messages, isLoading } = conversation
const text = ref('')
const isExpanded = ref(false)
const inputRef = ref(null)
const showHistory = ref(false)
const showTopicList = ref(false)
const showFeatureDrawer = ref(false)
const showReminderDetail = ref(false)
const selectedReminder = ref(null)
const explodingReminders = ref(new Set())
const isTopicExploding = ref(false)
const pendingImage = ref(null)
const fileInputRef = ref(null)
const reminders = ref([])
const topics = ref([])
let pressTimer = null
let isLongPress = false
let disposed = false
let unlistenReminder = null
let loadVersion = 0
const timers = new Set()

function later(action, delay) {
  const timer = setTimeout(() => {
    timers.delete(timer)
    if (!disposed) action()
  }, delay)
  timers.add(timer)
  return timer
}

async function loadAgentData() {
  const agentId = getActiveAgentId()
  const version = ++loadVersion
  try {
    const [, reminderList, topicList] = await Promise.all([
      conversation.load(), getReminders(agentId), getTopics(agentId),
    ])
    if (disposed || version !== loadVersion || agentId !== getActiveAgentId()) return
    reminders.value = reminderList.filter(item => !item.delivered)
    topics.value = topicList
  } catch (error) {
    if (!disposed) ElMessage.error(error.message)
  }
}

function expandInput() {
  if (isLoading.value) { conversation.stop(); return }
  isExpanded.value = true
  nextTick(() => inputRef.value?.focus())
}
function collapseInput() { isExpanded.value = false }
function toSettings() { router.push('/settings') }
function toRemoteHub() { showFeatureDrawer.value = false; setAppMode('remote') }
function toGameAction() { showFeatureDrawer.value = false; router.push('/game/blackjack') }
function toGroupChatAction() { showFeatureDrawer.value = false; router.push('/group') }
function openHistory() { showHistory.value = true }

async function onSend(systemMsg = null) {
  const ephemeral = typeof systemMsg === 'string'
  const value = ephemeral ? systemMsg : text.value.trim()
  const image = ephemeral ? null : pendingImage.value
  if ((!value && !image) || isLoading.value) return
  const content = image ? [
    { type: 'text', text: value || '这张图片里有什么？' },
    { type: 'image_url', image_url: { url: image } },
  ] : value
  const agentId = getActiveAgentId()
  if (!ephemeral) { text.value = ''; pendingImage.value = null }
  isExpanded.value = false
  try {
    await conversation.send(content, { ephemeral, agentId })
    if (!disposed && agentId === getActiveAgentId()) await loadAgentData()
  } catch (error) {
    if (!disposed && error.code !== 'CANCELLED') ElMessage.error(error.message)
    if (!ephemeral && !error.persisted && !disposed && !text.value) {
      text.value = value
      pendingImage.value = image
    }
  }
}

async function regenerateAt(index) {
  if (isLoading.value) return
  try { await conversation.regenerate(index); await loadAgentData() }
  catch (error) { if (!disposed && error.code !== 'CANCELLED') ElMessage.error(error.message) }
}
function regenerateAndClose(index) { showHistory.value = false; regenerateAt(index) }
async function deleteMessageAt(index) {
  try { await conversation.remove(index); await loadAgentData() }
  catch (error) { ElMessage.error(error.message) }
}
async function copyMessage(message) {
  const value = visibleMessage(message.content)
  try { await navigator.clipboard.writeText(value) }
  catch {
    const area = document.createElement('textarea')
    area.value = value
    area.style.position = 'fixed'
    area.style.opacity = '0'
    document.body.appendChild(area)
    area.select()
    const copied = document.execCommand('copy')
    area.remove()
    if (!copied) ElMessage.error('复制失败')
  }
}
function triggerUpload() { fileInputRef.value?.click() }
async function handleImageUpload(event) {
  const file = event.target.files?.[0]
  if (!file) return
  if (!file.type.startsWith('image/') || file.size > 10 * 1024 * 1024) {
    ElMessage.error('请选择不超过 10MB 的图片')
    event.target.value = ''
    return
  }
  const reader = new FileReader()
  reader.onload = () => { if (!disposed) pendingImage.value = reader.result; event.target.value = '' }
  reader.onerror = () => { ElMessage.error('读取图片失败'); event.target.value = '' }
  reader.readAsDataURL(file)
}
function removeImage() { pendingImage.value = null }

async function deleteTopic(id) {
  try { await apiDeleteTopic(id); topics.value = topics.value.filter(item => item.id !== id) }
  catch (error) { ElMessage.error(error.message) }
}
async function toggleTopicReveal(id) {
  const topic = topics.value.find(item => item.id === id)
  if (!topic) return
  try { await updateTopicStatus(id, !topic.revealed); topic.revealed = !topic.revealed }
  catch (error) { ElMessage.error(error.message) }
}
async function deleteReminder(id) {
  try { await apiDeleteReminder(id); reminders.value = reminders.value.filter(item => item.id !== id) }
  catch (error) { ElMessage.error(error.message) }
}
function handleWaifuClick(event) {
  let area = 'general'
  if (event?.target) {
    const rect = event.target.getBoundingClientRect()
    const relativeY = (event.clientY - rect.top) / Math.max(rect.height, 1)
    area = relativeY < 0.3 ? 'head' : relativeY < 0.6 ? 'chest' : 'body'
  }
  window.dispatchEvent(new CustomEvent('ppc:waifu-click', { detail: { area } }))
}
function handlePressStart(reminder) {
  handlePressCancel()
  isLongPress = false
  pressTimer = later(() => {
    isLongPress = true
    selectedReminder.value = reminder
    showReminderDetail.value = true
    impactFeedback('Heavy').catch(() => {})
  }, 500)
}
function handlePressEnd(id) {
  handlePressCancel()
  if (!isLongPress) handleReminderClick(id)
}
function handlePressCancel() {
  if (pressTimer) { clearTimeout(pressTimer); timers.delete(pressTimer); pressTimer = null }
}
function handleReminderClick(id) {
  if (explodingReminders.value.has(id)) return
  impactFeedback('Medium').catch(() => {})
  explodingReminders.value.add(id)
  later(async () => { await deleteReminder(id); explodingReminders.value.delete(id) }, 300)
}
function handleTopicClick() {
  if (isTopicExploding.value) return
  impactFeedback('Medium').catch(() => {})
  isTopicExploding.value = true
  later(() => { showTopicList.value = true; isTopicExploding.value = false }, 300)
}
function formatReminderTime(value) {
  if (!value) return '--:--'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '--:--'
  return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
}

function getBubbleStyle(idx) {
  // 调整基础位置偏移，使其更紧凑并位于头像右上角区域
  const baseOffsets = [
    { top: '0%', right: '0%' },
    { top: '25%', right: '35%' },
    { top: '15%', right: '65%' },
    { top: '50%', right: '10%' },
    { top: '65%', right: '45%' },
    { top: '40%', right: '70%' },
    { top: '10%', right: '30%' },
    { top: '75%', right: '15%' }
  ]

  // 如果任务超过预设偏移量，加入微小的随机扰动避免完全重叠
  const offset = baseOffsets[idx % baseOffsets.length]
  const jitterX = idx >= baseOffsets.length ? Math.sin(idx) * 10 : 0
  const jitterY = idx >= baseOffsets.length ? Math.cos(idx) * 10 : 0

  // 随机动画延迟，让每个气泡浮动不同步
  const delay = idx * 0.7 + 's'
  const duration = 3 + (idx % 3) + 's'

  return {
    top: `calc(${offset.top} + ${jitterY}px)`,
    right: `calc(${offset.right} + ${jitterX}px)`,
    animationDelay: delay,
    animationDuration: duration,
    zIndex: 100 - idx // 越新的任务越靠前
  }
}


watch(() => getActiveAgentId(), () => {
  text.value = ''
  pendingImage.value = null
  loadAgentData()
})
function onAgentUpdated() { loadAgentData() }
onMounted(async () => {
  window.addEventListener('ppc:agent-switched', onAgentUpdated)
  await loadAgentData()
  try {
    const unlisten = await listen('ppc:reminder-due', () => loadAgentData())
    if (disposed) unlisten()
    else unlistenReminder = unlisten
  } catch (error) { console.warn('提醒状态监听失败：', error) }
})
onBeforeUnmount(() => {
  disposed = true
  loadVersion++
  handlePressCancel()
  for (const timer of timers) clearTimeout(timer)
  timers.clear()
  unlistenReminder?.()
  window.removeEventListener('ppc:agent-switched', onAgentUpdated)
})
</script>

<style>
.home {
  height: 100vh;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
  background: transparent;
}
.hero {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  overflow: visible;
  z-index: 1;
  padding-bottom: 90px; /* 增加底部间距，避开输入栏 */
}

/* 提醒气泡样式 */
.reminder-bubbles-container {
  position: absolute;
  top: 260px; /* 心理状态标签下方 */
  right: 15px;
  width: 140px;
  height: 200px;
  pointer-events: none;
  z-index: 10;
}

.reminder-bubble {
  position: absolute;
  pointer-events: auto;
  background: rgba(255, 255, 255, 0.4);
  backdrop-filter: blur(8px);
  border: 1.5px solid rgba(255, 255, 255, 0.6);
  border-radius: 50%;
  padding: 12px;
  min-width: 60px;
  min-height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow:
    0 4px 15px rgba(219, 39, 119, 0.1),
    inset 0 0 10px rgba(255, 255, 255, 0.5);
  cursor: pointer;
  animation: taskBubbleFloat 4s ease-in-out infinite;
  transition: all 0.3s ease;
}

.reminder-bubble:hover {
  transform: scale(1.1);
  background: rgba(255, 255, 255, 0.6);
  box-shadow: 0 6px 20px rgba(219, 39, 119, 0.2);
}

/* 炸裂动画状态 */
.bubble-shattering {
  animation:
    bubbleVibrate 0.1s linear infinite,
    bubbleShatter 0.3s cubic-bezier(0.19, 1, 0.22, 1) forwards !important;
  pointer-events: none;
}

@keyframes bubbleVibrate {
  0% {
    transform: translate(0, 0) rotate(0deg);
  }
  25% {
    transform: translate(1px, 1px) rotate(0.5deg);
  }
  50% {
    transform: translate(-1px, -1px) rotate(-0.5deg);
  }
  75% {
    transform: translate(1px, -1px) rotate(0.5deg);
  }
  100% {
    transform: translate(-1px, 1px) rotate(-0.5deg);
  }
}

@keyframes bubbleShatter {
  0% {
    transform: scale(1);
    opacity: 1;
    filter: blur(0);
  }
  30% {
    transform: scale(1.2);
    opacity: 0.8;
    filter: blur(2px);
  }
  100% {
    transform: scale(2);
    opacity: 0;
    filter: blur(15px);
  }
}

.bubble-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  max-width: 80px;
}

.task-text {
  font-family: 'ZCOOL KuaiLe', cursive;
  font-size: 12px;
  color: #db2777;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  line-height: 1.2;
}

.time-text {
  font-size: 10px;
  color: rgba(219, 39, 119, 0.6);
  margin-top: 2px;
}

@keyframes taskBubbleFloat {
  0%,
  100% {
    transform: translate(0, 0) rotate(0deg);
  }
  33% {
    transform: translate(3px, -8px) rotate(2deg);
  }
  66% {
    transform: translate(-3px, -4px) rotate(-2deg);
  }
}

/* 话题气泡样式 (左侧) */
.topic-bubbles-container {
  position: absolute;
  top: 280px; /* 比任务气泡稍低一点点 */
  left: 15px;
  width: 140px;
  height: 200px;
  pointer-events: none;
  z-index: 10;
}

/* 聚合话题气泡样式 */
.topic-aggregate-bubble {
  position: absolute;
  top: 10px;
  left: 10px;
  pointer-events: auto;
  background: rgba(255, 255, 255, 0.25);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border: 1.5px solid rgba(255, 255, 255, 0.4);
  border-radius: 50%;
  width: 60px;
  height: 60px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 15px rgba(59, 130, 246, 0.15);
  cursor: pointer;
  animation: topicBubbleFloat 5s ease-in-out infinite;
  transition: all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  z-index: 90;
}

.topic-aggregate-bubble:active {
  transform: scale(0.95);
  background: rgba(255, 255, 255, 0.35);
}

.topic-aggregate-bubble .bubble-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  position: relative;
  width: 100%;
  height: 100%;
  justify-content: center;
}

.topic-aggregate-bubble .badge {
  position: absolute;
  bottom: 0;
  right: 0;
  background: #ff4757;
  color: white;
  font-size: 10px;
  font-weight: bold;
  padding: 2px 6px;
  border-radius: 10px;
  min-width: 18px;
  text-align: center;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  transform: translate(20%, 20%);
}

.secret-icon {
  font-size: 18px;
  color: #3b82f6;
  margin-bottom: 2px;
  opacity: 0.8;
}

.topic-text {
  font-family: 'ZCOOL KuaiLe', cursive;
  font-size: 11px;
  color: #3b82f6;
  line-height: 1.2;
  text-align: center;
}

/* 话题列表子页面样式 */
.topic-list-card {
  max-height: 80vh;
}

.topic-list-content {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 4px;
}

.topic-item {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 16px;
  padding: 12px;
  transition: all 0.3s ease;
  cursor: pointer;
  position: relative;
  overflow: hidden;
}

.topic-item:active {
  transform: scale(0.98);
  background: rgba(255, 255, 255, 0.12);
}

.topic-item.is-revealed {
  background: rgba(64, 158, 255, 0.15);
  border-color: rgba(64, 158, 255, 0.3);
}

.topic-item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.topic-info {
  display: flex;
  align-items: center;
  gap: 6px;
}

.topic-info .secret-icon {
  font-size: 14px;
  margin-bottom: 0;
}

.topic-info .time-tag {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.6);
  background: rgba(0, 0, 0, 0.2);
  padding: 2px 6px;
  border-radius: 4px;
}

.topic-body {
  font-size: 15px;
  line-height: 1.4;
  color: #fff;
}

.topic-body .blur-text {
  filter: blur(4px);
  opacity: 0.7;
  transition: all 0.3s ease;
}

.topic-item:hover .blur-text {
  filter: blur(2px);
}

.topic-item .delete-btn {
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.4);
  padding: 4px;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2;
}

.topic-item .delete-btn:active {
  color: #ff4757;
  background: rgba(255, 71, 87, 0.1);
}

@keyframes topicBubbleFloat {
  0%,
  100% {
    transform: translate(0, 0) rotate(0deg);
  }
  50% {
    transform: translate(-4px, 6px) rotate(-1deg);
  }
}

.inputbar-mini {
  position: fixed;
  bottom: 30px;
  left: 20px;
  right: 20px;
  display: flex;
  align-items: center;
  gap: 12px;
  z-index: 100;
}

.expand-btn {
  flex: 1;
  height: 50px;
  background: rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(15px);
  -webkit-backdrop-filter: blur(15px);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 25px;
  color: white;
  display: flex;
  align-items: center;
  padding: 0 20px;
  gap: 12px;
  font-size: 15px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}

.expand-btn:active {
  transform: scale(0.95);
  background: rgba(255, 255, 255, 0.25);
}

.mini-tools {
  display: flex;
  gap: 8px;
}

.tool-btn-mini {
  width: 50px;
  height: 50px;
  border-radius: 25px;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(15px);
  -webkit-backdrop-filter: blur(15px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.3s ease;
}

.tool-btn-mini:active {
  transform: scale(0.9);
}

/* 展开后的样式 */
.expanded-input-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(5px);
  z-index: 1000;
  display: flex;
  align-items: flex-end;
}

.expanded-input-card {
  width: 100%;
  background: rgba(28, 28, 30, 0.95);
  backdrop-filter: blur(20px);
  border-radius: 24px 24px 0 0;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  box-shadow: 0 -10px 40px rgba(0, 0, 0, 0.3);
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header .title {
  color: #fff;
  font-weight: 600;
  font-size: 17px;
}

.close-btn {
  background: rgba(255, 255, 255, 0.1);
  border: none;
  width: 32px;
  height: 32px;
  border-radius: 16px;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.expanded-textarea {
  width: 100%;
  height: 150px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  padding: 16px;
  color: white;
  font-size: 16px;
  resize: none;
  outline: none;
  font-family: inherit;
}

.expanded-textarea:focus {
  border-color: #409eff;
  background: rgba(255, 255, 255, 0.08);
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.footer-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.tool-btn {
  background: rgba(255, 255, 255, 0.1);
  border: none;
  width: 36px;
  height: 36px;
  border-radius: 10px;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tool-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.image-preview-container {
  position: relative;
  width: fit-content;
  margin-bottom: 8px;
}

.image-preview {
  max-width: 200px;
  max-height: 150px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  object-fit: cover;
}

.remove-image-btn {
  position: absolute;
  top: -8px;
  right: -8px;
  background: #ff4757;
  border: none;
  width: 20px;
  height: 20px;
  border-radius: 10px;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 10px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.hint {
  color: rgba(255, 255, 255, 0.4);
  font-size: 12px;
}

.send-btn {
  background: #409eff;
  border: none;
  padding: 10px 24px;
  border-radius: 20px;
  color: white;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.send-btn:active:not(:disabled) {
  transform: scale(0.95);
}

/* 动画 */
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

.fade-slide-enter-from,
.fade-slide-leave-to {
  opacity: 0;
  transform: translateY(100%);
}

.fade-slide-enter-to,
.fade-slide-leave-from {
  opacity: 1;
  transform: translateY(0);
}

.bubble-container {
  display: grid;
  gap: 2px;
}
.bubble {
  max-width: 75%;
  position: relative;
  border-radius: 20px;
  padding: 12px 16px;
  margin: 8px 0;
  word-break: break-word;
  line-height: 1.6;
  font-size: 15px;
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.05);
  transition: all 0.2s ease;
  animation: bubbleFloat 5.6s ease-in-out infinite;
}
.bubble.user {
  margin-left: auto;
  background: linear-gradient(135deg, #3b82f6, #2563eb);
  color: #fff;
  border: none;
  font-family: 'Quicksand', sans-serif;
  border-radius: 20px 20px 4px 20px;
}
.bubble.assistant {
  margin-right: auto;
  background: rgba(255, 255, 255, 0.4);
  backdrop-filter: blur(12px) saturate(150%);
  color: #db2777;
  border: 1.5px solid rgba(255, 255, 255, 0.5);
  font-family: 'ZCOOL KuaiLe', cursive;
  font-size: 17px;
  border-radius: 24px 24px 24px 8px;
  box-shadow:
    0 8px 32px rgba(219, 39, 119, 0.1),
    inset 0 0 15px rgba(255, 255, 255, 0.5),
    inset 0 -5px 10px rgba(219, 39, 119, 0.05);
  position: relative;
}
.bubble.assistant::before {
  content: '';
  position: absolute;
  top: 6px;
  left: 12px;
  width: 20px;
  height: 10px;
  background: rgba(255, 255, 255, 0.6);
  border-radius: 50%;
  transform: rotate(-15deg);
  filter: blur(1px);
  pointer-events: none;
  display: block !important;
}
.bubble.assistant::after {
  content: '';
  position: absolute;
  left: -7px;
  bottom: 12px;
  width: 12px;
  height: 12px;
  background: rgba(255, 255, 255, 0.4);
  backdrop-filter: blur(12px);
  transform: rotate(45deg);
  border-left: 1.5px solid rgba(255, 255, 255, 0.5);
  border-bottom: 1.5px solid rgba(255, 255, 255, 0.5);
  z-index: -1;
}
.bubble pre {
  background: #1e293b;
  color: #e2e8f0;
  border-radius: 8px;
  padding: 8px;
  overflow: auto;
}
.bubble code {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 4px;
  padding: 2px 4px;
}
@keyframes bubbleFloat {
  0%,
  100% {
    transform: translateY(0) rotate(0deg);
  }
  33% {
    transform: translateY(-4px) rotate(1deg);
  }
  66% {
    transform: translateY(-2px) rotate(-1deg);
  }
}
.bubble-tools {
  display: flex;
  gap: 6px;
  align-items: center;
  max-width: 68%;
}
.bubble-tools.user {
  margin-left: auto;
  justify-content: flex-end;
}
.bubble-tools.assistant {
  margin-right: auto;
  justify-content: flex-start;
}
.tool-btn {
  width: 22px;
  height: 22px;
  display: grid;
  place-items: center;
  border-radius: 6px;
  border: 1px solid #e5e7eb;
  background: #f9fafb;
  color: #6b7280;
}
.tool-btn:disabled {
  opacity: 0.5;
}
.input {
  height: 40px;
  border-radius: 12px;
  border: 1px solid rgba(0, 0, 0, 0.08);
  background: rgba(255, 255, 255, 0.5);
  backdrop-filter: blur(4px);
  color: #111;
  padding: 0 12px;
  transition: all 0.2s;
}
.input:focus {
  background: rgba(255, 255, 255, 0.8);
  border-color: #3b82f6;
  outline: none;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}
.btn {
  width: 40px;
  height: 40px;
  display: grid;
  place-items: center;
  padding: 0;
  border-radius: 12px;
  background: linear-gradient(135deg, #3b82f6, #2563eb);
  color: #fff;
  border: none;
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.2);
  cursor: pointer;
}
.btn-secondary {
  height: 40px;
  padding: 0 14px;
  border-radius: 12px;
  border: 1px solid rgba(0, 0, 0, 0.08);
  background: rgba(255, 255, 255, 0.6);
  backdrop-filter: blur(4px);
  color: #374151;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.8);
  border-color: rgba(0, 0, 0, 0.15);
}
@media (max-width: 768px) {
  .hero {
    min-height: 400px;
  }
}

/* 功能抽屉样式 */
.feature-drawer-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(5px);
  z-index: 2000;
  display: flex;
  align-items: flex-end;
}

.feature-drawer-card {
  width: 100%;
  background: rgba(28, 28, 30, 0.95);
  backdrop-filter: blur(20px);
  border-radius: 24px 24px 0 0;
  padding: 20px;
  padding-bottom: calc(20px + env(safe-area-inset-bottom));
  display: flex;
  flex-direction: column;
  gap: 16px;
  box-shadow: 0 -10px 40px rgba(0, 0, 0, 0.3);
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.drawer-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.drawer-title {
  color: #fff;
  font-weight: 600;
  font-size: 17px;
}

.drawer-close {
  background: rgba(255, 255, 255, 0.1);
  border: none;
  width: 32px;
  height: 32px;
  border-radius: 16px;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.feature-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  padding: 10px 0;
}

.feature-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.feature-icon {
  width: 56px;
  height: 56px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  color: #fff;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.1);
  transition: all 0.2s ease;
}

.feature-item:active .feature-icon {
  transform: scale(0.9);
  background: rgba(255, 255, 255, 0.2);
}

.game-icon {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.5), rgba(37, 99, 235, 0.5));
}

.group-icon {
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.5), rgba(5, 150, 105, 0.5));
}

.remote-icon {
  background: linear-gradient(135deg, rgba(245, 158, 11, 0.5), rgba(217, 119, 6, 0.5));
}

.feature-label {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.8);
}
</style>
