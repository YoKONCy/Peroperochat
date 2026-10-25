import { reactive, nextTick } from 'vue'
import { remoteClient } from './remote'
import { loadRemoteResource } from './remoteMedia'
import { currentToken, assertToken, onModeInvalidated } from '../stores/mode'
import { remoteState } from '../stores/remote'

const defaults = () => ({ version: null, name: '', source: '', loaded: false, loading: false, busy: false, available: true })
export const remoteUserAvatar = reactive(defaults())
let revision = 0, pending = null, media = null, refreshAfterSave = false
onModeInvalidated(() => {
  revision++; pending = null; refreshAfterSave = false
  media?.release(); media = null
  Object.assign(remoteUserAvatar, defaults())
})
function valid(token, stamp) {
  assertToken(token, 'remote')
  return stamp === revision
}
async function applyProfile(profile, token, stamp, cachedOnly = false) {
  if (!profile || typeof profile.name !== 'string' || !(profile.version === null || typeof profile.version === 'string' && /^[\da-f-]{36}$/i.test(profile.version))) throw new Error('暂时无法读取用户头像')
  if (!valid(token, stamp)) return
  let loaded = null
  try {
    if (profile.version && (profile.version !== remoteUserAvatar.version || !media)) {
      loaded = await loadRemoteResource({ kind: 'owner_avatar', version: profile.version }, { cachedOnly })
      if (loaded.mimeType !== 'image/png' || !loaded.previewable) throw new Error('头像图片暂时无法显示')
      const image = new Image(); image.src = loaded.url; await image.decode()
    }
    if (!valid(token, stamp)) return
    const previous = media
    if (!profile.version) media = null
    else if (loaded) { media = loaded; loaded = null }
    Object.assign(remoteUserAvatar, { version: profile.version, name: profile.name, source: media?.url || '', loaded: true, available: true })
    await nextTick()
    if (previous !== media) previous?.release()
  } finally { loaded?.release() }
}
export async function refreshRemoteUserAvatar(force = false) {
  const token = currentToken(); assertToken(token, 'remote')
  if (remoteUserAvatar.busy) { refreshAfterSave ||= force; return }
  if (pending) { pending.again ||= force; return pending.promise }
  if (!force && remoteUserAvatar.loaded) return
  const entry = { again: false, promise: null }
  pending = entry; remoteUserAvatar.loading = true
  entry.promise = (async () => {
    do {
      entry.again = false
      const stamp = revision
      if (!remoteUserAvatar.loaded) {
        try { const saved = await remoteClient.query({ kind: 'owner_avatar' }, { cachedOnly: true, token }); if (saved.data) await applyProfile(saved.data, token, stamp, true) } catch { /* 初次使用时没有本机头像，继续读取服务端。 */ }
        if (!valid(token, stamp)) return
      }
      if (['offline', 'connecting', 'reconnecting'].includes(remoteState.connection)) return
      let result
      try {
        result = await remoteClient.query({ kind: 'owner_avatar' }, { token })
      } catch (error) {
        if (!valid(token, stamp)) return
        if (error.code !== 'REMOTE_NOT_FOUND') throw error
        media?.release(); media = null
        Object.assign(remoteUserAvatar, { version: null, name: '', source: '', loaded: true, available: false })
      }
      if (result) await applyProfile(result.data, token, stamp, result.cached === true)
      assertToken(token, 'remote')
    } while (entry.again && !remoteUserAvatar.busy)
  })().finally(() => { if (pending === entry) { pending = null; remoteUserAvatar.loading = false } })
  return entry.promise
}
export function quietlyRefreshRemoteUserAvatar() { refreshRemoteUserAvatar(true).catch(() => {}) }
export function remoteUserAvatarImageFailed(source) {
  if (!source || source !== remoteUserAvatar.source) return
  media?.release(); media = null; remoteUserAvatar.source = ''
}
export async function saveRemoteUserAvatar(png = null) {
  const token = currentToken(); assertToken(token, 'remote')
  if (remoteUserAvatar.busy) return
  const stamp = ++revision
  remoteUserAvatar.busy = true
  try {
    if (png && (png.type !== 'image/png' || !png.size || png.size > 5 * 1024 * 1024)) throw new Error('请重新裁切头像后再保存')
    const result = png
      ? await remoteClient.manageUpload('owner_avatar', null, { name: 'avatar.png', mimeType: 'image/png', bytes: Array.from(new Uint8Array(await png.arrayBuffer())) }, { token })
      : await remoteClient.manage({ operation: 'owner_avatar.reset' }, { token })
    if (valid(token, stamp)) {
      try { await applyProfile(result.data, token, stamp) }
      catch {
        if (valid(token, stamp)) {
          // 保存已成功，图片加载失败只补读取，不诱导再次上传或删除。
          media?.release(); media = null
          Object.assign(remoteUserAvatar, { source: '', loaded: false })
          refreshAfterSave = true
        }
      }
    }
  } finally {
    if (stamp === revision) {
      remoteUserAvatar.busy = false
      // 保存期间的推送在写操作结束后再读取，不让旧读取把新头像覆盖回去。
      if (refreshAfterSave) { refreshAfterSave = false; quietlyRefreshRemoteUserAvatar() }
    }
  }
}
