// 缓存由组件各自持有，销毁后释放，不在角色或远程连接之间共享正文。
export function createTextRenderCache(render, maxEntries = 48, maxCharacters = 600000) {
  const entries = new Map()
  let characters = 0
  return value => {
    const text = typeof value === 'string' ? value : ''
    if (entries.has(text)) {
      const cached = entries.get(text)
      entries.delete(text); entries.set(text, cached)
      return cached
    }
    const html = render(text)
    const size = text.length + html.length
    if (size > maxCharacters) return html
    while (entries.size && (entries.size >= maxEntries || characters + size > maxCharacters)) {
      const oldest = entries.keys().next().value
      characters -= oldest.length + entries.get(oldest).length
      entries.delete(oldest)
    }
    entries.set(text, html); characters += size
    return html
  }
}
