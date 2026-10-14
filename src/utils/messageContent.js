const METADATA_TAGS = ['PEROCUE', 'MEMORY', 'CLICK_MESSAGES', 'IDLE_MESSAGES', 'BACK_MESSAGES', 'REMINDER', 'TOPIC']

export function messageText(content) {
  if (typeof content === 'string') return content
  if (Array.isArray(content)) return content.filter(part => part.type === 'text').map(part => part.text || '').join('\n')
  return ''
}

export function visibleMessage(content) {
  let text = messageText(content)
  if (text === '__loading__') return '正在思考...'
  for (const tag of METADATA_TAGS) {
    text = text.replace(new RegExp(`<${tag}>[\\s\\S]*?<\\/${tag}>`, 'g'), '')
    const pending = text.indexOf(`<${tag}>`)
    if (pending !== -1) text = text.slice(0, pending)
  }
  return text.replace(/<(nit(?:-[0-9a-fA-F]{4})?)>[\s\S]*?<\/\1>/gi, '')
    .replace(/\[\[\[NIT_CALL\]\]\][\s\S]*?\[\[\[NIT_END\]\]\]/g, '').trim()
}

export function displayMessage(message) {
  if (message.status === 'pending' && !messageText(message.content)) return { ...message, content: '__loading__' }
  if (message.error && !messageText(message.content)) return { ...message, content: message.error }
  return message
}
