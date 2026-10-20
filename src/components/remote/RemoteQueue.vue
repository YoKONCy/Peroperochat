<template>
  <aside v-if="items.length || snapshot?.paused || error" class="remote-queue" aria-label="待发送消息队列">
    <header><strong><RemoteIcon name="chat" />{{ snapshot?.paused ? '队列已暂停' : '消息队列' }}<small>{{ items.length }}</small></strong><button type="button" :disabled="changing || !online" @click="control">{{ snapshot?.paused ? '继续发送' : '暂停队列' }}</button></header>
    <p v-if="error" class="remote-help" role="alert">{{ error }}<button type="button" @click="refresh">重新同步</button></p>
    <p v-if="snapshot?.reason" class="remote-help">{{ snapshot.reason }}</p>
    <div class="remote-queue-items">
      <article v-for="item in items" :key="item.id"><span>{{ remoteStatus(item.state) }}{{ item.immediate ? ' · 优先' : '' }}</span><p>{{ item.input?.content }}</p><small v-if="item.input?.attachmentIds?.length">{{ item.input.attachmentIds.length }} 个附件</small><p v-if="item.error" class="remote-help">{{ item.error }}</p>
        <form v-if="editing === item.id" @submit.prevent="change(item, 'edit', edited)"><textarea v-model="edited" rows="2" maxlength="131072" aria-label="编辑队列消息" /><button :disabled="changing || !edited.trim()">保存</button><button type="button" @click="editing = null">保留原文</button></form>
        <div v-else class="remote-queue-actions"><button v-if="item.state === 'queued'" :disabled="changing || !online" @click="change(item, 'immediate')">立即提交</button><button v-if="item.state === 'queued'" :disabled="changing || !online" @click="editing = item.id; edited = item.input.content">编辑</button><button v-if="['failed', 'interrupted'].includes(item.state)" :disabled="changing || !online" @click="change(item, 'continue')">继续此条</button><button v-if="item.state === 'queued' && !item.immediate" :disabled="changing || !online || ordered[0]?.id === item.id" aria-label="向前移动" @click="move(item, -1)"><RemoteIcon name="up" /></button><button v-if="item.state === 'queued' && !item.immediate" :disabled="changing || !online || ordered.at(-1)?.id === item.id" aria-label="向后移动" @click="move(item, 1)"><RemoteIcon name="down" /></button><button :disabled="changing || !online" @click="change(item, 'cancel')">取消</button></div>
      </article>
    </div>
  </aside>
</template>
<script setup>
import { computed, ref, watch, onMounted, onBeforeUnmount } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
import { remoteState, remoteViews, remoteExecutions, readRemote, performRemote, installQueuedLive } from '../../stores/remote'
import { currentToken, assertToken } from '../../stores/mode'
import { remoteStatus } from '../../utils/remoteMarkdown'
const props = defineProps({ threadId: { type: String, required: true }, agentId: { type: String, required: true }, visible: Boolean })
const emit = defineEmits(['updated'])
const token = currentToken(), query = { kind: 'queue', id: props.threadId }
const snapshot = computed(() => remoteViews.get(JSON.stringify(query))?.data)
const online = computed(() => remoteState.connection === 'online')
const items = computed(() => (snapshot.value?.items || []).filter(item => ['queued', 'failed', 'interrupted'].includes(item.state)))
const ordered = computed(() => items.value.filter(item => item.state === 'queued' && !item.immediate))
const changing = ref(false), error = ref(''), editing = ref(null), edited = ref('')
let disposed = false, loading = false, timer = null, previousRun = ''
function guard() { assertToken(token, 'remote'); if (disposed) throw new Error('队列面板已关闭') }
async function refresh() {
  if (loading || disposed || document.hidden || !props.visible) return
  loading = true
  try {
    const next = await readRemote(query); guard(); error.value = ''
    if (remoteViews.get(JSON.stringify(query))?.cached) return
    const running = next.items.find(item => item.state === 'running')?.id || ''
    if (next.active || running) {
      const live = await readRemote({ kind: 'queue_live', id: props.threadId }, { allowCached: false }); guard()
      if (live && live.principalId === props.agentId) installQueuedLive(live)
    } else {
      for (const execution of remoteExecutions.values()) if (execution.requestId.startsWith('queue-live:') && execution.threadId === props.threadId && execution.state === 'observing') { execution.state = 'completed'; execution.needsRefresh = true }
    }
    if (running !== previousRun) { previousRun = running; emit('updated') }
  } catch (failure) { if (!disposed) error.value = failure.message }
  finally { loading = false }
}
async function mutate(action) {
  if (changing.value || !online.value) return
  changing.value = true; error.value = ''
  try { guard(); await performRemote(action); guard(); editing.value = null; await refresh(); emit('updated') }
  catch (failure) { if (!disposed) { await refresh(); if (!disposed) error.value = failure.message } }
  finally { if (!disposed) changing.value = false }
}
const control = () => mutate({ kind: 'queue_control', threadId: props.threadId, paused: !snapshot.value?.paused })
const change = (item, action, content = null) => mutate({ kind: 'queue_item', threadId: props.threadId, id: item.id, revision: item.revision, action, content })
function move(item, offset) {
  const order = ordered.value.map(({ id, revision }) => ({ id, revision })), index = order.findIndex(value => value.id === item.id), target = index + offset
  if (index < 0 || target < 0 || target >= order.length) return
  ;[order[index], order[target]] = [order[target], order[index]]
  return mutate({ kind: 'queue_order', threadId: props.threadId, order })
}
async function poll() { await refresh(); if (!disposed) timer = setTimeout(poll, document.hidden || !props.visible ? 10000 : 2500) }
const wake = () => { if (!document.hidden) refresh() }
watch(() => props.visible, value => { if (value) refresh() })
onMounted(() => { poll(); document.addEventListener('visibilitychange', wake); window.addEventListener('online', wake) })
onBeforeUnmount(() => { disposed = true; clearTimeout(timer); document.removeEventListener('visibilitychange', wake); window.removeEventListener('online', wake) })
</script>
<style scoped>
.remote-queue { flex: none; margin: 8px 14px 0; padding: 10px 12px; border: 1px solid var(--room-line); border-radius: 14px; background: var(--room-accent-soft); font-size: 11px; max-height: 30%; overflow: auto; }.remote-queue header, .remote-queue strong, .remote-queue-actions { display: flex; align-items: center; gap: 8px; }.remote-queue header { justify-content: space-between; }.remote-queue svg { width: 15px; height: 15px; }.remote-queue strong { font-weight: 600; }.remote-queue button { border: 0; background: transparent; color: var(--room-accent); font: inherit; padding: 6px 3px; }.remote-queue button:disabled { opacity: .4; }.remote-queue article { border-top: 1px solid var(--room-line); padding-top: 8px; margin-top: 8px; }.remote-queue article > p { white-space: pre-wrap; overflow-wrap: anywhere; max-height: 4.8em; overflow: auto; line-height: 1.6; margin: 5px 0; }.remote-queue article > span, .remote-queue article > small { color: var(--room-muted); font-size: 10px; }.remote-queue textarea { width: 100%; padding: 8px; font: inherit; border: 1px solid var(--room-line); border-radius: 8px; background: var(--room-surface); color: var(--room-ink); }
</style>
