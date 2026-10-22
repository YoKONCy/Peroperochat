<template>
  <details class="remote-reasoning" :data-reasoning-kind="node.kind" :data-streaming="active && node.props.phase === 'preview'" @toggle="$emit('resize')">
    <summary><span class="soft-thought-symbol"><RemoteIcon name="thought" /></span><strong>{{ node.kind === 'native-reasoning' ? '模型思考' : '思考过程' }}</strong><span class="chat-reasoning-meta"><span v-if="active && node.props.phase === 'preview'" class="chat-reasoning-receiving">接收中</span><span v-if="Number.isSafeInteger(node.props.turn)">第 {{ node.props.turn }} 轮</span><small v-if="duration">{{ duration }}</small></span><RemoteIcon class="remote-trace-chevron" name="down" /></summary>
    <div class="remote-reasoning-content"><RemoteRichText v-if="node.props.source" :source="node.props.source" :node-id="node.nodeId" :streaming="active && node.props.phase === 'preview'" :context="context" @resource="$emit('resource', $event)" @resize="$emit('resize')" /><p v-else class="remote-help">{{ active && node.props.phase === 'preview' ? '等待模型返回思考内容…' : '这一步没有额外的思考说明。' }}</p></div>
  </details>
</template>
<script setup>
import { computed } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteRichText from './RemoteRichText.vue'
import { traceDuration } from '../../utils/remoteTrace'
const props = defineProps({ node: { type: Object, required: true }, active: Boolean, context: Object })
defineEmits(['resource', 'resize'])
const duration = computed(() => traceDuration(props.node.props.durationMs))
</script>
