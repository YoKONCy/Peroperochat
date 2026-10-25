import test from 'node:test'
import assert from 'node:assert/strict'
import { createRemoteReadCoordinator } from '../src/services/remoteReadCoordinator.js'
import { observeRemoteResult } from '../src/services/remoteObservation.js'
import { elapsedLabel, runTimestamp } from '../src/utils/runTiming.js'

function deferred() {
  let resolve, reject
  const promise = new Promise((yes, no) => { resolve = yes; reject = no })
  return { promise, resolve, reject }
}

test('相同读取只发一次，恢复后新读取不被悬挂的旧读取占用', async () => {
  const old = deferred(), fresh = deferred()
  let calls = 0
  const reads = createRemoteReadCoordinator({ query: () => ++calls === 1 ? old.promise : fresh.promise, assertActive: () => {} })
  const first = reads.read({ kind: 'queue', id: 'a' }, { token: '连接一' })
  const duplicate = reads.read({ id: 'a', kind: 'queue' }, { token: '连接一' })
  const rejected = Promise.all([assert.rejects(first, { code: 'REMOTE_STALE_READ' }), assert.rejects(duplicate, { code: 'REMOTE_STALE_READ' })])
  assert.equal(calls, 1)
  reads.reset()
  const next = reads.read({ kind: 'queue', id: 'a' }, { token: '连接一' })
  old.resolve({ data: '旧队列' }); fresh.resolve({ data: '新队列' })
  await rejected
  assert.deepEqual(await next, { data: '新队列' })
  assert.equal(calls, 2)
})

test('恢复发生在重试间隔时，不再启动旧读取的下一次请求', async () => {
  const paused = deferred(), entered = deferred()
  let calls = 0
  const reads = createRemoteReadCoordinator({ assertActive: () => {},
    query: async () => { calls++; throw Object.assign(new Error('内容已更新'), { code: 'REMOTE_STALE_READ' }) },
    pause: () => { entered.resolve(); return paused.promise },
  })
  const pending = reads.read({ kind: 'conversation', id: 'a' }, { token: '连接一' })
  const rejected = assert.rejects(pending, { code: 'REMOTE_STALE_READ' })
  await entered.promise
  reads.reset(); paused.resolve()
  await rejected
  assert.equal(calls, 1)
})

test('结束前端等待不会取消或重新调用原请求', async () => {
  const native = deferred()
  let completions = 0
  const operation = native.promise.then(() => { completions++; return '已经保存' })
  await assert.rejects(observeRemoteResult(operation, { timeout: 0 }), { code: 'REMOTE_NETWORK' })
  native.resolve()
  assert.equal(await operation, '已经保存')
  assert.equal(completions, 1)
})

test('切屏很久后立即核对截止时间，不依赖后台定时器运行', async t => {
  const originalDocument = Object.getOwnPropertyDescriptor(globalThis, 'document')
  const foreground = new EventTarget(), operation = deferred()
  let now = 1000
  t.mock.method(Date, 'now', () => now)
  Object.defineProperty(globalThis, 'document', { value: foreground, configurable: true })
  try {
    const pending = observeRemoteResult(operation.promise, { timeout: 75000 })
    const rejected = assert.rejects(pending, { code: 'REMOTE_NETWORK' })
    now += 30 * 60 * 1000
    foreground.dispatchEvent(new Event('visibilitychange'))
    await rejected
    operation.resolve('迟到的结果')
  } finally {
    if (originalDocument) Object.defineProperty(globalThis, 'document', originalDocument)
    else delete globalThis.document
  }
})

test('等待计时接受服务端时间戳，长时间切屏后直接显示累计时长', () => {
  const start = 1792857600000
  assert.equal(runTimestamp(new Date(start).toISOString()), start)
  assert.equal(runTimestamp(start), start)
  assert.equal(runTimestamp('无效时间'), null)
  assert.equal(runTimestamp(null), null)
  assert.equal(elapsedLabel(9000), '9s')
  assert.equal(elapsedLabel(61_000), '1:01')
  assert.equal(elapsedLabel(3_661_000), '1:01:01')
  assert.equal(elapsedLabel(-1000), '0s')
})
