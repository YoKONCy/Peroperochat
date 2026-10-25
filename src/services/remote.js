import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { currentToken, assertToken, sameToken } from '../stores/mode'
import { normalizeError } from './localChatClient'
import { presentRemoteError } from './remoteFeedback.js'
import { createRemoteReadCoordinator } from './remoteReadCoordinator.js'
import { observeRemoteResult } from './remoteObservation.js'

const observedReads = new Set(['remote_connect', 'remote_session_status', 'remote_pending_requests', 'remote_notifications', 'remote_ui_read', 'remote_subscribe'])
const readObservers = new Set()

async function scoped(command, args = {}, token = currentToken()) {
  assertToken(token, 'remote')
  const observer = observedReads.has(command) ? new AbortController() : null
  if (observer) readObservers.add(observer)
  try {
    const operation = invoke(command, { ...args, token })
    const result = await (observer ? observeRemoteResult(operation, { signal: observer.signal }) : operation)
    assertToken(token, 'remote')
    if (result?.modeToken && !sameToken(result.modeToken, token)) throw new Error('远程响应运行身份不匹配')
    return result
  } catch (value) {
    const error = presentRemoteError(normalizeError(value))
    const requestId = args.requestId || args.request?.requestId
    if (requestId) error.requestId = requestId
    throw error
  } finally { if (observer) readObservers.delete(observer) }
}
async function control(command, args) {
  try { return await invoke(command, args) } catch (error) { throw presentRemoteError(normalizeError(error)) }
}
const reads = createRemoteReadCoordinator({ query: (query, options) => scoped('remote_query', { query, cachedOnly: options.cachedOnly }, options.token), assertActive: token => assertToken(token, 'remote') })
export const remoteConnections = {
  decodePairing: raw => control('remote_decode_pairing', { raw, modeToken: currentToken() }),
  list: () => control('remote_profiles'),
  save: input => control('remote_save_profile', { input, modeToken: currentToken() }),
  remove: id => control('remote_delete_profile', { id }),
}
export const remoteClient = {
  resetReads: () => { reads.reset(); for (const observer of readObservers) observer.abort(); readObservers.clear() },
  modelControls: (threadId, token) => scoped('remote_model_controls', { threadId }, token),
  prepareRegeneration: (input, token) => scoped('remote_prepare_regeneration', { ...input, requestId: crypto.randomUUID() }, token),
  commitRegeneration: (id, token) => scoped('remote_commit_regeneration', { id }, token),
  regenerationRecovery: (threadId, token) => scoped('remote_regeneration_recovery', { threadId }, token),
  discardRegeneration: (id, token) => scoped('remote_discard_regeneration', { id }, token),
  confirmRewind: (input, token) => scoped('remote_confirm_rewind', { ...input, requestId: crypto.randomUUID() }, token),
  manage: (request, { requestId = crypto.randomUUID(), token } = {}) => scoped('remote_manage', { request, requestId }, token),
  manageUpload: (kind, id, file, { fields = {}, requestId = crypto.randomUUID(), token } = {}) => scoped('remote_manage_upload', { kind, id, file, fields, requestId }, token),
  tts: async (input, { requestId = crypto.randomUUID(), token } = {}) => {
    const packet = await scoped('remote_voice_tts', { input, requestId }, token)
    const bytes = new Uint8Array(packet)
    const size = bytes.length >= 6 ? bytes[4] * 256 + bytes[5] : 0
    if (String.fromCharCode(...bytes.subarray(0, 4)) !== 'PCTS' || !size || size > 256 || bytes.length <= 6 + size) throw new Error('朗读服务未返回音频')
    const mimeType = new TextDecoder().decode(bytes.subarray(6, 6 + size))
    if (!mimeType.startsWith('audio/')) throw new Error('朗读服务未返回音频')
    return { data: { mimeType, bytes: bytes.subarray(6 + size) } }
  },
  cancelTts: (requestId, token) => scoped('remote_cancel_tts', { requestId }, token),
  exportText: (name, content, token) => scoped('remote_export_text', { name, content }, token),
  query: (query, { cachedOnly = false, token = currentToken() } = {}) => reads.read(query, { cachedOnly, token }),
  action: (action, { requestId = crypto.randomUUID(), token } = {}) => scoped('remote_action', { action, requestId }, token),
  chat: (input, { requestId = crypto.randomUUID(), token } = {}) => scoped('remote_chat', { request: { ...input, requestId } }, token),
  connect: (principalId, token, force = false) => scoped('remote_connect', { principalId, force }, token),
  sessionStatus: token => scoped('remote_session_status', {}, token),
  logoutDevice: token => scoped('remote_logout_device', {}, token),
  notifications: token => scoped('remote_notifications', {}, token),
  ackNotification: (principalId, receipt, token) => scoped('remote_ack_notification', { principalId, receipt }, token),
  backgroundRuntime: token => scoped('remote_background_runtime', {}, token),
  configureBackground: (enabled, token) => scoped('remote_configure_background', { enabled }, token),
  backgroundSettings: (kind, token) => scoped('remote_background_settings', { kind }, token),
  consumeNotificationOpen: token => scoped('remote_consume_notification_open', {}, token),
  subscribe: (subscription, token) => scoped('remote_subscribe', { subscription }, token),
  submitInput: (input, { requestId = crypto.randomUUID(), token } = {}) => scoped('remote_surface_input', { input, requestId }, token),
  pendingRequests: token => scoped('remote_pending_requests', {}, token),
  reviewRequest: (requestId, fingerprint, note, token) => scoped('remote_review_request', { requestId, fingerprint, note }, token),
  reconcileRequest: (requestId, token) => scoped('remote_reconcile_request', { requestId }, token),
  uiRead: (scope, token) => scoped('remote_ui_read', { scope }, token),
  uiWrite: (scope, value, token) => scoped('remote_ui_write', { scope, value }, token),
  upload: (threadId, file, { requestId = crypto.randomUUID(), token } = {}) => scoped('remote_upload', { threadId, file, requestId }, token),
  resource: (resource, { range = null, cachedOnly = false, token } = {}) => scoped('remote_resource', { resource, range, cachedOnly }, token),
  resourceInfo: (resource, token, { cachedOnly = false } = {}) => scoped('remote_resource_info', { resource, cachedOnly }, token),
  resolveResource: (context, source, token, { cachedOnly = false } = {}) => scoped('remote_resolve_resource', { agentId: context.agentId, threadId: context.threadId, taskId: context.taskId || null, source, cachedOnly }, token),
  htmlPreview: (resource, token) => scoped('remote_html_preview', { resource }, token),
  downloadResource: (resource, name, id, token) => scoped('remote_download_resource', { resource, name, id }, token),
  cancelDownload: (id, token) => scoped('remote_cancel_download', { id }, token),
  listen: callback => listen('remote-event', ({ payload }) => {
    try { assertToken(payload?.modeToken, 'remote') } catch { return }
    callback(payload)
  }),
}
