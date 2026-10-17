import { reactive, shallowReactive } from 'vue'
import { remoteClient } from '../../services/remote'
import { currentToken, assertToken, onModeInvalidated } from '../mode'
import { applyRemoteFrame, createRemoteSurfaceState, installRemoteProjection } from '../../services/remoteSurface'

// 独立模式 Store 不参与远程数据投影；服务端是消息、任务与角色状态的权威。
export const remoteState = shallowReactive({ connection: 'offline', sessionId: null, agents: [], activeAgentId: null, error: null, pendingError: null, ready: false })
export const remoteViews = reactive(new Map())
export const remoteExecutions = reactive(new Map())
export const remotePendingRequests = reactive(new Map())
let disposeEvents = null
let listener = null
let generation = 0
let refreshTimer = null
let bootstrapTimer = null
let initialization = null
let pendingVersion = 0
let reconciliation = null
const blockedCodes = new Set(['REMOTE_AUTH_REQUIRED', 'REMOTE_IDENTITY_CHANGED', 'REMOTE_INCOMPATIBLE', 'CREDENTIAL_UNAVAILABLE'])
const offlineCodes = new Set(['REMOTE_NETWORK', 'REMOTE_HTTP', 'REMOTE_RATE_LIMIT'])

function key(query) { return JSON.stringify(query) }
export function clearRemoteState() {
  generation++
  clearTimeout(refreshTimer)
  refreshTimer = null
  clearTimeout(bootstrapTimer)
  bootstrapTimer = null
  initialization = null
  pendingVersion++
  reconciliation = null
  remoteState.connection = 'offline'
  remoteState.sessionId = null
  remoteState.agents = []
  remoteState.activeAgentId = null
  remoteState.error = null
  remoteState.pendingError = null
  remoteState.ready = false
  remoteViews.clear()
  remoteExecutions.clear()
  remotePendingRequests.clear()
}
onModeInvalidated(clearRemoteState)

function scheduleBootstrap(delay = 5000) {
  if (bootstrapTimer) return
  let token
  try { token = currentToken(); assertToken(token, 'remote') } catch { return }
  bootstrapTimer = setTimeout(() => {
    bootstrapTimer = null
    try { assertToken(token, 'remote') } catch { return }
    initializeRemote().catch(error => {
      try { assertToken(token, 'remote') } catch { return }
      remoteState.error = error.message
    })
  }, delay)
}

export async function refreshRemotePendingRequests() {
  const token = currentToken()
  const version = ++pendingVersion
  const pending = await remoteClient.pendingRequests(token)
  assertToken(token, 'remote')
  if (version !== pendingVersion) return
  if (!Array.isArray(pending)) throw new Error('远程待核对操作记录格式无效')
  remotePendingRequests.clear()
  for (const item of pending) remotePendingRequests.set(item.requestId, item)
  remoteState.pendingError = null
}

export async function reviewRemoteRequest(requestId, note, { confirmed = false } = {}) {
  const token = currentToken()
  assertToken(token, 'remote')
  const pending = remotePendingRequests.get(requestId)
  if (!confirmed || !pending || pending.state !== 'unknown' || !note?.trim()) throw new Error('请先核对服务端状态，并明确确认核对说明')
  await remoteClient.reviewRequest(requestId, pending.fingerprint, note, token)
  assertToken(token, 'remote')
  await refreshRemotePendingRequests()
}

export async function reconcileRemotePendingRequests() {
  const token = currentToken()
  assertToken(token, 'remote')
  const version = generation
  if (reconciliation?.version === version) return reconciliation.promise
  const promise = (async () => {
    await refreshRemotePendingRequests()
    const candidates = [...remotePendingRequests.values()].filter(item => item.state === 'unknown' && item.context?.kind === 'chat' && item.result?.messageId)
    const workers = Array.from({ length: Math.min(2, candidates.length) }, async () => {
      while (candidates.length) {
        assertToken(token, 'remote')
        await remoteClient.reconcileRequest(candidates.shift().requestId, token)
      }
    })
    const results = await Promise.allSettled(workers)
    assertToken(token, 'remote')
    await refreshRemotePendingRequests()
    const failure = results.find(result => result.status === 'rejected')
    if (failure) remoteState.pendingError = failure.reason?.message || '远程操作核对暂未完成'
  })()
  reconciliation = { version, promise }
  try { await promise } finally { if (reconciliation?.promise === promise) reconciliation = null }
}

function scheduleRefresh() {
  if (refreshTimer) return
  const token = currentToken()
  refreshTimer = setTimeout(async () => {
    refreshTimer = null
    try { assertToken(token, 'remote') } catch { return }
    const queries = [...remoteViews.values()].filter(view => view.stale && !view.loading && !(view.query.kind === 'conversation' && view.query.beforeCursor)).map(view => view.query)
    // 只刷新显示投影，限制并发；从不重新发送聊天、审批或任务派发。
    const workers = Array.from({ length: Math.min(4, queries.length) }, async () => {
      while (queries.length) {
        try { assertToken(token, 'remote') } catch { return }
        await readRemote(queries.shift()).catch(() => {})
      }
    })
    await Promise.all(workers)
  }, 150)
}

async function listenOnce() {
  if (disposeEvents) return
  if (!listener) listener = remoteClient.listen(event => {
    if (event.kind === 'connection') {
      if (event.data.state === 'offline' && event.data.sessionId !== remoteState.sessionId) return
      remoteState.connection = event.data.state
      if (event.data.state === 'online') {
        remoteState.sessionId = event.data.sessionId
        if (event.data.principalId) remoteState.activeAgentId = event.data.principalId
        if (!remoteState.ready) scheduleBootstrap(150)
        const token = currentToken()
        reconcileRemotePendingRequests().catch(error => {
          try { assertToken(token, 'remote') } catch { return }
          remoteState.pendingError = error.message
        })
      }
      if (event.data.message) remoteState.error = event.data.message
      if (['auth_required', 'incompatible'].includes(event.data.state)) { clearTimeout(bootstrapTimer); bootstrapTimer = null }
      if (event.data.state !== 'online') {
        remoteState.ready = false
        for (const view of remoteViews.values()) view.stale = true
      } else scheduleRefresh()
    }
    if (event.kind === 'gateway') {
      // Gateway 只触发读模型失效；不把通知回放当作新消息或自行计算业务状态。
      for (const view of remoteViews.values()) view.stale = true
      scheduleRefresh()
      return
    }
    const execution = remoteExecutions.get(event.requestId)
    if (!execution) return
    if (event.kind === 'surface') {
      const frame = event.data
      try {
        applyRemoteFrame(execution.presentation, frame)
        execution.needsRefresh ||= execution.presentation.needsRefresh
        execution.frame = frame
        execution.revision++
      } catch (error) { execution.needsRefresh = true; execution.error = error.message }
    } else if (event.kind === 'done' || event.kind === 'error') {
      // 事件仅表示观察结果，命令和持久化回执决定最终操作状态。
      execution.observedState = event.kind === 'done' ? 'completed' : 'failed'
      execution.needsRefresh = true
    }
  }).then(dispose => { disposeEvents = dispose }).finally(() => { listener = null })
  await listener
}
export async function readRemote(query, { cachedOnly = false, allowCached = true } = {}) {
  const token = currentToken()
  assertToken(token, 'remote')
  const id = key(query)
  if (!remoteViews.has(id) && remoteViews.size >= 64) {
    const disposable = [...remoteViews].find(([, view]) => !view.loading)
    if (disposable) remoteViews.delete(disposable[0])
    else throw new Error('远程读取请求过多，请稍后再试')
  }
  const previous = remoteViews.get(id)
  const requestVersion = (previous?.requestVersion || 0) + 1
  remoteViews.set(id, { ...previous, query, requestVersion, loading: true, error: null })
  try {
    const result = await remoteClient.query(query, { cachedOnly, token })
    assertToken(token, 'remote')
    if (remoteViews.get(id)?.requestVersion === requestVersion) {
      remoteViews.set(id, { query, requestVersion, loading: false, data: result.data, cached: result.cached, stale: result.cached, error: null })
      if (query.kind === 'conversation' && !query.beforeCursor && !result.cached && !cachedOnly) {
        for (const execution of remoteExecutions.values()) if (execution.threadId === query.id && execution.state !== 'observing') execution.needsRefresh = false
      }
      if (query.kind === 'agents' && Array.isArray(result.data)) remoteState.agents = result.data
    }
    return result.data
  } catch (error) {
    try { assertToken(token, 'remote') } catch { throw error }
    if (!cachedOnly && allowCached && offlineCodes.has(error.code)) {
      try {
        const cached = await remoteClient.query(query, { cachedOnly: true, token })
        assertToken(token, 'remote')
        if (cached.data !== null && cached.data !== undefined) {
          if (remoteViews.get(id)?.requestVersion === requestVersion) {
            remoteViews.set(id, { query, requestVersion, loading: false, data: cached.data, cached: true, stale: true, error: error.message })
            if (query.kind === 'agents' && Array.isArray(cached.data)) remoteState.agents = cached.data
          }
          return cached.data
        }
      } catch { /* 原网络错误保留给调用者，不把缓存读取失败伪装成成功。 */ }
    }
    assertToken(token, 'remote')
    const view = remoteViews.get(id)
    if (view?.requestVersion === requestVersion) Object.assign(view, { loading: false, error: error.message, stale: true })
    if (error.code === 'REMOTE_STALE_READ') scheduleRefresh()
    throw error
  }
}
export async function initializeRemote() {
  const token = currentToken()
  assertToken(token, 'remote')
  const version = generation
  if (initialization?.version === version) return initialization.promise
  const promise = (async () => {
    await listenOnce()
    await refreshRemotePendingRequests()
    const cached = await readRemote({ kind: 'agents' }, { cachedOnly: true })
    assertToken(token, 'remote')
    if (Array.isArray(cached)) {
      remoteState.agents = cached
      remoteState.activeAgentId ||= cached.find(agent => agent.isEnabled)?.id || cached[0]?.id || null
    }
    try {
      // 原生先持有恢复目标；即使没有缓存角色且此刻离线，也会继续尝试发现角色。
      const connected = await remoteClient.connect(remoteState.activeAgentId, token)
      assertToken(token, 'remote')
      remoteState.sessionId = connected.sessionId
      remoteState.connection = 'online'
      remoteState.activeAgentId = connected.principalId
      const agents = await readRemote({ kind: 'agents' }, { allowCached: false })
      assertToken(token, 'remote')
      if (version !== generation) return
      if (!Array.isArray(agents)) throw new Error('远程角色列表格式无效')
      remoteState.agents = agents
      const principal = agents.find(agent => agent.id === remoteState.activeAgentId)?.id || agents.find(agent => agent.isEnabled)?.id || agents[0]?.id
      if (!principal) throw Object.assign(new Error('服务端尚无可用角色'), { code: 'REMOTE_NOT_FOUND' })
      if (principal !== remoteState.activeAgentId) {
        const selected = await remoteClient.connect(principal, token)
        assertToken(token, 'remote')
        remoteState.activeAgentId = selected.principalId
        remoteState.sessionId = selected.sessionId
      }
      remoteState.ready = true
      remoteState.error = null
      clearTimeout(bootstrapTimer)
      bootstrapTimer = null
      scheduleRefresh()
      reconcileRemotePendingRequests().catch(error => {
        try { assertToken(token, 'remote') } catch { return }
        remoteState.pendingError = error.message
      })
    } catch (error) {
      assertToken(token, 'remote')
      remoteState.ready = false
      remoteState.error = error.message
      if (blockedCodes.has(error.code)) {
        remoteState.connection = error.code === 'REMOTE_AUTH_REQUIRED' ? 'auth_required' : 'incompatible'
        clearTimeout(bootstrapTimer)
        bootstrapTimer = null
      } else scheduleBootstrap()
    }
  })()
  initialization = { version, promise }
  try { await promise } finally { if (initialization?.promise === promise) initialization = null }
}
export async function performRemote(action, options) {
  const token = currentToken()
  try { return (await remoteClient.action(action, { ...options, token })).data }
  finally { await afterRemoteWrite(token) }
}
export async function submitRemoteInput(input, options) {
  const token = currentToken()
  try { return (await remoteClient.submitInput(input, { ...options, token })).data.projection }
  finally { await afterRemoteWrite(token) }
}
async function afterRemoteWrite(token) {
  try {
    assertToken(token, 'remote')
    for (const view of remoteViews.values()) view.stale = true
    scheduleRefresh()
    await refreshRemotePendingRequests()
    if (remoteState.connection === 'online') reconcileRemotePendingRequests().catch(error => {
      try { assertToken(token, 'remote') } catch { return }
      remoteState.pendingError = error.message
    })
  } catch (error) {
    try { assertToken(token, 'remote') } catch { return }
    // 保留原命令结果，单独公开核对记录刷新错误，供后续界面展示。
    remoteState.pendingError = error.message
  }
}
export async function sendRemoteChat(input, { requestId = crypto.randomUUID() } = {}) {
  const token = currentToken()
  assertToken(token, 'remote')
  await listenOnce()
  assertToken(token, 'remote')
  const execution = reactive({ requestId, threadId: input.threadId, input: { ...input }, state: 'observing', frame: null, presentation: createRemoteSurfaceState(), revision: 0, needsRefresh: false, error: null })
  // 展示缓存可以释放，待核对操作由独立的持久化记录恢复，不占用活跃执行名额。
  if (remoteExecutions.size >= 30) {
    for (const [id, item] of remoteExecutions) {
      if (item.state !== 'observing') remoteExecutions.delete(id)
      if (remoteExecutions.size < 30) break
    }
    if (remoteExecutions.size >= 30) throw new Error('同时观察的远程执行过多，请稍后再试')
  }
  remoteExecutions.set(requestId, execution)
  try {
    await remoteClient.chat(input, { requestId, token })
    assertToken(token, 'remote')
    execution.state = 'completed'
  } catch (error) {
    try { assertToken(token, 'remote') } catch { throw error }
    execution.state = ['not_sent', 'rejected', 'failed'].includes(error.outcome) ? error.outcome : 'unknown'
    execution.error = error.message
    throw error
  } finally {
    await afterRemoteWrite(token)
    try {
      assertToken(token, 'remote')
      execution.needsRefresh = true
      const projection = await readRemote({ kind: 'conversation', id: input.threadId, pageSize: 60, beforeCursor: null }, { allowCached: false })
      installRemoteProjection(execution.presentation, projection)
      execution.needsRefresh = false
    } catch { /* 断线保留原分区待核对状态，恢复后显式读取。 */ }
  }
  return execution
}
export async function reconnectRemote() {
  await initializeRemote()
}
