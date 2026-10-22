// 只识别正文里的全角括号；代码、链接目标与 HTML 属性仍由 Markdown 词法器处理。
export const parentheticalExtension = {
  name: 'ppcAside', level: 'inline',
  start(source) { const index = source.indexOf('（'); return index < 0 ? undefined : index },
  tokenizer(source) {
    if (source[0] !== '（') return
    let depth = 0, ticks = 0
    for (let index = 0; index < Math.min(source.length, 8192); index++) {
      const char = source[index]
      if (char === '\\') { index++; continue }
      if (char === '`') {
        let end = index; while (source[end] === '`') end++
        const count = end - index
        if (!ticks) ticks = count; else if (ticks === count) ticks = 0
        index = end - 1; continue
      }
      if (ticks) continue
      if (char === '\n' && source[index + 1] === '\n') return
      if (char === '（' && ++depth > 8) return
      if (char === '）' && --depth === 0) {
        const raw = source.slice(0, index + 1), text = raw.slice(1, -1)
        if (!text.trim()) return
        return { type: 'ppcAside', raw, tokens: this.lexer.inlineTokens(text) }
      }
    }
  },
  renderer(token) { return `<span class="message-aside"><span class="aside-parenthesis">（</span>${this.parser.parseInline(token.tokens)}<span class="aside-parenthesis">）</span></span>` },
  childTokens: ['tokens'],
}

// 保持行内公式为一个文本节点，交给后续公式渲染处理，避免被括号旁白拆开。
const formulaProtection = {
  name: 'ppcInlineFormula', level: 'inline',
  start(source) { const index = source.indexOf('$'); return index < 0 ? undefined : index },
  tokenizer(source) { const match = source.match(/^\$(?!\$)([^$\n]+?)\$(?!\$)/); if (match) return { type: 'ppcInlineFormula', raw: match[0], text: match[0] } },
  renderer(token) { return `<span class="message-inline-formula">${token.text.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;')}</span>` },
}
export const expressionExtensions = [formulaProtection, parentheticalExtension]

// 原文流式尾部只追加文本；括号闭合时只替换这一小段，稳定正文与选区不重建。
export function appendAsideText(root, state, value) {
  const append = text => {
    if (!text) return
    if (!state.node) { state.node = document.createTextNode(''); root.append(state.node) }
    state.node.appendData(text)
    if (state.depth && state.node.length - state.start > 8192) state.depth = 0
  }
  let cursor = 0
  // 按连续文字批量写入，避免一帧里为每个汉字单独触发 DOM 更新。
  for (const match of value.matchAll(/[（）\n]/g)) {
    append(value.slice(cursor, match.index))
    const char = match[0], offset = state.node?.length || 0
    append(char); cursor = match.index + 1
    if (char === '（') { if (!state.depth) state.start = offset; state.depth++ }
    else if (char === '）' && state.depth && --state.depth === 0) {
      const text = state.node.data.slice(state.start)
      if (text.slice(1, -1).trim()) {
        state.node.deleteData(state.start, text.length)
        const aside = document.createElement('span'); aside.className = 'message-aside aside-arriving'
        const start = document.createElement('span'), end = document.createElement('span')
        start.className = end.className = 'aside-parenthesis'; start.textContent = '（'; end.textContent = '）'
        aside.append(start, text.slice(1, -1), end); root.append(aside); state.node = null
      }
    }
    if (char === '\n') state.depth = 0
  }
  append(value.slice(cursor))
}
