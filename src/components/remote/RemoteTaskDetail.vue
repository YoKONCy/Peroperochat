<template>
  <RoomDrawer :title="tr('任务详情')" variant="editor" :busy="writing" :before-close="beforeClose" @close="$emit('close')">
    <div class="task-detail-scroll">
      <RemoteFeedback :message="error || loadError" /><RemoteFeedback v-if="unresolved" :message="tr('操作结果待确认')" :action-label="tr('去核对')" @action="$emit('pending')" />
      <p v-if="loading && !task" class="room-empty">{{ tr('正在读取任务…') }}</p>
      <template v-if="task">
        <header class="task-detail-hero"><RemoteAvatar :id="task.agentId" :label="agentName" /><div><small>{{ agentName }} · {{ tr(task.category === 'resident' ? '系统常驻' : task.parentTaskId ? '子任务' : '后台任务') }}</small><h2>{{ task.title }}</h2><span class="task-status-badge" :data-state="task.status">{{ tr(taskStatusLabels[task.status] || '状态待同步') }}</span></div><button class="room-icon-button" :disabled="loading || writing" :aria-label="tr('刷新')" @click="load"><RemoteIcon name="refresh" /></button></header>
        <p v-if="task.currentStage" class="task-current-stage">{{ task.currentStage }}</p>
        <div v-if="taskProgress(task.progress) !== null" class="task-progress-line"><progress :value="task.progress" max="100" /><small>{{ task.progress }}%</small></div>
        <p v-if="task.category === 'resident'" class="remote-help">{{ tr('系统常驻状态随服务自动维护，不需要手动派发或恢复。') }}</p>
        <div v-else class="task-detail-controls"><RemoteAction v-if="permissions.pause" icon="pause" :disabled="blocked" @click="control('pause')">{{ tr('暂停任务') }}</RemoteAction><RemoteAction v-if="permissions.resume" icon="play" tone="primary" :disabled="blocked || !!task.lifecycle?.pendingEffect && !confirmed" @click="control('resume')">{{ tr('继续任务') }}</RemoteAction><RemoteAction v-if="permissions.update" icon="edit" :disabled="blocked" @click="startEditing">{{ tr('调整任务') }}</RemoteAction><RemoteAction v-if="permissions.cancel" icon="stop" tone="danger" :disabled="blocked" @click="ask('cancel')">{{ tr('取消任务') }}</RemoteAction><RemoteAction v-if="permissions.retry" icon="refresh" :disabled="blocked" @click="ask('retry')">{{ tr('重新派发') }}</RemoteAction><RemoteAction v-if="permissions.delete" icon="trash" tone="danger" :disabled="blocked" @click="ask('delete')">{{ tr('删除记录') }}</RemoteAction><RemoteAction v-if="finished && !task.readAt" icon="check" tone="quiet" :disabled="blocked" @click="direct('read')">{{ tr('标为已读') }}</RemoteAction></div>
        <label v-if="task.category !== 'resident' && task.lifecycle?.pendingEffect" class="task-pending-effect"><input v-model="confirmed" type="checkbox" :disabled="writing" /><span>{{ tr('我已核对上一次动作的结果') }}<small>{{ task.lifecycle.pendingEffect.toolName }}</small></span></label>
        <RemoteSurface v-for="surface in interactions" :key="surface.surfaceId" :surface="surface" :disabled="blocked" @resource="$emit('resource', $event)" @updated="updated" />
        <nav class="task-center-tabs" :aria-label="tr('任务内容')"><button v-for="item in sections" :key="item.id" :class="{ selected: section === item.id }" :aria-pressed="section === item.id" @click="section = item.id"><RemoteIcon :name="item.icon" />{{ tr(item.label) }}</button></nav>
        <template v-if="section === 'overview'">
          <section class="task-detail-section"><h3>{{ tr('任务要求') }}</h3><p>{{ task.instruction }}</p><p v-if="task.lifecycle?.successCriteria"><strong>{{ tr('完成条件') }}</strong><br />{{ task.lifecycle.successCriteria }}</p><p v-if="task.lifecycle?.continuation">{{ task.lifecycle.continuation }}</p></section>
          <section v-if="task.result || task.errorMessage" class="task-detail-section"><h3>{{ tr(task.result ? '任务结果' : '需要处理') }}</h3><RemoteRichText v-if="task.result" :source="task.result" node-id="task-result" :context="context" @resource="$emit('resource', $event)" /><p v-if="task.errorMessage" class="task-failure-copy">{{ task.errorMessage }}</p></section>
          <section v-if="task.lifecycle?.wait" class="task-detail-section"><h3>{{ tr('正在等待') }}</h3><p v-if="task.lifecycle.wait.kind === 'time'">{{ taskTime(task.lifecycle.wait.key) }}</p><button v-else-if="task.lifecycle.wait.kind === 'task'" type="button" class="task-child-row" @click="openTask(task.lifecycle.wait.key)"><RemoteIcon name="tasks" /><span>{{ tr('查看等待中的关联任务') }}</span><RemoteIcon name="next" /></button><p v-else>{{ tr('回复上方的问题或确认请求后，任务会继续。') }}</p></section>
          <RemoteTaskPlan v-if="task.plan" :plan="task.plan" />
          <ol v-else-if="task.lifecycle?.plan?.length" class="task-lifecycle-plan"><li v-for="(step, index) in task.lifecycle.plan" :key="index"><RemoteIcon :name="step.done ? 'check' : 'diamond'" />{{ step.title }}</li></ol>
          <section v-if="task.lifecycle?.artifacts?.length" class="task-detail-section"><h3>{{ tr('交付文件') }}</h3><button v-for="path in task.lifecycle.artifacts" :key="path" type="button" class="task-artifact" @click="$emit('resource', { source: path, context })"><RemoteIcon name="files" /><span>{{ path }}</span><RemoteIcon name="next" /></button></section>
          <details v-if="task.lifecycle?.evidence?.length" class="task-detail-section"><summary>{{ tr('执行依据') }}</summary><p v-for="item in task.lifecycle.evidence" :key="item.toolCallId">{{ item.description }}</p></details>
          <section v-if="deliveries.length" class="task-detail-section"><h3>{{ tr('结果投递') }}</h3><article v-for="item in deliveries" :key="item.id" class="task-delivery"><header><strong>{{ tr(item.channel === 'social' ? '社交会话' : item.channel === 'chat' ? '对话' : '通知') }}</strong><span>{{ tr(deliveryLabels[item.status] || '状态待同步') }}</span></header><p v-if="item.lastError">{{ item.lastError }}</p><div v-if="['uncertain', 'failed'].includes(item.status)"><RemoteAction v-if="item.status === 'uncertain'" icon="check" :disabled="blocked" @click="ask('delivery-sent', item.id)">{{ tr('已确认送达') }}</RemoteAction><RemoteAction icon="refresh" :disabled="blocked" @click="ask('delivery-retry', item.id)">{{ tr('重新投递') }}</RemoteAction></div></article></section>
          <dl class="remote-metadata"><dt>{{ tr('创建时间') }}</dt><dd>{{ taskTime(task.createdAt) }}</dd><dt>{{ tr('开始时间') }}</dt><dd>{{ taskTime(task.startedAt) || '—' }}</dd><dt v-if="task.completedAt">{{ tr('结束时间') }}</dt><dd v-if="task.completedAt">{{ taskTime(task.completedAt) }}</dd><dt v-if="task.lifecycle?.deadline">{{ tr('截止时间') }}</dt><dd v-if="task.lifecycle?.deadline">{{ taskTime(task.lifecycle.deadline) }}</dd></dl>
          <button v-if="task.targetThreadId || task.threadId" type="button" class="room-secondary-button" @click="$emit('thread', { agentId: task.agentId, threadId: task.targetThreadId || task.threadId })"><RemoteIcon name="chat" />{{ tr('打开关联会话') }}</button>
        </template>
        <template v-else-if="section === 'trace'"><RemoteFeedback :message="projectionError" /><RemoteSurface v-for="surface in traceSurfaces" :key="surface.surfaceId" :surface="surface" :disabled="blocked" @resource="$emit('resource', $event)" @updated="updated" /><p v-if="!traceSurfaces.length" class="room-empty">{{ tr('暂时没有执行轨迹') }}</p></template>
        <template v-else><button v-if="task.parentTaskId" type="button" class="room-secondary-button" @click="openTask(task.parentTaskId)"><RemoteIcon name="back" />{{ tr('查看父任务') }}</button><button v-for="child in children" :key="child.id" type="button" class="task-child-row" @click="openTask(child.id, child.title)"><RemoteIcon name="tasks" /><span>{{ child.title }}<small>{{ tr(taskStatusLabels[child.status] || '状态待同步') }}</small></span><RemoteIcon name="next" /></button><p v-if="!children.length" class="room-empty">{{ tr('没有关联子任务') }}</p><div v-if="childPage > 1 || childMore" class="remote-pagination"><button :disabled="loading || childPage === 1" @click="childPage--">{{ tr('上一页') }}</button><span>{{ childPage }}</span><button :disabled="loading || !childMore" @click="childPage++">{{ tr('下一页') }}</button></div></template>
      </template>
    </div>
    <RoomDrawer v-if="editing" :title="tr('调整任务')" :busy="writing" :before-close="closeEditor" @close="editing = false"><form class="task-edit-form" @submit.prevent="control('update')"><RemoteFeedback :message="error" /><p v-if="task.revision !== draftRevision" class="remote-help">{{ tr('任务已更新，草稿仍保留。请重新查看任务后再调整。') }}</p><label class="remote-field">{{ tr('任务要求') }}<textarea v-model="draft.instruction" rows="4" required maxlength="4000" :disabled="writing" /></label><label class="remote-field">{{ tr('完成条件') }}<textarea v-model="draft.successCriteria" rows="2" maxlength="2000" :disabled="writing" /></label><label class="remote-field">{{ tr('截止时间') }}<input v-model="draft.deadline" type="datetime-local" :required="!!task.lifecycle?.deadline" :disabled="writing" /></label><label class="task-pending-effect"><input v-model="draft.readOnly" type="checkbox" :disabled="writing" />{{ tr('仅查询与读取，服务重启后可自动继续') }}</label><div class="task-edit-budgets"><label v-for="item in budgetFields" :key="item.key" class="remote-field">{{ tr(item.label) }}<input v-model.number="draft[item.key]" type="number" min="1" :max="item.max" required :disabled="writing" /></label></div><RemoteAction type="submit" icon="check" tone="primary" :disabled="blocked || !dirty || task.revision !== draftRevision">{{ tr('保存调整') }}</RemoteAction></form></RoomDrawer>
    <RoomDrawer v-if="confirmation" :title="tr(confirmCopy.title)" :busy="writing" @close="confirmation = null"><p class="task-confirm-copy">{{ tr(confirmCopy.body) }}</p><RemoteFeedback :message="error" /><div class="remote-purpose-actions"><RemoteAction icon="back" :disabled="writing" @click="confirmation = null">{{ tr('返回') }}</RemoteAction><RemoteAction :icon="confirmation.action === 'delete' ? 'trash' : 'check'" :tone="['delete', 'cancel'].includes(confirmation.action) ? 'danger' : 'primary'" :disabled="blocked" @click="confirmOperation">{{ tr(confirmCopy.label) }}</RemoteAction></div></RoomDrawer>
    <RoomDrawer v-if="discard" :title="tr('放弃尚未保存的修改？')" @close="cancelDiscard"><div class="remote-purpose-actions"><RemoteAction icon="back" @click="cancelDiscard">{{ tr('继续编辑') }}</RemoteAction><RemoteAction icon="trash" tone="danger" @click="discardDraft">{{ tr('放弃修改') }}</RemoteAction></div></RoomDrawer>
  </RoomDrawer>
</template>
<script setup>
import { computed, ref, watch, onMounted, onBeforeUnmount } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RemoteAvatar from './RemoteAvatar.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import RemoteSurface from './RemoteSurface.vue'
import RemoteRichText from './RemoteRichText.vue'
import RemoteTaskPlan from './RemoteTaskPlan.vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { remoteViews, remoteState, readRemote } from '../../stores/remote'
import { finishedTaskStatuses, taskPermissions, taskProgress, taskStatusLabels, taskTime } from '../../utils/taskCenter'
import { t } from '../../i18n/remote'
const props = defineProps({ taskId: { type: String, required: true } })
const emit = defineEmits(['close', 'pending', 'resource', 'thread', 'detail'])
const { token, read, write, run, guard, error, busy, writing, unresolved } = useRemoteManagement()
const tr = (key, values) => t(key, values, 'tasks')
const taskQuery = { kind: 'task', id: props.taskId }, projectionQuery = { kind: 'task_projection', id: props.taskId }, deliveryQuery = { kind: 'task_deliveries', id: props.taskId }
const task = computed(() => remoteViews.get(JSON.stringify(taskQuery))?.data)
const projection = computed(() => remoteViews.get(JSON.stringify(projectionQuery))?.data)
const deliveries = computed(() => remoteViews.get(JSON.stringify(deliveryQuery))?.data || [])
const context = computed(() => ({ agentId: task.value?.agentId, threadId: task.value?.threadId, taskId: props.taskId }))
const interactions = computed(() => task.value?.category === 'resident' ? [] : (projection.value?.surfaces || []).filter(surface => surface.surfaceId === `background-task:${props.taskId}`).map(surface => ({ ...surface, taskId: props.taskId, nodes: surface.nodes.filter(node => ['input', 'approval'].includes(node.kind)) })).filter(surface => surface.nodes.length))
const traceSurfaces = computed(() => (projection.value?.surfaces || []).filter(surface => surface.surfaceId !== `background-task:${props.taskId}`).map(surface => ({ ...surface, taskId: props.taskId })))
const permissions = computed(() => taskPermissions(task.value))
const finished = computed(() => finishedTaskStatuses.includes(task.value?.status))
const agentName = computed(() => remoteState.agents.find(agent => agent.id === task.value?.agentId)?.name || task.value?.agentId)
const blocked = computed(() => busy.value || unresolved.value || remoteState.connection !== 'online')
const section = ref('overview'), confirmed = ref(false), loading = ref(false), loadError = ref(''), projectionError = ref(''), childPage = ref(1)
const sections = [{ id: 'overview', label: '进展与结果', icon: 'tasks' }, { id: 'trace', label: '执行轨迹', icon: 'route' }, { id: 'related', label: '关联任务', icon: 'group' }]
const childrenQuery = computed(() => ({ kind: 'tasks', agentId: task.value?.agentId || null, status: null, category: 'agent_task', parentTaskId: props.taskId, page: childPage.value, pageSize: 30 }))
const childData = computed(() => remoteViews.get(JSON.stringify(childrenQuery.value))?.data)
const children = computed(() => childData.value?.items || []), childMore = computed(() => childPage.value * 30 < (childData.value?.total || 0))
const deliveryLabels = { pending: '待投递', sending: '发送中', sent: '已送达', uncertain: '送达待确认', failed: '投递失败', superseded: '已失效' }
const confirmation = ref(null), editing = ref(false), draft = ref({}), discard = ref(false)
let draftRevision = null, baseline = '', decision = null, disposed = false, timer = null, discardTarget = 'editor'
const dirty = computed(() => editing.value && JSON.stringify(draft.value) !== baseline)
const budgetFields = [{ key: 'maxSegments', label: '执行段上限', max: 1000 }, { key: 'maxLlmCalls', label: '模型调用上限', max: 10000 }, { key: 'maxToolCalls', label: '工具调用上限', max: 50000 }]
const confirmCopies = { cancel: { title: '取消这项任务？', body: '将停止后续执行，已经完成的操作不会撤销。', label: '确认取消' }, delete: { title: '删除任务记录？', body: '只删除这条已结束任务的记录，删除后无法恢复。', label: '删除记录' }, retry: { title: '重新派发任务？', body: '会创建一项新任务重新执行，请先确认已有结果。', label: '重新派发' }, 'delivery-sent': { title: '确认结果已送达？', body: '请先检查接收端是否已经收到这份结果。', label: '已确认送达' }, 'delivery-retry': { title: '重新投递结果？', body: '请先确认接收端尚未收到，避免重复投递。', label: '重新投递' } }
const confirmCopy = computed(() => confirmCopies[confirmation.value?.action] || confirmCopies.cancel)
async function load() {
  if (disposed || document.hidden || loading.value || writing.value) return
  loading.value = true; loadError.value = ''
  try {
    await readRemote(taskQuery); guard()
    if (task.value?.category !== 'resident') {
      const results = await Promise.allSettled([readRemote(projectionQuery), readRemote(deliveryQuery), readRemote(childrenQuery.value)]); guard()
      projectionError.value = results[0].status === 'rejected' ? results[0].reason.message : ''
      loadError.value = results.slice(1).find(result => result.status === 'rejected')?.reason.message || ''
    }
  } catch (failure) { if (!disposed) loadError.value = failure.message }
  finally { if (!disposed) loading.value = false }
}
function notify() { window.dispatchEvent(new CustomEvent('ppc:task-center-changed', { detail: { token, id: props.taskId } })) }
async function updated() { await load(); notify() }
function openTask(id, title) { emit('detail', { taskId: id, title: title || tr('任务详情'), query: { kind: 'task_projection', id } }) }
function ask(action, deliveryId) { if (!blocked.value) { error.value = ''; confirmation.value = { action, deliveryId, revision: task.value.revision } } }
async function control(action, extra = {}, revision = action === 'update' ? draftRevision : task.value?.revision) {
  if (blocked.value || !task.value || (action === 'resolve_delivery' ? task.value.category !== 'agent_task' || !Number.isSafeInteger(task.value.revision) : !permissions.value[action])) return
  await run(async () => {
    const current = await read('tasks.get', { id: props.taskId }); guard()
    if (current.revision !== revision || current.agentId !== task.value.agentId) throw new Error(tr('任务已更新，请查看最新状态后再操作。'))
    if (action === 'resume' && current.lifecycle?.pendingEffect && !confirmed.value) throw new Error(tr('请先核对上一次动作的结果。'))
    const body = { agentId: current.agentId, taskId: current.id, expectedRevision: revision, action, ...extra }
    if (action === 'resume') body.confirmUncertain = confirmed.value
    if (action === 'update') {
      if (!draft.value.instruction?.trim() || draft.value.instruction.length > 4000 || draft.value.successCriteria.length > 2000 || budgetFields.some(item => !Number.isInteger(draft.value[item.key]) || draft.value[item.key] < 1 || draft.value[item.key] > item.max)) throw new Error(tr('请检查填写的内容'))
      Object.assign(body, { instruction: draft.value.instruction, successCriteria: draft.value.successCriteria, recovery: draft.value.readOnly ? 'read_only' : 'confirm', ...Object.fromEntries(budgetFields.map(item => [item.key, draft.value[item.key]])), ...(draft.value.deadline ? { deadline: new Date(draft.value.deadline).toISOString() } : {}) })
    }
    await write('tasks.control', { body }); guard(); confirmation.value = null
    if (action === 'update') editing.value = false
  })
  if (!error.value) { await load(); notify() }
}
async function direct(action, revision = task.value?.revision) {
  if (blocked.value || task.value?.category === 'resident') return
  let result = null, completed = false
  await run(async () => {
    const current = await read('tasks.get', { id: props.taskId }); guard()
    if (current.revision !== revision || !finishedTaskStatuses.includes(current.status) || action === 'delete' && !taskPermissions(current).delete || action === 'retry' && !taskPermissions(current).retry) throw new Error(tr('任务已更新，请查看最新状态后再操作。'))
    result = await write(`tasks.${action}`, { id: props.taskId }); guard(); completed = true; confirmation.value = null
  })
  if (!completed) return
  notify()
  if (action === 'delete') emit('close')
  else if (result?.id && result.id !== props.taskId) openTask(result.id, result.title)
  else await load()
}
async function confirmOperation() {
  if (!confirmation.value) return
  const current = { ...confirmation.value }
  if (['read', 'retry', 'delete'].includes(current.action)) await direct(current.action, current.revision)
  else if (current.action.startsWith('delivery-')) await control('resolve_delivery', { deliveryId: current.deliveryId, deliveryDecision: current.action === 'delivery-sent' ? 'sent' : 'retry' }, current.revision)
  else await control(current.action, {}, current.revision)
}
function startEditing() {
  if (blocked.value || !permissions.value.update) return
  const value = task.value, lifecycle = value.lifecycle || {}, date = lifecycle.deadline ? new Date(lifecycle.deadline) : null
  draftRevision = value.revision
  draft.value = { instruction: value.instruction, successCriteria: lifecycle.successCriteria || '', readOnly: lifecycle.recovery === 'read_only', deadline: date && Number.isFinite(date.getTime()) ? new Date(date.getTime() - date.getTimezoneOffset() * 60000).toISOString().slice(0, 16) : '', maxSegments: lifecycle.maxSegments ?? 48, maxLlmCalls: lifecycle.maxLlmCalls ?? 192, maxToolCalls: lifecycle.maxToolCalls ?? 768 }
  baseline = JSON.stringify(draft.value); editing.value = true
}
function closeEditor() { if (dirty.value) { discardTarget = 'editor'; discard.value = true; return false }; return true }
function beforeClose() { if (dirty.value) { discardTarget = 'detail'; discard.value = true; return false }; return true }
function cancelDiscard() { discard.value = false; decision?.(false); decision = null }
function discardDraft() { editing.value = false; discard.value = false; decision?.(true); decision = null; if (discardTarget === 'detail') emit('close') }
async function prepareLeave() { if (writing.value) throw new Error(tr('正在处理，请稍等。')); if (dirty.value) { discardTarget = 'navigation'; discard.value = true; if (!await new Promise(resolve => { decision = resolve })) throw Object.assign(new Error(''), { code: 'UI_CANCELLED' }) } }
defineExpose({ prepareLeave })
function poll() { clearTimeout(timer); if (disposed) return; if (!document.hidden) load(); timer = setTimeout(poll, task.value?.category === 'resident' || finished.value ? 30000 : 5000) }
watch(() => [task.value?.id, task.value?.revision], () => { confirmed.value = false })
watch(childPage, load)
watch(() => remoteState.connection, state => { if (state === 'online') poll() })
onMounted(() => { poll(); document.addEventListener('visibilitychange', poll); window.addEventListener('pageshow', poll) })
onBeforeUnmount(() => { disposed = true; clearTimeout(timer); decision?.(false); document.removeEventListener('visibilitychange', poll); window.removeEventListener('pageshow', poll) })
</script>
