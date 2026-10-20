<template>
  <section class="remote-collection" :class="{ 'file-library': kind === 'library' && source === 'files' }">
    <header class="remote-section-heading"><div><span class="remote-eyebrow">{{ eyebrow }}</span><h1>{{ title }}</h1></div><button v-if="kind === 'tasks'" class="room-icon-button" aria-label="新建远程任务" :disabled="remoteState.connection !== 'online' || !agentId" @click="$emit('create-task')"><RoomIcon name="plus" /></button><button class="room-icon-button" aria-label="刷新列表" :disabled="loading" @click="reload"><RoomIcon name="refresh" /></button></header>
    <div v-if="kind === 'tasks'" class="remote-task-filter"><span>任务状态</span><ChoicePicker v-model="taskStatus" label="任务状态" :options="taskStatuses" /></div>
    <template v-if="kind === 'library'"><div class="remote-segments"><button v-for="item in sources" :key="item.id" :aria-pressed="source === item.id" :class="{ selected: source === item.id }" @click="source = item.id">{{ item.label }}</button></div><RemoteFilePath v-if="source === 'files'" :path="path" :disabled="loading" @navigate="navigateFiles" /><form class="remote-search" @submit.prevent="search"><RoomIcon name="search" /><input v-model="searchText" :placeholder="source === 'files' ? '在文件夹里找找…' : '搜索当前连接中的资料'" aria-label="搜索资料" maxlength="200" /><button type="submit">搜索</button></form></template>
    <p v-if="kind === 'inbox'" class="remote-help">缓存最近 256 条待阅通知；明确标为已读后才发送回执。Android 可在连接设置开启原生后台接收。审批、问题与待核对请求仍以服务端状态为准。</p>
    <template v-if="kind === 'inbox'"><p v-if="remoteState.notificationError" class="room-error">{{ remoteState.notificationError }}</p><article v-for="notice in notices" :id="`remote-notice-${JSON.stringify([notice.principalId, notice.notificationId])}`" :key="`${notice.principalId}:${notice.notificationId}:${notice.revision}`" class="remote-input-card"><span class="remote-eyebrow">{{ notice.principalId }} · {{ notice.replayed ? '历史通知' : '待阅通知' }}</span><h3>{{ notice.title || 'infOS 通知' }}</h3><p class="remote-notice-body">{{ notice.body }}</p><button class="room-text-button" :disabled="acknowledging || remoteState.connection !== 'online'" @click="acknowledge(notice)">标为已读</button></article></template>
    <p v-if="error" class="room-error" role="alert">{{ error }}</p>
    <p v-if="cached" class="remote-cache-note">正在显示缓存，内容可能已经更新。</p>
    <p v-if="kind === 'library' && source === 'files' && loading && rows.length" class="file-opening-note" role="status">正在读取文件…</p>
    <p v-if="loading && !rows.length" class="room-empty">正在读取…</p>
    <p v-else-if="!rows.length && (kind !== 'inbox' || !notices.length)" class="remote-empty"><RoomIcon :name="kind === 'inbox' ? 'check' : 'document'" /><strong>{{ kind === 'inbox' ? '暂时没有需要处理的事项' : '这里还没有内容' }}</strong><span>{{ kind === 'tasks' ? '角色执行的任务会出现在这里。' : kind === 'library' ? '可以换个关键词，或查看其他分类。' : '新的审批和问题会在这里汇合。' }}</span></p>
    <div class="remote-record-list"><button v-for="row in rows" :key="`${row.category}:${row.id || row.requestId || row.path}`" class="remote-record" @click="open(row)"><RemoteFileCover v-if="row.category === 'files' && row.type !== 'directory'" :name="row.name" /><span v-else class="remote-record-symbol"><RoomIcon :name="row.category === 'pending' ? 'help' : row.category === 'approval' ? 'sliders' : row.type === 'directory' ? 'wardrobe' : kind === 'tasks' ? 'model' : 'document'" /></span><span><small>{{ category(row) }}</small><strong>{{ row.title || row.question || row.name || row.summary || row.event?.summary || '查看详情' }}</strong><em v-if="row.instruction || row.description || row.summary">{{ row.instruction || row.description || row.summary }}</em><small v-if="row.status">{{ remoteStatus(row.status) }}</small></span><RoomIcon name="arrow" /></button></div>
    <div v-if="kind !== 'inbox' && (kind === 'tasks' || source !== 'files')" class="remote-pagination"><button :disabled="loading || page <= 1" @click="page--">上一页</button><span>第 {{ page }} 页</span><button :disabled="loading || !hasMore" @click="page++">下一页</button></div>
  </section>
</template>
<script setup>
import { managementOperationLabel } from '../../services/remoteManagement'
import { computed, ref, watch, onBeforeUnmount } from 'vue'
import RoomIcon from './RemoteIcon.vue'
import RemoteFilePath from './RemoteFilePath.vue'
import '../../styles/remote-files.css'
import ChoicePicker from '../companion/ChoicePicker.vue'
import RemoteFileCover from './RemoteFileCover.vue'
import { readRemote, remoteViews, remotePendingRequests, refreshRemotePendingRequests, remoteNotifications, remoteState, refreshRemoteNotifications, acknowledgeRemoteNotification } from '../../stores/remote'
import { currentToken, assertToken } from '../../stores/mode'
import { remoteItems, remoteStatus } from '../../utils/remoteMarkdown'
const props = defineProps({ kind: String, agentId: String, threadId: String })
const emit = defineEmits(['detail', 'thread', 'pending', 'create-task', 'resource'])
const token = currentToken()
const page = ref(1), source = ref('files'), path = ref('.'), searchText = ref(''), keyword = ref(''), loading = ref(false), error = ref(''), queries = ref([]), taskStatus = ref(null)
const acknowledging = ref(false)
const notices = computed(() => [...remoteNotifications.values()])
const sources = [{ id: 'files', label: '文件' }, { id: 'memories', label: '记忆' }, { id: 'documents', label: '文档' }]
const taskStatuses = [{ value: null, label: '全部' }, { value: 'running', label: '执行中' }, { value: 'waiting_input', label: '等待回复' }, { value: 'paused', label: '已暂停' }, { value: 'completed', label: '已完成' }, { value: 'failed', label: '执行失败' }]
let version = 0, disposed = false
const title = computed(() => ({ tasks: '任务', inbox: '通知', library: '资料' }[props.kind]))
const eyebrow = computed(() => ({ tasks: '把进展留在身边', inbox: '等你看一眼', library: '一起积累的内容' }[props.kind]))
const views = computed(() => queries.value.map(query => remoteViews.get(JSON.stringify(query))))
const cached = computed(() => views.value.some(view => view?.cached))
const rows = computed(() => {
  const values = views.value.flatMap((view, index) => remoteItems(view?.data?.nodes || view?.data).map(item => ({ ...item, category: props.kind === 'inbox' ? index === 0 ? 'approval' : 'question' : props.kind === 'tasks' ? 'task' : source.value }))).filter(item => props.kind !== 'library' || source.value !== 'files' || `${item.name || ''} ${item.path || ''}`.toLowerCase().includes(keyword.value.toLowerCase()))
  return props.kind === 'inbox' ? [...[...remotePendingRequests.values()].filter(item => item.state === 'unknown').map(item => ({ ...item, id: item.requestId, category: 'pending', title: '发送结果待核对', summary: item.context?.input?.content || item.context?.input?.instruction || managementOperationLabel(item.context?.operation) || '请查看服务端状态后再确认' })), ...values] : values
})
const hasMore = computed(() => { const data = views.value[0]?.data; return typeof data?.total === 'number' ? page.value * 30 < data.total : rows.value.length >= 30 })
function guard() { assertToken(token, 'remote'); if (disposed) throw new Error('页面已关闭') }
async function reload() {
  const request = ++version; loading.value = true; error.value = ''
  try {
    guard()
    const next = props.kind === 'tasks' ? [{ kind: 'tasks', agentId: props.agentId || null, status: taskStatus.value, page: page.value, pageSize: 30 }]
      : props.kind === 'inbox' ? [{ kind: 'approvals' }, { kind: 'questions' }]
        : source.value === 'files' ? props.threadId && props.agentId ? [{ kind: 'workspace_tree', agentId: props.agentId, threadId: props.threadId, path: path.value }] : []
          : source.value === 'memories' ? props.agentId ? [{ kind: 'memories', agentId: props.agentId, query: keyword.value, page: page.value, pageSize: 30 }] : []
            : [{ kind: 'documents', query: keyword.value, page: page.value, pageSize: 30 }]
    queries.value = next
    const inboxResults = props.kind === 'inbox' ? await Promise.allSettled([refreshRemotePendingRequests(), refreshRemoteNotifications()]) : []
    guard()
    const results = await Promise.allSettled(next.map(query => readRemote(query)))
    guard()
    if (request !== version) return
    const failure = [...inboxResults, ...results].find(result => result.status === 'rejected')
    if (failure) error.value = failure.reason.message
    if (source.value === 'files' && props.kind === 'library' && !next.length) error.value = '先选择一条会话，再查看它的文件。'
  } catch (failure) { if (!disposed && request === version) error.value = failure.message } finally { if (!disposed && request === version) loading.value = false }
}
function search() { keyword.value = searchText.value.trim(); if (page.value !== 1) page.value = 1; else reload() }
function navigateFiles(target) { searchText.value = ''; keyword.value = ''; path.value = target }
async function acknowledge(notice) {
  if (acknowledging.value) return
  acknowledging.value = true; error.value = ''
  try { guard(); await acknowledgeRemoteNotification(notice); guard() }
  catch (failure) { if (!disposed) error.value = failure.message }
  finally { if (!disposed) acknowledging.value = false }
}
function category(row) { return ({ approval: '等待审批', question: '等待回复', pending: '待核对', memories: '事件记忆', documents: '文档资料', files: row.type === 'directory' ? '文件夹' : '会话文件' }[row.category] || '远程任务') }
async function open(row) {
  if (row.category === 'pending') { emit('pending', row); return }
  if (props.kind === 'tasks' || row.taskId) { emit('detail', { title: row.title || '任务详情', query: { kind: 'task_projection', id: row.taskId || row.id }, taskId: row.taskId || row.id }); return }
  if (props.kind === 'inbox') { emit('thread', { threadId: row.threadId, agentId: row.agentId }); return }
  if (source.value === 'files') {
    if (row.type === 'directory') { navigateFiles(row.path); return }
    if (loading.value) return
    const request = version, query = { kind: 'workspace_file', agentId: props.agentId, threadId: props.threadId, path: row.path }
    loading.value = true; error.value = ''
    try {
      const file = await readRemote(query, { allowCached: false }); guard()
      if (request !== version) return
      if (!file?.version) throw new Error('文件版本暂不可用，请刷新后重试')
      emit('resource', { kind: 'workspace', agentId: query.agentId, threadId: query.threadId, path: query.path, version: file.version, name: file.name || row.name })
    } catch (failure) { if (!disposed && request === version) error.value = failure.message }
    finally { if (!disposed && request === version) loading.value = false }
    return
  }
  emit('detail', { title: row.title || row.summary || '记忆来源', query: { kind: source.value === 'memories' ? 'memory_source' : 'document', id: row.id }, summary: row.summary || row.event?.summary })
}
watch(() => [props.kind, props.agentId, props.threadId], () => { page.value = 1; path.value = '.'; reload() }, { immediate: true })
watch([page, path], reload)
watch(source, () => { page.value = 1; path.value = '.'; searchText.value = ''; keyword.value = ''; reload() })
watch(taskStatus, () => { if (page.value !== 1) page.value = 1; else reload() })
onBeforeUnmount(() => { disposed = true; version++ })
</script>
