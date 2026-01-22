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
      <div class="placeholder"></div>
    </div>

    <!-- 消息列表 -->
    <div class="chat-list" ref="chatListRef">
      <div v-for="(msg, index) in messages" :key="index" :class="['message-row', msg.role === 'user' ? 'user-row' : 'agent-row']">
        <!-- 头像 -->
        <div class="avatar" v-if="msg.role !== 'user'">
          <div class="avatar-img" :class="msg.name.toLowerCase()">
            {{ msg.name[0] }}
          </div>
        </div>
        
        <!-- 消息体 -->
        <div class="message-bubble">
          <div class="sender-name" v-if="msg.role !== 'user'">{{ msg.name }}</div>
          <div class="bubble-content markdown-body" v-html="renderMarkdown(msg.content)"></div>
        </div>

        <!-- 用户头像 (右侧) -->
        <div class="avatar user" v-if="msg.role === 'user'">
          <div class="avatar-img user">Me</div>
        </div>
      </div>
      
      <div v-if="processingAgent" class="typing-indicator">
        <span>{{ processingAgent }} 正在输入...</span>
      </div>
    </div>

    <!-- 输入栏 -->
    <div class="input-bar">
      <textarea 
        v-model="input" 
        class="chat-input" 
        placeholder="..." 
        @keyup.ctrl.enter="sendMessage"
      ></textarea>
      <button class="send-btn" @click="sendMessage" :disabled="!!processingAgent || !input.trim()">
        <el-icon><Promotion /></el-icon>
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowLeft, Promotion } from '@element-plus/icons-vue'
import { marked } from 'marked'
import DOMPurify from 'dompurify'
import { chat, AGENTS, getDefaultPrompts, getRelevantMemories, saveMemory } from '../api'

const router = useRouter()
const messages = ref([])
const input = ref('')
const chatListRef = ref(null)
const processingAgent = ref(null) // 当前正在回复的角色名

// 颜色映射
const AGENT_COLORS = {
  pero: '#ec4899', // Pink
  nana: '#8b5cf6'  // Purple
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
})

// 保存群聊历史
watch(messages, (newVal) => {
  localStorage.setItem('ppc.group.messages', JSON.stringify(newVal))
  scrollToBottom()
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
  
  // 2. 触发群聊回复逻辑 (顺序: Pero -> Nana)
  // 可以随机顺序，或者根据 @ 提及？简单起见，这里固定顺序或随机
  const agents = ['pero', 'nana']
  // 简单的轮流发言逻辑，或者都发言
  
  for (const agentId of agents) {
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
2. 你正在一个群聊中，成员包括你、其他AI和用户。
3. 请根据上下文回复用户或其他人的话。
4. 字数控制在 50 字以内，保持简短。
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
  background: #f1f5f9;
}

.header {
  background: white;
  padding: 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid #e2e8f0;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
}

.back-btn {
  padding: 8px;
  cursor: pointer;
}

.header-title {
  text-align: center;
}
.header-title h2 { margin: 0; font-size: 18px; }
.header-title span { font-size: 12px; color: #64748b; }

.placeholder { width: 32px; }

.chat-list {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.message-row {
  display: flex;
  gap: 10px;
  max-width: 85%;
}

.user-row {
  align-self: flex-end;
  flex-direction: row-reverse;
}

.agent-row {
  align-self: flex-start;
}

.avatar-img {
  width: 40px;
  height: 40px;
  border-radius: 20px;
  background: #cbd5e1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: bold;
  font-size: 14px;
}

.avatar-img.pero { background: #ec4899; }
.avatar-img.nana { background: #8b5cf6; }
.avatar-img.user { background: #3b82f6; }

.message-bubble {
  background: white;
  padding: 10px 14px;
  border-radius: 16px;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  position: relative;
}

.user-row .message-bubble {
  background: #3b82f6;
  color: white;
  border-bottom-right-radius: 4px;
}

.agent-row .message-bubble {
  border-top-left-radius: 4px;
}

.sender-name {
  font-size: 12px;
  color: #64748b;
  margin-bottom: 4px;
}

.bubble-content {
  font-size: 14px;
  line-height: 1.5;
  word-break: break-word;
}

.user-row .bubble-content {
  color: white;
}

.input-bar {
  background: white;
  padding: 10px 16px;
  display: flex;
  gap: 10px;
  align-items: flex-end;
  border-top: 1px solid #e2e8f0;
}

.chat-input {
  flex: 1;
  border: 1px solid #e2e8f0;
  border-radius: 20px;
  padding: 10px 16px;
  font-family: inherit;
  font-size: 14px;
  resize: none;
  height: 44px;
  line-height: 20px;
  outline: none;
}

.send-btn {
  width: 44px;
  height: 44px;
  border-radius: 22px;
  background: #3b82f6;
  color: white;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.send-btn:disabled {
  background: #cbd5e1;
  cursor: not-allowed;
}

.typing-indicator {
  font-size: 12px;
  color: #94a3b8;
  text-align: center;
  padding: 5px;
}
</style>