<template><CharacterAvatar :src="source" :label="label" :fit="fit" :aria-busy="loading" @error="imageFailed" /></template>
<script setup>
import { ref, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import CharacterAvatar from '../companion/CharacterAvatar.vue'
import { acquireRemoteAvatar, invalidateRemoteAvatar, remoteAvatarRevisions } from '../../services/remoteMedia'
import { remoteState } from '../../stores/remote'
const props = defineProps({ id: String, label: String, fit: { type: String, default: 'cover' } })
const source = ref(''), loading = ref(false)
let version = 0, disposed = false, shownId = null, shownRevision = -1, decodingRetried = false, retries = 0, retryTimer = null
let resource = null
async function load(id, revision) {
  const request = ++version
  if (id !== shownId) { resource?.release(); resource = null; source.value = ''; shownId = id; shownRevision = -1; decodingRetried = false }
  if (!id || remoteState.connection !== 'online' || document.hidden || disposed) { loading.value = false; return }
  if (resource && shownRevision === revision) { loading.value = false; return }
  loading.value = true
  try {
    const loaded = await acquireRemoteAvatar(id)
    if (disposed || request !== version || !loaded.mimeType.startsWith('image/') || !loaded.previewable) { loaded.release(); return }
    const previous = resource
    resource = loaded; source.value = loaded.url; shownRevision = revision
    await nextTick(); previous?.release()
  } catch {
    // 短暂网络失败只延迟补一次；保留已显示头像，不进行持续轮询。
    if (!disposed && request === version && retries++ === 0 && remoteState.connection === 'online') retryTimer = setTimeout(() => { retryTimer = null; load(id, revision) }, 1500)
  }
  finally { if (!disposed && request === version) loading.value = false }
}
function refresh() { if (!document.hidden && !resource) load(props.id, remoteAvatarRevisions.get(props.id) || 0) }
function imageFailed(failedSource) {
  if (disposed || failedSource !== source.value) return
  resource?.release(); resource = null; source.value = ''; shownRevision = -1
  // 解码失败只主动重试一次，避免损坏图片形成无限请求。
  if (!decodingRetried && remoteState.connection === 'online' && !disposed) { decodingRetried = true; try { invalidateRemoteAvatar(props.id) } catch { /* 切换运行模式后不刷新旧连接。 */ } }
}
watch(() => [props.id, remoteAvatarRevisions.get(props.id) || 0, remoteState.connection], ([id, revision]) => { clearTimeout(retryTimer); retries = 0; load(id, revision) }, { immediate: true })
onMounted(() => document.addEventListener('visibilitychange', refresh))
onBeforeUnmount(() => { disposed = true; version++; clearTimeout(retryTimer); resource?.release(); document.removeEventListener('visibilitychange', refresh) })
</script>
