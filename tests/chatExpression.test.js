import test from 'node:test'
import assert from 'node:assert/strict'
import { Marked } from 'marked'
import { expressionExtensions } from '../src/utils/parenthetical.js'
import { reasoningLevel, effectiveReasoning, recordedReasoning } from '../src/utils/reasoningAppearance.js'

const parser = new Marked({ extensions: expressionExtensions })
test('全角括号保留强调与链接，代码、公式、半角括号不套旁白', () => {
  const html = parser.parse('台词（轻轻**挥手**，看[这里](https://example.com)）\n\n`（代码）` 和 $（公式）$ 与 fn(x)')
  assert.match(html, /message-aside/)
  assert.match(html, /<strong>挥手<\/strong>/)
  assert.match(html, /<a href="https:\/\/example.com">这里<\/a>/)
  assert.match(html, /<code>（代码）<\/code>/)
  assert.match(html, /message-inline-formula">\$（公式）\$/)
  assert.equal((html.match(/class="message-aside"/g) || []).length, 1)
})
test('未闭合括号保留原文，闭合后才渲染，嵌套与跨片段强调不丢字', () => {
  assert.doesNotMatch(parser.parse('（尚未说完'), /message-aside/)
  const html = parser.parse('（先坐下（往旁边挪一点），**慢慢来**）')
  assert.match(html, /往旁边挪一点/)
  assert.match(html, /<strong>慢慢来<\/strong>/)
  assert.equal((html.match(/class="message-aside"/g) || []).length, 2)
})
test('自动直接使用服务端有效档位；不传参数与未记录档位严格区分', () => {
  assert.equal(effectiveReasoning({ reasoningEffort: null, effective: { reasoningEffort: 'high' } }), 'high')
  assert.equal(effectiveReasoning({ reasoningEffort: null, effective: { reasoningEffort: null } }), 'default')
  assert.equal(effectiveReasoning(null), 'unknown')
  assert.equal(recordedReasoning({}), 'unknown')
  assert.equal(recordedReasoning({ reasoningEffort: null }), 'default')
  assert.equal(reasoningLevel('none'), 'off')
  assert.equal(reasoningLevel('not-a-level'), 'unknown')
})
