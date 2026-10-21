import { Marked } from 'marked'
import DOMPurify from 'dompurify'
import { isResourceReference } from '../services/remoteResources'

export const escapeRich = value => String(value ?? '').replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;')
const lexer = new Marked({ gfm: true, breaks: true })
lexer.use({ extensions: [{ name: 'ppcFormula', level: 'block', start: source => source.search(/(?:^|\n)[ \t]*(?:\$\$|\\\[)/), tokenizer(source) { const match = source.match(/^[ \t]*(?:\$\$[ \t]*\n?([\s\S]*?)\$\$|\\\[[ \t]*\n?([\s\S]*?)\\\])[ \t]*(?:\n|$)/); if (match) return { type: 'ppcFormula', raw: match[0], text: (match[1] ?? match[2]).trim() } } }] })
const blockTags = /<\/?(?:html|body|main|article|section|div|header|footer|nav|form|svg|script|style)\b[^>]*>/gi
function stateLine(line, state) {
  const marker = line.match(/^[ \t]{0,3}(`{3,}|~{3,})/)
  if (state.math) { if (line.includes(state.math)) state.math = ''; return false }
  if (marker) {
    if (!state.fence) state.fence = marker[1]
    else if (marker[1][0] === state.fence[0] && marker[1].length >= state.fence.length && !line.slice(marker[0].length).trim()) { state.fence = ''; return true }
  } else if (!state.fence) {
    const trimmed = line.trimStart()
    if (trimmed.startsWith('$$') && !trimmed.slice(2).includes('$$')) state.math = '$$'
    if (trimmed.startsWith('\\[') && !trimmed.slice(2).includes('\\]')) state.math = '\\]'
    for (const tag of line.matchAll(blockTags)) {
      if (/^<\//.test(tag[0])) state.depth = Math.max(0, state.depth - 1)
      else if (!/\/>$/.test(tag[0])) state.depth++
    }
    return !state.math && !state.depth && !line.trim()
  }
  return false
}
const scanState = () => ({ fence: '', math: '', depth: 0 })
// 与 infOS 保持相同原则：只扫描新增完整行，最近一段保留为活跃尾部。
export class RemoteMarkdownSegmenter {
  constructor() { this.reset() }
  reset() { this.source = ''; this.lineStart = 0; this.searched = 0; this.cutoff = 0; this.boundary = 0; this.state = scanState(); this.blocks = []; this.committed = false }
  update(source, committed = false) {
    if (source === this.source && committed === this.committed) return this.result || []
    if (this.committed || !source.startsWith(this.source)) this.reset()
    this.source = source; this.committed = committed
    if (committed) {
      // 结束时仍不把未闭合围栏、公式或 HTML 当成完整结构执行。
      const state = scanState(); let offset = 0, safe = 0
      for (const line of source.match(/[^\n]*\n|[^\n]+$/g) || []) { stateLine(line, state); offset += line.length; if (!state.fence && !state.math && !state.depth) safe = offset }
      const parts = stableParts(source.slice(0, safe))
      if (safe < source.length) parts.push({ id: `raw:${safe}`, kind: 'raw', source: source.slice(safe) })
      this.result = parts; return parts
    }
    let newline
    while ((newline = source.indexOf('\n', this.searched)) !== -1) {
      const end = newline + 1, line = source.slice(this.lineStart, end)
      if (stateLine(line, this.state)) {
        if (this.boundary > this.cutoff) { this.blocks.push(...stableParts(source.slice(this.cutoff, this.boundary), this.cutoff)); this.cutoff = this.boundary }
        this.boundary = end
      }
      this.lineStart = end; this.searched = end
    }
    this.searched = source.length
    this.result = [...this.blocks]
    if (this.cutoff < source.length) this.result.push({ id: 'tail', kind: 'raw', source: source.slice(this.cutoff), streaming: true })
    return this.result
  }
}
function unwrap(source) { return source.match(/^\s*```(?:markdown|md)\s*\n([\s\S]*?)\n?```\s*$/i)?.[1] ?? source }
function stableParts(source, start = 0) {
  let offset = start
  return lexer.lexer(unwrap(source)).map(token => {
    const id = `block:${offset}`; offset += token.raw.length
    if (token.type === 'space') return null
    if (token.type === 'ppcFormula') return { id, kind: 'formula', source: token.text }
    if (token.type === 'code') {
      const language = (token.lang || 'text').trim().split(/\s+/)[0].toLowerCase()
      const kind = ['mermaid', 'flowchart', 'graph'].includes(language) ? 'diagram' : ['html', 'htm', 'svg'].includes(language) && /<(?:html|body|main|div|svg|section|article|!doctype)\b/i.test(token.text) ? 'web' : ['peromath', 'math', 'latex'].includes(language) ? 'formula' : 'code'
      return { id, kind, source: token.text, language }
    }
    if (token.type === 'html' && /<(?:html|body|main|div|svg|section|article|!doctype)\b/i.test(token.raw)) return { id, kind: 'web', source: token.raw }
    const link = token.type === 'paragraph' && token.raw.trim().match(/^!?\[([^\]\n]*)\]\((?:<([^>\n]+)>|([^\s]+))(?:\s+"[^"]*")?\)$/)
    if (link && (isResourceReference(link[2] || link[3]) || token.raw.trim().startsWith('!['))) return { id, kind: 'resource', source: link[2] || link[3], name: link[1] }
    const math = token.raw.trim().match(/^(?:\$\$|\\\[)\s*([\s\S]*?)\s*(?:\$\$|\\\])$/)
    // 保留块的语义，让流式拆分后的标题、列表仍有完整的阅读节奏。
    return { id, kind: math ? 'formula' : 'markdown', format: token.type, source: math ? math[1] : token.raw }
  }).filter(Boolean)
}
export { stableParts as richParts }

// 富文本仅产生净化后的正文；媒体与文件通过受控 Vue 组件加载。
export function renderRemoteMarkdown(source) {
  const renderer = {
    html: ({ text }) => /^<\/?(?:span|mark|small|kbd|sub|sup|u|s|b|i|strong|em)(?:\s[^>]*)?>$|^<br\s*\/?>$/i.test(text.trim()) ? text : `<code>${escapeRich(text)}</code>`,
    image: ({ href, text }) => `<a href="#" class="remote-resource-chip" data-resource-source="${escapeRich(href)}">${escapeRich(text || '查看图片')}</a>`,
    codespan: ({ text }) => /\.html?$/i.test(text) && isResourceReference(text) ? `<a href="#" class="remote-resource-chip" data-resource-source="${escapeRich(text)}"><code>${escapeRich(text)}</code></a>` : `<code>${escapeRich(text)}</code>`,
    link(token) {
      const label = this.parser.parseInline(token.tokens)
      if (isResourceReference(token.href)) return `<a href="#" class="remote-resource-chip" data-resource-source="${escapeRich(token.href)}">${label}</a>`
      try { const url = new URL(token.href); if (!['https:', 'http:'].includes(url.protocol) || url.username || url.password) throw new Error(); return `<a href="${escapeRich(url.href)}" rel="noopener noreferrer">${label}</a>` } catch { return label }
    },
  }
  const parser = new Marked({ gfm: true, breaks: true, renderer, tokenizer: { del: () => undefined } })
  let fence = ''
  const normalized = source.split('\n').map(line => { const match = line.match(/^[ \t]*(`{3,}|~{3,})/); if (match) { if (!fence) fence = match[1]; else if (match[1][0] === fence[0] && match[1].length >= fence.length) fence = ''; return line }; return fence ? line : line.replace(/^[ \t]{4,}(?=#{1,6}(?:[ \t]+|$))/, '') }).join('\n')
  const clean = DOMPurify.sanitize(parser.parse(normalized), { ADD_ATTR: ['data-resource-source'], FORBID_TAGS: ['img', 'audio', 'video', 'iframe', 'object', 'embed', 'style', 'script', 'form', 'button'], FORBID_ATTR: ['style'] })
  const fragment = document.createElement('template'); fragment.innerHTML = clean
  for (const input of fragment.content.querySelectorAll('input')) {
    input.disabled = true; input.tabIndex = -1
    if (input.type === 'checkbox') input.closest('li')?.classList.add('remote-task-item')
  }
  for (const table of fragment.content.querySelectorAll('table')) { const box = document.createElement('div'); box.className = 'remote-rich-table'; box.tabIndex = 0; box.setAttribute('aria-label', '表格，可左右滑动'); table.replaceWith(box); box.append(table) }
  return fragment.innerHTML
}

// HTML 没有同源、原生桥或网络权限；先解析再放进文档，CSP 位于任何源内容之前。
export function embeddedRemoteDocument(source, channel) {
  source = typeof source === 'string' ? source : ''
  // template 内容保持惰性，避免在净化之前由 HTML 解析文档预取外部图片。
  const template = document.createElement('template'); template.innerHTML = source
  for (const node of template.content.querySelectorAll('base,meta[http-equiv],iframe,frame,object,embed,link[rel="import"]')) node.remove()
  const attributes = tag => {
    const match = source.match(new RegExp(`<${tag}\\b([^>]*)>`, 'i'))
    if (!match) return ''
    const holder = document.createElement('template'); holder.innerHTML = `<div ${match[1]}></div>`
    return [...(holder.content.firstElementChild?.attributes || [])].filter(attr => ['class', 'style', 'lang', 'dir'].includes(attr.name)).map(attr => `${attr.name}="${escapeRich(attr.value)}"`).join(' ')
  }
  const policy = "default-src 'none'; img-src data: blob:; media-src data: blob:; style-src 'unsafe-inline'; script-src 'unsafe-inline'; font-src data:; connect-src 'none'; form-action 'none'; base-uri 'none'"
  const bridge = `<script>(()=>{let id=0,last=0;const send=()=>{id=0;const h=Math.min(720,Math.max(120,document.documentElement.scrollHeight));if(h!==last){last=h;parent.postMessage({type:'ppc-preview-size',channel:${JSON.stringify(channel)},height:h},'*')}};const watch=()=>{if(!id)id=requestAnimationFrame(send)};new ResizeObserver(watch).observe(document.documentElement);addEventListener('load',watch);watch()})()</script>`
  return `<!doctype html><html ${attributes('html')}><head><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="${policy}"><meta name="referrer" content="no-referrer"><style>html,body{margin:0;min-height:1px}*{box-sizing:border-box}img,video{max-width:100%}</style></head><body ${attributes('body')}>${template.innerHTML}${bridge}</body></html>`
}
