<template>
  <div v-if="blocks.length" class="remote-rich-text journal-prose">
    <template v-for="block in blocks" :key="block.id">
      <ChatMarkdown v-if="block.kind === 'markdown'" class="markdown-body" :html="roomMarkdown(block.source)" @resize="$emit('resize')" />
      <pre v-else-if="block.kind === 'raw'" class="remote-rich-raw">{{ block.source }}</pre>
      <RemoteRichBlock v-else :block="block" @resize="$emit('resize')" />
    </template>
  </div>
  <ChatMarkdown v-else class="markdown-body" :html="plain" @resize="$emit('resize')" />
</template>
<script setup>
import { computed } from 'vue'
import ChatMarkdown from '../ChatMarkdown.vue'
import RemoteRichBlock from '../remote/RemoteRichBlock.vue'
import { RemoteMarkdownSegmenter } from '../../utils/remoteRichContent'
import { roomMarkdown } from '../../utils/roomMarkdown'
import { visibleMessage } from '../../utils/messageContent'
const props = defineProps({ source: String, pending: Boolean })
defineEmits(['resize'])
const plain = computed(() => roomMarkdown(props.source))
const blocks = computed(() => {
  if (props.pending || !/^(?: {0,3}(?:`{3}|~{3})|\s*\$\$|\s*\\\[)/m.test(props.source || '')) return []
  return new RemoteMarkdownSegmenter(true).update(visibleMessage(props.source), true)
})
</script>
