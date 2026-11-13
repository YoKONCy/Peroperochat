<template>
  <div class="remote-stream-tail" :class="{ 'is-receiving': active, 'markdown-body': formatted }" :aria-label="active ? '正在接收的正文' : '未完成的正文'" @click="activateLink">
    <div v-if="formatted" v-html="html" />
    <pre v-else class="remote-rich-raw">{{ source }}</pre>
    <p v-if="failure" class="remote-rich-incomplete" role="alert">{{ failure }}</p>
  </div>
</template>
<script setup>
import { computed, ref, watch } from 'vue'
import { renderRemoteMarkdown } from '../../utils/remoteRichContent'
import { openRemoteLink } from '../../utils/remoteMarkdown'
import { resolveRemoteResource } from '../../services/remoteResources'
import { currentToken } from '../../stores/mode'
const props = defineProps({ source: { type: String, default: '' }, active: Boolean, formatted: { type: Boolean, default: true }, context: Object })
const emit = defineEmits(['resize', 'resource'])
const failure = ref('')
// 仅重排活跃尾段，已经完成的段落由上层保留独立节点。
const html = computed(() => props.formatted ? renderRemoteMarkdown(props.source) : '')
async function activateLink(event) {
  const link = event.target.closest('a')
  if (!link || !event.currentTarget.contains(link)) return
  event.preventDefault(); event.stopPropagation()
  try {
    if (link.dataset.resourceSource) emit('resource', await resolveRemoteResource(link.dataset.resourceSource, props.context, currentToken()))
    else await openRemoteLink(event)
  } catch (error) { failure.value = error.message }
}
watch(() => props.source, () => emit('resize'), { flush: 'post' })
</script>
<style scoped>
.remote-stream-tail.markdown-body { white-space: normal; }
</style>
