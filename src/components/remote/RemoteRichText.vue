<template>
  <div class="remote-rich-text">
    <template v-for="block in blocks" :key="`${nodeId}:${block.id}`">
      <RemoteStreamText v-if="block.kind === 'raw'" :source="block.source" :active="streaming && !suspended && !!block.streaming" @resize="$emit('resize')" />
      <RemoteRichBlock v-else :block="block" :context="context" @resource="$emit('resource', $event)" @resize="$emit('resize')" />
    </template>
    <p v-if="incomplete" class="remote-rich-incomplete"><RemoteIcon name="clock" />{{ interrupted ? '回复未完成，已收内容保留' : '结构未闭合，保留原文' }}</p>
    <p v-else-if="streaming && suspended" class="remote-rich-incomplete"><RemoteIcon name="clock" />暂未继续接收，已收内容保留</p>
  </div>
</template>
<script setup>
import { shallowRef, watch, computed } from 'vue'
import { RemoteMarkdownSegmenter } from '../../utils/remoteRichContent'
import RemoteStreamText from './RemoteStreamText.vue'
import RemoteRichBlock from './RemoteRichBlock.vue'
import RemoteIcon from './RemoteIcon.vue'
const props = defineProps({ source: { type: String, default: '' }, nodeId: { type: String, default: 'body' }, streaming: Boolean, suspended: Boolean, interrupted: Boolean, context: Object })
defineEmits(['resource', 'resize'])
let segmenter = new RemoteMarkdownSegmenter()
const blocks = shallowRef([])
watch(() => props.nodeId, () => { segmenter = new RemoteMarkdownSegmenter() })
watch(() => [props.source, props.streaming, props.nodeId], () => { blocks.value = segmenter.update(typeof props.source === 'string' ? props.source : '', !props.streaming) }, { immediate: true })
const incomplete = computed(() => !props.streaming && blocks.value.some(block => block.kind === 'raw'))
</script>
