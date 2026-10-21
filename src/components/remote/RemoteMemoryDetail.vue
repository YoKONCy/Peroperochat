<template>
  <RoomDrawer title="记忆" variant="editor" @close="$emit('close')">
    <section class="remote-memory-reader">
      <header><span><RoomIcon name="memory" />记下的事</span><button type="button" class="room-icon-button" aria-label="刷新记忆" :disabled="loading" @click="reload"><RoomIcon :class="{ 'room-spinning': loading }" name="refresh" /></button></header>
      <RemoteFeedback :message="error" />
      <p v-if="loading && !memory" class="room-empty">正在展开记忆…</p>
      <template v-if="memory || detail.summary">
        <h2>{{ summary || '一段记忆' }}</h2>
        <RemoteRichText v-if="narrative && narrative !== summary" :source="narrative" node-id="memory-body" :context="context" @resource="$emit('resource', $event)" />
        <p v-if="memory?.createdAt" class="memory-date">{{ date(memory.createdAt) }}</p>
      </template>
      <p v-else-if="!loading" class="room-empty">暂时没能打开这条记忆。<button class="room-text-button" @click="reload">重试</button></p>
      <section v-if="memory" class="memory-chat-source">
        <div><h3>相关聊天</h3><button type="button" class="memory-source-button" :disabled="loading || sourceLoading" @click="loadSource"><RoomIcon name="chat" />{{ sourceLoading ? '正在查找…' : source ? '重新查找' : '查看相关聊天' }}</button></div>
        <div v-if="source && (source.available === false || !source.messages?.length)" class="memory-source-empty"><RoomIcon name="chat" /><div><strong>暂时找不到对应的聊天记录</strong><p>这条记忆仍然保留。</p></div></div>
        <template v-else-if="source">
          <article v-for="message in source.messages" :key="message.id" class="memory-source-message"><small>{{ message.role === 'user' ? '你' : message.role === 'assistant' ? '角色' : '记录' }}</small><RemoteRichText :source="message.content" :node-id="String(message.id)" :context="context" @resource="$emit('resource', $event)" /></article>
          <button v-if="context.threadId && context.agentId" type="button" class="memory-source-button" @click="$emit('thread', { ...context })">打开这段会话<RoomIcon name="next" /></button>
        </template>
      </section>
    </section>
  </RoomDrawer>
</template>
<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RoomIcon from './RemoteIcon.vue'
import RemoteRichText from './RemoteRichText.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import { remoteClient } from '../../services/remote'
import { readRemote } from '../../stores/remote'
import { currentToken, assertToken } from '../../stores/mode'
const props = defineProps({ detail: { type: Object, required: true } })
defineEmits(['close', 'resource', 'thread'])
const token = currentToken(), memory = ref(null), source = ref(null), loading = ref(false), sourceLoading = ref(false), error = ref('')
const summary = computed(() => memory.value?.summary || memory.value?.event?.summary || props.detail.summary || '')
const narrative = computed(() => memory.value?.narrative || memory.value?.event?.narrative || '')
const context = computed(() => ({ agentId: memory.value?.agentId || memory.value?.event?.agentId, threadId: memory.value?.origin?.threadId || memory.value?.event?.origin?.threadId }))
let disposed = false, generation = 0
function date(value) { const result = new Date(value); return Number.isNaN(result.getTime()) ? '' : result.toLocaleDateString() }
async function reload() {
  if (loading.value || sourceLoading.value) return
  const request = ++generation; loading.value = true; error.value = ''
  try {
    const result = await remoteClient.manage({ operation: 'memories.get', id: String(props.detail.query.id), params: {}, body: null }, { token })
    assertToken(token, 'remote')
    if (!disposed && request === generation) { memory.value = result.data; source.value = null }
  } catch (failure) { if (!disposed && request === generation) error.value = failure.message }
  finally { if (!disposed && request === generation) loading.value = false }
}
async function loadSource() {
  if (loading.value || sourceLoading.value) return
  const request = generation; sourceLoading.value = true; error.value = ''
  try {
    const result = await readRemote({ kind: 'memory_source', id: props.detail.query.id })
    assertToken(token, 'remote')
    if (!disposed && request === generation) source.value = result
  } catch (failure) { if (!disposed && request === generation) error.value = failure.message }
  finally { if (!disposed && request === generation) sourceLoading.value = false }
}
onMounted(reload)
onBeforeUnmount(() => { disposed = true; generation++ })
</script>
