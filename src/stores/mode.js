import { computed, shallowReactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { normalizeError } from '../services/localChatClient'

// 仅展示原生控制投影；前端永远不能自行授予运行权或在离线时切回独立模式。
export const modeState = shallowReactive({ snapshot: null, loading: true, error: null })
export const isStandalone = computed(() => modeState.snapshot?.phase === 'stable' && modeState.snapshot.activeTarget?.mode === 'standalone')
export const isRemote = computed(() => modeState.snapshot?.phase === 'stable' && modeState.snapshot.activeTarget?.mode === 'remote')
const invalidators = new Set()
let initialized = null
let dispose = null

export function sameToken(a, b) {
  return !!a && !!b && a.installationId === b.installationId && a.epoch === b.epoch &&
    a.target?.mode === b.target?.mode && a.target?.profileId === b.target?.profileId
}
export function currentToken() {
  const snapshot = modeState.snapshot
  if (!snapshot || snapshot.phase !== 'stable' || !snapshot.activeTarget) throw modeError()
  return { installationId: snapshot.installationId, epoch: snapshot.activationEpoch, target: { ...snapshot.activeTarget } }
}
export function modeError() {
  return Object.assign(new Error('当前运行模式未就绪或已经切换，请刷新'), { code: 'MODE_INACTIVE' })
}
export function assertToken(token, mode) {
  const active = currentToken()
  if (!sameToken(token, active) || (mode && active.target.mode !== mode)) throw modeError()
}
export function onModeInvalidated(callback) {
  invalidators.add(callback)
  return () => invalidators.delete(callback)
}
function apply(snapshot) {
  if (!snapshot || snapshot.schemaVersion !== 1 || !Number.isSafeInteger(snapshot.revision) || !Number.isSafeInteger(snapshot.activationEpoch)) throw modeError()
  const previous = modeState.snapshot
  if (previous?.installationId === snapshot.installationId && previous.revision >= snapshot.revision) return
  modeState.snapshot = snapshot
  modeState.error = snapshot.lastError || null
  if (!previous || previous.installationId !== snapshot.installationId || previous.activationEpoch !== snapshot.activationEpoch ||
      previous.phase !== snapshot.phase || previous.activeTarget?.profileId !== snapshot.activeTarget?.profileId || previous.activeTarget?.mode !== snapshot.activeTarget?.mode) {
    for (const callback of invalidators) {
      try { callback(snapshot) } catch (error) { console.error('模式展示缓存清理失败：', error) }
    }
  }
}
export async function refreshMode() {
  try { apply(await invoke('mode_snapshot')); return modeState.snapshot }
  catch (error) {
    modeState.snapshot = null
    modeState.error = normalizeError(error).message
    for (const callback of invalidators) {
      try { callback(null) } catch (failure) { console.error('模式展示缓存清理失败：', failure) }
    }
    throw normalizeError(error)
  }
}
export function initializeMode() {
  if (initialized) return initialized
  initialized = (async () => {
    dispose = await listen('mode-event', ({ payload }) => {
      try { apply(payload) } catch (error) { modeState.snapshot = null; modeState.error = error.message }
    })
    try {
      const snapshot = await refreshMode()
      // 旧远程入口已退役；仅保留待重新确认的地址，移除 Web 存储中的旧明文令牌。
      try {
        const legacy = JSON.parse(localStorage.getItem('remote_config') || '{}')
        if (typeof legacy.url === 'string') localStorage.setItem('ppc.remote.pendingEndpoint', legacy.url.slice(0, 2048))
        localStorage.removeItem('remote_config')
        localStorage.removeItem('app_mode')
      } catch { localStorage.removeItem('remote_config'); localStorage.removeItem('app_mode') }
      return snapshot
    } finally { modeState.loading = false }
  })().catch(error => {
    initialized = null
    dispose?.()
    dispose = null
    modeState.loading = false
    modeState.error = normalizeError(error).message
    throw error
  })
  return initialized
}
export async function switchMode(target) {
  modeState.loading = true
  try {
    // 切换由原生层串行化，失败时重新读取闭门状态；不乐观更新当前模式。
    apply(await invoke('mode_switch', { target }))
    return modeState.snapshot
  } catch (error) {
    await refreshMode().catch(() => {})
    throw normalizeError(error)
  } finally { modeState.loading = false }
}
