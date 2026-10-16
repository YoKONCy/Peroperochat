<template>
  <div class="remote-chat-view">
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
        @keydown="handleComposerKey"
      ></textarea>
      <button class="send-btn pixel-btn" aria-label="发送消息" @click="sendMessage" :disabled="!userInput.trim() || isSending">
        <RoomIcon name="send" />
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick } from 'vue'
import { marked } from 'marked'
import DOMPurify from 'dompurify'
import RoomIcon from '../companion/RoomIcon.vue'

const props = defineProps({
  pcUrl: String,
  token: String
})

const messages = ref([])
function handleComposerKey(event) { if (event.key === 'Enter' && !event.shiftKey && !event.isComposing && event.keyCode !== 229) { event.preventDefault(); sendMessage() } }
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
.remote-chat-view { height: 100%; display: flex; flex-direction: column; background: var(--room-page); }
.chat-messages { flex: 1; min-height: 0; overflow-y: auto; overscroll-behavior: contain; padding: 24px max(18px, calc((100% - 800px) / 2)); }
.message-row { display: flex; align-items: flex-start; gap: 10px; margin-bottom: 20px; }
.user-row { justify-content: flex-end; }
.avatar-box { width: 34px; height: 38px; border-radius: 10px; overflow: hidden; background: var(--room-accent-soft); flex: none; }
.avatar-img { width: 100%; height: 100%; object-fit: cover; }
.user-icon { display: grid; place-items: center; height: 100%; color: var(--room-muted); font-size: 11px; }
.bubble-container { min-width: 0; max-width: min(80%, 600px); }
.sender-name { font-size: 11px; color: var(--room-muted); margin: 0 0 7px; }
.bubble { background: var(--room-surface); color: var(--room-ink); border: 1px solid var(--room-line); border-radius: 4px 15px 15px; padding: 12px 15px; }
.user-bubble { background: var(--room-user-bubble); border-color: transparent; border-radius: 15px 4px 15px 15px; }
.message-time { margin-top: 6px; font-size: 10px; color: var(--room-muted); }
.user-row .message-time { text-align: right; }
.streaming-dot { width: 5px; height: 5px; margin-top: 7px; border-radius: 50%; background: var(--room-accent); animation: remote-pulse 1s infinite; }
.input-area { display: flex; align-items: flex-end; gap: 10px; padding: 12px max(16px, calc((100% - 800px) / 2)) calc(12px + env(safe-area-inset-bottom)); background: var(--room-surface); border-top: 1px solid var(--room-line); }
.pixel-textarea { flex: 1; min-width: 0; min-height: 44px; max-height: 120px; padding: 11px 12px; border: 1px solid var(--room-line); border-radius: 11px; outline: 0; resize: vertical; background: var(--room-surface); color: var(--room-ink); font: inherit; font-size: 16px; line-height: 22px; }
.pixel-textarea:focus { border-color: var(--room-accent); }
.send-btn { display: grid; place-items: center; width: 44px; height: 44px; border: 0; border-radius: 11px; flex: none; background: var(--room-accent); color: white !important; }
.send-btn:disabled { background: var(--room-accent-soft); color: var(--room-accent) !important; }
@keyframes remote-pulse { 0%, 100% { opacity: .3; } 50% { opacity: 1; } }
</style>
