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
  list: () => control('remote_profiles'),
  save: input => control('remote_save_profile', { input, modeToken: currentToken() }),
  remove: id => control('remote_delete_profile', { id }),
}
export const remoteClient = {
  query: (query, { cachedOnly = false, token } = {}) => scoped('remote_query', { query, cachedOnly }, token),
  action: (action, { requestId = crypto.randomUUID(), token } = {}) => scoped('remote_action', { action, requestId }, token),
  chat: (input, { requestId = crypto.randomUUID(), token } = {}) => scoped('remote_chat', { request: { ...input, requestId } }, token),
  connect: (principalId, token, force = false) => scoped('remote_connect', { principalId, force }, token),
  sessionStatus: token => scoped('remote_session_status', {}, token),
  logoutDevice: token => scoped('remote_logout_device', {}, token),
  notifications: token => scoped('remote_notifications', {}, token),
  ackNotification: (principalId, receipt, token) => scoped('remote_ack_notification', { principalId, receipt }, token),
  subscribe: (subscription, token) => scoped('remote_subscribe', { subscription }, token),
  submitInput: (input, { requestId = crypto.randomUUID(), token } = {}) => scoped('remote_surface_input', { input, requestId }, token),
  pendingRequests: token => scoped('remote_pending_requests', {}, token),
  reviewRequest: (requestId, fingerprint, note, token) => scoped('remote_review_request', { requestId, fingerprint, note }, token),
  reconcileRequest: (requestId, token) => scoped('remote_reconcile_request', { requestId }, token),
  uiRead: (scope, token) => scoped('remote_ui_read', { scope }, token),
  uiWrite: (scope, value, token) => scoped('remote_ui_write', { scope, value }, token),
  upload: (threadId, file, { requestId = crypto.randomUUID(), token } = {}) => scoped('remote_upload', { threadId, file, requestId }, token),
  resource: (resource, { range = null, token } = {}) => scoped('remote_resource', { resource, range }, token),
  listen: callback => listen('remote-event', ({ payload }) => {
    try { assertToken(payload?.modeToken, 'remote') } catch { return }
    callback(payload)
  }),
}
