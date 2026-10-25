import test from 'node:test'
import assert from 'node:assert/strict'
import { WHOLESOME_MODE_KEY, ASSISTANT_PREFILL_KEY, MAX_ASSISTANT_PREFILL_LENGTH, parseAnswerPreferences, changedAnswerPreferences } from '../src/utils/answerPreferences.js'

test('预填充的默认模式与 infOS 一致，无效旧值不会自动启用', () => {
  assert.deepEqual(parseAnswerPreferences().prefill, { enabled: false, content: '', mode: 'once' })
  for (const value of [{ enabled: true, content: '开头', mode: 'auto' }, { enabled: 'true', content: '开头' }, { enabled: true, content: '字'.repeat(MAX_ASSISTANT_PREFILL_LENGTH + 1) }]) {
    assert.equal(parseAnswerPreferences({ [ASSISTANT_PREFILL_KEY]: JSON.stringify(value) }).prefill.enabled, false)
  }
  assert.equal(parseAnswerPreferences({ [ASSISTANT_PREFILL_KEY]: JSON.stringify({ enabled: true, content: '开头' }) }).prefill.mode, 'once')
})

test('启停与单次、持续模式均原样保存正文，专用入口不附带健全模式写入', () => {
  const baseline = parseAnswerPreferences({ [WHOLESOME_MODE_KEY]: 'false' })
  for (const mode of ['once', 'continuous']) {
    for (const enabled of [false, true]) {
      const prefill = { enabled, content: '  <reply>\n（先想一想）\n', mode }
      const items = changedAnswerPreferences({ ...baseline, prefill }, baseline)
      assert.deepEqual(items, [{ key: ASSISTANT_PREFILL_KEY, value: JSON.stringify(prefill) }])
      assert.deepEqual(parseAnswerPreferences({ [ASSISTANT_PREFILL_KEY]: items[0].value }).prefill, prefill)
    }
  }
  assert.deepEqual(changedAnswerPreferences(baseline, baseline), [])
})

test('正文上限遵循服务端的 UTF-16 长度，同时保留关闭状态下的内容', () => {
  const content = '😀'.repeat(MAX_ASSISTANT_PREFILL_LENGTH / 2)
  assert.equal(parseAnswerPreferences({ [ASSISTANT_PREFILL_KEY]: JSON.stringify({ enabled: false, content, mode: 'continuous' }) }).prefill.content, content)
  assert.deepEqual(parseAnswerPreferences({ [ASSISTANT_PREFILL_KEY]: JSON.stringify({ enabled: true, content: content + 'a', mode: 'continuous' }) }).prefill, { enabled: false, content: '', mode: 'once' })
})
