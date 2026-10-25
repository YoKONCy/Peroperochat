<template>
  <RemoteTaskDetail v-if="taskId" ref="taskReader" :task-id="taskId" @close="$emit('close')" @resource="$emit('resource', $event)" @thread="$emit('thread', $event)" @pending="$emit('pending')" @detail="$emit('detail', $event)" />
  <RemoteMemoryDetail ref="memoryReader" v-else-if="detail.query.kind === 'memory_source'" :key="detail.query.id" :detail="detail" @close="$emit('close')" @resource="$emit('resource', $event)" @thread="$emit('thread', $event)" @pending="$emit('pending')" />
  <RoomDrawer v-else :title="detail.title || '详情'" variant="editor" @close="$emit('close')"><div class="remote-detail" @click="openLink">
    <div class="remote-detail-toolbar"><button class="room-text-button" :disabled="loading" @click="reload"><RoomIcon name="refresh" />刷新</button><span v-if="view?.cached" class="remote-help">上次保存</span></div>
    <RemoteFeedback :message="error || view?.error" /><p v-if="loading && !data" class="room-empty">正在读取…</p>
    <div v-if="detail.summary" class="markdown-body" v-html="remoteMarkdown(detail.summary)" />
    <RemoteSurface v-for="surface in data?.surfaces || []" :key="surface.surfaceId" :surface="surface" :disabled="remoteState.connection !== 'online'" @updated="reload" @resource="$emit('resource', $event)" />
    <template v-if="detail.query.kind === 'plan'"><RemoteTaskPlan v-if="data" :plan="data" /><p v-else class="remote-help">此会话暂无行动清单。</p></template>
    <template v-if="detail.query.kind === 'document' && data"><h2>{{ data.title }}</h2><p class="remote-help">{{ remoteStatus(data.status) }} · {{ data.partCount }} 个片段</p><RemoteFeedback :message="data.error" /><section v-for="part in data.parts || []" :key="part.id" class="remote-document-part"><h3>{{ part.title || `片段 ${part.ordinal + 1}` }}</h3><div class="markdown-body" v-html="remoteMarkdown(part.summary)" /></section></template>
    <template v-if="detail.query.kind === 'workspace_file' && data"><dl class="remote-metadata"><dt>文件</dt><dd>{{ data.name || detail.title }}</dd><dt>大小</dt><dd>{{ data.sizeBytes ?? data.size }} 字节</dd></dl><button class="room-primary-button" @click="openFile">查看文件</button></template>
  </div></RoomDrawer>
</template>
<script setup>
import RemoteFeedback from './RemoteFeedback.vue'
import RemoteMemoryDetail from './RemoteMemoryDetail.vue'
import RemoteTaskDetail from './RemoteTaskDetail.vue'
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RoomIcon from './RemoteIcon.vue'
import RemoteSurface from './RemoteSurface.vue'
import RemoteTaskPlan from './RemoteTaskPlan.vue'
import { readRemote, remoteViews, remoteState } from '../../stores/remote'
import { currentToken, assertToken } from '../../stores/mode'
import { remoteMarkdown, remoteStatus, openRemoteLink } from '../../utils/remoteMarkdown'
const memoryReader = ref(null), taskReader = ref(null)
const taskId = computed(() => props.detail.taskId || (props.detail.query.kind === 'task_projection' ? props.detail.query.id : null))
async function prepareLeave() { await memoryReader.value?.prepareLeave?.(); await taskReader.value?.prepareLeave?.() }
defineExpose({ prepareLeave })
const props = defineProps({ detail: Object })
const emit = defineEmits(['close', 'resource', 'thread', 'pending', 'detail'])
const token = currentToken(), loading = ref(false), error = ref('')
const view = computed(() => remoteViews.get(JSON.stringify(props.detail.query))), data = computed(() => view.value?.data)
let disposed = false
async function reload() {
  loading.value = true; error.value = ''
  try { assertToken(token, 'remote'); await readRemote(props.detail.query); assertToken(token, 'remote') }
  catch (failure) { if (!disposed) error.value = failure.message } finally { if (!disposed) loading.value = false }
}
function openFile() { if (!data.value?.version) { error.value = '服务端未提供文件版本，请刷新后重试'; return }; emit('resource', { kind: 'workspace', agentId: props.detail.query.agentId, threadId: props.detail.query.threadId, path: props.detail.query.path, version: data.value.version, name: data.value.name || props.detail.title }) }
async function openLink(event) { try { await openRemoteLink(event) } catch (failure) { if (!disposed) error.value = failure.message } }
onMounted(() => { if (!taskId.value && props.detail.query.kind !== 'memory_source') reload() })
onBeforeUnmount(() => { disposed = true })
</script>
