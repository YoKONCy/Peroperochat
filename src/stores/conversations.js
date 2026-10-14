import { computed, shallowReactive } from 'vue'
import { localChat, publishCharacterState } from '../services/localChat'
import { activeAgentId } from './agents'
import { displayMessage, visibleMessage } from '../utils/messageContent'

const conversations = new Map()

function entryFor(threadId) {
  if (!conversations.has(threadId)) conversations.set(threadId, shallowReactive({
    threadId, messages: [], busy: false, loaded: false, revision: 0, controller: null, speaker: null,
  }))
  return conversations.get(threadId)
}

function mergeMessage(entry, message) {
  const items = entry.messages.slice()
  const index = items.findIndex(item => item.id === message.id)
  if (index < 0) items.push(message)
  else items[index] = message
  entry.messages = items.sort((a, b) => a.id - b.id)
  entry.revision++
}

async function refresh(entry, agentId, channel, force = false) {
  const revision = entry.revision
  const snapshot = await localChat.snapshot(agentId, channel)
  if (force || revision === entry.revision) {
    entry.messages = snapshot.messages
    entry.revision++
  }
  entry.loaded = true
  return snapshot
}

// 视图销毁只移除 Vue 观察者；正在进行的单机回复仍由后端完成并保存。
export function useLocalConversation(channel = 'desktop') {
  const current = computed(() => entryFor(channel === 'group' ? 'group:family' : `desktop:${activeAgentId.value}`))
  const messages = computed(() => current.value.messages.map(displayMessage))
  const isLoading = computed(() => current.value.busy)
  const processingAgentId = computed(() => current.value.speaker)

  async function load() {
    const entry = current.value
    await refresh(entry, activeAgentId.value, channel)
  }

  async function run(input, options = {}) {
    const agentId = options.agentId || activeAgentId.value
    const entry = entryFor(channel === 'group' ? 'group:family' : `desktop:${agentId}`)
    if (entry.busy) throw new Error('该会话正在回复，请稍后重试')
    entry.busy = true
    entry.controller = new AbortController()
    let started = false
    let preview = null
    let frameTimer = null
    const flushPreview = () => {
      frameTimer = null
      if (!preview) return
      mergeMessage(entry, preview)
      if (agentId === activeAgentId.value && channel === 'desktop') {
        window.dispatchEvent(new CustomEvent('ppc:chat', { detail: visibleMessage(preview.content) || '正在思考中...' }))
      }
    }
    try {
      if (!entry.loaded) await refresh(entry, agentId, channel)
      const command = options.command || 'local_chat'
      const request = { threadId: entry.threadId, ...(command === 'local_group_chat' ? {} : { agentId }), ...input }
      const result = await localChat.turn(request, {
        command,
        signal: entry.controller.signal,
        onEvent(event) {
          if (event.kind === 'started') {
            clearTimeout(frameTimer)
            frameTimer = null
            if (preview) flushPreview()
            entry.speaker = event.agentId
            started = true
            if (event.user) mergeMessage(entry, event.user)
            preview = { ...event.message, content: '', status: 'pending', error: null }
            mergeMessage(entry, preview)
          } else if (['done', 'error', 'cancelled'].includes(event.kind) && event.message) {
            clearTimeout(frameTimer)
            frameTimer = null
            preview = null
            mergeMessage(entry, event.message)
          } else if (event.kind === 'delta' && preview) {
            preview = { ...preview, content: preview.content + event.content }
            // 增量合并到每秒约 30 帧，避免逐 Token 重绘整个消息列表。
            if (!frameTimer) frameTimer = setTimeout(flushPreview, 33)
          }
        },
      })
      clearTimeout(frameTimer)
      frameTimer = null
      entry.messages = result.snapshot.messages
      entry.revision++
      publishCharacterState(result.state)
      if (agentId === activeAgentId.value && channel === 'desktop') {
        window.dispatchEvent(new CustomEvent('ppc:chat', { detail: visibleMessage(result.message.content) }))
      }
      return result
    } catch (error) {
      clearTimeout(frameTimer)
      frameTimer = null
      error.persisted = started
      try { await refresh(entry, agentId, channel, true) }
      catch (refreshError) { console.error('恢复聊天记录失败：', refreshError) }
      throw error
    } finally {
      clearTimeout(frameTimer)
      entry.busy = false
      entry.controller = null
      entry.speaker = null
    }
  }

  async function removeById(messageId, pair = true) {
    const entry = current.value
    if (entry.busy) throw new Error('请先停止回复，再删除消息')
    const snapshot = await localChat.deleteMessage(entry.threadId, messageId, pair)
    entry.messages = snapshot.messages
    entry.revision++
  }

  function remove(index, pair = true) {
    const message = current.value.messages[index]
    return message ? removeById(message.id, pair) : Promise.resolve()
  }

  return {
    messages, isLoading, processingAgentId, load, remove, removeById,
    send: (content, options = {}) => run({ content, ephemeral: !!options.ephemeral }, options),
    sendGroup: content => run({ content }, { command: 'local_group_chat' }),
    regenerate: (index) => {
      const message = current.value.messages[index]
      if (!message || message.role !== 'assistant') throw new Error('请选择要重新生成的回复')
      return run({ regenerateMessageId: message.id }, { agentId: message.agentId })
    },
    stop: () => current.value.controller?.abort(),
  }
}

export function clearConversationCache() {
  for (const entry of conversations.values()) {
    entry.controller?.abort()
    entry.messages = []
    entry.loaded = false
    entry.revision++
  }
}

export function projectNativeTurn(result) {
  if (!result) return
  const entry = entryFor(result.snapshot.thread.id)
  entry.messages = result.snapshot.messages
  entry.loaded = true
  entry.revision++
  publishCharacterState(result.state)
}
