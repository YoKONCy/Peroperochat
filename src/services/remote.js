import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { currentToken, assertToken, sameToken } from '../stores/mode'
import { normalizeError } from './localChatClient'

async function scoped(command, args = {}, token = currentToken()) {
  assertToken(token, 'remote')
  try {
    const result = await invoke(command, { ...args, token })
    assertToken(token, 'remote')
    if (result?.modeToken && !sameToken(result.modeToken, token)) throw new Error('远程响应运行身份不匹配')
    return result
  } catch (value) {
    const error = normalizeError(value)
    const requestId = args.requestId || args.request?.requestId
    if (requestId) error.requestId = requestId
    throw error
  }
}
async function control(command, args) {
  try { return await invoke(command, args) } catch (error) { throw normalizeError(error) }
}
export const remoteConnections = {
  decodePairing: raw => control('remote_decode_pairing', { raw, modeToken: currentToken() }),
  list: () => control('remote_profiles'),
  save: input => control('remote_save_profile', { input, modeToken: currentToken() }),
  remove: id => control('remote_delete_profile', { id }),
}
export const remoteClient = {
  modelControls: (threadId, token) => scoped('remote_model_controls', { threadId }, token),
  prepareRegeneration: (input, token) => scoped('remote_prepare_regeneration', { ...input, requestId: crypto.randomUUID() }, token),
  commitRegeneration: (id, token) => scoped('remote_commit_regeneration', { id }, token),
  regenerationRecovery: (threadId, token) => scoped('remote_regeneration_recovery', { threadId }, token),
  discardRegeneration: (id, token) => scoped('remote_discard_regeneration', { id }, token),
  confirmRewind: (input, token) => scoped('remote_confirm_rewind', { ...input, requestId: crypto.randomUUID() }, token),
  manage: (request, { requestId = crypto.randomUUID(), token } = {}) => scoped('remote_manage', { request, requestId }, token),
  manageUpload: (kind, id, file, { fields = {}, requestId = crypto.randomUUID(), token } = {}) => scoped('remote_manage_upload', { kind, id, file, fields, requestId }, token),
  tts: (input, { requestId = crypto.randomUUID(), token } = {}) => scoped('remote_voice_tts', { ...input, requestId }, token),
  exportText: (name, content, token) => scoped('remote_export_text', { name, content }, token),
  query: (query, { cachedOnly = false, token } = {}) => scoped('remote_query', { query, cachedOnly }, token),
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
  resource: (resource, { range = null, token } = {}) => scoped('remote_resource', { resource, range }, token),
  resourceInfo: (resource, token) => scoped('remote_resource_info', { resource }, token),
  resolveResource: (context, source, token) => scoped('remote_resolve_resource', { agentId: context.agentId, threadId: context.threadId, taskId: context.taskId || null, source }, token),
  htmlPreview: (resource, token) => scoped('remote_html_preview', { resource }, token),
  downloadResource: (resource, name, id, token) => scoped('remote_download_resource', { resource, name, id }, token),
  cancelDownload: (id, token) => scoped('remote_cancel_download', { id }, token),
  listen: callback => listen('remote-event', ({ payload }) => {
    try { assertToken(payload?.modeToken, 'remote') } catch { return }
    callback(payload)
  }),
}
