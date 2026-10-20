import test from 'node:test'
import assert from 'node:assert/strict'
import { createTextRenderCache } from '../src/utils/textRenderCache.js'

test('重复正文复用渲染结果，更新正文仍重新渲染', () => {
  let calls = 0
  const render = createTextRenderCache(text => { calls++; return `<p>${text}</p>` })
  assert.equal(render('你好'), '<p>你好</p>')
  assert.equal(render('你好'), '<p>你好</p>')
  assert.equal(calls, 1)
  assert.equal(render('你好呀'), '<p>你好呀</p>')
  assert.equal(calls, 2)
})

test('缓存受总字符预算约束，超长流式正文不会持续驻留', () => {
  const calls = []
  const render = createTextRenderCache(text => { calls.push(text); return text }, 2, 8)
  render('aa'); render('bb'); render('aa'); render('cc'); render('bb')
  assert.deepEqual(calls, ['aa', 'bb', 'cc', 'bb'])
  const large = 'a'.repeat(20)
  render(large); render(large)
  assert.equal(calls.filter(text => text === large).length, 2)
})

test('不同组件实例不共享正文或渲染结果', () => {
  const first = createTextRenderCache(text => `独立:${text}`)
  const second = createTextRenderCache(text => `远程:${text}`)
  assert.equal(first('消息'), '独立:消息')
  assert.equal(second('消息'), '远程:消息')
})
