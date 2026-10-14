import test from 'node:test'
import assert from 'node:assert/strict'
import { createLocalChatClient } from '../src/services/localChatClient.js'
import { visibleMessage, displayMessage } from '../src/utils/messageContent.js'

function deferred() {
  let resolve, reject
  const promise = new Promise((a, b) => { resolve = a; reject = b })
  return { promise, resolve, reject }
}

test('监听注册完成后才发起请求，并隔离其他角色与请求的事件', async () => {
  const registration = deferred()
  const completion = deferred()
  const calls = []
  const chunks = []
  let handler, removed = 0
  const client = createLocalChatClient({ uuid: () => '当前请求',
    listen: async (name, callback) => { handler = callback; await registration.promise; return () => removed++ },
    invoke: (name) => { calls.push(name); return completion.promise },
  })
  const result = client.turn({ threadId: 'desktop:pero', agentId: 'pero', content: '你好' }, { onEvent: event => chunks.push(event.content) })
  await Promise.resolve()
  assert.deepEqual(calls, [])
  registration.resolve()
  await new Promise(resolve => setImmediate(resolve))
  const emit = fields => handler({ payload: { requestId: '当前请求', threadId: 'desktop:pero', agentId: 'pero', kind: 'delta', ...fields } })
  emit({ requestId: '其他请求', content: '串包' })
  emit({ agentId: 'nana', content: '其他角色' })
  emit({ threadId: 'group:family', content: '其他会话' })
  emit({ content: '主人你好' })
  completion.resolve({ message: { id: 7, content: '主人你好' } })
  assert.equal((await result).message.id, 7)
  assert.deepEqual(chunks, ['主人你好'])
  assert.equal(removed, 1)
  emit({ content: '迟到数据' })
  assert.equal(chunks.length, 1)
})

test('启动前取消不会发出请求，注册期间取消也会清理监听', async () => {
  const controller = new AbortController()
  controller.abort()
  let calls = 0
  const client = createLocalChatClient({ invoke: async () => calls++, listen: async () => calls++ })
  await assert.rejects(client.turn({}, { signal: controller.signal }), { code: 'CANCELLED' })
  assert.equal(calls, 0)

  const registration = deferred()
  const second = new AbortController()
  let removed = 0
  const waiting = createLocalChatClient({ invoke: async () => calls++, listen: async () => { await registration.promise; return () => removed++ } })
    .turn({}, { signal: second.signal })
  second.abort()
  registration.resolve()
  await assert.rejects(waiting, { code: 'CANCELLED' })
  assert.equal(calls, 0)
  assert.equal(removed, 1)
})

test('取消会调用后端并等待终态，后端失败一定拒绝 Promise', async () => {
  const controller = new AbortController()
  const completion = deferred()
  let cancelled = 0, removed = 0
  const client = createLocalChatClient({ uuid: () => '取消请求',
    listen: async () => () => removed++,
    invoke: async name => {
      if (name === 'local_cancel_chat') { cancelled++; return }
      return completion.promise
    },
  })
  const waiting = client.turn({ threadId: 'desktop:pero', agentId: 'pero' }, { signal: controller.signal })
  await new Promise(resolve => setImmediate(resolve))
  controller.abort()
  assert.equal(cancelled, 1)
  completion.reject({ code: 'CANCELLED', message: '已停止回复' })
  await assert.rejects(waiting, { code: 'CANCELLED', message: '已停止回复' })
  assert.equal(removed, 1)

  const broken = createLocalChatClient({ listen: async () => { throw new Error('监听失败') }, invoke: async () => { throw new Error('不应执行') } })
  await assert.rejects(broken.turn({}), /监听失败/)
})

test('展示过滤器隐藏协议残片且保留普通 HTML 与代码', () => {
  assert.equal(visibleMessage('你好<MEMORY>{"content":"秘密"}</MEMORY>'), '你好')
  assert.equal(visibleMessage('你好<PEROCUE>{"mood":"'), '你好')
  assert.equal(visibleMessage('<DIV>示例</DIV>'), '<DIV>示例</DIV>')
  const message = { id: 8, timestamp: 123, status: 'pending', content: '' }
  const display = displayMessage(message)
  assert.equal(display.content, '__loading__')
  assert.equal(display.timestamp, 123)
  assert.equal(message.content, '')
})
