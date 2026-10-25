import test from 'node:test'
import assert from 'node:assert/strict'
import { activeTaskStatuses, taskQueries, taskPermissions, taskProgress, taskDateBoundary, taskTime, taskResultQuery, opensTaskResult, newestTasks } from '../src/utils/taskCenter.js'

test('活跃任务覆盖真实六种状态，历史由服务端先过滤再分页，常驻状态独立查询', () => {
  const active = taskQueries({ agentId: 'pero', keyword: ' 路线 ' })
  assert.deepEqual(active.map(query => query.status), activeTaskStatuses)
  assert(active.every(query => query.category === 'agent_task' && query.agentId === 'pero' && query.keyword === '路线'))
  assert.deepEqual(taskQueries({ view: 'history', page: 3 }), [{ kind: 'tasks', agentId: null, status: null, page: 3, pageSize: 30, category: 'agent_task', view: 'history' }])
  assert.equal(taskQueries({ view: 'history', status: 'running' }).length, 0)
  assert.equal(taskQueries({ status: 'interrupted' }).length, 0)
  assert.equal(taskQueries({ view: 'resident' })[0].category, 'resident')
})

test('只允许有效普通任务执行对应操作，重派和删除遵守待确认动作与取消状态', () => {
  const task = { id: 'a', category: 'agent_task', revision: 4, status: 'waiting_input' }
  assert.equal(taskPermissions(task).pause, true)
  assert.equal(taskPermissions(task).resume, false)
  for (const status of ['paused', 'blocked', 'waiting', 'failed']) assert.equal(taskPermissions({ ...task, status }).resume, true)
  for (const status of ['completed', 'failed', 'cancelled']) {
    const finished = { ...task, status }
    assert.equal(taskPermissions(finished).retry, true)
    assert.equal(taskPermissions(finished).delete, true)
    assert.equal(taskPermissions({ ...finished, lifecycle: { pendingEffect: { callId: 'old' } } }).retry, false)
    assert.equal(taskPermissions({ ...finished, metadata: { cancellationPending: true } }).delete, false)
  }
  for (const invalid of [null, { ...task, category: 'resident' }, { ...task, revision: undefined }, { ...task, revision: 0 }, { ...task, status: 'interrupted' }]) assert(Object.values(taskPermissions(invalid)).every(value => !value))
})

test('任务去重以修订号为准，未知进度不能伪装为零或推算进度', () => {
  const current = { id: 'a', revision: 5, createdAt: '2026-10-25', updatedAt: '2026-10-25' }
  const old = { ...current, revision: 4, updatedAt: '2026-10-26' }
  const other = { id: 'b', revision: 1, createdAt: '2026-10-24' }
  assert.deepEqual(newestTasks([old, other, current, old]), [current, other])
  assert.equal(taskProgress(0), 0)
  assert.equal(taskProgress(100), 100)
  for (const value of [null, undefined, -1, 101, '50', Infinity]) assert.equal(taskProgress(value), null)
})

test('日期范围按本机时区换算为服务端 UTC 文本，任务时间兼容数据库与标准时间戳', () => {
  const date = new Date('2026-10-25T00:00:00')
  assert.equal(taskDateBoundary('2026-10-25'), date.toISOString().replace('T', ' ').slice(0, 19))
  assert.equal(taskDateBoundary('2026-10-25', true), new Date('2026-10-25T23:59:59').toISOString().replace('T', ' ').slice(0, 19))
  assert.equal(taskTime('2026-10-25 08:00:00'), taskTime('2026-10-25T08:00:00Z'))
  assert.equal(taskDateBoundary(''), '')
})

test('完成事件只触发权威读取，补发通知不重复打开结果，普通通知任务也不自动打开', () => {
  const payload = { action: 'background_task_completed', task: { id: 'a' } }
  assert.deepEqual(taskResultQuery(payload), { kind: 'task', id: 'a' })
  assert.equal(taskResultQuery(payload, true), null)
  assert.equal(taskResultQuery({ ...payload, replayed: true }), null)
  assert.equal(taskResultQuery({ ...payload, action: 'background_task_failed' }), null)
  const task = { category: 'agent_task', status: 'completed', completionAction: 'open_result' }
  assert.equal(opensTaskResult(task), true)
  assert.equal(opensTaskResult({ ...task, completionAction: 'notify' }), false)
  assert.equal(opensTaskResult({ ...task, status: 'running' }), false)
  assert.equal(opensTaskResult({ ...task, category: 'resident' }), false)
})
