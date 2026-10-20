import { remoteClient } from './remote'
import { currentToken, assertToken, onModeInvalidated } from '../stores/mode'
import { reactive } from 'vue'

const urls = new Set()
const avatars = new Map()
export const remoteAvatarRevisions = reactive(new Map())
export function invalidateRemoteAvatar(id) {
  const token = currentToken(); assertToken(token, 'remote')
  const key = `${token.target.profileId}:${token.epoch}:${id}`, entry = avatars.get(key)
  avatars.delete(key)
  if (entry && entry.refs === 0) entry.media?.release()
  remoteAvatarRevisions.set(id, (remoteAvatarRevisions.get(id) || 0) + 1)
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
    Promise.resolve().then(() => { assertToken(job.token, 'remote'); return loadRemoteResource({ kind: 'agent_avatar', id: job.id }) })
      .then(job.resolve, job.reject).finally(() => { avatarLoads--; pumpAvatars() })
  }
}
export async function acquireRemoteAvatar(id) {
  const token = currentToken()
  assertToken(token, 'remote')
  const key = `${token.target.profileId}:${token.epoch}:${id}`
  let entry = avatars.get(key)
  if (!entry) {
    entry = { refs: 0, media: null }
    entry.promise = new Promise((resolve, reject) => { avatarQueue.push({ id, token, resolve, reject }); pumpAvatars() }).then(media => { entry.media = media; return media })
    avatars.set(key, entry)
  }
  entry.refs++
  try {
    const media = await entry.promise
    assertToken(token, 'remote')
    let released = false
    return { ...media, release() {
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
  const result = await remoteClient.resource(resource, { ...options, token })
  assertToken(token, 'remote')
  // HTML、SVG 等主动内容只作为文本或下载数据；不注入应用 DOM。
  const mime = result.mimeType.split(';')[0].trim().toLowerCase()
  const passive = /^(image\/(png|jpeg|webp|gif|avif|bmp|x-icon)|audio\/[a-z0-9.+-]+|video\/[a-z0-9.+-]+|application\/pdf)$/.test(mime)
  const blob = new Blob([new Uint8Array(result.bytes)], { type: passive ? mime : 'text/plain' })
  const url = URL.createObjectURL(blob)
  urls.add(url)
  return {
    url, blob, mimeType: mime, contentRange: result.contentRange, previewable: passive,
    release() { URL.revokeObjectURL(url); urls.delete(url) },
  }
}
