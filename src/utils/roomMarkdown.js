import { Marked } from 'marked'
import { expressionExtensions } from './parenthetical'
import DOMPurify from 'dompurify'
import { visibleMessage } from './messageContent'

const parser = new Marked({ gfm: true, breaks: true, extensions: expressionExtensions })

export function roomMarkdown(content) {
  const html = DOMPurify.sanitize(parser.parse(visibleMessage(content)), { USE_PROFILES: { html: true } })
  const fragment = document.createElement('template')
  fragment.innerHTML = html
  for (const link of fragment.content.querySelectorAll('a')) {
    link.target = '_blank'
    link.rel = 'noopener noreferrer'
  }
  return fragment.innerHTML
}
