import { ref, watch, onBeforeUnmount } from 'vue'
import { currentToken, assertToken } from '../stores/mode'
import { remoteClient } from '../services/remote'
import { manageRemote, remotePendingRequests, remoteState } from '../stores/remote'

// 管理配置不进入共享投影或本机偏好；离开页面即释放，避免密钥随离线缓存落盘。
export function useRemoteManagement() {
  const token = currentToken(), error = ref(''), busy = ref(false), writing = ref(false), unresolved = ref(false)
  const unresolvedId = ref(null)
  let disposed = false
  onBeforeUnmount(() => { disposed = true })
  function guard() { assertToken(token, 'remote'); if (disposed) throw new Error('页面已关闭') }
  async function read(operation, { id = null, params = {}, body = null } = {}) {
    guard()
    const result = await remoteClient.manage({ operation, id: id == null ? null : String(id), params, body }, { token })
    guard(); return result.data
  }
  async function write(operation, { id = null, params = {}, body = null, requestId } = {}) {
    guard()
    if (unresolved.value) throw new Error('上次操作结果待核对，请先在通知中完成核对后重新打开此页')
    writing.value = true
    try {
      return await manageRemote({ operation, id: id == null ? null : String(id), params, body }, { token, requestId })
    } catch (failure) {
      fail(failure)
      throw failure
    } finally { writing.value = false }
  }
  function fail(failure) { if (!['not_sent', 'rejected', 'failed'].includes(failure.outcome)) { unresolvedId.value = failure.requestId || null; unresolved.value = true } }
  watch(() => [unresolved.value, unresolvedId.value, remotePendingRequests.has(unresolvedId.value), remoteState.pendingError], () => {
    if (unresolved.value && unresolvedId.value && !remoteState.pendingError && !remotePendingRequests.has(unresolvedId.value)) { unresolved.value = false; unresolvedId.value = null }
  })
  async function run(callback) {
    if (busy.value) return
    busy.value = true; error.value = ''
    try { await callback(); guard() }
    catch (failure) { if (!disposed) error.value = failure.message }
    finally { if (!disposed) busy.value = false }
  }
  return { token, error, busy, writing, unresolved, read, write, run, guard, fail }
}
