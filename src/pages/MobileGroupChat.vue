<template>
  <div class="group-chat-container" :style="viewportStyle">
    
    <!-- 顶部导航 -->
    <CompanionHeader title="家庭群聊" :subtitle="replySettings.filter(a => a.enabled).map(a => a.name).join(' · ')" @back="goHome"><button class="room-icon-button" aria-label="设置群聊成员" :aria-expanded="showSettings" @click="showSettings = !showSettings"><RoomIcon name="sliders" /></button></CompanionHeader>

    <!-- 回复控制面板 -->
    <transition name="slide-down">
      <div v-if="showSettings" class="reply-settings-panel">
        <div class="settings-header">
          <span>群聊成员</span>
          <span class="settings-hint">拖动排序，点击开关</span>
        </div>
        <div class="agent-control-list">
          <div 
            v-for="(agent, index) in replySettings" 
            :key="agent.id"
            class="agent-control-item"
            :class="{ 
              disabled: !agent.enabled,
              dragging: touchStartIndex === index 
            }"
            draggable="true"
            @dragstart="handleDragStart(index)"
            @dragover.prevent
            @drop="handleDrop(index)"
            @touchstart="handleTouchStart(index, $event)"
            @touchmove.prevent
            @touchend="handleTouchEnd($event)"
          >
            <div class="drag-handle">
              <RoomIcon name="menu" />
            </div>
            <div class="agent-avatar-mini" :class="agent.id">
              <img v-if="agent.avatar" :src="agent.avatar" class="avatar-mini-img" :alt="agent.name" />
              <span v-else>{{ agent.name[0] }}</span>
            </div>
            <div class="agent-info">
              <span class="agent-name">{{ agent.name }}</span>
            </div>
            <div class="agent-toggle">
              <el-switch v-model="agent.enabled" size="small" />
            </div>
          </div>
        </div>
      </div>
    </transition>

    <!-- 消息列表 -->
    <div class="chat-list" ref="chatListRef">
      <!-- 查看更多按钮 -->
      <div v-if="hasMoreMessages" class="load-more-container">
        <div class="load-more-btn" @click="loadMore">
          <RoomIcon name="up" />
          <span>加载更多消息</span>
        </div>
      </div>

      <div v-for="msg in displayedMessages" :key="msg.id" :class="[
        'message-row', 
        msg.role === 'user' ? 'user-row' : 'agent-row',
        msg.name ? msg.name.toLowerCase() + '-msg' : ''
      ]">
        <!-- 角色头像 (左侧) -->
        <div class="avatar" v-if="msg.role !== 'user'">
          <div 
            class="avatar-img" 
            :class="[msg.name.toLowerCase(), getBubbleShapeClass(msg.agentId)]"
          >
            <img v-if="getAgentAvatar(msg.agentId)" :src="getAgentAvatar(msg.agentId)" class="avatar-real-img" :alt="msg.name" />
            <span v-else>{{ msg.name[0] }}</span>
          </div>
        </div>
        
        <!-- 消息体 -->
        <div class="message-bubble-container">
          <div 
            class="message-bubble"
            :class="[
              msg.role === 'user' ? 'user-bubble' : 'agent-bubble',
              msg.name ? msg.name.toLowerCase() + '-bubble' : '',
              msg.name ? getBubbleShapeClass(msg.agentId) : ''
            ]"
            :style="msg.name ? getBubbleStyle(msg.agentId) : {}"
          >
            <div class="sender-name" v-if="msg.role !== 'user'" :style="getNameStyle(msg.agentId)">
              {{ msg.name }}
            </div>
            <div class="bubble-content markdown-body" v-html="renderMarkdown(msg.content)"></div>
          </div>
          <div class="message-actions">
            <div class="action-btn delete" @click="deleteMessage(msg.id)">
              <RoomIcon name="trash" />
            </div>
          </div>
        </div>

        <!-- 用户头像 (右侧) -->
        <div class="avatar" v-if="msg.role === 'user'">
          <div class="avatar-img user">Me</div>
        </div>
      </div>
      
      <!-- 打字指示器 -->
      <div v-if="processingAgent" class="typing-indicator">
        <div class="typing-avatar">{{ processingAgent[0] }}</div>
        <div class="typing-bubble">
          <div class="typing-dot"></div>
          <div class="typing-dot"></div>
          <div class="typing-dot"></div>
        </div>
        <span class="typing-text">{{ processingAgent }}正在输入...</span>
      </div>
    </div>

    <!-- 输入栏 -->
    <div class="input-bar-container">
      <div class="input-bar">
        <textarea 
          v-model="input" 
          class="chat-input" 
          placeholder="和大家聊聊…" aria-label="群聊消息"
          @keydown="handleComposerKey"
          rows="1"
          @input="adjustTextareaHeight"
          ref="textareaRef"
        ></textarea>
        <button class="send-btn" aria-label="发送消息" @click="sendMessage" :disabled="!!processingAgent || isLoading || !input.trim()">
          <RoomIcon name="send" />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, nextTick, watch, computed } from 'vue'
import { useRouter } from 'vue-router'
import CompanionHeader from '../components/companion/CompanionHeader.vue'
import RoomIcon from '../components/companion/RoomIcon.vue'
import { useRoomViewport } from '../composables/useRoomViewport'
import { marked } from 'marked'
import DOMPurify from 'dompurify'
import { ElMessageBox, ElMessage } from 'element-plus'
import { localInvoke } from '../services/localChat'
import { useLocalConversation } from '../stores/conversations'
import { visibleMessage } from '../utils/messageContent'
import { useRoleManager, getRoleTheme } from '../composables/useRoleManager'

const router = useRouter()
const { viewportStyle } = useRoomViewport()
function handleComposerKey(event) { if (event.key === 'Enter' && !event.shiftKey && !event.isComposing && event.keyCode !== 229) { event.preventDefault(); sendMessage() } }

const { roles, roleList, loadRoles } = useRoleManager()

const conversation = useLocalConversation('group')
const { messages, isLoading, processingAgentId } = conversation
let disposed = false
onBeforeUnmount(() => { disposed = true })
const displayCount = ref(30) // 初始渲染30条
const input = ref('')
const chatListRef = ref(null)
const textareaRef = ref(null)
const processingAgent = computed(() => roles[processingAgentId.value]?.name || null)

const showSettings = ref(false)
const replySettings = ref([])

// 按稳定角色身份读取展示资源，名称仅用于显示。
function getAgentAvatar(id) { return roles[id]?.avatar || null }

// 获取气泡形状类名
function getBubbleShapeClass(roleId) {
  const theme = getRoleTheme(roleId)
  return `bubble-shape-${theme.bubbleShape}`
}

// 获取气泡样式
function getBubbleStyle(roleId) {
  const theme = getRoleTheme(roleId)
  return {
    '--bubble-primary': theme.primary,
    '--bubble-secondary': theme.secondary,
    '--bubble-accent': theme.accent,
    '--bubble-bg': theme.background,
    '--bubble-text': theme.text,
    '--bubble-border': theme.borderColor
  }
}

// 获取名称样式
function getNameStyle(roleId) {
  const theme = getRoleTheme(roleId)
  return {
    color: theme.primary
  }
}

// 计算属性：当前显示的消息
const displayedMessages = computed(() => {
  return messages.value.slice(-displayCount.value)
})

// 计算属性：是否还有更多消息
const hasMoreMessages = computed(() => {
  return messages.value.length > displayCount.value
})

// 加载更多
function loadMore() {
  const scrollHeightBefore = chatListRef.value?.scrollHeight || 0
  displayCount.value += 30
  
  // 保持滚动位置
  nextTick(() => {
    if (chatListRef.value) {
      const scrollHeightAfter = chatListRef.value.scrollHeight
      chatListRef.value.scrollTop = scrollHeightAfter - scrollHeightBefore
    }
  })
}

let draggedIndex = null

function handleDragStart(index) {
  draggedIndex = index
}

function handleDrop(index) {
  if (draggedIndex === null) return
  const item = replySettings.value.splice(draggedIndex, 1)[0]
  replySettings.value.splice(index, 0, item)
}

// 移动端触摸支持
let touchStartY = 0
const touchStartIndex = ref(null)

function handleTouchStart(index, e) {
  touchStartY = e.touches[0].clientY
  touchStartIndex.value = index
}

function handleTouchEnd(e) {
  if (touchStartIndex.value === null) return
  
  const touchEndY = e.changedTouches[0].clientY
  const deltaY = touchEndY - touchStartY
  
  // 估算移动了多少个位置 (每个 item 高度约 50px)
  const moveSteps = Math.round(deltaY / 50)
  if (moveSteps !== 0) {
    let targetIndex = touchStartIndex.value + moveSteps
    targetIndex = Math.max(0, Math.min(targetIndex, replySettings.value.length - 1))
    
    if (touchStartIndex.value !== targetIndex) {
      draggedIndex = touchStartIndex.value
      handleDrop(targetIndex)
    }
  }
  
  touchStartIndex.value = null
}

let settingsReady = false
let settingsWrites = Promise.resolve()
let settingsError = null
watch(replySettings, value => {
  if (!settingsReady) return
  const members = value.map(({ id, enabled }) => ({ id, enabled }))
  settingsWrites = settingsWrites.then(async () => {
    await localInvoke('local_group_settings', { members })
    settingsError = null
  }).catch(error => { settingsError = error; if (!disposed) ElMessage.error(error.message) })
}, { deep: true })
watch(roleList, async () => {
  settingsReady = false
  try {
    await settingsWrites
    replySettings.value = await localInvoke('local_group_settings')
    await nextTick()
  } catch (error) { if (!disposed) ElMessage.error(error.message) }
  finally { settingsReady = true }
})

function adjustTextareaHeight() {
  const el = textareaRef.value
  if (!el) return
  el.style.height = '40px'
  el.style.height = (el.scrollHeight) + 'px'
}

async function deleteMessage(messageId) {
  try {
    await ElMessageBox.confirm('确定要删除这条消息吗？', '提示', {
      confirmButtonText: '删除', cancelButtonText: '取消', type: 'warning', customClass: 'ppc-message-box',
    })
    await conversation.removeById(messageId, false)
    ElMessage.success('删除成功')
  } catch (error) { if (error instanceof Error) ElMessage.error(error.message) }
}

function goHome() {
  router.push('/')
}

function renderMarkdown(text) {
  return DOMPurify.sanitize(marked.parse(visibleMessage(text)))
}

function scrollToBottom() {
  nextTick(() => {
    if (chatListRef.value) {
      chatListRef.value.scrollTop = chatListRef.value.scrollHeight
    }
  })
}

onMounted(async () => {
  try {
    await loadRoles()
    await conversation.load()
    replySettings.value = await localInvoke('local_group_settings')
    await nextTick()
    settingsReady = true
    scrollToBottom()
  } catch (error) { if (!disposed) ElMessage.error(error.message) }
})

// 消息只从本地后端投影恢复，视图监听不再写入配置。
watch(() => messages.value.length, () => nextTick(scrollToBottom))

async function sendMessage() {
  const value = input.value.trim()
  if (!value || isLoading.value) return
  input.value = ''
  nextTick(adjustTextareaHeight)
  try {
    await settingsWrites
    if (settingsError) throw settingsError
    await conversation.sendGroup(value)
  } catch (error) {
    if (!disposed && error.code !== 'CANCELLED') ElMessage.error(error.message)
    if (!error.persisted && !input.value) input.value = value
  }
}
</script>

<style scoped>
.group-chat-container { position: fixed; inset: var(--room-top, 0px) 0 auto; height: var(--room-height, 100dvh); display: flex; flex-direction: column; background: var(--room-page); overflow: hidden; }
.reply-settings-panel { background: var(--room-surface); border-bottom: 1px solid var(--room-line); padding: 18px max(20px, calc((100% - 800px) / 2)); flex: none; max-height: 35%; overflow-y: auto; }
.settings-header { display: flex; justify-content: space-between; gap: 12px; margin-bottom: 14px; font-size: 13px; font-weight: 500; }
.settings-hint { color: var(--room-muted); font-size: 11px; font-weight: 400; }
.agent-control-list { display: flex; flex-direction: column; gap: 8px; }
.agent-control-item { display: flex; align-items: center; gap: 12px; padding: 10px; border: 1px solid var(--room-line); border-radius: 11px; background: var(--room-subtle); }
.agent-control-item.disabled { opacity: .55; }
.agent-control-item.dragging { border-color: var(--room-accent); }
.drag-handle { color: var(--room-muted); cursor: grab; display: flex; }
.agent-avatar-mini { display: grid; place-items: center; width: 32px; height: 32px; border-radius: 10px; background: var(--room-accent-soft); color: var(--room-accent); overflow: hidden; }
.avatar-mini-img { width: 100%; height: 100%; object-fit: cover; }
.agent-info { flex: 1; font-size: 13px; }
.chat-list { flex: 1; min-height: 0; overflow-y: auto; overscroll-behavior: contain; padding: 26px max(20px, calc((100% - 800px) / 2)); }
.load-more-container { display: flex; justify-content: center; margin-bottom: 24px; }
.load-more-btn { display: flex; align-items: center; gap: 5px; color: var(--room-muted); font-size: 12px; cursor: pointer; }
.load-more-btn svg { width: 15px; height: 15px; }
.message-row { display: flex; align-items: flex-start; gap: 10px; margin-bottom: 22px; }
.user-row { justify-content: flex-end; }
.avatar { flex: none; }
.avatar-img { width: 34px; height: 38px; display: grid; place-items: center; border-radius: 11px; overflow: hidden; background: var(--room-accent-soft); font-size: 12px; color: var(--room-accent); }
.avatar-real-img { width: 100%; height: 100%; object-fit: cover; }
.avatar-img.user { background: var(--room-line); color: var(--room-muted); }
.message-bubble-container { max-width: min(80%, 600px); min-width: 0; }
.message-bubble { border: 1px solid var(--room-line); border-radius: 4px 15px 15px; padding: 12px 15px; background: var(--room-surface); color: var(--room-ink); }
.message-bubble.user-bubble { background: var(--room-user-bubble); border-color: transparent; border-radius: 15px 4px 15px 15px; }
.sender-name { font-size: 11px; font-weight: 500; margin-bottom: 7px; }
.message-actions { display: flex; justify-content: flex-end; margin-top: 4px; }
.message-actions .action-btn { display: flex; padding: 5px; border-radius: 6px; color: var(--room-muted); cursor: pointer; }
.message-actions .action-btn svg { width: 13px; height: 13px; }
.message-actions .delete:hover { color: var(--room-danger); background: #fff0f3; }
.typing-indicator { display: flex; align-items: center; gap: 10px; color: var(--room-muted); font-size: 11px; }
.typing-avatar { display: grid; place-items: center; width: 32px; height: 34px; background: var(--room-accent-soft); border-radius: 10px; }
.typing-bubble { display: flex; gap: 4px; padding: 10px; border: 1px solid var(--room-line); background: white; border-radius: 12px; }
.typing-dot { width: 4px; height: 4px; border-radius: 50%; background: var(--room-accent); animation: typing-pulse 1.2s infinite; }
.typing-dot:nth-child(2) { animation-delay: .15s; }
.typing-dot:nth-child(3) { animation-delay: .3s; }
.input-bar-container { padding: 12px max(16px, calc((100% - 800px) / 2)) calc(12px + env(safe-area-inset-bottom)); background: var(--room-surface); border-top: 1px solid var(--room-line); flex: none; }
.input-bar { display: flex; align-items: flex-end; gap: 10px; padding: 6px; border: 1px solid var(--room-line); border-radius: 15px; }
.chat-input { flex: 1; min-width: 0; min-height: 42px; max-height: 120px; padding: 10px; border: 0; outline: 0; resize: none; background: transparent; color: var(--room-ink); font: inherit; font-size: 16px; line-height: 22px; }
.send-btn { width: 42px; height: 42px; flex: none; display: grid; place-items: center; border: 0; border-radius: 11px; background: var(--room-accent); color: white !important; }
.send-btn:disabled { background: var(--room-accent-soft); color: var(--room-accent) !important; }
.slide-down-enter-active, .slide-down-leave-active { transition: opacity .2s; }
.slide-down-enter-from, .slide-down-leave-to { opacity: 0; }
@keyframes typing-pulse { 0%, 100% { opacity: .3; } 50% { opacity: 1; } }
</style>

<style>
/* 全局样式用于覆盖 Element Plus 组件 */
.ppc-message-box {
  border-radius: 16px !important;
  border: none !important;
  box-shadow: 0 10px 30px rgba(0,0,0,0.1) !important;
  max-width: 300px !important;
}

.ppc-message-box .el-message-box__header {
  padding-top: 20px !important;
}

.ppc-message-box .el-message-box__title {
  font-weight: 700 !important;
  color: #1e293b !important;
}

.ppc-message-box .el-message-box__btns button {
  border-radius: 10px !important;
  padding: 8px 16px !important;
}

.ppc-message-box .el-button--primary {
  background: #ef4444 !important;
  border-color: #ef4444 !important;
}
</style>
