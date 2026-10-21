import { remoteClient } from './remote'
import { loadRemoteResource } from './remoteMedia'
import { nativeResource, externalResourceUrl } from './remoteResources'
import { documentKind, officePreview, csvPreview } from './remoteDocuments'
import { currentToken, assertToken, onModeInvalidated } from '../stores/mode'

export const mediaKinds = new Set(['image', 'audio', 'video'])
const textKinds = new Set(['text', 'csv', 'markdown', 'html', 'svg', 'rtf'])
const officeKinds = new Set(['docx', 'xlsx', 'pptx', 'odt', 'ods', 'odp', 'epub', 'zip'])
export const remoteFileLabel = kind => ({ image: '图片 / GIF', audio: '音频', video: '视频', pdf: 'PDF 文档', markdown: 'Markdown', html: 'HTML 页面', svg: 'SVG 矢量图', doc: 'Word 文档', docx: 'Word 文档', xlsx: '电子表格', pptx: '幻灯片', csv: 'CSV 表格', odt: '开放文本文档', ods: '开放电子表格', odp: '开放演示文稿', epub: '电子书', rtf: '富文本文档', text: '文本 / 代码', zip: '压缩包' }[kind] || '文件')
export async function remoteFileInfo(resource, token = currentToken()) {
  assertToken(token, 'remote')
  if (resource.kind === 'external') return { name: resource.name || new URL(resource.url).pathname.split('/').pop() || '在线文件', mimeType: resource.mimeType || 'application/octet-stream', size: resource.size ?? null, ranges: false }
  return (await remoteClient.resourceInfo(nativeResource(resource), token)).data
}
async function readBytes(resource, info, limit, signal, partial = false) {
  const token = currentToken(); assertToken(token, 'remote'); signal?.throwIfAborted()
  if (resource.kind === 'external') {
    const response = await fetch(externalResourceUrl(resource.url), { signal, credentials: 'omit', referrerPolicy: 'no-referrer' })
    if (!response.ok || !response.body) throw new Error('在线文件暂时无法读取，可打开来源或保存原件')
    const reader = response.body.getReader(), chunks = []; let size = 0
    try { while (true) { const { done, value } = await reader.read(); if (done) break; if (size + value.length > limit) { if (!partial) throw new Error('文件超过本次预览范围，请保存原件'); chunks.push(value.slice(0, limit - size)); size = limit; break }; chunks.push(value); size += value.length } }
    finally { await reader.cancel().catch(() => {}) }
    assertToken(token, 'remote'); signal?.throwIfAborted(); return new Blob(chunks, { type: info.mimeType })
  }
  const size = info.size, native = nativeResource(resource)
  if (size !== null && size > limit && !partial) throw new Error('文件较大，请保存原件后查看')
  if (partial && (size === null || size > limit) && !info.ranges) throw new Error('服务端不支持分段预览，请保存原件查看')
  const end = Math.min(size ?? limit, limit), chunks = []
  if (size !== null && size <= 8 * 1024 * 1024 && (!partial || size <= limit) || size === null && limit <= 8 * 1024 * 1024 && !partial) {
    const loaded = await loadRemoteResource(native); try { signal?.throwIfAborted(); return loaded.blob } finally { loaded.release() }
  }
  if (!info.ranges) throw new Error('服务端不支持分段读取，请保存原件查看')
  for (let start = 0; start < end; start += 2 * 1024 * 1024) {
    signal?.throwIfAborted(); assertToken(token, 'remote')
    const stop = Math.min(end - 1, start + 2 * 1024 * 1024 - 1)
    const part = await remoteClient.resource(native, { range: { start, end: stop }, token })
    const span = part.contentRange?.match(/^bytes (\d+)-(\d+)\/(\d+)$/)
    if (!span || Number(span[1]) !== start || Number(span[2]) > stop || Number(span[2]) - start + 1 !== part.bytes.length || size !== null && (Number(span[3]) !== size || Number(span[2]) !== stop)) throw new Error('文件内容已变化，请重新打开')
    chunks.push(new Uint8Array(part.bytes))
    if (Number(span[2]) + 1 === Number(span[3])) break
  }
  signal?.throwIfAborted(); assertToken(token, 'remote'); return new Blob(chunks, { type: info.mimeType })
}
function archivePreview(bytes, kind, signal) {
  return new Promise((resolve, reject) => {
    const worker = new Worker(new URL('./remoteDocument.worker.js', import.meta.url), { type: 'module' })
    const finish = (error, data) => { clearTimeout(timer); worker.terminate(); signal?.removeEventListener('abort', abort); error ? reject(error) : resolve(data) }
    const abort = () => finish(new Error('文件预览已关闭'))
    const timer = setTimeout(() => finish(new Error('文档解析超时，可保存原件查看')), 10000)
    worker.onmessage = ({ data }) => finish(data.error ? new Error(data.error) : null, data)
    worker.onerror = () => finish(new Error('文档损坏或格式暂不支持，可保存原件查看'))
    signal?.addEventListener('abort', abort, { once: true }); if (signal?.aborted) { abort(); return }
    worker.postMessage({ bytes, kind }, [bytes])
  })
}
const previewJobs = []
let previewLoads = 0
onModeInvalidated(() => { for (const job of previewJobs.splice(0)) { job.options.signal?.removeEventListener('abort', job.abort); job.reject(new Error('运行模式已改变')) } })
function pumpPreviews() {
  while (previewLoads < 2 && previewJobs.length) {
    const job = previewJobs.shift(); job.options.signal?.removeEventListener('abort', job.abort)
    try { assertToken(job.token, 'remote'); job.options.signal?.throwIfAborted() } catch (error) { job.reject(error); continue }
    previewLoads++
    prepareFile(job.resource, job.info, job.options).then(result => {
      try { assertToken(job.token, 'remote'); job.options.signal?.throwIfAborted(); job.resolve(result) }
      catch (error) { result.media?.release(); job.reject(error) }
    }, job.reject).finally(() => { previewLoads--; pumpPreviews() })
  }
}
export function prepareRemoteFile(resource, info, options = {}) {
  const token = currentToken(); assertToken(token, 'remote')
  return new Promise((resolve, reject) => {
    const job = { resource, info, options, token, resolve, reject }
    job.abort = () => { const index = previewJobs.indexOf(job); if (index >= 0) { previewJobs.splice(index, 1); reject(new Error('预览已关闭')) } }
    if (options.signal?.aborted) { reject(new Error('预览已关闭')); return }
    options.signal?.addEventListener('abort', job.abort, { once: true }); previewJobs.push(job); pumpPreviews()
  })
}
async function prepareFile(resource, info, { compact = false, signal } = {}) {
  const kind = documentKind(info.name || resource.name, info.mimeType.split(';')[0].toLowerCase()), result = { kind, info, text: null, media: null, sections: [], archive: null, notice: '' }
  if (info.size === 0) return { ...result, notice: '这是一个空文件' }
  if (kind === 'pdf') return result
  if (resource.kind === 'external' && mediaKinds.has(kind)) return { ...result, media: { url: externalResourceUrl(resource.url), release() {} } }
  const media = mediaKinds.has(kind), text = textKinds.has(kind)
  if (!media && !text && !officeKinds.has(kind) && kind !== 'doc') return { ...result, notice: '此格式可保存原件，再使用设备上的应用查看' }
  const limit = text ? compact ? 64 * 1024 : 256 * 1024 : media ? compact ? 8 * 1024 * 1024 : 100 * 1024 * 1024 : 8 * 1024 * 1024
  const blob = await readBytes(resource, info, limit, signal, text)
  signal?.throwIfAborted()
  if (media) {
    // Blob 仅交给被动媒体元素；不让远程 MIME 产生可执行网页。
    const mime = /^(image\/(png|jpeg|gif|webp|avif|bmp|x-icon)|audio\/[\w.+-]+|video\/[\w.+-]+)$/.test(info.mimeType.split(';')[0]) ? info.mimeType : ({ image: 'image/png', audio: 'audio/mpeg', video: 'video/mp4' }[kind])
    const url = URL.createObjectURL(new Blob([blob], { type: mime })); result.media = { url, release() { URL.revokeObjectURL(url) } }; return result
  }
  if (text) {
    const value = await blob.text(); signal?.throwIfAborted()
    if (kind === 'csv') result.sections = csvPreview(value)
    else if (kind === 'rtf') result.text = rtfText(value)
    else result.text = value
    if (info.size === null || info.size > limit) result.notice = `仅预览前 ${limit / 1024} KiB，完整内容可保存原件查看`
    return result
  }
  if (kind === 'doc') {
    const data = await readLegacyDoc(await blob.arrayBuffer(), signal)
    return { ...result, sections: [{ title: '正文速览', text: data }], notice: '仅提取正文，不执行宏；完整排版请保存原件查看' }
  }
  const data = await archivePreview(await blob.arrayBuffer(), kind, signal); signal?.throwIfAborted()
  if (kind === 'zip') result.archive = data
  else result.sections = officePreview(kind, data.files)
  result.notice = data.truncated ? '内容较多，仅展示部分文档分部；完整内容请保存原件' : kind === 'zip' ? '仅浏览目录，不执行压缩包内文件' : '正文与表格速览，不执行宏或外部引用；完整排版请保存原件'
  return result
}
function rtfText(source) {
  return source.replace(/\{\\\*[^{}]*(?:\{[^{}]*\}[^{}]*)*\}/g, '').replace(/\\(?:fonttbl|colortbl|stylesheet|info)[\s\S]*?\}/g, '').replace(/\\u(-?\d+)\??/g, (_, value) => String.fromCharCode((Number(value) + 65536) % 65536)).replace(/\\'([0-9a-f]{2})/gi, (_, hex) => new TextDecoder('windows-1252').decode(Uint8Array.of(parseInt(hex, 16)))).replace(/\\(?:par|line)\b\s?/g, '\n').replace(/\\tab\b\s?/g, '\t').replace(/\\[a-z]+-?\d*\s?/gi, '').replace(/[{}]/g, '').slice(0, 256 * 1024)
}
async function readLegacyDoc(bytes, signal) {
  // 旧 Word 的复合文档解析放在 Worker 内，避免阻塞消息滚动。
  return new Promise((resolve, reject) => {
    const worker = new Worker(new URL('./remoteLegacyDoc.worker.js', import.meta.url), { type: 'module' })
    const finish = (error, text) => { clearTimeout(timer); worker.terminate(); signal?.removeEventListener('abort', abort); error ? reject(error) : resolve(text) }
    const abort = () => finish(new Error('文件预览已关闭')), timer = setTimeout(() => finish(new Error('旧 Word 文档解析超时，请保存原件')), 8000)
    worker.onmessage = ({ data }) => finish(data.error ? new Error(data.error) : null, data.text)
    worker.onerror = () => finish(new Error('旧 Word 文档无法速览，请保存原件'))
    signal?.addEventListener('abort', abort, { once: true }); if (signal?.aborted) { abort(); return }; worker.postMessage(bytes, [bytes])
  })
}
