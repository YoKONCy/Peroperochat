<template>
  <RoomDrawer :title="detail.title || '详情'" subtitle="来自 infOS 的当前状态" variant="editor" @close="$emit('close')"><div class="remote-detail" @click="openLink">
    <div class="remote-detail-toolbar"><button class="room-text-button" :disabled="loading || busy" @click="reload"><RoomIcon name="refresh" />刷新</button><span v-if="view?.cached" class="remote-help">上次保存</span></div>
    <RemoteFeedback :message="error || view?.error" /><p v-if="loading && !data" class="room-empty">正在读取…</p>
    <div v-if="detail.summary" class="markdown-body" v-html="remoteMarkdown(detail.summary)" />
    <RemoteTaskPlan v-if="task?.plan" :plan="task.plan" />
    <RemoteSurface v-for="surface in data?.surfaces || []" :key="surface.surfaceId" :surface="surface" :disabled="remoteState.connection !== 'online'" @updated="reload" @resource="$emit('resource', $event)" />
    <template v-if="detail.query.kind === 'plan'"><RemoteTaskPlan v-if="data" :plan="data" /><p v-else class="remote-help">此会话暂无行动清单。</p></template>
    <template v-if="detail.query.kind === 'document' && data"><h2>{{ data.title }}</h2><p class="remote-help">{{ remoteStatus(data.status) }} · {{ data.partCount }} 个片段</p><RemoteFeedback :message="data.error" /><section v-for="part in data.parts || []" :key="part.id" class="remote-document-part"><h3>{{ part.title || `片段 ${part.ordinal + 1}` }}</h3><div class="markdown-body" v-html="remoteMarkdown(part.summary)" /></section></template>
    <template v-if="detail.query.kind === 'memory_source' && data"><p v-if="data.available === false" class="remote-help">原始会话内容已不可用。</p><article v-for="message in data.messages || []" :key="message.id" class="remote-source-message"><small>{{ message.role === 'user' ? '你' : '角色' }}</small><div class="markdown-body" v-html="remoteMarkdown(message.content)" /></article></template>
    <template v-if="detail.query.kind === 'workspace_file' && data"><dl class="remote-metadata"><dt>文件</dt><dd>{{ data.name || detail.title }}</dd><dt>大小</dt><dd>{{ data.sizeBytes ?? data.size }} 字节</dd></dl><button class="room-primary-button" @click="openFile">查看文件</button></template>
    <div v-if="detail.taskId && task" class="remote-task-controls"><span class="remote-eyebrow">任务操作</span><button v-if="['running','queued'].includes(task.status)" :disabled="busy || !online" @click="control('pause')">暂停</button><button v-if="task.status === 'paused'" :disabled="busy || !online" @click="control('resume')">继续</button><button v-if="task.status === 'interrupted'" :disabled="busy || !online" @click="control('resume-interrupted')">继续中断的任务</button><button v-if="task.status === 'failed'" :disabled="busy || !online" @click="control('retry')">重试</button><button v-if="!['completed','failed','cancelled'].includes(task.status)" :disabled="busy || !online" @click="confirmCancel = true">取消任务</button><div v-if="confirmCancel" class="remote-inline-confirm"><span>确认取消这个任务？</span><button :disabled="busy" @click="control('cancel')">确认取消</button><button @click="confirmCancel = false">继续保留</button></div><button v-if="task.threadId" @click="$emit('thread', { threadId: task.threadId, agentId: task.agentId })">打开关联会话</button></div>
  </div></RoomDrawer>
</template>
<script setup>
import RemoteFeedback from './RemoteFeedback.vue'
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RoomIcon from './RemoteIcon.vue'
import RemoteSurface from './RemoteSurface.vue'
import RemoteTaskPlan from './RemoteTaskPlan.vue'
import { readRemote, remoteViews, remoteState, performRemote } from '../../stores/remote'
import { currentToken, assertToken } from '../../stores/mode'
import { remoteMarkdown, remoteStatus, openRemoteLink } from '../../utils/remoteMarkdown'
const props = defineProps({ detail: Object })
const emit = defineEmits(['close', 'resource', 'thread'])
const token = currentToken(), loading = ref(false), error = ref(''), busy = ref(false), confirmCancel = ref(false)
const view = computed(() => remoteViews.get(JSON.stringify(props.detail.query))), data = computed(() => view.value?.data)
const task = computed(() => props.detail.taskId ? remoteViews.get(JSON.stringify({ kind: 'task', id: props.detail.taskId }))?.data : null)
const online = computed(() => remoteState.connection === 'online')
let disposed = false
async function reload() {
  loading.value = true; error.value = ''
  try { assertToken(token, 'remote'); await readRemote(props.detail.query); assertToken(token, 'remote'); if (props.detail.taskId && !disposed) await readRemote({ kind: 'task', id: props.detail.taskId }) }
  catch (failure) { if (!disposed) error.value = failure.message } finally { if (!disposed) loading.value = false }
}
async function control(action) {
  if (busy.value) return
  busy.value = true; error.value = ''
  try { assertToken(token, 'remote'); await performRemote({ kind: 'task_control', id: props.detail.taskId, action }); assertToken(token, 'remote'); if (!disposed) { confirmCancel.value = false; await reload() } }
  catch (failure) { if (!disposed) error.value = failure.message } finally { if (!disposed) busy.value = false }
}
function openFile() { if (!data.value?.version) { error.value = '服务端未提供文件版本，请刷新后重试'; return }; emit('resource', { kind: 'workspace', agentId: props.detail.query.agentId, threadId: props.detail.query.threadId, path: props.detail.query.path, version: data.value.version, name: data.value.name || props.detail.title }) }
async function openLink(event) { try { await openRemoteLink(event) } catch (failure) { if (!disposed) error.value = failure.message } }
onMounted(reload)
onBeforeUnmount(() => { disposed = true })
</script>
