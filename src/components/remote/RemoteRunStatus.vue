<template>
  <div class="remote-execution-status" :data-state="node.props.state" role="status"><span class="remote-status-dot" /><span>{{ node.props.message || remoteStatus(node.props.state) }}</span><ElapsedTime :started-at="startedAt" :running="running" :duration-ms="node.props.totalDurationMs ?? node.props.outputDurationMs" :prefix="running ? t('已等待', [], 'execution') : ''" /></div>
</template>
<script setup>
import { computed } from 'vue'
import ElapsedTime from '../companion/ElapsedTime.vue'
import { remoteStatus } from '../../utils/remoteMarkdown'
import { t } from '../../i18n/remote'
const props = defineProps({ node: { type: Object, required: true }, surface: { type: Object, required: true }, startedAt: [Number, String] })
const running = computed(() => props.surface.state === 'open' && !props.surface.suspended && ['thinking', 'assembling', 'calling', 'generating', 'queued', 'running'].includes(props.node.props.state))
</script>
<style scoped>
.remote-execution-status { flex-wrap: wrap; row-gap: 4px; }
.remote-execution-status > span:nth-child(2) { min-width: 0; overflow-wrap: anywhere; }
</style>
