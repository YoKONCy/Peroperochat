<template>
  <section class="task-center">
    <div v-if="filterCount" class="task-filter-summary"><button type="button" :title="filterSummary" @click="openFilters">{{ filterSummary }}</button><button type="button" @click="clearFilters">{{ tr('清除') }}</button></div>
    <div v-if="view === 'history'" class="task-history-toolbar"><RemoteAction icon="check" tone="quiet" :disabled="!online || writing || unresolved" @click="markAllRead">{{ tr('全部标为已读') }}</RemoteAction></div>
    <RemoteFeedback v-if="active" :message="error || actionError" />
    <RemoteFeedback v-if="active && unresolved" :message="tr('操作结果待确认')" :action-label="tr('去核对')" @action="$emit('pending')" />
    <p v-if="loading && !items.length" class="room-empty">{{ tr('正在读取任务…') }}</p><p v-else-if="!items.length && !error" class="room-empty">{{ tr(view === 'history' ? '还没有历史任务' : view === 'resident' ? '暂无系统常驻状态' : '现在没有待处理的任务') }}</p>
    <div class="task-center-cards"><button v-for="task in items" :key="task.id" type="button" class="task-center-card" :data-state="task.status" @click="open(task)"><header><RemoteAvatar :id="task.agentId" :label="agentName(task.agentId)" /><span>{{ agentName(task.agentId) }}</span><i v-if="view === 'history' && !task.readAt" class="task-unread-dot" :aria-label="tr('未读')" /><strong>{{ tr(taskStatusLabels[task.status] || '状态待同步') }}</strong></header><h2>{{ task.title || tr('一项任务') }}</h2><p>{{ task.currentStage || task.lifecycle?.continuation || task.instruction }}</p><div v-if="taskProgress(task.progress) !== null" class="task-progress-line"><progress :value="task.progress" max="100" /><small>{{ task.progress }}%</small></div><footer><span>{{ taskTime(task.createdAt) }}</span><span>{{ tr(task.status === 'waiting_input' ? '查看并回复' : task.status === 'blocked' ? '查看原因' : '查看详情') }}<RemoteIcon name="next" /></span></footer></button></div>
    <RemoteAction v-if="view === 'active' && hasMore" icon="down" :disabled="loading" @click="more">{{ tr('加载更多') }}</RemoteAction>
    <div v-if="view !== 'active' && (page > 1 || hasMore)" class="remote-pagination"><button :disabled="loading || page === 1" @click="goToPage(page - 1)">{{ tr('上一页') }}</button><span>{{ page }}</span><button :disabled="loading || !hasMore" @click="goToPage(page + 1)">{{ tr('下一页') }}</button></div>
    <RoomDrawer v-if="filterOpen" :title="tr('筛选任务')" @close="filterOpen = false">
      <form class="task-filter-form" @submit.prevent="applyFilters">
        <RemoteFeedback :message="filterError" />
        <label class="remote-search"><RemoteIcon name="search" /><input v-model="filterDraft.keyword" :placeholder="tr('搜索任务')" :aria-label="tr('搜索任务')" maxlength="200" /></label>
        <label class="remote-field">{{ tr('任务角色') }}<RemoteSelect v-model="filterDraft.agentId" :label="tr('任务角色')" :options="agentOptions" /></label>
        <label class="remote-field">{{ tr('任务状态') }}<RemoteSelect v-model="filterDraft.status" :label="tr('任务状态')" :options="statusOptions" /></label>
        <div class="task-filter-dates">
          <label class="remote-field">{{ tr('开始日期') }}<input v-model="filterDraft.from" type="date" :max="filterDraft.to || undefined" /></label>
          <label class="remote-field">{{ tr('结束日期') }}<input v-model="filterDraft.to" type="date" :min="filterDraft.from || undefined" /></label>
        </div>
        <button type="submit" hidden />
      </form>
      <template #footer><div class="remote-purpose-actions"><RemoteAction icon="close" @click="clearFilters">{{ tr('清除') }}</RemoteAction><RemoteAction icon="check" tone="primary" @click="applyFilters">{{ tr('应用') }}</RemoteAction></div></template>
    </RoomDrawer>
  </section>
</template>
<script setup>
import { computed, reactive, ref, toRef, watch, onMounted, onBeforeUnmount } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
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
const props = defineProps({ active: Boolean, view: { type: String, default: 'active' } })
const emit = defineEmits(['detail', 'pending'])
const { run, write, error: actionError, writing, unresolved } = useRemoteManagement()
const tr = (key, values) => t(key, values, 'tasks')
const token = currentToken(), view = toRef(props, 'view'), selectedAgent = ref(''), status = ref(''), keyword = ref(''), page = ref(1), queries = ref([]), loading = ref(false), error = ref('')
const online = computed(() => remoteState.connection === 'online')
const from = ref(''), to = ref('')
const filterOpen = ref(false), filterError = ref('')
const filterDraft = reactive({ keyword: '', agentId: '', status: '', from: '', to: '' })
const filterCount = computed(() => [keyword.value, selectedAgent.value, status.value, from.value || to.value].filter(Boolean).length)
const filterSummary = computed(() => [keyword.value && `“${keyword.value}”`, selectedAgent.value && agentName(selectedAgent.value), status.value && tr(taskStatusLabels[status.value]), (from.value || to.value) && `${from.value || '…'} – ${to.value || '…'}`].filter(Boolean).join(' · '))
const agentOptions = computed(() => [{ value: '', label: tr('全部角色') }, ...remoteState.agents.filter(agent => !agent.isUser).map(agent => ({ value: agent.id, label: agent.name }))])
const statusOptions = computed(() => [{ value: '', label: tr('全部状态') }, ...(view.value === 'active' ? activeTaskStatuses : view.value === 'history' ? finishedTaskStatuses : [...activeTaskStatuses, ...finishedTaskStatuses]).map(value => ({ value, label: tr(taskStatusLabels[value]) }))])
const snapshots = computed(() => queries.value.map(query => remoteViews.get(JSON.stringify(query))?.data))
const items = computed(() => newestTasks(snapshots.value.flatMap(data => data?.items || [])).filter(task => task.category === (view.value === 'resident' ? 'resident' : 'agent_task') && (view.value !== 'active' || activeTaskStatuses.includes(task.status)) && (view.value !== 'history' || finishedTaskStatuses.includes(task.status))))
const total = computed(() => view.value === 'active' ? queries.value.filter(query => query.page === 1).reduce((sum, query) => sum + (remoteViews.get(JSON.stringify(query))?.data?.total || 0), 0) : snapshots.value[0]?.total || 0)
const count = computed(() => {
  const counted = view.value === 'active' ? queries.value.filter(query => query.page === 1) : queries.value
  return counted.length && counted.every(query => typeof remoteViews.get(JSON.stringify(query))?.data?.total === 'number') ? total.value : null
})
const hasMore = computed(() => queries.value.some((query, index) => query.page * query.pageSize < (snapshots.value[index]?.total || 0) && (view.value !== 'active' || !queries.value.some(other => other.status === query.status && other.page === query.page + 1))))
let disposed = false, version = 0, timer = null, scheduled = null
function prepareLeave() { if (writing.value) throw new Error(tr('正在处理，请稍等。')); filterOpen.value = false }
defineExpose({ prepareLeave, reload, loading, count, filterCount, openFilters })
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
function openFilters() {
  Object.assign(filterDraft, { keyword: keyword.value, agentId: selectedAgent.value, status: status.value, from: from.value, to: to.value })
  filterError.value = ''; filterOpen.value = true
}
function applyFilters() {
  if (filterDraft.from && !taskDateBoundary(filterDraft.from) || filterDraft.to && !taskDateBoundary(filterDraft.to) || filterDraft.from && filterDraft.to && filterDraft.from > filterDraft.to) {
    filterError.value = tr('日期范围无效'); return
  }
  keyword.value = filterDraft.keyword.trim(); selectedAgent.value = filterDraft.agentId; status.value = filterDraft.status
  from.value = filterDraft.from; to.value = filterDraft.to; filterOpen.value = false
}
function clearFilters() {
  keyword.value = ''; selectedAgent.value = ''; status.value = ''; from.value = ''; to.value = ''; filterOpen.value = false
}
function goToPage(value) {
  if (loading.value || value < 1 || view.value === 'active') return
  version++; queries.value = []; page.value = value; reload()
}
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
watch(view, () => { status.value = ''; filterOpen.value = false })
watch([view, selectedAgent, status, keyword, from, to], reset)
watch(() => props.active, active => { if (!active) { version++; loading.value = false; filterOpen.value = false }; poll() }, { immediate: true })
watch(online, connected => { if (connected) poll() })
watch(() => remoteState.pendingTaskResult, openPendingResult)
onMounted(() => { document.addEventListener('visibilitychange', poll); window.addEventListener('pageshow', poll); window.addEventListener('ppc:task-center-changed', changed) })
onBeforeUnmount(() => { disposed = true; version++; clearTimeout(timer); clearTimeout(scheduled); document.removeEventListener('visibilitychange', poll); window.removeEventListener('pageshow', poll); window.removeEventListener('ppc:task-center-changed', changed) })
</script>
