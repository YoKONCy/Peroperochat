<template><RoomDrawer :title="name || '查看附件'" subtitle="来自当前 infOS 连接" variant="editor" @close="$emit('close')"><div class="remote-resource"><p v-if="loading" class="room-empty">正在读取附件…</p><p v-if="error" class="room-error">{{ error }}</p><template v-if="media"><img v-if="media.previewable && media.mimeType.startsWith('image/')" :src="media.url" :alt="name" /><audio v-else-if="media.mimeType.startsWith('audio/')" :src="media.url" controls /><video v-else-if="media.mimeType.startsWith('video/')" :src="media.url" controls playsinline /><pre v-else-if="text !== null">{{ text }}</pre><p v-else class="remote-help">文件已读取，可以保存后使用支持此格式的应用查看。</p><a :href="media.url" :download="name || '附件'" class="room-secondary-button">保存文件</a></template></div></RoomDrawer></template>
<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import { loadRemoteResource } from '../../services/remoteMedia'
const props = defineProps({ resource: Object, name: String })
defineEmits(['close'])
const media = ref(null), loading = ref(true), error = ref(''), text = ref(null)
let disposed = false
onMounted(async () => {
  try {
    const loaded = await loadRemoteResource(props.resource)
    if (disposed) { loaded.release(); return }
    media.value = loaded
    if (/^(text\/|application\/(json|xml)|image\/svg\+xml)/.test(loaded.mimeType)) { const content = await loaded.blob.text(); if (!disposed) text.value = content.slice(0, 256 * 1024) }
  } catch (failure) { if (!disposed) error.value = failure.message } finally { if (!disposed) loading.value = false }
})
onBeforeUnmount(() => { disposed = true; media.value?.release() })
</script>
