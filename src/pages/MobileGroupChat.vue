<template>
  <div class="group-chat-container">
    <!-- 装饰性背景 -->
    <div class="bg-decoration bg-1"></div>
    <div class="bg-decoration bg-2"></div>
    <div class="bg-decoration bg-3"></div>
    
    <!-- 顶部导航 -->
    <div class="header">
      <div class="back-btn" @click="goHome">
        <el-icon><ArrowLeft /></el-icon>
      </div>
      <div class="header-title">
        <h2>🏠 家庭群聊</h2>
        <span class="subtitle">{{ replySettings.filter(a => a.enabled).map(a => a.name).join(' • ') }}</span>
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
          <span>🎀 群聊成员设置</span>
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
              <el-icon><Menu /></el-icon>
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
          <el-icon><ArrowUp /></el-icon>
          <span>加载更多消息</span>
        </div>
      </div>

      <div v-for="(msg, index) in displayedMessages" :key="index" :class="[
        'message-row', 
        msg.role === 'user' ? 'user-row' : 'agent-row',
        msg.name ? msg.name.toLowerCase() + '-msg' : ''
      ]">
        <!-- 角色头像 (左侧) -->
        <div class="avatar" v-if="msg.role !== 'user'">
          <div 
            class="avatar-img" 
            :class="[msg.name.toLowerCase(), getBubbleShapeClass(msg.name.toLowerCase())]"
          >
            <img v-if="getAgentAvatar(msg.name)" :src="getAgentAvatar(msg.name)" class="avatar-real-img" :alt="msg.name" />
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
              msg.name ? getBubbleShapeClass(msg.name.toLowerCase()) : ''
            ]"
            :style="msg.name ? getBubbleStyle(msg.name.toLowerCase()) : {}"
          >
            <div class="sender-name" v-if="msg.role !== 'user'" :style="getNameStyle(msg.name.toLowerCase())">
              {{ msg.name }}
            </div>
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
          placeholder="和大家说点什么吧～ 💕" 
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
import { ref, onMounted, nextTick, watch, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowLeft, Promotion, Delete, Operation, Menu, ArrowUp } from '@element-plus/icons-vue'
import { marked } from 'marked'
import DOMPurify from 'dompurify'
import { ElMessageBox, ElMessage } from 'element-plus'
import { getRelevantMemories, saveMemory, getConfig, saveConfig, chat } from '../api'
import { useRoleManager, getRoleTheme } from '../composables/useRoleManager'

const router = useRouter()

const { roles, roleList, loadRoles } = useRoleManager()

const messages = ref([])
const displayCount = ref(30) // 初始渲染30条
const input = ref('')
const chatListRef = ref(null)
const textareaRef = ref(null)
const processingAgent = ref(null) // 当前正在回复的角色名

const showSettings = ref(false)
const replySettings = ref([])

// 根据角色名获取头像路径
function getAgentAvatar(name) {
  if (!name) return null
  const id = name.toLowerCase()
  const role = roles[id]
  return role?.avatar || null
}

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
  saveConfig('ppc.group.replySettings', JSON.stringify(replySettings.value))
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

watch(replySettings, (newVal) => {
  saveConfig('ppc.group.replySettings', JSON.stringify(newVal))
}, { deep: true })

// 监听角色变化，更新 replySettings 中的名称
watch(roleList, (newRoleList) => {
  const roleMap = new Map(newRoleList.map(r => [r.id, r]))
  replySettings.value.forEach(setting => {
    const role = roleMap.get(setting.id)
    if (role) {
      setting.name = role.name
    }
  })
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
  // 移除元数据标签 (改为白名单模式，防止误伤)
  const clean = text.replace(/<(PEROCUE|MEMORY|IDLE_MESSAGES|CLICK_MESSAGES|BACK_MESSAGES|REMINDER|TOPIC)>[\s\S]*?<\/\1>/g, '').trim()
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

// 加载角色和群聊历史
onMounted(async () => {
  // 加载所有角色
  await loadRoles()

  // 加载群聊消息历史
  const saved = await getConfig('ppc.group.messages')
  if (saved) {
    try {
      messages.value = JSON.parse(saved)
      scrollToBottom()
    } catch { /* ignore */ }
  }

  // 加载回复设置
  const savedSettings = await getConfig('ppc.group.replySettings')
  if (savedSettings) {
    try {
      replySettings.value = JSON.parse(savedSettings)
    } catch { /* ignore */ }
  }

  // 如果没有保存的回复设置，初始化一个（默认包含所有存在的角色）
  if (replySettings.value.length === 0) {
    replySettings.value = roleList.value.map(role => ({
      id: role.id,
      name: role.name,
      enabled: true
    }))
  } else {
    // 确保回复设置中的角色都存在，并添加可能的新角色
    const existingIds = new Set(roleList.value.map(r => r.id))
    const settingIds = new Set(replySettings.value.map(s => s.id))
    
    // 添加新增的角色
    roleList.value.forEach(role => {
      if (!settingIds.has(role.id)) {
        replySettings.value.push({
          id: role.id,
          name: role.name,
          enabled: true
        })
      }
    })
    
    // 移除不存在的角色
    replySettings.value = replySettings.value.filter(s => existingIds.has(s.id))
  }
})

// 保存群聊历史
watch(messages, (newVal, oldVal) => {
  saveConfig('ppc.group.messages', JSON.stringify(newVal))
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

// 获取指定 Agent 当前部位的对应台词
async function getCurrentBodyLines(agentId) {
  let cur = {}
  try {
    const saved = await getConfig(`ppc.${agentId}.waifu.texts`)
    if (saved) cur = JSON.parse(saved)
  } catch { /* ignore */ }

  const lines = {
    head: [],
    chest: [],
    body: []
  }

  // 提取部位台词 (click_head_01, click_head_02...)
  for (let i = 1; i <= 20; i++) {
    const slot = String(i).padStart(2, '0')
    if (cur[`click_head_${slot}`]) lines.head.push(cur[`click_head_${slot}`])
    if (cur[`click_chest_${slot}`]) lines.chest.push(cur[`click_chest_${slot}`])
    if (cur[`click_body_${slot}`]) lines.body.push(cur[`click_body_${slot}`])
  }

  // 如果没有台词，返回空
  if (lines.head.length === 0 && lines.chest.length === 0 && lines.body.length === 0) {
    return null
  }

  return `[当前交互台词设定]
头部: ${lines.head.join(' / ') || '无'}
胸部: ${lines.chest.join(' / ') || '无'}
身体: ${lines.body.join(' / ') || '无'}`
}

// 获取当前环境信息 Prompt (时间、地点、天气)
async function getEnvPrompt() {
  const d = new Date()
  const weekdays = ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六']
  const timeStr = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')} ${weekdays[d.getDay()]}`
  
  const location = await getConfig('ppc.location') || '未知地点'
  const weather = await getConfig('ppc.weather') || '未知天气'
  
  return `[当前环境信息]\n时间: ${timeStr}\n地点: ${location}\n天气: ${weather}`
}

async function generateResponse(agentId) {
  processingAgent.value = roles[agentId]?.name
  
  try {
    const config = roles[agentId]
    
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
    const otherAgents = Object.values(roles)
      .filter(a => a.name !== config.name)
      .map(a => a.name)
      .join('、')
    
    // 获取当前部位台词
    const bodyLines = await getCurrentBodyLines(agentId)
    const bodyLinesPrompt = bodyLines ? `\n${bodyLines}\n` : ''

    // 获取环境信息
    const envPrompt = await getEnvPrompt()
    
    // 获取用户信息
    const userName = await getConfig('ppc.userName') || ''
    const userPersona = await getConfig('ppc.userPersonaText') || ''
    const userSettingPrompt = (userName || userPersona) 
      ? `\n[用户信息]\n姓名: ${userName || '主人'}\n描述: ${userPersona || '暂无描述'}\n`
      : ''

    const systemPrompt = config.persona_prompt + '\n' + config.system_prompt + memoryContext + bodyLinesPrompt + '\n' + envPrompt + userSettingPrompt + '\n' + `
<Output_Constraint>
回复要求:
1. 你的名字是 ${config.name}。
2. 你正在大家庭的群聊中，成员包括你、${otherAgents}和主人。
3. 请根据上下文回复用户或其他人的话。
4. 最好控制在3句话，30字以内，保持简短；将行为动作用括号“（）”包裹来表示。
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
/* ============================================
   萌系可爱风UI - 家庭群聊
   ============================================ */

.group-chat-container {
  min-height: 100vh;
  background: linear-gradient(180deg, #fffafb 0%, #fef3f8 50%, #fdf4ff 100%);
  position: relative;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 装饰性背景元素 */
.bg-decoration {
  position: fixed;
  border-radius: 50%;
  opacity: 0.4;
  pointer-events: none;
  z-index: 0;
  filter: blur(60px);
}

.bg-1 {
  width: 300px;
  height: 300px;
  background: linear-gradient(135deg, #fce7f3 0%, #fbcfe8 100%);
  top: -100px;
  left: -100px;
}

.bg-2 {
  width: 250px;
  height: 250px;
  background: linear-gradient(135deg, #ede9fe 0%, #ddd6fe 100%);
  top: 40%;
  right: -80px;
}

.bg-3 {
  width: 280px;
  height: 280px;
  background: linear-gradient(135deg, #dbeafe 0%, #bfdbfe 100%);
  bottom: -120px;
  left: 20%;
}

/* 顶部导航 */
.header {
  position: sticky;
  top: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.85rem 1.25rem;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(25px);
  -webkit-backdrop-filter: blur(25px);
  border-bottom: 2px solid rgba(251, 207, 232, 0.5);
  z-index: 100;
  box-shadow: 0 4px 20px rgba(236, 72, 153, 0.08);
}

.back-btn {
  width: 44px;
  height: 44px;
  border-radius: 16px;
  background: linear-gradient(135deg, #fff1f2 0%, #ffe4e6 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  color: #db2777;
  border: 2px solid #fecaca;
  box-shadow: 0 2px 8px rgba(236, 72, 153, 0.15);
}

.back-btn:hover {
  transform: scale(0.95) translateY(-1px);
  box-shadow: 0 4px 12px rgba(236, 72, 153, 0.25);
  background: linear-gradient(135deg, #fecaca 0%, #fda4af 100%);
}

.back-btn .el-icon {
  font-size: 1.3rem;
}

.header-title {
  text-align: center;
  flex: 1;
}

.header-title h2 {
  margin: 0;
  font-size: 1.15rem;
  color: #831843;
  font-weight: 700;
  letter-spacing: 0.5px;
}

.header-title .subtitle {
  font-size: 0.8rem;
  color: #a855f7;
  font-weight: 500;
}

.header-actions {
  display: flex;
  gap: 0.75rem;
  width: 44px;
  justify-content: flex-end;
}

.action-trigger {
  width: 44px;
  height: 44px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  color: #a855f7;
  background: linear-gradient(135deg, #f5f3ff 0%, #ede9fe 100%);
  border: 2px solid #ddd6fe;
  box-shadow: 0 2px 8px rgba(168, 85, 247, 0.15);
}

.action-trigger:hover {
  transform: scale(0.95) translateY(-1px);
  box-shadow: 0 4px 12px rgba(168, 85, 247, 0.25);
  background: linear-gradient(135deg, #ddd6fe 0%, #c4b5fd 100%);
}

.action-trigger.active {
  background: linear-gradient(135deg, #a855f7 0%, #7c3aed 100%);
  color: white;
  border-color: #7c3aed;
  box-shadow: 0 4px 16px rgba(168, 85, 247, 0.4);
}

/* 回复设置面板 */
.reply-settings-panel {
  background: rgba(255, 255, 255, 0.92);
  backdrop-filter: blur(30px);
  -webkit-backdrop-filter: blur(30px);
  padding: 1rem 1.25rem;
  border-bottom: 2px solid rgba(221, 214, 254, 0.6);
  animation: slideDownBounce 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
  z-index: 99;
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.85rem;
}

.settings-header span:first-child {
  font-weight: 700;
  color: #7c3aed;
  font-size: 0.95rem;
}

.settings-hint {
  font-size: 0.75rem;
  color: #a78bfa;
}

.agent-control-list {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.agent-control-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  background: linear-gradient(135deg, #faf5ff 0%, #f5f3ff 100%);
  border-radius: 16px;
  border: 2px solid #e9d5ff;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.agent-control-item:hover {
  border-color: #c084fc;
  background: linear-gradient(135deg, #f3e8ff 0%, #ede9fe 100%);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(192, 132, 252, 0.2);
}

.agent-control-item.disabled {
  opacity: 0.55;
  filter: grayscale(30%);
}

.agent-control-item.dragging {
  opacity: 0.9;
  border-color: #a855f7;
  background: linear-gradient(135deg, #e9d5ff 0%, #ddd6fe 100%);
  box-shadow: 0 6px 20px rgba(168, 85, 247, 0.3);
  transform: scale(1.02);
}

.drag-handle {
  cursor: grab;
  color: #c4b5fd;
  padding: 0.25rem;
  transition: all 0.2s;
}

.drag-handle:hover {
  color: #a855f7;
  transform: scale(1.1);
}

.drag-handle:active {
  cursor: grabbing;
}

.agent-avatar-mini {
  width: 34px;
  height: 34px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 0.9rem;
  color: white;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  overflow: hidden;
}

.avatar-mini-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 50%;
}

.agent-avatar-mini.pero {
  background: linear-gradient(135deg, #ec4899 0%, #db2777 100%);
}

.agent-avatar-mini.nana {
  background: linear-gradient(135deg, #a855f7 0%, #7c3aed 100%);
}

.agent-info {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: 1;
}

.agent-name {
  font-weight: 600;
  color: #581c87;
  font-size: 0.95rem;
}

.agent-toggle {
  display: flex;
  align-items: center;
}

/* 消息列表 */
.chat-list {
  flex: 1;
  overflow-y: auto;
  padding: 1.25rem;
  padding-bottom: 130px;
  display: flex;
  flex-direction: column;
  gap: 1.1rem;
  position: relative;
  z-index: 1;
}

/* 消息行 */
.message-row {
  display: flex;
  gap: 0.5rem;
  align-items: flex-start;
  max-width: 100%;
  width: 100%;
  animation: messageSlideIn 0.35s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.message-row.user-row {
  flex-direction: row-reverse;
}

/* 头像 */
.avatar {
  flex-shrink: 0;
}

.avatar-img {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 1rem;
  color: white;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  border: 3px solid white;
  overflow: hidden;
}

.avatar-real-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 50%;
}

.avatar-img:hover {
  transform: scale(1.12) rotate(-5deg);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.25);
}

.avatar-img.pero {
  background: linear-gradient(135deg, #f472b6 0%, #db2777 100%);
}

.avatar-img.nana {
  background: linear-gradient(135deg, #a78bfa 0%, #7c3aed 100%);
}

.avatar-img.user {
  background: linear-gradient(135deg, #60a5fa 0%, #3b82f6 100%);
}

/* 消息气泡容器 */
.message-bubble-container {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  width: 100%;
}

.user-row .message-bubble-container {
  align-items: flex-end;
}

/* 气泡 */
.message-bubble {
  width: 100%;
  max-width: 100%;
  padding: 1rem 1.25rem;
  box-shadow: 0 3px 12px rgba(0, 0, 0, 0.06);
  position: relative;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  border: 2px solid var(--bubble-border, rgba(251, 207, 232, 0.6));
  word-wrap: break-word;
  overflow-wrap: break-word;
  box-sizing: border-box;
}

.message-bubble:hover {
  box-shadow: 0 5px 16px rgba(0, 0, 0, 0.1);
  transform: translateY(-1px);
}

/* 用户气泡 */
.user-bubble {
  background: linear-gradient(135deg, #60a5fa 0%, #3b82f6 100%);
  color: white;
  border-radius: 22px;
  border-bottom-right-radius: 8px;
  border-color: rgba(59, 130, 246, 0.35);
}

/* 角色气泡 - 使用CSS变量 */
.agent-bubble {
  background: var(--bubble-bg, #fff1f2);
  color: var(--bubble-text, #831843);
  border-color: var(--bubble-border, rgba(251, 207, 232, 0.6));
}

.agent-bubble.bubble-shape-rounded {
  border-radius: 22px;
  border-top-left-radius: 8px;
}

.agent-bubble.bubble-shape-cloud {
  border-radius: 26px;
  border-top-left-radius: 12px;
  border-bottom-right-radius: 12px;
}

.agent-bubble.bubble-shape-speech {
  border-radius: 22px;
  border-top-left-radius: 8px;
}

.sender-name {
  font-size: 0.8rem;
  font-weight: 700;
  margin-bottom: 0.35rem;
  letter-spacing: 0.5px;
}

.bubble-content {
  line-height: 1.7;
  word-wrap: break-word;
  overflow-wrap: break-word;
  font-size: 0.95rem;
}

/* 消息操作 */
.message-actions {
  display: flex;
  gap: 0.4rem;
  opacity: 0;
  transition: all 0.2s;
  padding-left: 0.25rem;
}

.message-row:hover .message-actions {
  opacity: 1;
}

.action-btn {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  font-size: 0.85rem;
  background: rgba(255, 255, 255, 0.9);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.action-btn:hover {
  transform: scale(1.15);
}

.action-btn.delete {
  color: #f43f5e;
}

.action-btn.delete:hover {
  background: #fee2e2;
}

/* 打字指示器 */
.typing-indicator {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.6rem 0;
  color: #a855f7;
  font-weight: 500;
}

.typing-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: linear-gradient(135deg, #a78bfa 0%, #7c3aed 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 0.95rem;
  box-shadow: 0 3px 10px rgba(168, 85, 247, 0.3);
  animation: avatarPulse 1.5s ease-in-out infinite;
}

.typing-bubble {
  display: flex;
  gap: 0.3rem;
  padding: 0.75rem 1rem;
  background: linear-gradient(135deg, #f5f3ff 0%, #ede9fe 100%);
  border-radius: 16px;
  border: 2px solid #e9d5ff;
  box-shadow: 0 3px 10px rgba(168, 85, 247, 0.15);
}

.typing-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: linear-gradient(135deg, #a78bfa 0%, #7c3aed 100%);
  animation: typingBounce 1.4s infinite ease-in-out both;
  box-shadow: 0 2px 6px rgba(168, 85, 247, 0.3);
}

.typing-dot:nth-child(1) { animation-delay: -0.32s; }
.typing-dot:nth-child(2) { animation-delay: -0.16s; }

.typing-text {
  font-size: 0.85rem;
  color: #a855f7;
}

/* 输入栏 */
.input-bar-container {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 0.85rem 1.25rem;
  background: rgba(255, 255, 255, 0.92);
  backdrop-filter: blur(30px);
  -webkit-backdrop-filter: blur(30px);
  border-top: 2px solid rgba(251, 207, 232, 0.6);
  z-index: 100;
  box-shadow: 0 -4px 20px rgba(236, 72, 153, 0.08);
}

.input-bar {
  display: flex;
  gap: 0.85rem;
  align-items: flex-end;
}

.chat-input {
  flex: 1;
  border: 3px solid #fce7f3;
  border-radius: 20px;
  padding: 0.85rem 1.1rem;
  font-size: 1rem;
  font-family: inherit;
  resize: none;
  max-height: 130px;
  outline: none;
  background: linear-gradient(135deg, #fff 0%, #fffafb 100%);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  line-height: 1.6;
  color: #831843;
  font-weight: 500;
  box-shadow: 0 2px 8px rgba(236, 72, 153, 0.08);
}

.chat-input::placeholder {
  color: #f9a8d4;
}

.chat-input:focus {
  border-color: #f472b6;
  box-shadow: 0 0 0 4px rgba(244, 114, 182, 0.15), 0 4px 12px rgba(236, 72, 153, 0.15);
  background: white;
}

.send-btn {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  border: none;
  background: linear-gradient(135deg, #ec4899 0%, #db2777 100%);
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  font-size: 1.2rem;
  flex-shrink: 0;
  box-shadow: 0 4px 16px rgba(236, 72, 153, 0.4);
}

.send-btn:hover:not(:disabled) {
  transform: scale(1.1) rotate(-5deg);
  box-shadow: 0 6px 20px rgba(236, 72, 153, 0.5);
}

.send-btn:active:not(:disabled) {
  transform: scale(0.95);
}

.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  box-shadow: 0 2px 8px rgba(236, 72, 153, 0.2);
}

/* 加载更多 */
.load-more-container {
  display: flex;
  justify-content: center;
  margin-bottom: 1rem;
}

.load-more-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.6rem 1.2rem;
  background: linear-gradient(135deg, #fff 0%, #faf5ff 100%);
  border-radius: 2rem;
  border: 2px solid #e9d5ff;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  color: #7c3aed;
  font-size: 0.85rem;
  font-weight: 600;
  box-shadow: 0 2px 8px rgba(168, 85, 247, 0.1);
}

.load-more-btn:hover {
  background: linear-gradient(135deg, #f5f3ff 0%, #ede9fe 100%);
  border-color: #c084fc;
  color: #6d28d9;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(168, 85, 247, 0.2);
}

/* Markdown 样式 */
.markdown-body {
  color: inherit;
}

.markdown-body p {
  margin: 0.5rem 0;
}

.markdown-body p:first-child {
  margin-top: 0;
}

.markdown-body p:last-child {
  margin-bottom: 0;
}

.markdown-body strong {
  font-weight: 700;
}

.markdown-body em {
  font-style: italic;
}

.markdown-body code {
  background: rgba(0, 0, 0, 0.08);
  padding: 0.15rem 0.45rem;
  border-radius: 0.5rem;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 0.9em;
}

.user-bubble .markdown-body code {
  background: rgba(255, 255, 255, 0.25);
}

/* 动画 */
@keyframes typingBounce {
  0%, 80%, 100% { 
    transform: scale(0.7);
    opacity: 0.5;
  }
  40% { 
    transform: scale(1.1);
    opacity: 1;
  }
}

@keyframes avatarPulse {
  0%, 100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.08);
  }
}

@keyframes messageSlideIn {
  0% {
    opacity: 0;
    transform: translateY(20px) scale(0.92);
  }
  100% {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes slideDownBounce {
  0% {
    opacity: 0;
    transform: translateY(-20px);
  }
  60% {
    transform: translateY(5px);
  }
  100% {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 过渡动画 */
.slide-down-enter-active,
.slide-down-leave-active {
  transition: all 0.35s cubic-bezier(0.4, 0, 0.2, 1);
}

.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-15px);
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}

/* 滚动条美化 */
.chat-list::-webkit-scrollbar {
  width: 8px;
}

.chat-list::-webkit-scrollbar-track {
  background: transparent;
  margin: 10px 0;
}

.chat-list::-webkit-scrollbar-thumb {
  background: linear-gradient(135deg, #f472b6 0%, #c084fc 100%);
  border-radius: 4px;
  box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.5);
}

.chat-list::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(135deg, #ec4899 0%, #a855f7 100%);
}

/* 响应式 */
@media (max-width: 600px) {
  .header {
    padding: 0.7rem 1rem;
  }

  .header-title h2 {
    font-size: 1rem;
  }

  .chat-list {
    padding: 1rem;
    padding-bottom: 110px;
    gap: 1rem;
  }

  .input-bar-container {
    padding: 0.7rem 1rem;
  }

  .message-bubble {
    max-width: 95%;
    padding: 0.9rem 1.1rem;
  }

  .avatar-img {
    width: 44px;
    height: 44px;
  }
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