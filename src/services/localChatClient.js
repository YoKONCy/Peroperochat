export function normalizeError(value) {
  if (value instanceof Error) return value
  const error = new Error(typeof value === 'string' ? value : value?.message || '本地操作失败')
  error.code = value?.code || 'LOCAL_ERROR'
  if (typeof value?.outcome === 'string') error.outcome = value.outcome
  return error
}

// 先注册观察者，再发起命令；命令结果是终态和持久化事实的唯一来源。
export function createLocalChatClient({ invoke, listen, uuid = () => crypto.randomUUID() }) {
  return {
    async snapshot(agentId, channel = 'desktop') {
      try { return await invoke('local_snapshot', { agentId, channel }) }
      catch (error) { throw normalizeError(error) }
    },

    async deleteMessage(threadId, messageId, pair = true) {
      try { return await invoke('local_delete_message', { threadId, messageId, pair }) }
      catch (error) { throw normalizeError(error) }
    },

    async turn(input, { signal, onEvent, command = 'local_chat' } = {}) {
      const request = { ...input, requestId: input.requestId || uuid() }
      let active = true
      let cancellationError = null
      const cancel = () => {
        // 停止观察与取消执行分别处理；取消失败不能冒充成功。
        invoke('local_cancel_chat', { requestId: request.requestId }).catch(error => {
          cancellationError = normalizeError(error)
        })
      }
      if (signal?.aborted) {
        const error = new Error('已停止回复')
        error.code = 'CANCELLED'
        throw error
      }
      let unlisten
      try {
        unlisten = await listen('local-chat-event', ({ payload }) => {
          if (!active || payload.requestId !== request.requestId ||
              payload.threadId !== request.threadId || (request.agentId && payload.agentId !== request.agentId)) return
          if (payload.kind === 'started' && signal?.aborted) cancel()
          if (!signal?.aborted) onEvent?.(payload)
        })
        signal?.addEventListener('abort', cancel, { once: true })
        if (signal?.aborted) {
          const error = new Error('已停止回复')
          error.code = 'CANCELLED'
          throw error
        }
        const result = await invoke(command, { request })
        if (cancellationError) throw cancellationError
        return result
      } catch (error) {
        throw normalizeError(error)
      } finally {
        active = false
        signal?.removeEventListener('abort', cancel)
        unlisten?.()
      }
    },
  }
}
