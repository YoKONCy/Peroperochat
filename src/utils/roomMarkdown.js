import { Marked } from 'marked'
import { expressionExtensions } from './parenthetical'
import DOMPurify from 'dompurify'
import { visibleMessage } from './messageContent'

const parser = new Marked({ gfm: true, async: false, breaks: true, extensions: expressionExtensions })

export function roomMarkdown(content) {
  const html = DOMPurify.sanitize(parser.parse(visibleMessage(content), { async: false }), { USE_PROFILES: { html: true } })
  const fragment = document.createElement('template')
  fragment.innerHTML = html
  for (const link of fragment.content.querySelectorAll('a')) {
    link.target = '_blank'
    link.rel = 'noopener noreferrer'
  }
  for (const table of fragment.content.querySelectorAll('table')) {
    const wrapper = document.createElement('div')
    wrapper.className = 'journal-table'
    wrapper.tabIndex = 0
    table.replaceWith(wrapper)
    wrapper.append(table)
  }
  return fragment.innerHTML
}
