import test from 'node:test'
import assert from 'node:assert/strict'
import { remoteFeedbackMessage, presentRemoteError, createRemoteFeedbackLifecycle } from '../src/services/remoteFeedback.js'
import { createRemoteReadCoordinator } from '../src/services/remoteReadCoordinator.js'

const failure = code => Object.assign(new Error('内部说明'), { code })
const active = () => {}
function deferred() { let resolve; const promise = new Promise(done => { resolve = done }); return { promise, resolve } }

test('合并相同读取，并区分连接、离线缓存与最新内容', async () => {
  const completion = deferred(), requests = []
  const reader = createRemoteReadCoordinator({ assertActive: active, query: async (query, options) => { requests.push(options); await completion.promise; return options } })
  const a = reader.read({ kind: 'thread', id: 'a' }, { token: 'one' })
  const b = reader.read({ id: 'a', kind: 'thread' }, { token: 'one' })
  const cached = reader.read({ kind: 'thread', id: 'a' }, { token: 'one', cachedOnly: true })
  const other = reader.read({ kind: 'thread', id: 'a' }, { token: 'two' })
  assert.equal(requests.length, 3)
  completion.resolve()
  assert.equal(await a, await b)
  assert.equal((await cached).cachedOnly, true)
  assert.equal((await other).token, 'two')
  await reader.read({ kind: 'thread', id: 'a' }, { token: 'one' })
  assert.equal(requests.length, 4)
})

test('读取碰上写入时静默重试，成功后返回最新结果', async () => {
  let calls = 0
  const waits = []
  const reader = createRemoteReadCoordinator({ assertActive: active, pause: async delay => { waits.push(delay) }, query: async () => { if (++calls < 4) throw failure('REMOTE_STALE_READ'); return { data: '最新回复' } } })
  assert.deepEqual(await reader.read({ kind: 'conversation' }), { data: '最新回复' })
  assert.equal(calls, 4)
  assert.deepEqual(waits, [50, 150, 400])
})

test('持续变化会结束本轮读取，不无限重试或暴露内部冲突', async () => {
  let calls = 0
  const reader = createRemoteReadCoordinator({ assertActive: active, pause: async () => {}, query: async () => { calls++; throw failure('REMOTE_STALE_READ') } })
  await assert.rejects(reader.read({ kind: 'conversation' }), error => error.code === 'REMOTE_STALE_READ' && remoteFeedbackMessage(error) === '')
  assert.equal(calls, 4)
})

test('网络与授权错误不在读取协调器内重发，失败不污染后续重试', async () => {
  for (const code of ['REMOTE_NETWORK', 'REMOTE_AUTH_REQUIRED', 'REMOTE_CONFLICT']) {
    let calls = 0
    const reader = createRemoteReadCoordinator({ assertActive: active, query: async () => { if (++calls === 1) throw failure(code); return { data: '恢复' } } })
    await assert.rejects(reader.read({ kind: 'thread' }), { code })
    assert.equal(calls, 1)
    assert.deepEqual(await reader.read({ kind: 'thread' }), { data: '恢复' })
  }
})

test('切换模式后不继续旧连接的延迟重试', async () => {
  let allowed = true, calls = 0
  const reader = createRemoteReadCoordinator({ assertActive: () => { if (!allowed) throw failure('MODE_INACTIVE') }, pause: async () => { allowed = false }, query: async () => { calls++; throw failure('REMOTE_STALE_READ') } })
  await assert.rejects(reader.read({ kind: 'thread' }), { code: 'MODE_INACTIVE' })
  assert.equal(calls, 1)
})

test('错误文案保留机器状态与未确认结果，隐藏实现细节', () => {
  const original = Object.assign(failure('REMOTE_NETWORK'), { outcome: 'unknown', requestId: 'request-one' })
  const error = presentRemoteError(original)
  assert.match(error.message, /通知/)
  assert.equal(error.code, 'REMOTE_NETWORK')
  assert.equal(error.requestId, 'request-one')
  assert.equal(error.outcome, 'unknown')
  assert.equal(remoteFeedbackMessage(failure('REMOTE_STALE_READ')), '')
  assert.equal(remoteFeedbackMessage('读取期间远程状态已刷新或发生操作，请重新读取'), '')
  assert.doesNotMatch(remoteFeedbackMessage(failure('REMOTE_PROTOCOL')), /信封|JSON|投影/)
  assert.equal(remoteFeedbackMessage('请先选择角色'), '请先选择角色')
})

test('普通提示自动收起、支持暂停，重复后台错误不会续期占屏', () => {
  let time = 1000, next = 0, visible = false, dismissals = 0
  const timers = new Map()
  const feedback = createRemoteFeedbackLifecycle({ show: state => { visible = state }, dismissed: () => { dismissals++ }, now: () => time, schedule: (callback, delay) => { const id = ++next; timers.set(id, { callback, delay }); return id }, cancel: id => timers.delete(id) })
  feedback.update('连接失败')
  assert.equal(visible, true)
  assert.equal([...timers.values()][0].delay, 5500)
  feedback.hold(); assert.equal(timers.size, 0)
  feedback.resume(); assert.equal(timers.size, 1)
  const expire = [...timers.values()][0].callback
  expire(); assert.equal(visible, false); assert.equal(dismissals, 1)
  time += 6000; feedback.update(''); feedback.update('连接失败')
  assert.equal(visible, false); assert.equal(timers.size, 0)
  feedback.update('新问题'); assert.equal(visible, true)
  feedback.dismiss(); assert.equal(visible, false)
  time += 31000; feedback.update('连接失败'); assert.equal(visible, true)
  feedback.dispose(); assert.equal(timers.size, 0)
  feedback.update('退出后迟到的提示'); assert.equal(timers.size, 0)
})
