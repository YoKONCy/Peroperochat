<template>
  <details class="remote-reasoning" :data-streaming="active && node.props.phase === 'preview'">
    <summary><RemoteIcon name="orbit" /><strong>{{ node.kind === 'native-reasoning' ? '模型思考' : '思考过程' }}</strong><span v-if="Number.isSafeInteger(node.props.turn)">第 {{ node.props.turn }} 轮</span><span class="remote-reasoning-phase">{{ node.props.phase === 'preview' ? active ? '接收中' : '记录未完成' : '已记录' }}</span><small v-if="duration">{{ duration }}</small><RemoteIcon class="remote-trace-chevron" name="down" /></summary>
    <div class="remote-reasoning-content"><RemoteTraceText v-if="node.props.source" :value="node.props.source" /><p v-else class="remote-help">{{ active && node.props.phase === 'preview' ? '等待模型返回思考内容…' : '服务端未提供思考正文。' }}</p></div>
  </details>
</template>
<script setup>
import { computed } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteTraceText from './RemoteTraceText.vue'
import { traceDuration } from '../../utils/remoteTrace'
const props = defineProps({ node: { type: Object, required: true }, active: Boolean })
const duration = computed(() => traceDuration(props.node.props.durationMs))
</script>
