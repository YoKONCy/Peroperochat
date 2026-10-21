import { remoteClient } from './remote'
import { currentToken, assertToken } from '../stores/mode'

const fileExtension = /\.(?:html?|svg|png|jpe?g|gif|webp|avif|bmp|ico|mp4|webm|mov|m4v|mkv|ogv|mp3|wav|ogg|oga|opus|m4a|aac|flac|pdf|zip|7z|rar|tar|gz|docx?|docm|xlsx?|xlsm|xlsb|pptx?|pptm|odt|ods|odp|rtf|epub|txt|md|markdown|csv|json|xml|ya?ml|toml|log)(?:[?#]|$)/i
export function isResourceReference(value = '') {
  return /^(?:attachment:|workspace:|resource:|arca:|file:|[a-z]:[\\/]|\.\.?\/)/i.test(value) ||
    (/^https?:\/\//i.test(value) ? fileExtension.test(value) || /\/api\/attachments\/[^/]+\/content(?:[?#]|$)/.test(value) : /^(?:\/|(?:[^:/#\s]+\/)*[^:/#\s]+\.[\w]+(?:#L\d+)?$)/.test(value))
}
export function externalResourceUrl(value) {
  const url = new URL(value)
  if (!['https:', 'http:'].includes(url.protocol) || url.username || url.password) throw new Error('此文件地址不能直接打开')
  return url.href
}
export function nativeResource(value) {
  if (value.kind === 'attachment') return { kind: 'attachment', id: String(value.id) }
  if (value.kind === 'workspace') return { kind: 'workspace', agentId: value.agentId, threadId: value.threadId, path: value.path, version: value.version }
  if (value.kind === 'reference') return { kind: 'reference', agentId: value.agentId, threadId: value.threadId, taskId: value.taskId || null, source: value.source }
  throw new Error('此资源需要先解析来源')
}
function fromDescriptor(value, context = {}) {
  const meta = { name: value.name, mimeType: value.mimeType, size: value.sizeBytes ?? value.size, caption: value.caption }
  if (value.id) return { ...meta, kind: 'attachment', id: String(value.id) }
  const origin = value.origin
  if (origin?.type === 'workspace') return { ...meta, kind: 'reference', agentId: origin.agentId || context.agentId, threadId: origin.threadId || context.threadId, taskId: origin.taskId || context.taskId, source: `workspace:${encodeURIComponent(origin.value)}` }
  if (origin?.type === 'url') return { ...meta, kind: 'external', url: externalResourceUrl(origin.value) }
  if (origin?.type === 'arca') return { ...meta, kind: 'knowledge', id: origin.value }
  throw new Error('文件来源不完整，请刷新会话后重试')
}
export async function resolveRemoteResource(value, context = {}, token = currentToken()) {
  assertToken(token, 'remote')
  if (typeof value === 'object' && value) {
    if (value.kind === 'knowledge') return value
    if (['attachment', 'workspace', 'reference'].includes(value.kind)) return { ...nativeResource(value), name: value.name, mimeType: value.mimeType, size: value.size, caption: value.caption }
    if (value.kind === 'external') return { ...value, url: externalResourceUrl(value.url) }
    if (value.source) return resolveRemoteResource(value.source, { ...context, ...value.context }, token)
    return fromDescriptor(value, context)
  }
  const source = String(value || '').trim()
  if (!source || source.length > 2048) throw new Error('文件引用为空或过长')
  // 即使附件看起来有 ID，也由服务端解析入口核对当前会话权限。
  if (!context.agentId || !context.threadId) throw new Error('此文件缺少所属会话，请从原会话打开')
  const { data } = await remoteClient.resolveResource(context, source, token)
  assertToken(token, 'remote')
  return fromDescriptor(data, context)
}

const toolResourceCache = new WeakMap()
export function filesFromTool(node) {
  const result = node.kind === 'tool-result' ? node : node.resultNode
  if (!result || result.props.isError) return []
  const cached = toolResourceCache.get(result)
  if (cached) return cached
  let value = result.props.result
  try {
    if (typeof value === 'string') value = JSON.parse(value)
    if (typeof value?.ok === 'boolean' && typeof value.output === 'string') { if (!value.ok) return []; value = JSON.parse(value.output) }
    const resources = []
    if (node.props.name === 'send_file' && value?.success === true && value.resource) resources.push(value.resource)
    if (['read_image', 'document', 'take_screenshot', 'screen_capture', 'browser_screenshot', 'browser_page_image', 'automation_execute'].includes(node.props.name)) {
      const images = value?.imagePreview ? [value.imagePreview] : (value?.images || value?.screenshots || []).slice(0, 8).map(item => item.imagePreview)
      resources.push(...images.filter(item => item?.kind === 'image'))
    }
    const files = resources.filter(item => item && typeof item.id === 'string' && /^[\w-]{1,128}$/.test(item.id) && typeof item.name === 'string' && typeof item.mimeType === 'string' && Number.isFinite(item.sizeBytes) && item.sizeBytes >= 0)
    toolResourceCache.set(result, files); return files
  } catch { toolResourceCache.set(result, []); return [] }
}
