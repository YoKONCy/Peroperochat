<template>
  <SoftDisclosure class="remote-reasoning expression-reasoning" :data-effort="effort" :data-reasoning-kind="node.kind" :data-streaming="active && node.props.phase === 'preview'" @resize="$emit('resize')">
    <template #summary><span class="expression-thought-symbol"><ReasoningIcon :level="effort" :running="running" /></span><strong>{{ tr(running ? '思考中' : '思考过程') }}</strong><span class="chat-reasoning-meta"><small v-if="effort !== 'unknown'" class="expression-effort-label">{{ tr(reasoningLabels[effort]) }}</small><small v-if="duration">{{ duration }}</small></span><RemoteIcon class="remote-trace-chevron" name="down" /></template>
    <div class="remote-reasoning-content"><RemoteRichText v-if="node.props.source" :source="node.props.source" :node-id="node.nodeId" :streaming="active && node.props.phase === 'preview'" :context="context" @resource="$emit('resource', $event)" @resize="$emit('resize')" /><p v-else class="remote-help">{{ tr(running ? '等待模型返回思考内容…' : '这一步没有额外的思考说明。') }}</p></div>
  </SoftDisclosure>
</template>
<script setup>
import { computed } from 'vue'
import SoftDisclosure from '../companion/SoftDisclosure.vue'
import RemoteIcon from './RemoteIcon.vue'
import ReasoningIcon from '../companion/ReasoningIcon.vue'
import { recordedReasoning, reasoningLabels } from '../../utils/reasoningAppearance'
import { t } from '../../i18n/remote'
import RemoteRichText from './RemoteRichText.vue'
import { traceDuration } from '../../utils/remoteTrace'
const props = defineProps({ node: { type: Object, required: true }, active: Boolean, context: Object })
defineEmits(['resource', 'resize'])
const tr = (key, values) => t(key, values, 'expression')
const effort = computed(() => recordedReasoning(props.node.props))
const running = computed(() => props.active && props.node.props.phase === 'preview')
const duration = computed(() => traceDuration(props.node.props.durationMs))
</script>
