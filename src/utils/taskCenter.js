// 与 infOS 的任务状态、分类和控制面板保持一致；服务端始终决定最终状态。
export const activeTaskStatuses = ['queued', 'running', 'paused', 'waiting_input', 'waiting', 'blocked']
export const finishedTaskStatuses = ['completed', 'failed', 'cancelled']
export const taskStatusLabels = { queued: '排队中', running: '执行中', paused: '已暂停', waiting_input: '等你回复', waiting: '等待条件', blocked: '需要处理', completed: '已完成', failed: '执行失败', cancelled: '已取消' }
export function taskQueries({ view = 'active', agentId = null, status = '', keyword = '', page = 1, from = '', to = '' } = {}) {
  const base = { kind: 'tasks', agentId: agentId || null, status: status || null, page, pageSize: view === 'active' ? 100 : 30, category: view === 'resident' ? 'resident' : 'agent_task', ...(keyword.trim() ? { keyword: keyword.trim() } : {}), ...(from ? { from } : {}), ...(to ? { to } : {}) }
  if (view === 'active') return (status ? activeTaskStatuses.filter(value => value === status) : activeTaskStatuses).map(value => ({ ...base, status: value }))
  if (view === 'history') return status && !finishedTaskStatuses.includes(status) ? [] : [{ ...base, view: 'history' }]
  return view === 'resident' ? [{ ...base }] : []
}
export function taskPermissions(task) {
  const mutable = !!task && task.category === 'agent_task' && Number.isSafeInteger(task.revision) && task.revision > 0
  return {
    pause: mutable && ['queued', 'running', 'waiting', 'waiting_input'].includes(task.status),
    resume: mutable && ['paused', 'blocked', 'waiting', 'failed'].includes(task.status),
    cancel: mutable && activeTaskStatuses.includes(task.status),
    update: mutable && [...activeTaskStatuses, 'failed'].includes(task.status),
    retry: mutable && finishedTaskStatuses.includes(task.status) && !task.lifecycle?.pendingEffect,
    delete: mutable && finishedTaskStatuses.includes(task.status) && !task.metadata?.cancellationPending,
  }
}
export function taskProgress(value) { return typeof value === 'number' && Number.isFinite(value) && value >= 0 && value <= 100 ? value : null }
export function taskTime(value) { if (!value) return ''; let stamp = String(value).replace(' ', 'T'); if (/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?$/.test(stamp)) stamp += 'Z'; const date = new Date(stamp); return Number.isFinite(date.getTime()) ? date.toLocaleString() : '' }
// infOS 按数据库的 UTC 文本比较创建时间，筛选边界需要与其格式一致。
export function taskDateBoundary(day, end = false) { if (!day) return ''; const date = new Date(`${day}T${end ? '23:59:59' : '00:00:00'}`); return Number.isFinite(date.getTime()) ? date.toISOString().replace('T', ' ').slice(0, 19) : '' }
export function taskResultQuery(payload, replayed = false) { return !replayed && !payload?.replayed && payload?.action === 'background_task_completed' && typeof payload.task?.id === 'string' && payload.task.id ? { kind: 'task', id: payload.task.id } : null }
export function opensTaskResult(task) { return task?.category === 'agent_task' && task.status === 'completed' && task.completionAction === 'open_result' }
export function newestTasks(values) {
  const tasks = new Map()
  for (const task of values) {
    if (!task?.id) continue
    const old = tasks.get(task.id)
    if (!old || (task.revision || 0) > (old.revision || 0) || task.revision === old.revision && String(task.updatedAt) > String(old.updatedAt)) tasks.set(task.id, task)
  }
  return [...tasks.values()].sort((a, b) => String(b.createdAt).localeCompare(String(a.createdAt)))
}
