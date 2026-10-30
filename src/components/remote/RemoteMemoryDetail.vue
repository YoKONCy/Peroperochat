<template>
  <RoomDrawer title="记忆" variant="editor" :busy="memoryActionsBusy" @close="$emit('close')">
    <section class="remote-memory-reader">
      <header><span><RoomIcon name="memory" />记下的事</span><button type="button" class="room-icon-button" aria-label="刷新记忆" :disabled="loading || sourceLoading || memoryActionsBusy" @click="reload"><RoomIcon :class="{ 'room-spinning': loading }" name="refresh" /></button></header>
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
    <template v-if="memory" #footer><RemoteMemoryActions ref="memoryActions" :record="memory" :disabled="loading || sourceLoading || editing" @edit="editing = true" @changed="memoryChanged" @busy="memoryActionsBusy = $event" @pending="$emit('pending')" /></template>
  </RoomDrawer>
  <RemoteArchiveEditor v-if="editing && memory" ref="archiveEditor" kind="memories" :record="memory" @close="editing = false" @saved="edited" @changed="memoryChanged" @pending="$emit('pending')" />
</template>
<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RoomIcon from './RemoteIcon.vue'
import RemoteRichText from './RemoteRichText.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import RemoteArchiveEditor from './RemoteArchiveEditor.vue'
import RemoteMemoryActions from './RemoteMemoryActions.vue'
import { remoteClient } from '../../services/remote'
import { readRemote } from '../../stores/remote'
import { currentToken, assertToken } from '../../stores/mode'
const props = defineProps({ detail: { type: Object, required: true } })
const emit = defineEmits(['close', 'resource', 'thread', 'pending'])
const editing = ref(false), archiveEditor = ref(null), memoryActions = ref(null), memoryActionsBusy = ref(false)
async function prepareLeave() { await memoryActions.value?.prepareLeave?.(); await archiveEditor.value?.prepareLeave?.(); editing.value = false }
defineExpose({ prepareLeave })
function edited(value) { memory.value = value; source.value = null; editing.value = false; error.value = ''; window.dispatchEvent(new CustomEvent('ppc:memories-changed', { detail: { token, id: value.id, action: 'edit', agentId: value.agentId } })) }
async function memoryChanged(change) {
  if (change.action === 'delete') { emit('close'); return }
  memory.value = { ...memory.value, status: change.action === 'archive' ? 'archived' : 'active' }
  await reload()
}
const token = currentToken(), memory = ref(null), source = ref(null), loading = ref(false), sourceLoading = ref(false), error = ref('')
const summary = computed(() => memory.value?.summary || memory.value?.event?.summary || memory.value?.narrative?.slice(0, 100) || props.detail.summary || '')
const narrative = computed(() => memory.value?.narrative || memory.value?.event?.narrative || '')
const context = computed(() => ({ agentId: memory.value?.agentId || memory.value?.event?.agentId, threadId: memory.value?.origin?.threadId || memory.value?.event?.origin?.threadId }))
let disposed = false, generation = 0
function date(value) { const result = new Date(value); return Number.isNaN(result.getTime()) ? '' : result.toLocaleDateString() }
async function reload() {
  if (loading.value || sourceLoading.value || memoryActionsBusy.value) return
  const request = ++generation; loading.value = true; error.value = ''
  try {
    const result = await remoteClient.manage({ operation: 'memories.get', id: String(memory.value?.id || props.detail.query.id), params: {}, body: null }, { token })
    assertToken(token, 'remote')
    if (!disposed && request === generation) { memory.value = result.data; source.value = null }
  } catch (failure) { if (!disposed && request === generation) error.value = failure.message }
  finally { if (!disposed && request === generation) loading.value = false }
}
async function loadSource() {
  if (loading.value || sourceLoading.value || memoryActionsBusy.value) return
  const request = generation; sourceLoading.value = true; error.value = ''
  try {
    const result = await readRemote({ kind: 'memory_source', id: memory.value?.id || props.detail.query.id })
    assertToken(token, 'remote')
    if (!disposed && request === generation) source.value = result
  } catch (failure) { if (!disposed && request === generation) error.value = failure.message }
  finally { if (!disposed && request === generation) sourceLoading.value = false }
}
onMounted(reload)
onBeforeUnmount(() => { disposed = true; generation++ })
</script>
