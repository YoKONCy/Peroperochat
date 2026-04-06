<template>
  <div class="remote-chat-view pixel-font">
    <!-- 聊天记录区域 -->
    <div class="chat-messages" ref="scrollContainer">
      <div v-for="msg in messages" :key="msg.id || msg.timestamp" 
           :class="['message-row', msg.role === 'user' ? 'user-row' : 'assistant-row']">
        
        <div class="avatar-box pixel-border-sm" v-if="msg.role === 'assistant'">
          <img :src="getFullUrl(getAgentAvatar(msg.senderId))" class="avatar-img" />
        </div>

        <div class="bubble-container">
          <div class="sender-name" v-if="msg.role === 'assistant'">{{ msg.senderId || 'Assistant' }}</div>
          <div :class="['bubble', msg.role === 'user' ? 'user-bubble' : 'assistant-bubble', 'pixel-border-sm']">
            <div class="message-content markdown-body" v-html="renderMarkdown(msg.content)"></div>
            <div v-if="msg.isStreaming" class="streaming-dot"></div>
          </div>
          <div class="message-time">{{ formatTime(msg.timestamp) }}</div>
        </div>

        <div class="avatar-box pixel-border-sm" v-if="msg.role === 'user'">
          <div class="user-icon">Me</div>
        </div>
      </div>
    </div>

    <!-- 输入区域 -->
    <div class="input-area pixel-border-sm">
      <textarea 
        v-model="userInput" 
        placeholder="发送指令到 PeroCore..." 
        class="pixel-textarea"
        @keydown.enter.prevent="sendMessage"
      ></textarea>
      <button class="send-btn pixel-btn" @click="sendMessage" :disabled="!userInput.trim() || isSending">
        <i class="fas fa-paper-plane"></i>
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick } from 'vue'
import { marked } from 'marked'
import DOMPurify from 'dompurify'

const props = defineProps({
  pcUrl: String,
  token: String
})

const messages = ref([])
const userInput = ref('')
const isSending = ref(false)
const scrollContainer = ref(null)
const agents = ref([])

const getFullUrl = (path) => {
  if (!path) return ''
  if (path.startsWith('http')) return path
  return `${props.pcUrl.replace('/api', '')}${path}`
}

const getAgentAvatar = (id) => {
  const agent = agents.value.find(a => a.id === id)
  return agent?.avatar || '/api/agents/pero/avatar'
}

const formatTime = (ts) => {
  if (!ts) return ''
  const date = new Date(ts * 1000)
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

const renderMarkdown = (text) => {
  if (!text) return ''
  const html = marked.parse(text)
  return DOMPurify.sanitize(html)
}

const scrollToBottom = async () => {
  await nextTick()
  if (scrollContainer.value) {
    scrollContainer.value.scrollTop = scrollContainer.value.scrollHeight
  }
}

const fetchHistory = async () => {
  try {
    const res = await fetch(`${props.pcUrl}/api/history?limit=30`, {
      headers: { 'Authorization': `Bearer ${props.token}` }
    })
    if (res.ok) {
      const data = await res.json()
      messages.value = data.messages.reverse()
      scrollToBottom()
    }
  } catch (e) {
    console.error('获取历史记录失败:', e)
  }
}

const fetchAgents = async () => {
  try {
    const res = await fetch(`${props.pcUrl}/api/agents`, {
      headers: { 'Authorization': `Bearer ${props.token}` }
    })
    if (res.ok) {
      agents.value = await res.json()
    }
  } catch (e) {
    console.error('获取 Agent 列表失败:', e)
  }
}

const sendMessage = async () => {
  if (!userInput.value.trim() || isSending.value) return
  
  const content = userInput.value
  userInput.value = ''
  isSending.value = true

  // 乐观更新
  const userMsg = {
    role: 'user',
    content: content,
    timestamp: Date.now() / 1000
  }
  messages.value.push(userMsg)
  scrollToBottom()

  try {
    const res = await fetch(`${props.pcUrl}/api/chat/stream`, {
      method: 'POST',
      headers: { 
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${props.token}`
      },
      body: JSON.stringify({
        message: content,
        agent_id: 'active'
      })
    })

    if (res.ok) {
      const reader = res.body.getReader()
      const decoder = new TextDecoder()
      let assistantMsg = {
        role: 'assistant',
        content: '',
        timestamp: Date.now() / 1000,
        isStreaming: true,
        senderId: 'pero' // 默认，稍后可能更新
      }
      messages.value.push(assistantMsg)

      while (true) {
        const { done, value } = await reader.read()
        if (done) break
        
        const chunk = decoder.decode(value)
        const lines = chunk.split('\n')
        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const data = line.slice(6)
            if (data === '[DONE]') {
              assistantMsg.isStreaming = false
              break
            }
            try {
              const json = JSON.parse(data)
              if (json.content) {
                assistantMsg.content += json.content
                scrollToBottom()
              }
            } catch {
              // ignore invalid JSON chunk
            }
          }
        }
      }
    }
  } catch (e) {
    console.error('发送失败:', e)
    messages.value.push({
      role: 'system',
      content: '发送失败，请检查网络连接',
      timestamp: Date.now() / 1000
    })
  } finally {
    isSending.value = false
  }
}

onMounted(() => {
  fetchAgents()
  fetchHistory()
})
</script>

<style scoped>
.remote-chat-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #000;
}

.chat-messages {
  flex: 1;
  overflow-y: auto;
  padding: 15px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.message-row {
  display: flex;
  gap: 12px;
  max-width: 85%;
}

.user-row {
  align-self: flex-end;
  flex-direction: row;
}

.assistant-row {
  align-self: flex-start;
}

.avatar-box {
  width: 36px;
  height: 36px;
  flex-shrink: 0;
  background: #333;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.user-icon {
  font-size: 10px;
  color: #fff;
}

.bubble-container {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.user-row .bubble-container {
  align-items: flex-end;
}

.sender-name {
  font-size: 10px;
  color: #0ff;
  text-transform: uppercase;
}

.bubble {
  padding: 10px 14px;
  font-size: 14px;
  line-height: 1.5;
  word-break: break-word;
}

.user-bubble {
  background: #004488;
  color: #fff;
}

.assistant-bubble {
  background: #222;
  color: #0f0;
}

.message-time {
  font-size: 8px;
  color: #666;
}

.streaming-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  background: #0f0;
  margin-left: 5px;
  animation: blink 1s infinite;
}

@keyframes blink {
  0%, 100% { opacity: 0; }
  50% { opacity: 1; }
}

.input-area {
  display: flex;
  padding: 10px;
  gap: 10px;
  background: #111;
}

.pixel-textarea {
  flex: 1;
  height: 50px;
  background: #000;
  border: 2px solid #fff;
  color: #0f0;
  padding: 8px;
  font-family: inherit;
  font-size: 14px;
  outline: none;
  resize: none;
}

.send-btn {
  width: 50px;
  background: #0088ff;
  display: flex;
  align-items: center;
  justify-content: center;
}

.pixel-border-sm {
  border: 2px solid #fff;
}

/* Markdown 样式 */
:deep(.markdown-body) {
  font-size: 14px;
}
:deep(.markdown-body p) {
  margin-bottom: 8px;
}
:deep(.markdown-body code) {
  background: #1a1a1a;
  padding: 2px 4px;
  border: 1px solid #444;
}
</style>
