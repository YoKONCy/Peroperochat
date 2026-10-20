<template>
  <RoomDrawer title="文件预览" subtitle="infOS · 随身资料" variant="file" @close="$emit('close')">
    <div class="remote-resource remote-file-reader">
      <header class="file-identity">
        <RemoteFileCover :name="fileName" />
        <div><span class="file-origin"><RemoteIcon :name="resource.kind === 'workspace' ? 'folder' : 'attach'" />{{ resource.kind === 'workspace' ? '会话工作区' : '聊天附件' }}</span><h3 :title="fileName">{{ fileName }}</h3><p><span>{{ typeLabel }}</span><i /><span>{{ info ? formatFileSize(info.size) : '正在连接' }}</span></p></div>
        <RemoteIcon class="file-identity-ornament" name="orbit" />
      </header>
      <div ref="readingBody" class="file-reading-body">
        <RemoteFileState v-if="loading" title="把文件轻轻展开…" description="正在读取远程内容" loading />
        <RemoteFileState v-else-if="error" icon="alert" title="暂时没能展开" :description="error"><span class="file-state-hint">仍可在下方保存原件</span></RemoteFileState>
        <RemotePdf v-else-if="kind === 'pdf' && info" :resource="resource" :info="info" @page-change="resetReadingScroll" />
        <template v-else-if="media && ['image', 'audio', 'video'].includes(kind)">
          <div class="file-view-bar"><span>{{ kind === 'image' ? '画面预览' : kind === 'audio' ? '音频播放' : '视频播放' }}</span><div v-if="kind === 'image'" class="file-zoom-actions"><button :disabled="imageZoom === 1" aria-label="缩小图片" @click="imageZoom = Math.max(1, imageZoom - .5)"><RemoteIcon name="zoom-out" /></button><button @click="imageZoom = 1">{{ Math.round(imageZoom * 100) }}%</button><button :disabled="imageZoom >= 3" aria-label="放大图片" @click="imageZoom += .5"><RemoteIcon name="zoom-in" /></button></div></div>
          <div class="file-media-stage" :class="`is-${kind}`" tabindex="0" :aria-label="`${fileName}预览区`">
            <img v-if="kind === 'image'" :src="media.url" :alt="fileName" :class="{ 'is-zoomed': imageZoom > 1 }" :style="imageZoom > 1 ? { width: `${imageZoom * 100}%` } : {}" @error="playbackError" />
            <div v-else-if="kind === 'audio'" class="file-audio-player"><div class="file-audio-art" aria-hidden="true"><RemoteIcon name="audio" /><span /><span /><span /><span /><span /></div><p>{{ fileName }}</p><audio :src="media.url" controls preload="metadata" @error="playbackError" /></div>
            <video v-else :src="media.url" controls playsinline preload="metadata" @error="playbackError" />
          </div>
        </template>
        <section v-else-if="text !== null" class="file-text-preview">
          <div class="file-view-bar"><span>文本预览</span><button :aria-pressed="wrapText" @click="wrapText = !wrapText"><RemoteIcon name="wrap" />{{ wrapText ? '自动换行' : '保留原行' }}</button></div>
          <pre class="file-code" :class="{ 'is-unwrapped': !wrapText }" tabindex="0" aria-label="文件正文">{{ text }}</pre>
        </section>
        <section v-else-if="sections.length" class="file-document-reader">
          <nav v-if="sections.length > 1 && sections.length <= 5" class="file-section-tabs" aria-label="文档章节"><button v-for="(section, index) in sections" :key="index" :aria-pressed="sectionIndex === index" :class="{ selected: sectionIndex === index }" @click="sectionIndex = index">{{ section.title }}</button></nav>
          <RemoteSelect v-else-if="sections.length > 5" :model-value="String(sectionIndex)" label="选择文档章节" :options="sections.map((section, index) => ({ value: String(index), label: section.title }))" @update:model-value="sectionIndex = Number($event)" />
          <div class="file-view-bar"><span>{{ activeSection.title }}</span><small>{{ activeSection.rows ? '表格速览' : '正文速览' }}</small></div>
          <div v-if="activeSection.rows" class="file-sheet" tabindex="0" aria-label="工作表内容"><table><thead><tr><th scope="col">#</th><th v-for="(_, c) in sheetColumns" :key="c" scope="col">{{ columnLabel(c) }}</th></tr></thead><tbody><tr v-for="(row, r) in activeSection.rows" :key="r"><th scope="row">{{ r + 1 }}</th><td v-for="(_, c) in sheetColumns" :key="c">{{ row[c] || '' }}</td></tr></tbody></table></div>
          <article v-else class="file-document-paper"><span class="file-paper-marker" aria-hidden="true"><RemoteIcon name="diamond" /></span><p>{{ activeSection.text }}</p><small>{{ sectionIndex + 1 }} / {{ sections.length }}</small></article>
          <details class="file-reading-note"><summary>关于内容速览<RemoteIcon name="down" /></summary><p>显示正文与表格内容，完整排版、图片和公式请保存原件查看。表格最多显示 200 行、30 列。</p></details>
        </section>
        <section v-else-if="archive" class="file-archive-reader">
          <div class="file-view-bar"><span>压缩包目录</span><small>{{ archive.count }} 项</small></div>
          <label class="file-inline-search"><RemoteIcon name="search" /><input v-model="archiveQuery" placeholder="查找目录中的文件" maxlength="200" aria-label="筛选压缩包目录" /><button v-if="archiveQuery" aria-label="清空筛选" @click="archiveQuery = ''"><RemoteIcon name="close" /></button></label>
          <ul v-if="archiveEntries.length" class="file-archive-list"><li v-for="(entry, index) in archiveEntries" :key="index"><span class="file-folder-icon" v-if="entry.directory"><RemoteIcon name="folder" /></span><RemoteFileCover v-else :name="entry.name" /><span :title="entry.name">{{ entry.name }}<small>{{ entry.directory ? '文件夹' : formatFileSize(entry.size) }}</small></span></li></ul>
          <RemoteFileState v-else icon="search" title="没有找到这个文件" description="试试短一点的名字" />
          <button v-if="archiveMatches.length > archiveLimit" class="file-load-more" @click="archiveLimit += 80">再展开一些<RemoteIcon name="down" /><small>{{ archiveEntries.length }} / {{ archiveMatches.length }}</small></button>
          <p class="file-reading-caption">仅浏览目录 · 最多显示 600 项</p>
        </section>
        <RemoteFileState v-else title="先把它收进口袋吧" :description="notice || '这个格式适合保存后查看'" icon="download" />
        <p v-if="notice && (media || sections.length || archive)" class="file-inline-notice"><RemoteIcon name="files" />{{ notice }}</p>
      </div>
    </div>
    <template #footer>
      <div class="file-save-dock">
        <p v-if="saveError" class="file-save-message is-error" role="alert"><RemoteIcon name="alert" />{{ saveError }}</p>
        <p v-else-if="saved" class="file-save-message" role="status"><RemoteIcon name="check" />{{ saved }}</p>
        <template v-if="saving"><div class="file-transfer-label"><span role="status">{{ progress.phase === 'saving' ? '选择位置，收好这份文件' : '正在把文件带到身边…' }}</span><strong>{{ progress.phase === 'saving' ? '保存中' : percent === null ? formatFileSize(progress.received) : `${percent}%` }}</strong></div><div class="file-transfer-track" :class="{ indeterminate: percent === null }" role="progressbar" aria-label="下载进度" :aria-valuenow="percent" aria-valuemin="0" aria-valuemax="100"><i :style="{ width: `${percent ?? 24}%` }" /></div></template>
        <div class="file-save-actions"><span><RemoteIcon :name="saving ? 'download' : 'shield'" />{{ saving ? progress.phase === 'saving' ? '正在交给系统保存' : `${formatFileSize(progress.received)} / ${formatFileSize(progress.total)}` : '保留完整原件' }}</span><button v-if="saving" class="file-button secondary" :disabled="cancelling" @click="cancel">{{ cancelling ? '正在取消…' : '取消保存' }}</button><button v-else class="file-button primary" @click="save"><RemoteIcon name="download" />保存到设备</button></div>
      </div>
    </template>
  </RoomDrawer>
</template>
<script setup>
import { ref, computed, watch, onMounted, onBeforeUnmount, defineAsyncComponent } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteFileCover from './RemoteFileCover.vue'
import RemoteFileState from './RemoteFileState.vue'
import RemoteSelect from './RemoteSelect.vue'
import '../../styles/remote-files.css'
import { loadRemoteResource } from '../../services/remoteMedia'
import { remoteClient } from '../../services/remote'
import { currentToken, assertToken } from '../../stores/mode'
import { documentKind, formatFileSize, officePreview, csvPreview } from '../../services/remoteDocuments'

const RemotePdf = defineAsyncComponent(() => import('./RemotePdf.vue'))
const props = defineProps({ resource: Object, name: String })
defineEmits(['close'])
const token = currentToken(), info = ref(null), kind = ref(''), media = ref(null), loading = ref(true), error = ref(''), text = ref(null)
const sections = ref([]), archive = ref(null), notice = ref(''), saving = ref(false), cancelling = ref(false), saveError = ref(''), saved = ref('')
const fileName = computed(() => props.name || info.value?.name || 'infOS文件')
const imageZoom = ref(1), wrapText = ref(true), sectionIndex = ref(0), archiveQuery = ref('')
const readingBody = ref(null), archiveLimit = ref(80)
const activeSection = computed(() => sections.value[sectionIndex.value] || sections.value[0])
const sheetColumns = computed(() => Array.from({ length: Math.max(1, ...(activeSection.value?.rows || []).map(row => row.length)) }))
const archiveMatches = computed(() => (archive.value?.entries || []).filter(entry => entry.name.toLowerCase().includes(archiveQuery.value.trim().toLowerCase())))
const archiveEntries = computed(() => archiveMatches.value.slice(0, archiveLimit.value))
const typeLabel = computed(() => ({ pdf: 'PDF 文档', docx: 'Word 文档', xlsx: 'Excel 表格', pptx: '幻灯片', csv: 'CSV 表格', text: '文本文件', image: '图片', audio: '音频', video: '视频', zip: '压缩文件' }[kind.value] || '远程文件'))
function columnLabel(index) { return index < 26 ? String.fromCharCode(65 + index) : `A${String.fromCharCode(65 + index - 26)}` }
function resetReadingScroll() { readingBody.value?.scrollTo({ top: 0, behavior: 'auto' }) }
watch(sectionIndex, resetReadingScroll)
watch(archiveQuery, () => { archiveLimit.value = 80 })
const progress = ref({ phase: 'downloading', received: 0, total: null })
const percent = computed(() => progress.value.total > 0 ? Math.min(100, Math.floor(progress.value.received / progress.value.total * 100)) : null)
let disposed = false, worker, stopListen, downloadId, rejectDocument, documentTimer
function playbackError() { error.value = '手机无法预览此格式，可保存原件后使用其他应用查看' }
async function cancel() {
  if (!downloadId || cancelling.value) return
  cancelling.value = true
  try { await remoteClient.cancelDownload(downloadId, token) }
  catch (failure) { if (!disposed) { saveError.value = failure.message; cancelling.value = false } }
}
async function save() {
  if (saving.value) return
  saving.value = true; cancelling.value = false; saveError.value = ''; saved.value = ''
  downloadId = crypto.randomUUID(); progress.value = { phase: 'downloading', received: 0, total: info.value?.size }
  try {
    const result = await remoteClient.downloadResource(props.resource, fileName.value, downloadId, token)
    if (!disposed) saved.value = result.data?.saved ? result.data.path ? `已保存：${result.data.path}` : '文件已保存' : '已取消保存'
  } catch (failure) { if (!disposed) { if (failure.code === 'DOWNLOAD_CANCELLED') saved.value = '已取消保存'; else saveError.value = failure.message } }
  finally { downloadId = null; if (!disposed) { saving.value = false; cancelling.value = false } }
}
function readArchive(bytes) {
  return new Promise((resolve, reject) => {
    rejectDocument = reject
    worker = new Worker(new URL('../../services/remoteDocument.worker.js', import.meta.url), { type: 'module' })
    documentTimer = setTimeout(() => { worker?.terminate(); worker = null; rejectDocument = null; reject(new Error('文档预览超时，请下载原件查看')) }, 10000)
    worker.onmessage = ({ data }) => { clearTimeout(documentTimer); worker.terminate(); worker = null; rejectDocument = null; data.error ? reject(new Error(data.error)) : resolve(data) }
    worker.onerror = () => { clearTimeout(documentTimer); worker?.terminate(); worker = null; rejectDocument = null; reject(new Error('文档无法预览，请下载原件查看')) }
    worker.postMessage({ bytes, kind: kind.value }, [bytes])
  })
}
onMounted(async () => {
  try {
    stopListen = await remoteClient.listen(event => { if (event.kind === 'download' && event.data?.id === downloadId) progress.value = event.data })
    if (disposed) { stopListen(); return }
    info.value = (await remoteClient.resourceInfo(props.resource, token)).data
    if (disposed) return
    const mime = info.value.mimeType.split(';')[0].trim().toLowerCase()
    kind.value = documentKind(fileName.value, mime)
    if (info.value.size === 0) { kind.value = 'binary'; notice.value = '这是一个空文件。'; return }
    if (kind.value === 'pdf') {
      if (info.value.size > 512 * 1024 * 1024) notice.value = 'PDF 超过预览范围，请保存原件查看'
      else return
      kind.value = 'binary'; return
    }
    if (kind.value === 'binary') { notice.value = '此格式暂不支持应用内预览，可直接保存原件。'; return }
    const isText = ['text', 'csv'].includes(kind.value), size = info.value.size
    if (size === 0) { notice.value = '这是一个空文件。'; return }
    if (!isText && (size === null || size > 8 * 1024 * 1024)) { notice.value = '此文件较大，请保存原件后查看。PDF 支持分段预览，其他格式当前预览上限为 8 MiB。'; return }
    const partial = isText && (size === null || size > 256 * 1024)
    if (partial && !info.value.ranges) { notice.value = '服务端不支持分段读取，请保存原件查看。'; return }
    const loaded = await loadRemoteResource(props.resource, { range: partial ? { start: 0, end: 256 * 1024 - 1 } : null })
    if (disposed) { loaded.release(); return }
    media.value = loaded
    if (isText) {
      const content = (await loaded.blob.text()).slice(0, 256 * 1024)
      if (disposed) return
      if (kind.value === 'csv') sections.value = csvPreview(content); else text.value = content
      if (partial) notice.value = '仅预览前 256 KiB，保存原件可获取完整内容。'
    } else if (['docx', 'xlsx', 'pptx', 'zip'].includes(kind.value)) {
      const buffer = await loaded.blob.arrayBuffer(); if (disposed) return
      const data = await readArchive(buffer)
      if (disposed) return
      assertToken(token, 'remote')
      if (kind.value === 'zip') archive.value = data
      else { sections.value = officePreview(kind.value, data.files); if (!sections.value.length) notice.value = '未找到可预览的正文，请保存原件查看。' }
      if (data.truncated) notice.value = '文档内容较多，仅预览前 50 个文档分部；完整内容请保存原件查看。'
      loaded.release(); media.value = null
    }
  } catch (failure) { media.value?.release(); media.value = null; if (!disposed) error.value = failure.message }
  finally { if (!disposed) loading.value = false }
})
onBeforeUnmount(() => {
  disposed = true; stopListen?.(); clearTimeout(documentTimer); worker?.terminate(); rejectDocument?.(new Error('预览已关闭')); media.value?.release()
  if (downloadId) remoteClient.cancelDownload(downloadId, token).catch(() => {})
})
</script>
