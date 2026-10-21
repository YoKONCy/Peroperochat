<template>
  <aside v-if="items.length || snapshot?.paused || expanded" class="chat-queue-summary" aria-label="待发送消息">
    <button v-if="items.length || snapshot?.paused" type="button" class="chat-queue-chip" :data-attention="failed > 0 || !!error" :aria-expanded="expanded" aria-haspopup="dialog" @click="expanded = true"><RemoteIcon :name="snapshot?.paused ? 'pause' : failed ? 'alert' : 'queue'" /><span>{{ summary }}</span><RemoteIcon name="up" /></button>
    <RoomDrawer v-if="expanded" title="等着说的话" :subtitle="snapshot?.paused ? '已暂停，继续后按顺序发送' : '排好顺序，慢慢说'" :busy="changing" :before-close="beforeClose" @close="close">
      <RemoteFeedback :message="error" />
      <div v-if="discard" class="chat-queue-discard" role="alert"><p>这条消息还有未保存的修改。</p><button type="button" @click="discard = false">继续编辑</button><button type="button" @click="close">放弃修改并关闭</button></div>
      <div v-if="editing && !items.some(item => item.id === editing)" class="chat-queue-orphan"><p>这条消息已经发送或移出队列，你的修改仍在这里。</p><textarea v-model="edited" rows="3" aria-label="尚未保存的队列草稿" /><button type="button" @click="editing = null">放弃这份修改</button></div>
      <ol class="chat-queue-list">
        <li v-for="(item, index) in items" :key="item.id" :class="{ 'is-selected': selected === item.id }">
          <div class="chat-queue-row"><span class="chat-queue-number">{{ String(index + 1).padStart(2, '0') }}</span><button type="button" class="chat-queue-preview" :aria-expanded="selected === item.id" @click="select(item)"><span class="chat-queue-state" :data-failed="['failed', 'interrupted'].includes(item.state)">{{ remoteStatus(item.state) }}{{ item.immediate ? ' · 优先' : '' }}</span><span class="chat-queue-copy">{{ item.input?.content || '附件消息' }}</span><small v-if="item.input?.attachmentIds?.length || reference(item)"><RemoteIcon :name="reference(item) ? 'folder' : 'attach'" />{{ reference(item) || `${item.input.attachmentIds.length} 个附件` }}</small></button><button type="button" class="chat-queue-more" :aria-label="`第 ${index + 1} 条消息的操作`" @click="select(item, true)"><RemoteIcon name="more" /></button></div>
          <template v-if="selected === item.id">
            <p v-if="item.error" class="chat-queue-item-error">{{ remoteFeedbackMessage(item.error) || '这条消息尚未发送。' }}</p>
            <form v-if="editing === item.id" class="chat-queue-editor" @submit.prevent="change(item, 'edit', edited)"><p v-if="item.revision !== editingRevision" class="chat-queue-item-error">这条消息在别处更新了。<button type="button" @click="edit(item)">放弃修改，使用最新版本</button></p><textarea v-model="edited" rows="4" maxlength="131072" aria-label="编辑队列消息" /><div><button type="button" :disabled="changing" @click="editing = null">保留原文</button><button type="submit" :disabled="changing || !writable || !edited.trim() || item.revision !== editingRevision">保存修改</button></div></form>
            <div v-else class="chat-queue-item-actions">
              <button v-if="item.state === 'queued'" type="button" :disabled="changing || !writable" @click="change(item, 'immediate')"><RemoteIcon name="send" />立即发送</button>
              <button v-if="item.state === 'queued'" type="button" :disabled="changing || !writable" @click="edit(item)"><RemoteIcon name="edit" />编辑</button>
              <button v-if="['failed', 'interrupted'].includes(item.state)" type="button" :disabled="changing || !writable" @click="change(item, 'continue')"><RemoteIcon name="refresh" />继续此条</button>
              <button v-if="item.state === 'queued' && !item.immediate" type="button" :disabled="changing || !writable || ordered[0]?.id === item.id" aria-label="向前移动" @click="move(item, -1)"><RemoteIcon name="up" /></button>
              <button v-if="item.state === 'queued' && !item.immediate" type="button" :disabled="changing || !writable || ordered.at(-1)?.id === item.id" aria-label="向后移动" @click="move(item, 1)"><RemoteIcon name="down" /></button>
              <button type="button" class="is-danger" :disabled="changing || !writable" @click="change(item, 'cancel')"><RemoteIcon name="close" />取消</button>
            </div>
          </template>
        </li>
      </ol>
      <p v-if="!items.length" class="room-empty">这里的消息都已处理。</p>
      <template #footer><div class="chat-queue-footer"><span>{{ items.length }} 条消息</span><button type="button" class="room-secondary-button" :disabled="changing || !writable" @click="control"><RemoteIcon :name="snapshot?.paused ? 'play' : 'pause'" />{{ snapshot?.paused ? '继续发送' : '暂停排队' }}</button></div></template>
    </RoomDrawer>
  </aside>
</template>
<script setup>
import { computed, ref, watch, onMounted, onBeforeUnmount } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import { remoteFeedbackMessage } from '../../services/remoteFeedback.js'
import { remoteState, remoteViews, remoteExecutions, readRemote, performRemote, installQueuedLive } from '../../stores/remote'
import { currentToken, assertToken } from '../../stores/mode'
import { remoteStatus } from '../../utils/remoteMarkdown'
const props = defineProps({ threadId: { type: String, required: true }, agentId: { type: String, required: true }, visible: Boolean })
const emit = defineEmits(['updated', 'pending'])
const token = currentToken(), query = { kind: 'queue', id: props.threadId }
const snapshot = computed(() => remoteViews.get(JSON.stringify(query))?.data)
const online = computed(() => remoteState.connection === 'online')
const items = computed(() => (snapshot.value?.items || []).filter(item => ['queued', 'failed', 'interrupted'].includes(item.state)))
const ordered = computed(() => items.value.filter(item => item.state === 'queued' && !item.immediate))
const changing = ref(false), error = ref(''), editing = ref(null), edited = ref('')
const expanded = ref(false), selected = ref(null), discard = ref(false), editingRevision = ref(null), original = ref('')
const failed = computed(() => items.value.filter(item => ['failed', 'interrupted'].includes(item.state)).length)
const summary = computed(() => snapshot.value?.paused ? `${items.value.length} 条消息已暂停` : failed.value ? `${failed.value} 条消息待处理` : `${items.value.length} 条待发送`)
function select(item, force = false) { if (editing.value && editing.value !== item.id && dirty.value) { error.value = '先保存或放弃当前编辑，再查看另一条消息。'; return }; selected.value = !force && selected.value === item.id && editing.value !== item.id ? null : item.id }
const dirty = computed(() => !!editing.value && edited.value !== original.value)
const writable = computed(() => online.value && !remoteViews.get(JSON.stringify(query))?.cached)
function beforeClose() { if (dirty.value) { discard.value = true; return false }; return true }
function prepareLeave() { if (changing.value) throw new Error('队列操作还在进行，请稍等。'); if (dirty.value) { expanded.value = true; discard.value = true; throw new Error('队列里还有未保存的修改。') } }
defineExpose({ prepareLeave })
function close() { expanded.value = false; editing.value = null; discard.value = false }
function edit(item) { if (editing.value && editing.value !== item.id && dirty.value) { error.value = '先保存或放弃当前编辑，再修改另一条消息。'; return }; selected.value = item.id; editing.value = item.id; edited.value = original.value = item.input?.content || ''; editingRevision.value = item.revision }
function reference(item) { return item.input?.workspaceContext?.filePath || (item.input?.workspaceContext?.terminalId ? '终端引用' : '') }
let disposed = false, loading = false, timer = null, previousRun = ''
function guard() { assertToken(token, 'remote'); if (disposed) throw new Error('队列面板已关闭') }
async function refresh() {
  if (loading || disposed || document.hidden || !props.visible) return
  loading = true
  try {
    const next = await readRemote(query); guard(); error.value = ''
    if (remoteViews.get(JSON.stringify(query))?.cached) return
    if (!Array.isArray(next?.items)) return
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
  if (changing.value || !writable.value) return
  changing.value = true; error.value = ''
  try { guard(); await performRemote(action); guard(); editing.value = null; await refresh(); emit('updated') }
  catch (failure) { if (!disposed) { await refresh(); if (!disposed) error.value = failure.message } }
  finally { if (!disposed) changing.value = false }
}
const control = () => mutate({ kind: 'queue_control', threadId: props.threadId, paused: !snapshot.value?.paused })
const change = (item, action, content = null) => mutate({ kind: 'queue_item', threadId: props.threadId, id: item.id, revision: editing.value === item.id && action === 'edit' ? editingRevision.value : item.revision, action, content })
function move(item, offset) {
  const order = ordered.value.map(({ id, revision }) => ({ id, revision })), index = order.findIndex(value => value.id === item.id), target = index + offset
  if (index < 0 || target < 0 || target >= order.length) return
  ;[order[index], order[target]] = [order[target], order[index]]
  return mutate({ kind: 'queue_order', threadId: props.threadId, order })
}
async function poll() { await refresh(); if (!disposed) timer = setTimeout(poll, document.hidden || !props.visible ? 10000 : 2500) }
const wake = () => { if (!document.hidden) refresh() }
watch(() => props.visible, value => { if (value) refresh(); else close() })
watch(items, values => { if (selected.value && !values.some(item => item.id === selected.value) && !editing.value) selected.value = null })
onMounted(() => { poll(); document.addEventListener('visibilitychange', wake); window.addEventListener('online', wake) })
onBeforeUnmount(() => { disposed = true; clearTimeout(timer); document.removeEventListener('visibilitychange', wake); window.removeEventListener('online', wake) })
</script>
<style scoped>
.chat-queue-summary{display:flex;flex:none;min-width:0;padding:0 4px 6px}.chat-queue-chip{display:inline-flex;align-items:center;gap:7px;min-height:34px;max-width:min(100%,280px);padding:5px 11px;border:1px solid var(--room-line);border-radius:10px 10px 4px 10px;background:var(--room-surface);color:var(--room-accent);box-shadow:0 -3px 0 -1px var(--room-subtle);font:inherit;font-size:12px}.chat-queue-chip span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.chat-queue-chip svg{width:15px;height:15px;flex:none}.chat-queue-chip>svg:last-child{width:11px;height:11px;margin-left:4px;opacity:.65}.chat-queue-chip[data-attention=true]{color:var(--danger,#ad6971)}
.chat-queue-list{list-style:none;margin:0;padding:0;display:grid;gap:8px}.chat-queue-list>li{border:1px solid var(--room-line);border-radius:13px;background:var(--room-surface);overflow:hidden}.chat-queue-list>li.is-selected{border-color:color-mix(in srgb,var(--room-accent) 42%,var(--room-line));background:var(--room-subtle)}.chat-queue-row{display:flex;align-items:flex-start;gap:9px;padding:10px 10px 10px 12px}.chat-queue-number{padding-top:10px;color:var(--room-accent);font:11px/1.4 ui-monospace,monospace}.chat-queue-preview{flex:1;min-width:0;padding:4px 0;border:0;background:transparent;text-align:left;color:var(--room-ink);font:inherit}.chat-queue-state{display:block;margin-bottom:5px;color:var(--room-muted);font-size:11px}.chat-queue-state[data-failed=true]{color:var(--danger,#ad6971)}.chat-queue-copy{display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden;overflow-wrap:anywhere;font-size:14px;line-height:1.65}.is-selected .chat-queue-copy{display:block;white-space:pre-wrap;max-height:12em;overflow:auto}.chat-queue-preview small{display:flex;align-items:center;gap:4px;margin-top:5px;max-width:100%;overflow:hidden;text-overflow:ellipsis;color:var(--room-muted);font-size:11px}.chat-queue-preview small svg{width:13px;height:13px;flex:none}.chat-queue-more{display:grid;place-items:center;min-width:40px;height:40px;padding:10px;border:0;background:transparent;color:var(--room-muted)}.chat-queue-more svg{width:18px;height:18px}
.chat-queue-item-actions{display:flex;align-items:center;flex-wrap:wrap;gap:3px;margin:0 12px 9px 30px;border-top:1px solid var(--room-line);padding-top:6px}.chat-queue-item-actions button{display:inline-flex;align-items:center;gap:5px;min-height:42px;min-width:40px;padding:7px 9px;border:0;border-radius:8px;background:transparent;color:var(--room-accent);font:inherit;font-size:12px}.chat-queue-item-actions svg{width:16px;height:16px}.chat-queue-item-actions button.is-danger{margin-left:auto;color:var(--danger,#ad6971)}.chat-queue-list button:disabled{opacity:.4}.chat-queue-item-error{margin:0 14px 10px 30px;color:var(--danger,#ad6971);font-size:12px;line-height:1.6}
.chat-queue-editor,.chat-queue-orphan{padding:0 12px 12px}.chat-queue-editor textarea,.chat-queue-orphan textarea{box-sizing:border-box;width:100%;padding:10px 12px;border:1px solid var(--room-line);border-radius:10px;background:var(--room-surface);color:var(--room-ink);font:inherit;font-size:14px;line-height:1.7;resize:vertical}.chat-queue-editor>div,.chat-queue-footer{display:flex;align-items:center;justify-content:space-between;gap:10px;margin-top:8px}.chat-queue-editor button,.chat-queue-discard button{min-height:40px;border:0;border-radius:8px;padding:8px 12px;background:var(--room-accent-soft);color:var(--room-accent);font:inherit;font-size:12px}.chat-queue-editor button[type=submit]{background:var(--room-accent);color:var(--room-surface)}.chat-queue-footer{margin:0;color:var(--room-muted);font-size:12px}.chat-queue-footer button{display:inline-flex;align-items:center;gap:6px}.chat-queue-footer svg{width:16px;height:16px}.chat-queue-discard{margin:0 0 12px;padding:12px;border:1px solid var(--room-line);border-radius:12px;font-size:13px}.chat-queue-discard p{margin:0 0 8px}.chat-queue-discard button+button{margin-left:8px}
</style>
