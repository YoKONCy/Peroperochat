import DOMPurify from 'dompurify'

let highlightJob, mathJob, diagramJob, diagramQueue = Promise.resolve()
export async function highlightRemoteCode(source, language) {
  if (source.length > 32000) return null
  highlightJob ||= Promise.all([
    import('highlight.js/lib/core'), import('highlight.js/lib/languages/javascript'), import('highlight.js/lib/languages/typescript'),
    import('highlight.js/lib/languages/python'), import('highlight.js/lib/languages/json'), import('highlight.js/lib/languages/xml'),
    import('highlight.js/lib/languages/css'), import('highlight.js/lib/languages/bash'), import('highlight.js/lib/languages/powershell'), import('highlight.js/lib/languages/markdown'),
  ]).then(([core, ...languages]) => {
    const engine = core.default
    const names = [['javascript', 'js'], ['typescript', 'ts'], ['python', 'py'], ['json'], ['xml', 'html', 'svg'], ['css'], ['bash', 'shell', 'sh'], ['powershell', 'ps1'], ['markdown', 'md']]
    languages.forEach((module, index) => names[index].forEach(name => engine.registerLanguage(name, module.default)))
    return engine
  }).catch(error => { highlightJob = null; throw error })
  const engine = await highlightJob
  return engine.getLanguage(language) ? engine.highlight(source, { language, ignoreIllegals: true }).value : null
}
export async function remoteMath(source, displayMode = true) {
  if (source.length > 16000) throw new Error('公式较大，请查看原文')
  mathJob ||= Promise.all([import('katex'), import('katex/dist/katex.min.css')]).then(([module]) => module.default).catch(error => { mathJob = null; throw error })
  const engine = await mathJob
  return engine.renderToString(source, { displayMode, throwOnError: true, strict: 'ignore', trust: false, maxExpand: 1000, maxSize: 20, output: 'htmlAndMathml' })
}
export function remoteDiagram(source) {
  // Mermaid 的全局配置与临时 DOM 串行使用，多个消息同时出现时不互相覆盖。
  const job = diagramQueue.catch(() => {}).then(async () => {
    if (source.length > 32000) throw new Error('图表较大，请查看源码')
    diagramJob ||= import('mermaid').then(module => module.default).catch(error => { diagramJob = null; throw error })
    const engine = await diagramJob, id = `ppc-diagram-${crypto.randomUUID()}`
    engine.initialize({ startOnLoad: false, securityLevel: 'strict', theme: 'neutral', suppressErrorRendering: true, maxTextSize: 32000, maxEdges: 500, flowchart: { htmlLabels: false } })
    try {
      const result = await engine.render(id, source)
      const clean = DOMPurify.sanitize(result.svg, { USE_PROFILES: { svg: true, svgFilters: true }, FORBID_TAGS: ['script', 'foreignObject', 'a'], FORBID_ATTR: ['onload', 'onclick'] })
      const doc = new DOMParser().parseFromString(clean, 'image/svg+xml')
      if (doc.querySelector('parsererror')) throw new Error('图表暂时无法显示')
      for (const element of doc.querySelectorAll('*')) {
        for (const attribute of [...element.attributes]) {
          if (attribute.localName === 'href' && !attribute.value.startsWith('#')) element.removeAttributeNode(attribute)
          if ([...attribute.value.matchAll(/url\(\s*(['"]?)(.*?)\1\s*\)/gi)].some(match => !match[2].trim().startsWith('#'))) element.removeAttributeNode(attribute)
        }
      }
      for (const style of doc.querySelectorAll('style')) style.textContent = style.textContent.replace(/@import[^;]+;/gi, '').replace(/url\([^)]*\)/gi, 'none')
      return new XMLSerializer().serializeToString(doc.documentElement)
    } finally { document.getElementById(id)?.remove(); document.getElementById(`d${id}`)?.remove() }
  })
  diagramQueue = job; return job
}
