import { marked } from 'marked'
import DOMPurify from 'dompurify'
import { open } from '@tauri-apps/plugin-shell'

export function remoteMarkdown(value) {
  const html = DOMPurify.sanitize(marked.parse(typeof value === 'string' ? value : '', { gfm: true, breaks: true }), {
    USE_PROFILES: { html: true }, FORBID_TAGS: ['img', 'video', 'audio', 'iframe', 'object', 'embed', 'style', 'form', 'input', 'button'],
  })
  const fragment = document.createElement('template')
  fragment.innerHTML = html
  for (const link of fragment.content.querySelectorAll('a')) {
    try {
      const url = new URL(link.getAttribute('href'))
      if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password) throw new Error('链接协议不支持')
      link.href = url.href
      link.target = '_blank'
      link.rel = 'noopener noreferrer'
    } catch { link.removeAttribute('href') }
  }
  return fragment.innerHTML
}
export const remoteStatus = value => ({ running: '执行中', pending: '待处理', queued: '排队中', waiting_input: '等待回复', waiting_approval: '等待审批', waiting: '等待中', blocked: '受阻', thinking: '思考中', generating: '生成中', tool_failed: '工具执行失败', completed: '已完成', failed: '执行失败', paused: '已暂停', cancelled: '已取消', interrupted: '已中断', calling: '执行中', assembling: '准备参数', open: '进行中', committed: '已保存', unknown: '结果待核对', sending: '发送中', observing: '正在接收' }[value] || value || '')
export function remoteItems(data) { const values = Array.isArray(data) ? data : data?.items || data?.requests || data?.tasks || data?.entries || data?.nodes; return Array.isArray(values) ? values : [] }
export async function openRemoteLink(event) {
  const link = event.target.closest('a[href]')
  if (!link) return
  event.preventDefault()
  const url = new URL(link.href)
  if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password) throw new Error('此链接不支持直接打开')
  await open(url.href)
}
