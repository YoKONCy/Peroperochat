import { reactive, shallowReactive } from 'vue'
import { remoteClient } from '../../services/remote'
import { currentToken, assertToken, onModeInvalidated } from '../mode'
import { applyRemoteFrame, createRemoteSurfaceState, installRemoteProjection } from '../../services/remoteSurface'
import { remoteFeedbackMessage } from '../../services/remoteFeedback.js'

// 独立模式 Store 不参与远程数据投影；服务端是消息、任务与角色状态的权威。
export const remoteState = shallowReactive({ connection: 'offline', sessionId: null, agents: [], activeAgentId: null, error: null, pendingError: null, notificationError: null, ready: false, initializing: false, restoringView: false })
export const remoteViews = reactive(new Map())
export const remoteExecutions = reactive(new Map())
export const remotePendingRequests = reactive(new Map())
export const remoteNotifications = reactive(new Map())
export const remoteUnavailableThreads = reactive(new Set())
const pendingReads = new Map()
const terminalReads = new Map()
let readVersion = 0
let focusedThread = null
let disposeEvents = null
let listener = null
let generation = 0
let refreshTimer = null
let refreshAt = 0
let bootstrapTimer = null
let initialization = null
let pendingVersion = 0
let reconciliation = null
let notificationTimer = null
let notificationVersion = 0
let resuming = null
let lastResumeAt = 0
let queuedFrames = []
let queuedFrame = null
const blockedCodes = new Set(['REMOTE_AUTH_REQUIRED', 'REMOTE_IDENTITY_CHANGED', 'REMOTE_INCOMPATIBLE', 'CREDENTIAL_UNAVAILABLE', 'PROFILE_NOT_FOUND', 'INVALID_PROFILE'])
const offlineCodes = new Set(['REMOTE_NETWORK', 'REMOTE_HTTP', 'REMOTE_RATE_LIMIT'])

function blockConnection(error) {
  if (!blockedCodes.has(error.code)) return false
  remoteState.connection = ['REMOTE_AUTH_REQUIRED', 'CREDENTIAL_UNAVAILABLE'].includes(error.code) ? 'auth_required' : 'incompatible'
  remoteState.ready = false; remoteState.error = error.message
  clearTimeout(bootstrapTimer); bootstrapTimer = null
  return true
}

function key(query) { return JSON.stringify(query) }
function queryThread(query) {
  if (['thread', 'conversation', 'queue', 'queue_live', 'thread_tools'].includes(query.kind)) return query.id
  return query.threadId || null
}
function missingThread() { return Object.assign(new Error('这段对话已不可用，请选择其他会话。'), { code: 'REMOTE_NOT_FOUND' }) }
export function focusRemoteThread(id) {
  const next = id && !remoteUnavailableThreads.has(id) ? id : null
  focusedThread = next
  if (next && remoteState.connection === 'online') scheduleRefresh()
}
export function retryRemoteThreadReads(id) {
  if (!id || remoteUnavailableThreads.has(id)) return
  for (const [name, view] of remoteViews) if (queryThread(view.query) === id) {
    terminalReads.delete(name); Object.assign(view, { errorCode: null, retryAfter: 0, failures: 0, stale: true })
  }
  scheduleRefresh()
}
export function retireRemoteThread(id) {
  if (!id) return
  remoteUnavailableThreads.add(id)
  if (focusedThread === id) focusedThread = null
  // 只释放展示和刷新记录；原生草稿、未确认请求及服务端执行均不在这里删除。
  for (const [name, view] of remoteViews) if (queryThread(view.query) === id) { remoteViews.delete(name); terminalReads.delete(name) }
  for (const [name, execution] of remoteExecutions) if (execution.threadId === id) remoteExecutions.delete(name)
}
function assertReadable(query) { if (remoteUnavailableThreads.has(queryThread(query))) throw missingThread() }
function queryOnce(query, cachedOnly, token) {
  const name = `${cachedOnly ? 'cache' : 'server'}:${JSON.stringify(Object.keys(query).sort().map(name => [name, query[name]]))}`
  if (pendingReads.has(name)) return pendingReads.get(name)
  const request = remoteClient.query(query, { cachedOnly, token }).finally(() => { if (pendingReads.get(name) === request) pendingReads.delete(name) })
  pendingReads.set(name, request)
  return request
}
function autoRefreshable(view, ignoreDelay = false) {
  const thread = queryThread(view.query)
  return view.stale && !view.loading && !terminalReads.has(key(view.query)) && !remoteUnavailableThreads.has(thread)
    && (!thread || thread === focusedThread) && (ignoreDelay || !view.retryAfter || view.retryAfter <= Date.now())
    && !(view.query.kind === 'conversation' && view.query.beforeCursor)
}
export function clearRemoteState() {
  generation++
  cancelAnimationFrame(queuedFrame); queuedFrame = null; queuedFrames = []
  clearTimeout(refreshTimer)
  refreshTimer = null
  refreshAt = 0
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
  remoteState.initializing = false
  remoteState.restoringView = false
  remoteViews.clear()
  remoteUnavailableThreads.clear(); pendingReads.clear(); terminalReads.clear(); focusedThread = null
  remoteExecutions.clear()
  remotePendingRequests.clear()
  clearTimeout(notificationTimer); notificationTimer = null; notificationVersion++
  remoteNotifications.clear(); remoteState.notificationError = null
  resuming = null
  lastResumeAt = 0
}
onModeInvalidated(clearRemoteState)

export async function refreshRemoteNotifications() {
  const token = currentToken(), request = ++notificationVersion
  const result = await remoteClient.notifications(token)
  assertToken(token, 'remote')
  if (request !== notificationVersion) return
  if (!Array.isArray(result.data?.items)) throw new Error('暂时没能载入通知，请稍后再试。')
  remoteNotifications.clear()
  for (const item of result.data.items) remoteNotifications.set(`${item.principalId}:${item.notificationId}`, item)
  remoteState.notificationError = null
}
function scheduleNotifications() {
  if (notificationTimer) return
  const token = currentToken()
  notificationTimer = setTimeout(() => {
    notificationTimer = null
    try { assertToken(token, 'remote') } catch { return }
    refreshRemoteNotifications().catch(error => { try { assertToken(token, 'remote'); remoteState.notificationError = error.message } catch { /* 旧连接错误不写入新页面。 */ } })
  }, 150)
}
export async function acknowledgeRemoteNotification(item) {
  const token = currentToken()
  await remoteClient.ackNotification(item.principalId, { notificationId: item.notificationId, revision: item.revision }, token)
  assertToken(token, 'remote')
  await refreshRemoteNotifications()
}

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
  if (!Array.isArray(pending)) throw new Error('暂时没能读取操作记录，请稍后再试。')
  remotePendingRequests.clear()
  for (const item of pending) remotePendingRequests.set(item.requestId, item)
  remoteState.pendingError = null
}

export async function reviewRemoteRequest(requestId, note, { confirmed = false } = {}) {
  const token = currentToken()
  assertToken(token, 'remote')
  const pending = remotePendingRequests.get(requestId)
  if (!confirmed || !pending || pending.state !== 'unknown' || !note?.trim()) throw new Error('请先查看结果，并填写确认说明。')
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
    const candidates = [...remotePendingRequests.values()].filter(item => item.state === 'unknown' && item.context?.kind === 'chat' && (item.context?.transport === 'queue' || item.result?.messageId))
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

function scheduleRefresh(delay = 150) {
  const due = Date.now() + delay
  if (refreshTimer && refreshAt <= due) return
  clearTimeout(refreshTimer); refreshAt = due
  const token = currentToken()
  refreshTimer = setTimeout(async () => {
    refreshTimer = null
    refreshAt = 0
    try { assertToken(token, 'remote') } catch { return }
    if (document.hidden || remoteState.connection !== 'online') return
    const queries = [...remoteViews.values()].filter(view => autoRefreshable(view)).map(view => view.query)
    // 只刷新显示投影，限制并发；从不重新发送聊天、审批或任务派发。
    const workers = Array.from({ length: Math.min(4, queries.length) }, async () => {
      while (queries.length) {
        try { assertToken(token, 'remote') } catch { return }
        if (document.hidden || remoteState.connection !== 'online') return
        const query = queries.shift(), view = remoteViews.get(key(query))
        if (view && autoRefreshable(view)) await readRemote(query).catch(() => {})
      }
    })
    await Promise.all(workers)
    try { assertToken(token, 'remote') } catch { return }
    const retries = [...remoteViews.values()].filter(view => autoRefreshable(view, true) && view.retryAfter > Date.now())
    if (retries.length) scheduleRefresh(Math.max(150, Math.min(...retries.map(view => view.retryAfter)) - Date.now()))
  }, delay)
}

async function listenOnce() {
  if (disposeEvents) return
  if (!listener) listener = remoteClient.listen(event => {
    if (event.kind === 'connection') {
      if (event.data.state === 'offline' && (event.data.sessionId !== remoteState.sessionId || remoteState.connection === 'auth_required')) return
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
      if (event.data.state === 'online') remoteState.error = null
      else if (event.data.message) remoteState.error = remoteFeedbackMessage({ code: event.data.code, message: event.data.message })
      if (['auth_required', 'incompatible'].includes(event.data.state)) { clearTimeout(bootstrapTimer); bootstrapTimer = null }
      if (event.data.state !== 'online') {
        remoteState.ready = false
        for (const view of remoteViews.values()) view.stale = true
      } else scheduleRefresh()
    }
    if (event.kind === 'gateway' || event.kind === 'snapshot_required') {
      // 群聊增量由据点视图逐帧消费，避免每个字都触发单聊和管理页面的全量重读。
      if (event.kind === 'gateway' && event.data?.payload?.action?.startsWith('stronghold_')) {
        if (['stronghold_round_completed', 'stronghold_agent_completed'].includes(event.data.payload.action)) {
          for (const view of remoteViews.values()) if (['rooms', 'room'].includes(view.query.kind)) view.stale = true
        }
        return
      }
      if (event.kind === 'gateway' && event.data?.payload?.action === 'surface') {
        scheduleQueuedSurface(event.data.payload.frame)
        return
      }
      // Gateway 只触发读模型失效；不把通知回放当作新消息或自行计算业务状态。
      for (const view of remoteViews.values()) view.stale = true
      scheduleRefresh()
      if (event.kind === 'snapshot_required' || event.data?.payload?.action === 'durable_notification') scheduleNotifications()
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
export async function readRemote(query, { cachedOnly = false, allowCached = true, retryUnavailable = false } = {}) {
  const token = currentToken()
  assertToken(token, 'remote')
  if (!(retryUnavailable && query.kind === 'thread')) assertReadable(query)
  const id = key(query)
  if (!cachedOnly && terminalReads.has(id)) {
    if (!retryUnavailable) throw terminalReads.get(id)
    terminalReads.delete(id)
  }
  if (!remoteViews.has(id) && remoteViews.size >= 64) {
    const disposable = [...remoteViews].find(([, view]) => !view.loading)
    if (disposable) { remoteViews.delete(disposable[0]); terminalReads.delete(disposable[0]) }
    else throw new Error('正在载入其他内容，稍等一下再试吧。')
  }
  const previous = remoteViews.get(id)
  const requestVersion = ++readVersion
  remoteViews.set(id, { ...previous, query, requestVersion, loading: true, error: null })
  try {
    const result = await queryOnce(query, cachedOnly, token)
    assertToken(token, 'remote')
    if (retryUnavailable && query.kind === 'thread' && !cachedOnly && remoteViews.get(id)?.requestVersion === requestVersion) remoteUnavailableThreads.delete(query.id)
    assertReadable(query)
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
        const cached = await queryOnce(query, true, token)
        assertToken(token, 'remote'); assertReadable(query)
        if (cached.data !== null && cached.data !== undefined) {
          if (remoteViews.get(id)?.requestVersion === requestVersion) {
            const failures = (previous?.failures || 0) + 1, delay = Math.min(30000, 1000 * 2 ** Math.min(failures, 5))
            remoteViews.set(id, { query, requestVersion, loading: false, data: cached.data, cached: true, stale: true, error: error.message, errorCode: error.code, failures, retryAfter: Date.now() + delay })
            scheduleRefresh(delay)
            if (query.kind === 'agents' && Array.isArray(cached.data)) remoteState.agents = cached.data
          }
          return cached.data
        }
      } catch { /* 原网络错误保留给调用者，不把缓存读取失败伪装成成功。 */ }
    }
    assertToken(token, 'remote')
    const view = remoteViews.get(id)
    const terminal = ['REMOTE_NOT_FOUND', 'REMOTE_FORBIDDEN', 'REMOTE_INCOMPATIBLE', 'REMOTE_PROTOCOL', 'REMOTE_INVALID_INPUT'].includes(error.code)
    if (!cachedOnly && view?.requestVersion === requestVersion) {
      if (terminal) terminalReads.set(id, error)
      const failures = (previous?.failures || 0) + 1
      Object.assign(view, { loading: false, error: error.message, errorCode: error.code, stale: !terminal, failures, retryAfter: terminal ? 0 : Date.now() + Math.min(30000, 1000 * 2 ** Math.min(failures, 5)), syncing: error.code === 'REMOTE_STALE_READ' })
      if (error.code === 'REMOTE_NOT_FOUND') {
        if (query.kind === 'thread') retireRemoteThread(query.id)
        else if (['conversation', 'queue', 'queue_live'].includes(query.kind)) {
          // 队列或消息接口不支持不代表会话被删，只用会话详情确认失效。
          readRemote({ kind: 'thread', id: query.id }, { allowCached: false }).catch(() => {})
        }
      }
    } else if (cachedOnly && view?.requestVersion === requestVersion) Object.assign(view, { loading: false, stale: false, error: error.message })
    blockConnection(error)
    if (!terminal && !cachedOnly) scheduleRefresh(Math.max(1500, (view?.retryAfter || 0) - Date.now()))
    throw error
  }
}
export async function initializeRemote() {
  const token = currentToken()
  assertToken(token, 'remote')
  const version = generation
  if (initialization?.version === version) return initialization.promise
  remoteState.initializing = true
  if (!['online', 'auth_required', 'incompatible'].includes(remoteState.connection)) remoteState.connection = 'connecting'
  const promise = (async () => {
    try {
      await listenOnce()
      // 本机辅助记录读取失败单独展示，不阻断设备鉴权和 Gateway 恢复。
      await refreshRemotePendingRequests().catch(error => { assertToken(token, 'remote'); remoteState.pendingError = error.message })
      await refreshRemoteNotifications().catch(error => { assertToken(token, 'remote'); remoteState.notificationError = error.message })
      const cached = await readRemote({ kind: 'agents' }, { cachedOnly: true }).catch(() => null)
      assertToken(token, 'remote')
      if (Array.isArray(cached)) {
        remoteState.agents = cached
      }
      // 优先恢复本连接的上次会话角色；没有选择时交给原生保存的主体或服务端发现。
      if (!remoteState.activeAgentId) {
        const home = await remoteClient.uiRead({ kind: 'home' }, token).catch(error => { assertToken(token, 'remote'); remoteState.error = error.message; return null })
        assertToken(token, 'remote')
        if (!remoteState.activeAgentId && home?.data?.agentId) remoteState.activeAgentId = home.data.agentId
      }
      // 原生先持有恢复目标；即使没有缓存角色且此刻离线，也会继续尝试发现角色。
      const connected = await remoteClient.connect(remoteState.activeAgentId, token)
      assertToken(token, 'remote')
      remoteState.sessionId = connected.sessionId
      remoteState.connection = 'online'
      remoteState.activeAgentId = connected.principalId
      const agents = await readRemote({ kind: 'agents' }, { allowCached: false })
      assertToken(token, 'remote')
      if (version !== generation) return
      if (!Array.isArray(agents)) throw new Error('暂时没能载入角色，请稍后再试。')
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
      if (!blockConnection(error)) { remoteState.connection = 'offline'; scheduleBootstrap() }
    }
  })()
  initialization = { version, promise }
  try { await promise } finally { if (initialization?.promise === promise) { initialization = null; remoteState.initializing = false } }
}
export async function performRemote(action, options) {
  const token = options?.token || currentToken()
  assertToken(token, 'remote')
  try {
    const result = await remoteClient.action(action, { ...options, token })
    assertToken(token, 'remote')
    if (action.kind === 'delete_thread') retireRemoteThread(action.id)
    return result.data
  }
  finally { await afterRemoteWrite(token) }
}
export async function manageRemote(request, options) {
  const token = options?.token || currentToken()
  assertToken(token, 'remote')
  try { return (await remoteClient.manage(request, { ...options, token })).data }
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
  assertReadable({ kind: 'thread', id: input.threadId })
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
    const result = await remoteClient.chat(input, { requestId, token })
    assertToken(token, 'remote')
    execution.state = 'accepted'
    const query = { kind: 'queue', id: input.threadId }
    remoteViews.set(key(query), { query, data: result.data, cached: false, stale: false, loading: false })
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

function queueExecution(id, threadId) {
  if (remoteUnavailableThreads.has(threadId)) return null
  const key = `queue-live:${id}`
  if (!remoteExecutions.has(key)) {
    for (const [existing, value] of remoteExecutions) {
      if (remoteExecutions.size < 30) break
      if (value.state !== 'observing') remoteExecutions.delete(existing)
    }
    if (remoteExecutions.size >= 30) return null
    remoteExecutions.set(key, reactive({ requestId: key, threadId, state: 'observing', presentation: createRemoteSurfaceState(), revision: 0, needsRefresh: false, error: null }))
  }
  return remoteExecutions.get(key)
}
export function installQueuedLive(surface) {
  if (!surface || surface.state !== 'open') return
  const execution = queueExecution(surface.surfaceId, surface.threadId)
  if (!execution) return
  if (execution.presentation.retired.get(surface.surfaceId) === surface.generation) return
  const current = execution.presentation.surfaces.get(surface.surfaceId)
  if (current?.generation === surface.generation && current.sequence >= surface.sequence) return
  installRemoteProjection(execution.presentation, { protocolVersion: 1, threadId: surface.threadId, surfaces: [surface] })
  execution.state = 'observing'; execution.revision++
}
function applyQueuedSurface(frame) {
  if (!frame?.surfaceId || !frame.operation) return
  const operation = frame.operation
  const execution = operation.type === 'surface.open' ? queueExecution(frame.surfaceId, operation.threadId) : remoteExecutions.get(`queue-live:${frame.surfaceId}`)
  if (!execution) return
  try {
    applyRemoteFrame(execution.presentation, frame); execution.frame = frame; execution.revision++
    execution.needsRefresh ||= execution.presentation.needsRefresh
    if (['surface.commit', 'surface.fail', 'surface.dispose'].includes(operation.type)) {
      execution.state = operation.type === 'surface.fail' ? 'failed' : 'completed'; execution.needsRefresh = true
      for (const view of remoteViews.values()) if (['conversation', 'queue', 'queue_live', 'agent_state', 'active_executions'].includes(view.query.kind)) view.stale = true
      scheduleRefresh()
    }
  } catch (error) { execution.needsRefresh = true; execution.error = error.message }
}
function scheduleQueuedSurface(frame) {
  if (queuedFrames.length >= 256) {
    queuedFrames = []
    for (const view of remoteViews.values()) view.stale = true
    scheduleRefresh()
    return
  }
  queuedFrames.push(frame)
  if (queuedFrame !== null) return
  const token = currentToken()
  // 保留帧顺序，每个显示帧统一合成一次；后台积压超限时回到权威快照。
  queuedFrame = requestAnimationFrame(() => {
    queuedFrame = null
    const frames = queuedFrames; queuedFrames = []
    try { assertToken(token, 'remote') } catch { return }
    for (const item of frames) applyQueuedSurface(item)
  })
}
export async function reconnectRemote() {
  const token = currentToken()
  await remoteClient.connect(remoteState.activeAgentId, token, true)
  assertToken(token, 'remote')
  await initializeRemote()
  assertToken(token, 'remote')
  if (!remoteState.ready) throw new Error(remoteState.error || '服务端尚未就绪，请稍后再试')
}
export async function resumeRemote() {
  if (resuming) return resuming
  if (initialization) return initialization.promise
  if (['auth_required', 'incompatible'].includes(remoteState.connection)) return
  if (Date.now() - lastResumeAt < 4000) return
  lastResumeAt = Date.now()
  const token = currentToken()
  const promise = (async () => {
    try {
      try { await remoteClient.sessionStatus(token) }
      catch (error) { if (error.code !== 'REMOTE_NOT_FOUND') throw error }
      assertToken(token, 'remote')
      await reconnectRemote()
      assertToken(token, 'remote')
      for (const view of remoteViews.values()) view.stale = true
      scheduleRefresh(); scheduleNotifications()
    } catch (error) {
      try { assertToken(token, 'remote') } catch { return }
      remoteState.error = error.message
      if (!blockConnection(error)) { remoteState.connection = 'offline'; remoteState.ready = false; scheduleBootstrap() }
    }
  })()
  resuming = promise
  try { await promise } finally { if (resuming === promise) resuming = null }
}
