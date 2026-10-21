import test from 'node:test'
import assert from 'node:assert/strict'
import { activeFeedback, showFeedback, closeFeedback, clearFeedback, holdFeedback, resumeFeedback } from '../src/services/appFeedback.js'

function clock(t) {
  const original = { now: Date.now, schedule: globalThis.setTimeout, cancel: globalThis.clearTimeout }
  let now = 100000, serial = 0
  const timers = new Map()
  Date.now = () => now
  globalThis.setTimeout = (callback, delay) => { const id = ++serial; timers.set(id, { callback, at: now + delay }); return id }
  globalThis.clearTimeout = id => timers.delete(id)
  clearFeedback(); resumeFeedback('hidden')
  t.after(() => {
    clearFeedback(); resumeFeedback('hidden')
    Date.now = original.now; globalThis.setTimeout = original.schedule; globalThis.clearTimeout = original.cancel
  })
  return {
    timers,
    tick(duration) {
      const end = now + duration
      while (true) {
        const next = [...timers].filter(([, entry]) => entry.at <= end).sort((a, b) => a[1].at - b[1].at)[0]
        if (!next) break
        timers.delete(next[0]); now = next[1].at; next[1].callback()
      }
      now = end
    },
  }
}

test('重复后台错误不会让气泡续期，自动收起后短期不重弹', t => {
  const time = clock(t)
  let dismissals = 0
  showFeedback({ message: '连接中断', tone: 'error', onDismiss: () => dismissals++ })
  const id = activeFeedback.value.id
  time.tick(4000)
  showFeedback({ message: '连接中断', tone: 'error' })
  assert.equal(activeFeedback.value.id, id)
  time.tick(800)
  assert.equal(activeFeedback.value, null)
  assert.equal(dismissals, 1)
  showFeedback({ message: '连接中断', tone: 'error' })
  assert.equal(activeFeedback.value, null)
  time.tick(30000)
  showFeedback({ message: '连接中断', tone: 'error' })
  assert.equal(activeFeedback.value.message, '连接中断')
})

test('悬停、后台和聚焦同时暂停，恢复后只使用剩余时间', t => {
  const time = clock(t)
  showFeedback({ message: '文件已保存', tone: 'success' })
  time.tick(1000); holdFeedback('pointer'); holdFeedback('hidden')
  time.tick(10000); resumeFeedback('pointer')
  assert.equal(time.timers.size, 0)
  holdFeedback('focus'); resumeFeedback('hidden'); time.tick(2000)
  assert.ok(activeFeedback.value)
  resumeFeedback('focus'); time.tick(1799)
  assert.ok(activeFeedback.value)
  time.tick(1)
  assert.equal(activeFeedback.value, null)
})

test('离开来源组件只移除自己的气泡，队列中的旧提示不会迟到', t => {
  const time = clock(t)
  const first = showFeedback({ message: '第一条', tone: 'error' })
  const second = showFeedback({ message: '第二条', tone: 'info' })
  showFeedback({ message: '第三条', tone: 'info' })
  second.close(); first.close()
  assert.equal(activeFeedback.value.message, '第三条')
  holdFeedback('hidden')
  showFeedback({ message: '后台期间的提醒', tone: 'info' })
  time.tick(9000); resumeFeedback('hidden'); time.tick(4800)
  assert.equal(activeFeedback.value, null)
})

test('主动重试可以再次报告相同错误，自动提示仍保持去重', t => {
  clock(t)
  let dismissals = 0
  showFeedback({ message: '保存失败', tone: 'error', onDismiss: () => dismissals++ })
  closeFeedback(activeFeedback.value.id, true, true)
  showFeedback({ message: '保存失败', tone: 'error' })
  assert.equal(activeFeedback.value.message, '保存失败')
  assert.equal(dismissals, 1)
})

test('切换页面清空反馈，迟到的旧计时器不能关闭新页面提示', t => {
  const time = clock(t)
  let oldDismissals = 0
  showFeedback({ message: '旧页面', tone: 'error', onDismiss: () => oldDismissals++ })
  const lateTimer = [...time.timers.values()][0].callback
  showFeedback({ message: '旧页面排队提醒', tone: 'info' })
  clearFeedback()
  showFeedback({ message: '新页面', tone: 'success' })
  lateTimer()
  assert.equal(activeFeedback.value.message, '新页面')
  assert.equal(oldDismissals, 0)
  time.tick(2800)
  assert.equal(activeFeedback.value, null)
})
