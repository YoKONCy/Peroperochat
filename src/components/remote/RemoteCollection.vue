<template>
  <section class="remote-collection" :class="{ 'file-library': kind === 'library' && source === 'files' }">
    <header class="remote-section-heading"><div><span class="remote-eyebrow">{{ t(eyebrow) }}</span><h1>{{ t(title) }}</h1></div><button v-if="kind === 'library'" type="button" class="library-manage-button" :disabled="source === 'files' && (!threadId || !agentId)" @click="$emit('manage', managementEntry.section)"><RoomIcon :name="managementEntry.icon" />{{ t(managementEntry.label) }}</button><button v-if="kind === 'tasks'" class="room-icon-button" aria-label="新建远程任务" :disabled="remoteState.connection !== 'online' || !agentId" @click="$emit('create-task')"><RoomIcon name="plus" /></button><button class="room-icon-button" aria-label="刷新列表" :disabled="loading" @click="reload"><RoomIcon name="refresh" /></button></header>
    <template v-if="kind === 'tasks'">
      <section class="chromatic-task-overview" data-ui-tone="mint"><div class="chromatic-task-count"><strong>{{ runningCount }}</strong><small>{{ t('执行中') }}</small></div><div><h2>{{ t('小事，一件件做好。') }}</h2><p>{{ t('当前列表，共 {0} 项', [rows.length]) }}</p></div></section>
      <nav class="chromatic-task-filters" :aria-label="t('任务状态')"><button v-for="item in taskStatuses" :key="item.value || 'all'" :class="{ selected: taskStatus === item.value }" :aria-pressed="taskStatus === item.value" @click="taskStatus = item.value">{{ t(item.label) }}</button></nav>
    </template>
    <template v-if="kind === 'library'">
      <form class="remote-search chromatic-library-search" @submit.prevent="search"><RoomIcon name="search" /><input v-model="searchText" :placeholder="t(source === 'files' ? '在文件夹里找找…' : '搜索当前连接中的资料')" :aria-label="t('搜索资料')" maxlength="200" /><button type="submit">{{ t('搜索') }}</button></form>
      <nav class="chromatic-library-categories" :aria-label="t('资料分类')"><button v-for="item in sources" :key="item.id" :data-ui-tone="item.tone" :aria-pressed="source === item.id" :class="{ selected: source === item.id }" @click="source = item.id"><RoomIcon :name="item.icon" /><strong>{{ t(item.label) }}</strong><small>{{ t(item.description) }}</small></button></nav>
      <RemoteFilePath v-if="source === 'files'" :path="path" :disabled="loading" @navigate="navigateFiles" />
    </template>
    <template v-if="kind === 'inbox'"><RemoteFeedback v-if="active" :message="remoteState.notificationError" /><article v-for="notice in notices" :id="`remote-notice-${JSON.stringify([notice.principalId, notice.notificationId])}`" :key="`${notice.principalId}:${notice.notificationId}:${notice.revision}`" class="remote-input-card"><span class="remote-eyebrow">{{ notice.replayed ? '较早的提醒' : '新的提醒' }}</span><h3>{{ notice.title || 'infOS 通知' }}</h3><p class="remote-notice-body">{{ notice.body }}</p><button class="room-text-button" :disabled="acknowledging || remoteState.connection !== 'online'" @click="acknowledge(notice)">标为已读</button></article></template>
    <RemoteFeedback v-if="active" :message="error" />
    <p v-if="cached && remoteState.connection !== 'online'" class="remote-cache-note">离线中，显示上次保存的内容。</p>
    <p v-if="kind === 'library' && source === 'files' && loading && rows.length" class="file-opening-note" role="status">正在读取文件…</p>
    <p v-if="(loading || syncing) && !rows.length" class="room-empty">正在载入…</p>
    <p v-else-if="error && !rows.length && (kind !== 'inbox' || !notices.length)" class="remote-empty"><RoomIcon name="refresh" /><strong>暂时没能载入</strong><span>稍后点右上角刷新，再试一次。</span></p>
    <p v-else-if="!rows.length && (kind !== 'inbox' || !notices.length)" class="remote-empty"><RoomIcon :name="kind === 'inbox' ? 'check' : 'document'" /><strong>{{ kind === 'inbox' ? '暂时没有需要处理的事项' : '这里还没有内容' }}</strong><span>{{ kind === 'tasks' ? '角色执行的任务会出现在这里。' : kind === 'library' ? '可以换个关键词，或查看其他分类。' : '新的审批和问题会在这里汇合。' }}</span></p>
    <template v-if="featuredFile"><h3 class="chromatic-section-label">{{ t('文件预览') }}</h3><RemoteResourceCard v-if="active" :key="`${agentId}:${threadId}:${featuredFile.path}`" :value="{ source: `workspace:${encodeURIComponent(featuredFile.path)}`, name: featuredFile.name }" :context="{ agentId, threadId }" @open="$emit('resource', $event)" /></template>
    <div class="remote-record-list"><button v-for="row in displayRows" :key="`${row.category}:${row.id || row.requestId || row.path}`" class="remote-record" :class="{ 'chromatic-task-card': kind === 'tasks' }" :data-ui-tone="rowTone(row)" @click="open(row)"><RemoteFileCover v-if="row.category === 'files' && row.type !== 'directory'" :name="row.name" /><span v-else class="remote-record-symbol"><RoomIcon :name="row.category === 'pending' ? 'help' : row.category === 'approval' ? 'shield' : row.type === 'directory' ? 'folder' : kind === 'tasks' ? row.status === 'completed' ? 'check' : 'tasks' : source === 'memories' ? 'memory' : 'document'" /></span><span class="chromatic-record-copy"><small v-if="kind !== 'tasks'">{{ t(category(row)) }}</small><strong>{{ row.title || row.question || row.name || row.summary || row.event?.summary || t('查看详情') }}</strong><em v-if="recordSummary(row)">{{ recordSummary(row) }}</em><small v-if="row.status" class="chromatic-record-state">{{ t(remoteStatus(row.status)) }}</small></span><RoomIcon name="arrow" /></button></div>
    <div v-if="kind !== 'inbox' && (kind === 'tasks' || source !== 'files') && (page > 1 || hasMore)" class="remote-pagination"><button :disabled="loading || page <= 1" @click="page--">{{ t('上一页') }}</button><span>{{ t('第 {0} 页', [page]) }}</span><button :disabled="loading || !hasMore" @click="page++">{{ t('下一页') }}</button></div>
  </section>
</template>
<script setup>
import RemoteFeedback from './RemoteFeedback.vue'
import { managementOperationLabel } from '../../services/remoteManagement'
import { computed, ref, watch, onBeforeUnmount } from 'vue'
import RoomIcon from './RemoteIcon.vue'
import RemoteFilePath from './RemoteFilePath.vue'
import '../../styles/remote-files.css'
import { t } from '../../i18n/remote'
import RemoteFileCover from './RemoteFileCover.vue'
import RemoteResourceCard from './RemoteResourceCard.vue'
import { readRemote, remoteViews, remotePendingRequests, refreshRemotePendingRequests, remoteNotifications, remoteState, refreshRemoteNotifications, acknowledgeRemoteNotification } from '../../stores/remote'
import { currentToken, assertToken } from '../../stores/mode'
import { remoteItems, remoteStatus } from '../../utils/remoteMarkdown'
const props = defineProps({ kind: String, agentId: String, threadId: String, active: { type: Boolean, default: true } })
const emit = defineEmits(['detail', 'thread', 'pending', 'create-task', 'resource', 'manage'])
const token = currentToken()
const page = ref(1), source = ref('files'), path = ref('.'), searchText = ref(''), keyword = ref(''), loading = ref(false), error = ref(''), queries = ref([]), taskStatus = ref(null)
const acknowledging = ref(false)
const notices = computed(() => [...remoteNotifications.values()])
const sources = [{ id: 'files', label: '文件', icon: 'folder', tone: 'blue', description: '随时打开' }, { id: 'memories', label: '记忆', icon: 'memory', tone: 'violet', description: '一起积累' }, { id: 'documents', label: '文档', icon: 'document', tone: 'amber', description: '随时翻一翻' }]
const managementEntry = computed(() => ({ files: { section: 'workbench', label: '文件管理', icon: 'folder' }, memories: { section: 'memories', label: '记忆管理', icon: 'memory' }, documents: { section: 'documents', label: '文档管理', icon: 'edit' } })[source.value])
const taskStatuses = [{ value: null, label: '全部' }, { value: 'running', label: '执行中' }, { value: 'waiting_input', label: '等待回复' }, { value: 'paused', label: '已暂停' }, { value: 'completed', label: '已完成' }, { value: 'failed', label: '执行失败' }]
let version = 0, disposed = false
const title = computed(() => ({ tasks: '任务', inbox: '通知', library: '资料' }[props.kind]))
const eyebrow = computed(() => ({ tasks: '把进展留在身边', inbox: '等你看一眼', library: '一起积累的内容' }[props.kind]))
const views = computed(() => queries.value.map(query => remoteViews.get(JSON.stringify(query))))
const syncing = computed(() => views.value.some(view => view?.loading || view?.syncing))
const cached = computed(() => views.value.some(view => view?.cached))
const rows = computed(() => {
  const values = views.value.flatMap((view, index) => remoteItems(view?.data?.nodes || view?.data).map(item => ({ ...item, category: props.kind === 'inbox' ? index === 0 ? 'approval' : 'question' : props.kind === 'tasks' ? 'task' : source.value }))).filter(item => props.kind !== 'library' || source.value !== 'files' || `${item.name || ''} ${item.path || ''}`.toLowerCase().includes(keyword.value.toLowerCase()))
  return props.kind === 'inbox' ? [...[...remotePendingRequests.values()].filter(item => item.state === 'unknown').map(item => ({ ...item, id: item.requestId, category: 'pending', title: '有一项操作需要确认', summary: item.context?.input?.content || item.context?.input?.instruction || managementOperationLabel(item.context?.operation) || '查看结果，避免重复发送。' })), ...values] : values
})
const hasMore = computed(() => { const data = views.value[0]?.data; return typeof data?.total === 'number' ? page.value * 30 < data.total : rows.value.length >= 30 })
const runningCount = computed(() => rows.value.filter(row => ['running', 'dispatching'].includes(row.status)).length)
const featuredFile = computed(() => props.kind === 'library' && source.value === 'files' ? rows.value.find(row => row.type !== 'directory' && typeof row.path === 'string' && row.path.length) : null)
const displayRows = computed(() => featuredFile.value && props.active ? rows.value.filter(row => row.path !== featuredFile.value.path) : rows.value)
function rowTone(row) {
  if (props.kind === 'tasks') return ['waiting_input', 'paused', 'blocked'].includes(row.status) ? 'amber' : row.status === 'failed' ? 'danger' : 'mint'
  return ({ memories: 'violet', documents: 'amber', approval: 'amber', question: 'rose', pending: 'amber' })[row.category] || (row.type === 'directory' ? 'amber' : 'blue')
}
function recordSummary(row) {
  const summary = row.instruction || row.description || row.summary
  return typeof summary === 'string' && summary !== (row.title || row.question || row.name || row.summary || row.event?.summary) ? summary : ''
}
function guard() { assertToken(token, 'remote'); if (disposed) throw new Error('页面已关闭') }
async function reload() {
  if (!props.active) return
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
      const file = await readRemote(query); guard()
      if (request !== version) return
      if (!file?.version) throw new Error('文件版本暂不可用，请刷新后重试')
      emit('resource', { kind: 'workspace', agentId: query.agentId, threadId: query.threadId, path: query.path, version: file.version, name: file.name || row.name })
    } catch (failure) { if (!disposed && request === version) error.value = failure.message }
    finally { if (!disposed && request === version) loading.value = false }
    return
  }
  emit('detail', { title: source.value === 'memories' ? '记忆' : row.title || row.summary || '文档', query: { kind: source.value === 'memories' ? 'memory_source' : 'document', id: row.id }, summary: row.summary || row.event?.summary })
}
watch(() => [props.kind, props.agentId, props.threadId], () => { queries.value = []; page.value = 1; path.value = '.'; reload() }, { immediate: true })
watch(() => props.active, value => { if (value) reload(); else { version++; loading.value = false; error.value = '' } })
watch([page, path], reload)
watch(source, () => { page.value = 1; path.value = '.'; searchText.value = ''; keyword.value = ''; reload() })
watch(taskStatus, () => { if (page.value !== 1) page.value = 1; else reload() })
watch(() => remoteState.connection, (state, previous) => { if (state === 'online' && previous !== 'online') reload() })
onBeforeUnmount(() => { disposed = true; version++ })
</script>
