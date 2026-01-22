<template>
  <div class="group-chat-container">
    <!-- 顶部导航 -->
    <div class="header">
      <div class="back-btn" @click="goHome">
        <el-icon><ArrowLeft /></el-icon>
      </div>
      <div class="header-title">
        <h2>家庭群聊</h2>
        <span>Pero & Nana</span>
      </div>
      <div class="header-actions">
        <div class="action-trigger" @click="showSettings = !showSettings" :class="{ active: showSettings }">
          <el-icon><Operation /></el-icon>
        </div>
      </div>
    </div>

    <!-- 回复控制面板 -->
    <transition name="slide-down">
      <div v-if="showSettings" class="reply-settings-panel">
        <div class="settings-header">
          <span>回复顺序与启用</span>
          <span class="settings-hint">拖动排序，点击开关</span>
        </div>
        <div class="agent-control-list">
          <div 
            v-for="(agent, index) in replySettings" 
            :key="agent.id"
            class="agent-control-item"
            :class="{ disabled: !agent.enabled }"
            draggable="true"
            @dragstart="handleDragStart(index)"
            @dragover.prevent
            @drop="handleDrop(index)"
          >
            <div class="drag-handle">
              <el-icon><Menu /></el-icon>
            </div>
            <div class="agent-info">
              <div class="agent-dot" :class="agent.id"></div>
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
      <div v-for="(msg, index) in messages" :key="index" :class="[
        'message-row', 
        msg.role === 'user' ? 'user-row' : 'agent-row',
        msg.name ? msg.name.toLowerCase() + '-msg' : ''
      ]">
        <!-- 头像 -->
        <div class="avatar" v-if="msg.role !== 'user'">
          <div class="avatar-img" :class="msg.name.toLowerCase()">
            {{ msg.name[0] }}
          </div>
        </div>
        
        <!-- 消息体 -->
        <div class="message-bubble-container">
          <div class="message-bubble">
            <div class="sender-name" v-if="msg.role !== 'user'">{{ msg.name }}</div>
            <div class="bubble-content markdown-body" v-html="renderMarkdown(msg.content)"></div>
          </div>
          <div class="message-actions">
            <div class="action-btn delete" @click="deleteMessage(index)">
              <el-icon><Delete /></el-icon>
            </div>
          </div>
        </div>

        <!-- 用户头像 (右侧) -->
        <div class="avatar" v-if="msg.role === 'user'">
          <div class="avatar-img user">Me</div>
        </div>
      </div>
      
      <div v-if="processingAgent" class="typing-indicator">
        <div class="typing-dot"></div>
        <div class="typing-dot"></div>
        <div class="typing-dot"></div>
        <span>{{ processingAgent }} 正在思考...</span>
      </div>
    </div>

    <!-- 输入栏 -->
    <div class="input-bar-container">
      <div class="input-bar">
        <textarea 
          v-model="input" 
          class="chat-input" 
          placeholder="给大家发个消息吧..." 
          @keyup.ctrl.enter="sendMessage"
          rows="1"
          @input="adjustTextareaHeight"
          ref="textareaRef"
        ></textarea>
        <button class="send-btn" @click="sendMessage" :disabled="!!processingAgent || !input.trim()">
          <el-icon><Promotion /></el-icon>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowLeft, Promotion, Delete, Operation, Menu } from '@element-plus/icons-vue'
import { marked } from 'marked'
import DOMPurify from 'dompurify'
import { ElMessageBox, ElMessage } from 'element-plus'
import { chat, AGENTS, getDefaultPrompts, getRelevantMemories, saveMemory } from '../api'

const router = useRouter()
const messages = ref([])
const input = ref('')
const chatListRef = ref(null)
const textareaRef = ref(null)
const processingAgent = ref(null) // 当前正在回复的角色名

const showSettings = ref(false)
const replySettings = ref([
  { id: 'pero', name: 'Pero', enabled: true },
  { id: 'nana', name: 'Nana', enabled: true }
])

let draggedIndex = null

function handleDragStart(index) {
  draggedIndex = index
}

function handleDrop(index) {
  const item = replySettings.value.splice(draggedIndex, 1)[0]
  replySettings.value.splice(index, 0, item)
  localStorage.setItem('ppc.group.replySettings', JSON.stringify(replySettings.value))
}

watch(replySettings, (newVal) => {
  localStorage.setItem('ppc.group.replySettings', JSON.stringify(newVal))
}, { deep: true })

function adjustTextareaHeight() {
  const el = textareaRef.value
  if (!el) return
  el.style.height = '40px'
  el.style.height = (el.scrollHeight) + 'px'
}

function deleteMessage(index) {
  ElMessageBox.confirm(
    '确定要删除这条消息吗？',
    '提示',
    {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning',
      customClass: 'ppc-message-box'
    }
  ).then(() => {
    messages.value.splice(index, 1)
    ElMessage({
      type: 'success',
      message: '删除成功',
      duration: 1500
    })
  }).catch(() => {})
}

function goHome() {
  router.push('/')
}

function renderMarkdown(text) {
  if (!text) return ''
  // 移除元数据标签
  const clean = text.replace(/<[A-Z_]+>[\s\S]*?<\/[A-Z_]+>/g, '').trim()
  return DOMPurify.sanitize(marked.parse(clean))
}

function scrollToBottom() {
  nextTick(() => {
    if (chatListRef.value) {
      chatListRef.value.scrollTop = chatListRef.value.scrollHeight
    }
  })
}

// 解析并保存记忆
async function parseAndSaveMemory(text, msgTimestamp = null, agentId) {
  if (!text) return
  const memoryMatch = text.match(/<MEMORY>([\s\S]*?)<\/MEMORY>/)
  if (memoryMatch) {
    try {
      const content = memoryMatch[1].trim()
      const memoryData = JSON.parse(content)
      if (memoryData.content && memoryData.tags) {
        await saveMemory(memoryData, msgTimestamp, agentId)
        console.log(`[Group] ${agentId} memory saved:`, memoryData.content)
      }
    } catch (e) {
      console.error('[Group] Failed to parse MEMORY JSON:', e)
    }
  }
}

// 加载群聊历史
onMounted(() => {
  const saved = localStorage.getItem('ppc.group.messages')
  if (saved) {
    try {
      messages.value = JSON.parse(saved)
      scrollToBottom()
    } catch (e) {}
  }

  const savedSettings = localStorage.getItem('ppc.group.replySettings')
  if (savedSettings) {
    try {
      replySettings.value = JSON.parse(savedSettings)
    } catch (e) {}
  }
})

// 保存群聊历史
watch(messages, (newVal, oldVal) => {
  localStorage.setItem('ppc.group.messages', JSON.stringify(newVal))
  // 只有当消息增加时才滚动到底部（比如发送新消息或收到回复）
  if (newVal.length > (oldVal ? oldVal.length : 0)) {
    scrollToBottom()
  }
}, { deep: true })

async function sendMessage() {
  const text = input.value.trim()
  if (!text || processingAgent.value) return
  
  // 1. 添加用户消息
  messages.value.push({
    role: 'user',
    name: 'User',
    content: text,
    timestamp: Date.now()
  })
  input.value = ''
  nextTick(() => {
    adjustTextareaHeight()
  })
  
  // 2. 触发群聊回复逻辑 (遵循用户配置的顺序和启用状态)
  const agentsToReply = replySettings.value
    .filter(a => a.enabled)
    .map(a => a.id)
  
  for (const agentId of agentsToReply) {
    await generateResponse(agentId)
  }
}

async function generateResponse(agentId) {
  processingAgent.value = AGENTS[agentId].name
  
  try {
    const config = AGENTS[agentId]
    
    // 构造 Prompt
    // 0. 获取记忆
    let memoryContext = ''
    try {
      // 获取最后一条用户消息的内容作为检索上下文
      const lastUserMsg = messages.value.slice().reverse().find(m => m.role === 'user')
      if (lastUserMsg) {
        const mems = await getRelevantMemories(lastUserMsg.content, 5, agentId)
        if (mems && mems.length > 0) {
          memoryContext = '\n<RelevantMemories>\n' + 
            mems.map(m => `- ${m.content} (Tags: ${m.tags.join(',')})`).join('\n') + 
            '\n</RelevantMemories>'
        }
      }
    } catch (e) {
      console.warn('Memory retrieval failed:', e)
    }

    // 1. 系统提示词
    const systemPrompt = config.system_prompt + '\n' + config.persona_prompt + memoryContext + '\n' + `
<Output_Constraint>
回复要求:
1. 你的名字是 ${config.name}。
2. 你正在大家庭的群聊中，成员包括你、其他AI和主人。
3. 请根据上下文回复用户或其他人的话。
4. 最好控制在三句话以内，保持简短。
</Output_Constraint>
`
    // 2. 历史记录转换 (Perspective Shifting)
    const history = messages.value.map(m => {
      if (m.name.toLowerCase() === agentId) {
        return { role: 'assistant', content: m.content }
      } else {
        // 其他人都是 user，带上前缀
        const prefix = m.role === 'user' ? '' : `[${m.name}]: `
        return { role: 'user', content: prefix + m.content }
      }
    })

    const fullMessages = [
      { role: 'system', content: systemPrompt },
      ...history.slice(-60) // 取最近60条，增加上下文长度
    ]

    // 3. 调用 API
    // 注意：这里没有流式，简单等待
    // 为了更好的体验，可以使用 chatStream，这里简化用 chat
    const response = await chat(fullMessages, null, 0.7)
    
    // 4. 添加回复
    messages.value.push({
      role: 'assistant',
      name: config.name,
      content: response.content,
      timestamp: Date.now()
    })
    
    // 5. 保存记忆
    await parseAndSaveMemory(response.content, Date.now(), agentId)
    
  } catch (e) {
    console.error(`[Group] ${agentId} failed to respond:`, e)
  } finally {
    processingAgent.value = null
  }
}
</script>

<style scoped>
.group-chat-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #fdf2f8 0%, #f5f3ff 100%);
  position: relative;
  overflow: hidden;
}

/* 装饰性背景元素 */
.group-chat-container::before {
  content: '';
  position: absolute;
  top: -10%;
  right: -10%;
  width: 300px;
  height: 300px;
  background: radial-gradient(circle, rgba(236, 72, 153, 0.1) 0%, transparent 70%);
  z-index: 0;
}

.group-chat-container::after {
  content: '';
  position: absolute;
  bottom: -10%;
  left: -10%;
  width: 300px;
  height: 300px;
  background: radial-gradient(circle, rgba(139, 92, 246, 0.1) 0%, transparent 70%);
  z-index: 0;
}

.header {
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
  padding: 12px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid rgba(226, 232, 240, 0.5);
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05);
  z-index: 10;
}

.back-btn {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  cursor: pointer;
  transition: background 0.2s;
  color: #64748b;
}

.back-btn:active {
  background: rgba(0, 0, 0, 0.05);
}

.header-title {
  text-align: center;
}
.header-title h2 { 
  margin: 0; 
  font-size: 17px; 
  font-weight: 700;
  color: #1e293b;
}
.header-title span { 
  font-size: 11px; 
  color: #94a3b8; 
  font-weight: 500;
  letter-spacing: 0.5px;
}

.header-actions {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.action-trigger {
  width: 32px;
  height: 32px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #64748b;
  cursor: pointer;
  transition: all 0.2s;
  background: rgba(0, 0, 0, 0.03);
}

.action-trigger:hover, .action-trigger.active {
  background: #fdf2f8;
  color: #ec4899;
}

/* 回复设置面板 */
.reply-settings-panel {
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(10px);
  padding: 12px 16px;
  border-bottom: 1px solid rgba(226, 232, 240, 0.5);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
  z-index: 9;
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: 10px;
}

.settings-header span:first-child {
  font-size: 13px;
  font-weight: 700;
  color: #475569;
}

.settings-hint {
  font-size: 10px;
  color: #94a3b8;
}

.agent-control-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.agent-control-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: white;
  border-radius: 12px;
  border: 1px solid #f1f5f9;
  transition: all 0.2s;
  user-select: none;
}

.agent-control-item.disabled {
  opacity: 0.6;
  background: #f8fafc;
}

.drag-handle {
  cursor: grab;
  color: #cbd5e1;
  font-size: 16px;
}

.drag-handle:active {
  cursor: grabbing;
}

.agent-info {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
}

.agent-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.agent-dot.pero { background: #ec4899; }
.agent-dot.nana { background: #8b5cf6; }

.agent-name {
  font-size: 14px;
  font-weight: 600;
  color: #334155;
}

.agent-toggle {
  display: flex;
  align-items: center;
}

/* 动画 */
.slide-down-enter-active, .slide-down-leave-active {
  transition: all 0.3s ease;
}

.slide-down-enter-from, .slide-down-leave-to {
  transform: translateY(-20px);
  opacity: 0;
}

.placeholder { width: 40px; }

.chat-list {
  flex: 1;
  overflow-y: auto;
  padding: 20px 16px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  z-index: 1;
  scroll-behavior: smooth;
}

/* 隐藏滚动条但保留功能 */
.chat-list::-webkit-scrollbar {
  width: 4px;
}
.chat-list::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.1);
  border-radius: 2px;
}

.message-row {
  display: flex;
  gap: 12px;
  max-width: 90%;
  animation: message-in 0.3s ease-out;
}

@keyframes message-in {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.user-row {
  align-self: flex-end;
  flex-direction: row-reverse;
}

.agent-row {
  align-self: flex-start;
}

.avatar {
  flex-shrink: 0;
  margin-top: 4px;
}

.avatar-img {
  width: 38px;
  height: 38px;
  border-radius: 14px;
  background: #cbd5e1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 800;
  font-size: 14px;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  border: 2px solid white;
}

.avatar-img.pero { 
  background: linear-gradient(135deg, #f472b6, #db2777); 
}
.avatar-img.nana { 
  background: linear-gradient(135deg, #a78bfa, #7c3aed); 
}
.avatar-img.user { 
  background: linear-gradient(135deg, #60a5fa, #2563eb); 
}

.message-bubble-container {
  position: relative;
  display: flex;
  align-items: center;
  gap: 8px;
  max-width: 100%;
}

.user-row .message-bubble-container {
  flex-direction: row-reverse;
}

.message-bubble {
  max-width: 100%;
  padding: 12px 16px;
  border-radius: 20px;
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.05);
  position: relative;
  transition: transform 0.2s;
  z-index: 2;
}

.message-actions {
  opacity: 0;
  transition: opacity 0.2s, transform 0.2s;
  display: flex;
  gap: 4px;
  transform: translateX(5px);
}

.user-row .message-actions {
  transform: translateX(-5px);
}

.message-row:hover .message-actions {
  opacity: 1;
  transform: translateX(0);
}

.action-btn {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  background: white;
  color: #94a3b8;
  box-shadow: 0 2px 5px rgba(0,0,0,0.05);
  transition: all 0.2s;
}

.action-btn:hover {
  background: #fee2e2;
  color: #ef4444;
}

.action-btn.delete {
  font-size: 14px;
}

.user-row .message-bubble {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
  color: white;
  border-bottom-right-radius: 4px;
}

.agent-row .message-bubble {
  background: white;
  border-top-left-radius: 4px;
}

/* Pero 专属样式 */
.agent-row.pero-msg .message-bubble {
  background: linear-gradient(135deg, #ffffff, #fff1f2);
  border: 1px solid rgba(236, 72, 153, 0.1);
}

/* Nana 专属样式 */
.agent-row.nana-msg .message-bubble {
  background: linear-gradient(135deg, #ffffff, #f5f3ff);
  border: 1px solid rgba(139, 92, 246, 0.1);
}

.sender-name {
  font-size: 11px;
  font-weight: 700;
  margin-bottom: 4px;
  margin-left: 4px;
}

.pero-msg .sender-name { color: #db2777; }
.nana-msg .sender-name { color: #7c3aed; }

.bubble-content {
  font-size: 15px;
  line-height: 1.6;
  word-break: break-word;
}

.user-row .bubble-content {
  color: rgba(255, 255, 255, 0.95);
}

.agent-row .bubble-content {
  color: #334155;
}

.input-bar-container {
  padding: 16px;
  background: transparent;
  z-index: 10;
}

.input-bar {
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(10px);
  padding: 8px 12px;
  display: flex;
  gap: 10px;
  align-items: flex-end;
  border-radius: 24px;
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.5);
}

.chat-input {
  flex: 1;
  border: none;
  background: transparent;
  padding: 10px 4px;
  font-family: inherit;
  font-size: 15px;
  resize: none;
  height: 40px;
  max-height: 120px;
  line-height: 20px;
  outline: none;
  color: #1e293b;
}

.chat-input::placeholder {
  color: #94a3b8;
}

.send-btn {
  width: 40px;
  height: 40px;
  border-radius: 14px;
  background: linear-gradient(135deg, #3b82f6, #2563eb);
  color: white;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s;
  flex-shrink: 0;
  margin-bottom: 2px;
}

.send-btn:hover:not(:disabled) {
  transform: scale(1.05);
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.3);
}

.send-btn:active:not(:disabled) {
  transform: scale(0.95);
}

.send-btn:disabled {
  background: #e2e8f0;
  color: #94a3b8;
  cursor: not-allowed;
}

.typing-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: #94a3b8;
  padding: 0 20px 10px;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0% { opacity: 0.6; }
  50% { opacity: 1; }
  100% { opacity: 0.6; }
}

.typing-dot {
  width: 4px;
  height: 4px;
  background: #cbd5e1;
  border-radius: 50%;
  animation: dot-bounce 1.4s infinite ease-in-out both;
}

.typing-dot:nth-child(1) { animation-delay: -0.32s; }
.typing-dot:nth-child(2) { animation-delay: -0.16s; }

@keyframes dot-bounce {
  0%, 80%, 100% { transform: scale(0); }
  40% { transform: scale(1); }
}
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