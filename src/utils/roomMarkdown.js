import { marked } from 'marked'
import DOMPurify from 'dompurify'
import { visibleMessage } from './messageContent'

export function roomMarkdown(content) {
  const html = DOMPurify.sanitize(marked.parse(visibleMessage(content), { gfm: true, breaks: true }), { USE_PROFILES: { html: true } })
  const fragment = document.createElement('template')
  fragment.innerHTML = html
  for (const link of fragment.content.querySelectorAll('a')) {
    link.target = '_blank'
    link.rel = 'noopener noreferrer'
  }
  return fragment.innerHTML
}
