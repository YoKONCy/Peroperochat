import { clearCache, layout, prepare } from '@chenglou/pretext'
import { measureRichInlineStats, prepareRichInline } from '@chenglou/pretext/rich-inline'

let fontRevision = 0
let environmentObserver = null
let environmentFrame = null
let rebuildingObserver = false
let pixelRatio = window.devicePixelRatio
const subscribers = new Map()
const environmentAttributes = ['class', 'style', 'lang', 'data-ui', 'data-remote-theme', 'data-theme', 'data-tone']

function observeAncestors(element) {
  for (let ancestor = element.parentElement; ancestor; ancestor = ancestor.parentElement) {
    environmentObserver.observe(ancestor, { attributes: true, attributeFilter: environmentAttributes })
  }
}

function fontsChanged() {
  fontRevision++
  clearCache()
  for (const refresh of subscribers.values()) refresh()
}

function attributesChanged(records) {
  if (records.some(record => record.attributeName === 'lang')) {
    fontsChanged()
    return
  }
  for (const [root, notify] of subscribers) {
    if (records.some(record => record.target instanceof Element && record.target.contains(root))) notify()
  }
}

function viewportChanged() {
  if (environmentFrame !== null) return
  environmentFrame = requestAnimationFrame(() => {
    environmentFrame = null
    if (pixelRatio !== window.devicePixelRatio) {
      pixelRatio = window.devicePixelRatio
      fontsChanged()
      return
    }
    for (const refresh of subscribers.values()) refresh()
  })
}

function rebuildObserver() {
  if (rebuildingObserver) return
  rebuildingObserver = true
  // 一次切页可能卸载很多段落，只在这批卸载结束后重建祖先监听。
  queueMicrotask(() => {
    rebuildingObserver = false
    if (!environmentObserver) return
    attributesChanged(environmentObserver.takeRecords())
    environmentObserver.disconnect()
    for (const root of subscribers.keys()) observeAncestors(root)
  })
}

// 同一批消息共用环境监听，字号、主题或语言变化后重新读取实际文字样式。
export function observeTextLayout(element, refresh) {
  subscribers.set(element, refresh)
  if (!environmentObserver) {
    environmentObserver = new MutationObserver(attributesChanged)
    document.fonts?.addEventListener('loadingdone', fontsChanged)
    window.addEventListener('resize', viewportChanged, { passive: true })
  }
  // 只观察祖先的属性，正文自身的高度提示不会触发下一轮测量。
  observeAncestors(element)
  return () => {
    if (!subscribers.delete(element)) return
    if (!subscribers.size) {
      environmentObserver.disconnect()
      environmentObserver = null
      document.fonts?.removeEventListener('loadingdone', fontsChanged)
      window.removeEventListener('resize', viewportChanged)
      if (environmentFrame !== null) cancelAnimationFrame(environmentFrame)
      environmentFrame = null
      fontRevision++
      clearCache()
      return
    }
    rebuildObserver()
  }
}

export function readTextStyle(element) {
  const style = getComputedStyle(element)
  return {
    font: `${style.fontStyle} ${style.fontWeight} ${style.fontSize} ${style.fontFamily}`,
    lineHeight: Number.parseFloat(style.lineHeight),
    letterSpacing: Number.parseFloat(style.letterSpacing) || 0,
    whiteSpace: style.whiteSpace,
    wordBreak: style.wordBreak,
    display: style.display,
    paddingX: (Number.parseFloat(style.paddingLeft) || 0) + (Number.parseFloat(style.paddingRight) || 0),
    paddingY: (Number.parseFloat(style.paddingTop) || 0) + (Number.parseFloat(style.paddingBottom) || 0),
    borderX: (Number.parseFloat(style.borderLeftWidth) || 0) + (Number.parseFloat(style.borderRightWidth) || 0),
    borderY: (Number.parseFloat(style.borderTopWidth) || 0) + (Number.parseFloat(style.borderBottomWidth) || 0),
    marginX: (Number.parseFloat(style.marginLeft) || 0) + (Number.parseFloat(style.marginRight) || 0),
    boxSizing: style.boxSizing,
    minHeight: Number.parseFloat(style.minHeight) || 0,
    maxHeight: Number.parseFloat(style.maxHeight) || Infinity,
  }
}

// 每个正文或输入框持有有界缓存，宽度变化只计算换行，销毁组件时释放正文句柄。
export function createTextMeasurer(maxCharacters = 240000) {
  const entries = new Map()
  let characters = 0

  function cached(key, build) {
    if (entries.has(key)) {
      const value = entries.get(key)
      entries.delete(key)
      entries.set(key, value)
      return value
    }
    const value = build()
    if (key.length > maxCharacters) return value
    while (entries.size && (entries.size >= 128 || characters + key.length > maxCharacters)) {
      const oldest = entries.keys().next().value
      characters -= oldest.length
      entries.delete(oldest)
    }
    entries.set(key, value)
    characters += key.length
    return value
  }

  function measure(text, style, width) {
    if (!(width > 0) || !(style.lineHeight > 0)) return null
    try {
      /** @type {import('@chenglou/pretext').PrepareOptions} */
      const options = {
        whiteSpace: style.whiteSpace === 'pre-wrap' ? 'pre-wrap' : 'normal',
        wordBreak: style.wordBreak === 'keep-all' ? 'keep-all' : 'normal',
        letterSpacing: style.letterSpacing,
      }
      const key = JSON.stringify([fontRevision, document.documentElement.lang, style.font, options, text])
      const prepared = cached(key, () => prepare(text, style.font, options))
      return layout(prepared, width, style.lineHeight).height
    } catch {
      // 旧 WebView 或特殊字体采用浏览器原有排版。
      return null
    }
  }

  function measureInline(items, style, width) {
    if (!(width > 0) || !(style.lineHeight > 0)) return null
    try {
      /** @type {import('@chenglou/pretext/rich-inline').RichInlineOptions} */
      const options = { wordBreak: style.wordBreak === 'keep-all' ? 'keep-all' : 'normal' }
      const key = JSON.stringify([fontRevision, document.documentElement.lang, options, items])
      const prepared = cached(key, () => prepareRichInline(items, options))
      return measureRichInlineStats(prepared, width).lineCount * style.lineHeight
    } catch {
      return null
    }
  }

  return { measure, measureInline }
}
