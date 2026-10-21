<template>
  <div class="remote-operation-result">
    <template v-if="segments.length"><p class="remote-help">编译结果 · {{ value.tokens ?? '—' }} Token · {{ segments.length }} 个片段</p><article v-for="(segment, index) in segments" :key="segment.id || index" class="remote-result-message"><small>{{ segment.role }} · {{ segment.source || '片段 ' + (index + 1) }} · {{ segment.tokens ?? '—' }} Token</small><p>{{ segment.content }}</p></article><p v-for="(issue, index) in value.issues || []" :key="index" :class="issue.level === 'error' ? 'room-error' : 'remote-help'">{{ issue.message }}</p></template>
    <template v-else-if="messages.length"><article v-for="(message, index) in messages" :key="message.id || index" class="remote-result-message"><small>{{ message.role === 'user' ? '用户输入' : message.role === 'system' ? '系统提示' : '角色回复' }}</small><p>{{ message.content }}</p></article></template>
    <template v-else-if="Array.isArray(value)"><p class="remote-help">共 {{ value.length }} 项</p><article v-for="(item, index) in value.slice(0, limit)" :key="index" class="remote-result-item"><RemoteIcon :name="item?.level === 'error' || item?.error ? 'alert' : 'files'" /><div><strong>{{ item?.title || item?.name || item?.message || item?.label || '第 ' + (index + 1) + ' 项' }}</strong><p>{{ typeof item === 'string' ? item : item?.description || item?.content || item?.summary || item?.path || '' }}</p><RemoteDataDetails v-if="item && typeof item === 'object'" :value="item" /></div></article><RemoteAction v-if="value.length > limit" icon="down" @click="limit += 40">展开更多</RemoteAction></template>
    <RemoteRecordView v-else-if="value && typeof value === 'object'" :kind="operation?.split('.')[0]" :record="value" />
    <p v-else class="remote-result-text">{{ value == null ? '操作已完成，无额外返回内容。' : String(value) }}</p>
  </div>
</template>
<script setup>
import { computed, ref } from 'vue'
import RemoteDataDetails from './RemoteDataDetails.vue'
import RemoteRecordView from './RemoteRecordView.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteIcon from './RemoteIcon.vue'
const props = defineProps({ value: { default: null }, operation: String })
const limit = ref(40)
const messages = computed(() => Array.isArray(props.value?.messages) ? props.value.messages : [])
const segments = computed(() => Array.isArray(props.value?.segments) ? props.value.segments : [])
</script>
