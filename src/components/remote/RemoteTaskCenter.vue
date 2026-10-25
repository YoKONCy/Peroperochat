<template>
  <section class="task-center">
    <header class="task-center-toolbar"><div><strong>{{ total }}</strong><span>{{ tr(view === 'history' ? '条历史记录' : view === 'resident' ? '项系统常驻' : '项进行中任务') }}</span></div><RemoteAction icon="plus" tone="primary" :disabled="!online" @click="$emit('create-task')">{{ tr('新任务') }}</RemoteAction><button type="button" class="room-icon-button" :disabled="loading" :aria-label="tr('刷新')" @click="reload"><RemoteIcon name="refresh" /></button></header>
    <nav class="task-center-tabs" :aria-label="tr('任务分类')"><button v-for="item in views" :key="item.id" type="button" :class="{ selected: view === item.id }" :aria-pressed="view === item.id" @click="view = item.id"><RemoteIcon :name="item.icon" />{{ tr(item.label) }}</button></nav>
    <form class="task-center-filters" @submit.prevent="keyword = search.trim(); reset()"><label class="remote-search"><RemoteIcon name="search" /><input v-model="search" :placeholder="tr('搜索任务')" maxlength="200" /><button type="submit">{{ tr('搜索') }}</button></label><RemoteSelect v-model="selectedAgent" :label="tr('任务角色')" :options="agentOptions" /><RemoteSelect v-model="status" :label="tr('任务状态')" :options="statusOptions" /></form>
    <details class="task-date-filter"><summary><RemoteIcon name="calendar" />{{ tr('按创建时间筛选') }}</summary><div><label class="remote-field">{{ tr('开始日期') }}<input v-model="from" type="date" :max="to || undefined" /></label><label class="remote-field">{{ tr('结束日期') }}<input v-model="to" type="date" :min="from || undefined" /></label><RemoteAction v-if="from || to" icon="close" tone="quiet" @click="from = ''; to = ''">{{ tr('清除日期') }}</RemoteAction></div></details>
    <p v-if="view === 'resident'" class="remote-help">{{ tr('系统常驻状态随服务自动维护，不需要手动派发或恢复。') }}</p>
    <div v-if="view === 'history'" class="task-history-toolbar"><span>{{ tr('已结束的任务会保留在这里。') }}</span><RemoteAction icon="check" tone="quiet" :disabled="!online || writing || unresolved" @click="markAllRead">{{ tr('全部标为已读') }}</RemoteAction></div>
    <RemoteFeedback v-if="active" :message="error || actionError" />
    <RemoteFeedback v-if="active && unresolved" :message="tr('操作结果待确认')" :action-label="tr('去核对')" @action="$emit('pending')" />
    <p v-if="loading && !items.length" class="room-empty">{{ tr('正在读取任务…') }}</p><p v-else-if="!items.length && !error" class="room-empty">{{ tr(view === 'history' ? '还没有历史任务' : view === 'resident' ? '暂无系统常驻状态' : '现在没有待处理的任务') }}</p>
    <div class="task-center-cards"><button v-for="task in items" :key="task.id" type="button" class="task-center-card" :data-state="task.status" @click="open(task)"><header><RemoteAvatar :id="task.agentId" :label="agentName(task.agentId)" /><span>{{ agentName(task.agentId) }}</span><i v-if="view === 'history' && !task.readAt" class="task-unread-dot" :aria-label="tr('未读')" /><strong>{{ tr(taskStatusLabels[task.status] || '状态待同步') }}</strong></header><h2>{{ task.title || tr('一项任务') }}</h2><p>{{ task.currentStage || task.lifecycle?.continuation || task.instruction }}</p><div v-if="taskProgress(task.progress) !== null" class="task-progress-line"><progress :value="task.progress" max="100" /><small>{{ task.progress }}%</small></div><footer><span>{{ taskTime(task.createdAt) }}</span><span>{{ tr(task.status === 'waiting_input' ? '查看并回复' : task.status === 'blocked' ? '查看原因' : '查看详情') }}<RemoteIcon name="next" /></span></footer></button></div>
    <RemoteAction v-if="view === 'active' && hasMore" icon="down" :disabled="loading" @click="more">{{ tr('加载更多') }}</RemoteAction>
    <div v-if="view !== 'active' && (page > 1 || hasMore)" class="remote-pagination"><button :disabled="loading || page === 1" @click="page--">{{ tr('上一页') }}</button><span>{{ page }}</span><button :disabled="loading || !hasMore" @click="page++">{{ tr('下一页') }}</button></div>
  </section>
</template>
<script setup>
import { computed, ref, watch, onMounted, onBeforeUnmount } from 'vue'
import RemoteAvatar from './RemoteAvatar.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteSelect from './RemoteSelect.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import { remoteState, remoteViews, readRemote } from '../../stores/remote'
import { currentToken, assertToken, sameToken } from '../../stores/mode'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { activeTaskStatuses, finishedTaskStatuses, taskStatusLabels, taskQueries, taskProgress, taskTime, taskDateBoundary, newestTasks } from '../../utils/taskCenter'
import { t } from '../../i18n/remote'
const props = defineProps({ active: Boolean })
const emit = defineEmits(['detail', 'pending', 'create-task'])
const { run, write, error: actionError, writing, unresolved } = useRemoteManagement()
const tr = (key, values) => t(key, values, 'tasks')
const token = currentToken(), view = ref('active'), selectedAgent = ref(''), status = ref(''), search = ref(''), keyword = ref(''), page = ref(1), queries = ref([]), loading = ref(false), error = ref('')
const online = computed(() => remoteState.connection === 'online')
const from = ref(''), to = ref('')
const views = [{ id: 'active', label: '进行中', icon: 'tasks' }, { id: 'history', label: '历史记录', icon: 'diary' }, { id: 'resident', label: '系统常驻', icon: 'settings' }]
const agentOptions = computed(() => [{ value: '', label: tr('全部角色') }, ...remoteState.agents.filter(agent => !agent.isUser).map(agent => ({ value: agent.id, label: agent.name }))])
const statusOptions = computed(() => [{ value: '', label: tr('全部状态') }, ...(view.value === 'active' ? activeTaskStatuses : view.value === 'history' ? finishedTaskStatuses : [...activeTaskStatuses, ...finishedTaskStatuses]).map(value => ({ value, label: tr(taskStatusLabels[value]) }))])
const snapshots = computed(() => queries.value.map(query => remoteViews.get(JSON.stringify(query))?.data))
const items = computed(() => newestTasks(snapshots.value.flatMap(data => data?.items || [])).filter(task => task.category === (view.value === 'resident' ? 'resident' : 'agent_task') && (view.value !== 'active' || activeTaskStatuses.includes(task.status)) && (view.value !== 'history' || finishedTaskStatuses.includes(task.status))))
const total = computed(() => view.value === 'active' ? queries.value.filter(query => query.page === 1).reduce((sum, query) => sum + (remoteViews.get(JSON.stringify(query))?.data?.total || 0), 0) : snapshots.value[0]?.total || 0)
const hasMore = computed(() => queries.value.some((query, index) => query.page * query.pageSize < (snapshots.value[index]?.total || 0) && (view.value !== 'active' || !queries.value.some(other => other.status === query.status && other.page === query.page + 1))))
let disposed = false, version = 0, timer = null, scheduled = null
function prepareLeave() { if (writing.value) throw new Error(tr('正在处理，请稍等。')) }
defineExpose({ prepareLeave })
function agentName(id) { return remoteState.agents.find(agent => agent.id === id)?.name || id }
function base() { return taskQueries({ view: view.value, agentId: selectedAgent.value, status: status.value, keyword: keyword.value, page: page.value, from: taskDateBoundary(from.value), to: taskDateBoundary(to.value, true) }) }
async function reload() {
  if (!props.active || document.hidden || writing.value || loading.value) return
  const request = ++version; loading.value = true; error.value = ''
  if (!queries.value.length) queries.value = base()
  const pending = [...queries.value], failures = []
  const workers = Array.from({ length: Math.min(3, pending.length) }, async () => {
    while (pending.length && !disposed && request === version && props.active && !document.hidden) {
      try { await readRemote(pending.shift()); assertToken(token, 'remote') } catch (failure) { failures.push(failure) }
    }
  })
  await Promise.all(workers)
  if (!disposed && request === version) { loading.value = false; error.value = failures[0]?.message || '' }
}
function reset() { version++; loading.value = false; queries.value = []; page.value = 1; reload() }
function more() {
  if (loading.value || view.value !== 'active') return
  const next = queries.value.filter((query, index) => query.page * query.pageSize < (snapshots.value[index]?.total || 0) && !queries.value.some(other => other.status === query.status && other.page === query.page + 1)).map(query => ({ ...query, page: query.page + 1 }))
  queries.value = [...queries.value, ...next]; reload()
}
function open(task) { emit('detail', { title: task.title || tr('任务详情'), query: { kind: 'task_projection', id: task.id }, taskId: task.id }) }
async function markAllRead() { await run(async () => { await write('tasks.read_all'); await reload() }) }
function openPendingResult() { if (!disposed && props.active && !document.hidden && !writing.value && remoteState.pendingTaskResult) { const task = remoteState.pendingTaskResult; remoteState.pendingTaskResult = null; open(task) } }
function poll() { clearTimeout(timer); if (disposed || !props.active) return; if (!document.hidden) { reload(); openPendingResult() }; timer = setTimeout(poll, 15000) }
function changed(event) { if (event.detail?.token && !sameToken(event.detail.token, token)) return; clearTimeout(scheduled); scheduled = setTimeout(reload, 150) }
watch(view, () => { status.value = ''; reset() })
watch([selectedAgent, status, from, to], reset)
watch(page, () => { if (view.value !== 'active') { queries.value = []; reload() } })
watch(() => props.active, active => { if (!active) { version++; loading.value = false }; poll() }, { immediate: true })
watch(online, connected => { if (connected) poll() })
watch(() => remoteState.pendingTaskResult, openPendingResult)
onMounted(() => { document.addEventListener('visibilitychange', poll); window.addEventListener('pageshow', poll); window.addEventListener('ppc:task-center-changed', changed) })
onBeforeUnmount(() => { disposed = true; version++; clearTimeout(timer); clearTimeout(scheduled); document.removeEventListener('visibilitychange', poll); window.removeEventListener('pageshow', poll); window.removeEventListener('ppc:task-center-changed', changed) })
</script>
