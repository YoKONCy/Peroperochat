<template>
  <details v-if="steps.length" class="remote-task-plan"><summary><span>{{ plan.title || '行动清单' }}</span><small>已完成 {{ steps.filter(step => step.status === 'completed').length }} / {{ steps.length }} 步</small></summary><ol class="remote-trace-plan"><li v-for="(step, index) in steps" :key="step.id || index" :data-state="step.status"><RemoteIcon :name="step.status === 'completed' ? 'check' : step.status === 'blocked' ? 'alert' : 'diamond'" /><span>{{ step.title }}</span><small>{{ statusLabel(step.status) }}</small></li></ol><p class="remote-help">服务端记录的步骤进度</p></details>
</template>
<script setup>
import { computed } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
const props = defineProps({ plan: { type: Object, required: true } })
const statuses = new Set(['pending', 'in_progress', 'completed', 'blocked'])
const steps = computed(() => Array.isArray(props.plan.steps) && props.plan.steps.every(step => step && typeof step.title === 'string' && statuses.has(step.status)) ? props.plan.steps : [])
const statusLabel = value => ({ pending: '待处理', in_progress: '进行中', completed: '已完成', blocked: '受阻' }[value] || value)
</script>
