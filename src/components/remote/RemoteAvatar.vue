<template><CharacterAvatar :src="source" :label="label" /></template>
<script setup>
import { ref, watch, onBeforeUnmount } from 'vue'
import CharacterAvatar from '../companion/CharacterAvatar.vue'
import { acquireRemoteAvatar, remoteAvatarRevisions } from '../../services/remoteMedia'
const props = defineProps({ id: String, label: String })
const source = ref('')
let version = 0
let resource = null
watch(() => [props.id, remoteAvatarRevisions.get(props.id) || 0], async ([id]) => {
  const request = ++version
  resource?.release(); resource = null; source.value = ''
  if (!id) return
  try {
    const loaded = await acquireRemoteAvatar(id)
    if (request !== version || !loaded.mimeType.startsWith('image/') || !loaded.previewable) { loaded.release(); return }
    resource = loaded; source.value = loaded.url
  } catch { /* 离线或服务端没有头像时使用统一的角色轮廓。 */ }
}, { immediate: true })
onBeforeUnmount(() => { version++; resource?.release() })
</script>
