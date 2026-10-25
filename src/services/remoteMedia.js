import { remoteClient } from './remote'
import { currentToken, assertToken, onModeInvalidated } from '../stores/mode'
import { reactive } from 'vue'
import { avatarImageMime } from '../utils/avatarImage'
import { remoteState } from '../stores/remote'

const urls = new Set()
const avatars = new Map()
export const remoteAvatarRevisions = reactive(new Map())
export function invalidateRemoteAvatar(id) {
  const token = currentToken(); assertToken(token, 'remote')
  for (const [key, entry] of avatars) if (entry.id === id) { avatars.delete(key); if (entry.refs === 0) entry.media?.release() }
  remoteAvatarRevisions.set(id, (remoteAvatarRevisions.get(id) || 0) + 1)
}
export function invalidateAllRemoteAvatars() {
  const ids = new Set([...avatars.values()].map(entry => entry.id))
  for (const id of ids) invalidateRemoteAvatar(id)
}
const avatarQueue = []
let avatarLoads = 0
onModeInvalidated(() => {
  for (const url of urls) URL.revokeObjectURL(url)
  urls.clear()
  for (const job of avatarQueue.splice(0)) job.reject(new Error('运行模式已改变'))
  avatars.clear()
  remoteAvatarRevisions.clear()
})
function pumpAvatars() {
  while (avatarLoads < 3 && avatarQueue.length) {
    const job = avatarQueue.shift()
    avatarLoads++
    Promise.resolve().then(() => { assertToken(job.token, 'remote'); return loadRemoteResource({ kind: 'agent_avatar', id: job.id }, { cachedOnly: job.cachedOnly }) })
      .then(job.resolve, job.reject).finally(() => { avatarLoads--; pumpAvatars() })
  }
}
export async function acquireRemoteAvatar(id, { cachedOnly = false } = {}) {
  const token = currentToken()
  assertToken(token, 'remote')
  const key = `${token.target.profileId}:${token.epoch}:${id}:${cachedOnly ? 'cached' : 'live'}`
  let entry = avatars.get(key)
  if (entry?.media && entry.expiresAt <= Date.now() && !cachedOnly) { avatars.delete(key); if (!entry.refs) entry.media.release(); entry = null }
  if (!entry) {
    entry = { id, refs: 0, media: null, expiresAt: 0 }
    entry.promise = new Promise((resolve, reject) => { avatarQueue.push({ id, token, cachedOnly, resolve, reject }); pumpAvatars() }).then(media => {
      if (!media.previewable || !media.mimeType.startsWith('image/')) { media.release(); throw new Error('头像图片暂时无法显示') }
      entry.expiresAt = Date.now() + 60000; entry.media = media; return media
    })
    avatars.set(key, entry)
  }
  entry.refs++
  try {
    const media = await entry.promise
    assertToken(token, 'remote')
    let released = false
    return { ...media, expiresAt: entry.expiresAt, release() {
      if (released) return
      released = true; entry.refs--
      if (entry.refs === 0 && avatars.get(key) !== entry) { entry.media?.release(); return }
      for (const [cacheKey, cached] of avatars) if (avatars.size > 32 && cached.refs === 0) { cached.media?.release(); avatars.delete(cacheKey) }
    } }
  } catch (error) { entry.refs--; if (avatars.get(key) === entry) avatars.delete(key); throw error }
}
export async function uploadRemoteFile(threadId, file) {
  const token = currentToken()
  assertToken(token, 'remote')
  if (!file?.size || file.size > 8 * 1024 * 1024) throw new Error('请选择不超过 8 MiB 的附件')
  const bytes = Array.from(new Uint8Array(await file.arrayBuffer()))
  assertToken(token, 'remote')
  return (await remoteClient.upload(threadId, { name: file.name, mimeType: file.type || 'application/octet-stream', bytes }, { token })).data
}
export async function loadRemoteResource(resource, options = {}) {
  const token = currentToken()
  const cachedOnly = options.cachedOnly ?? ['offline', 'connecting', 'reconnecting'].includes(remoteState.connection)
  const result = await remoteClient.resource(resource, { ...options, cachedOnly, token })
  assertToken(token, 'remote')
  // HTML、SVG 等主动内容只作为文本或下载数据；不注入应用 DOM。
  const bytes = new Uint8Array(result.bytes)
  const mime = ['agent_avatar', 'owner_avatar'].includes(resource.kind) ? avatarImageMime(bytes) : result.mimeType.split(';')[0].trim().toLowerCase()
  const passive = /^(image\/(png|jpeg|webp|gif|avif|bmp|x-icon)|audio\/[a-z0-9.+-]+|video\/[a-z0-9.+-]+|application\/pdf)$/.test(mime)
  const blob = new Blob([bytes], { type: passive ? mime : 'text/plain' })
  const url = URL.createObjectURL(blob)
  urls.add(url)
  return {
    url, blob, mimeType: mime, contentRange: result.contentRange, previewable: passive, cached: result.cached === true,
    release() { URL.revokeObjectURL(url); urls.delete(url) },
  }
}
