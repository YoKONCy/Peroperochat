<template>
  <RoomDrawer title="文件预览" subtitle="infOS · 随身资料" variant="file" @close="$emit('close')">
    <div class="remote-resource remote-file-reader">
      <header class="file-identity">
        <RemoteFileCover :name="fileName" />
        <div><span class="file-origin"><RemoteIcon :name="resource.kind === 'external' ? 'globe' : resource.kind === 'attachment' ? 'attach' : 'folder'" />{{ resource.kind === 'external' ? '在线资源' : resource.kind === 'attachment' ? '聊天附件' : '会话工作区' }}</span><h3 :title="fileName">{{ fileName }}</h3><p><span>{{ typeLabel }}</span><i /><span>{{ info ? formatFileSize(info.size) : '正在连接' }}</span></p></div>
        <RemoteIcon class="file-identity-ornament" name="orbit" />
      </header>
      <div ref="readingBody" class="file-reading-body">
        <RemoteFileState v-if="loading" title="把文件轻轻展开…" description="正在读取远程内容" loading />
        <RemoteFileState v-else-if="error" icon="alert" title="暂时没能展开" :description="error"><span class="file-state-hint">仍可在下方保存原件</span><button class="file-button secondary" @click="load">重新读取</button></RemoteFileState>
        <RemotePdf v-else-if="kind === 'pdf' && info && nativeInput" :resource="nativeInput" :info="info" @page-change="resetReadingScroll" />
        <template v-else-if="media && ['image', 'audio', 'video'].includes(kind)">
          <div class="file-view-bar"><span>{{ kind === 'image' ? '画面预览' : kind === 'audio' ? '音频播放' : '视频播放' }}</span><div v-if="kind === 'image'" class="file-zoom-actions"><button :disabled="imageZoom === 1" aria-label="缩小图片" @click="imageZoom = Math.max(1, imageZoom - .5)"><RemoteIcon name="zoom-out" /></button><button @click="imageZoom = 1">{{ Math.round(imageZoom * 100) }}%</button><button :disabled="imageZoom >= 3" aria-label="放大图片" @click="imageZoom += .5"><RemoteIcon name="zoom-in" /></button></div></div>
          <div class="file-media-stage" :class="`is-${kind}`" tabindex="0" :aria-label="`${fileName}预览区`">
            <img v-if="kind === 'image'" :src="media.url" :alt="fileName" :class="{ 'is-zoomed': imageZoom > 1 }" :style="imageZoom > 1 ? { width: `${imageZoom * 100}%` } : {}" @error="playbackError" />
            <div v-else-if="kind === 'audio'" class="file-audio-player"><div class="file-audio-art" aria-hidden="true"><RemoteIcon name="audio" /><span /><span /><span /><span /><span /></div><p>{{ fileName }}</p><audio :src="media.url" controls preload="metadata" @error="playbackError" /></div>
            <video v-else :src="media.url" controls playsinline preload="metadata" @error="playbackError" />
          </div>
        </template>
        <section v-else-if="(text !== null || htmlTicket) && ['html', 'svg'].includes(kind) && (!notice || htmlTicket)" class="file-web-reader"><RemoteRichBlock :block="webBlock" :context="fileContext" expanded @refresh="load" /></section>
        <section v-else-if="text !== null && kind === 'markdown'" class="file-markdown-reader"><RemoteRichText :source="text" node-id="file-body" :context="fileContext" @resource="$emit('resource', $event)" /></section>
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
        <p v-if="notice && (media || text !== null || sections.length || archive)" class="file-inline-notice"><RemoteIcon name="files" />{{ notice }}</p>
      </div>
    </div>
    <template #footer>
      <div class="file-save-dock">
        <RemoteFeedback :message="saveError" />
        <RemoteFeedback :message="saved" tone="success" />
        <template v-if="saving"><div class="file-transfer-label"><span role="status">{{ progress.phase === 'saving' ? '选择位置，收好这份文件' : '正在把文件带到身边…' }}</span><strong>{{ progress.phase === 'saving' ? '保存中' : percent === null ? formatFileSize(progress.received) : `${percent}%` }}</strong></div><div class="file-transfer-track" :class="{ indeterminate: percent === null }" role="progressbar" aria-label="下载进度" :aria-valuenow="percent" aria-valuemin="0" aria-valuemax="100"><i :style="{ width: `${percent ?? 24}%` }" /></div></template>
        <div class="file-save-actions"><span><RemoteIcon :name="saving ? 'download' : 'shield'" />{{ saving ? progress.phase === 'saving' ? '正在交给系统保存' : `${formatFileSize(progress.received)} / ${formatFileSize(progress.total)}` : '保留完整原件' }}</span><button v-if="saving && downloadId" class="file-button secondary" :disabled="cancelling" @click="cancel">{{ cancelling ? '正在取消…' : '取消保存' }}</button><button v-else class="file-button primary" :disabled="saving" @click="save"><RemoteIcon :name="resource.kind === 'external' ? 'open' : 'download'" />{{ resource.kind === 'external' ? '使用系统打开' : '保存到设备' }}</button></div>
      </div>
    </template>
  </RoomDrawer>
</template>
<script setup>
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { open as openSystemUrl } from '@tauri-apps/plugin-shell'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import RemoteFileCover from './RemoteFileCover.vue'
import RemoteFileState from './RemoteFileState.vue'
import RemoteSelect from './RemoteSelect.vue'
import RemotePdf from './RemotePdf.vue'
import RemoteRichText from './RemoteRichText.vue'
import RemoteRichBlock from './RemoteRichBlock.vue'
import '../../styles/remote-files.css'
import '../../styles/remote-rich.css'
import { remoteClient } from '../../services/remote'
import { currentToken, assertToken } from '../../stores/mode'
import { documentKind, formatFileSize } from '../../services/remoteDocuments'
import { remoteFileInfo, prepareRemoteFile, remoteFileLabel } from '../../services/remoteFilePreview'
import { nativeResource, externalResourceUrl } from '../../services/remoteResources'
const props = defineProps({ resource: Object, name: String })
defineEmits(['close', 'resource'])
const token = currentToken(), info = ref(null), kind = ref(''), media = ref(null), loading = ref(true), error = ref(''), text = ref(null)
const sections = ref([]), archive = ref(null), notice = ref(''), saving = ref(false), cancelling = ref(false), saveError = ref(''), saved = ref('')
const fileName = computed(() => info.value?.name || props.name || 'infOS文件')
const htmlTicket = ref('')
const imageZoom = ref(1), wrapText = ref(true), sectionIndex = ref(0), archiveQuery = ref('')
const readingBody = ref(null), archiveLimit = ref(80)
const nativeInput = computed(() => props.resource.kind === 'external' ? null : nativeResource(props.resource))
const fileContext = computed(() => ({ agentId: props.resource.agentId, threadId: props.resource.threadId, taskId: props.resource.taskId }))
const webBlock = computed(() => ({ kind: 'web', source: text.value || '', language: kind.value === 'svg' ? 'svg' : 'html', previewUrl: htmlTicket.value }))
const activeSection = computed(() => sections.value[sectionIndex.value] || sections.value[0])
const sheetColumns = computed(() => Array.from({ length: Math.max(1, ...(activeSection.value?.rows || []).map(row => row.length)) }))
const archiveMatches = computed(() => (archive.value?.entries || []).filter(entry => entry.name.toLowerCase().includes(archiveQuery.value.trim().toLowerCase())))
const archiveEntries = computed(() => archiveMatches.value.slice(0, archiveLimit.value))
const typeLabel = computed(() => remoteFileLabel(kind.value))
function columnLabel(index) { return index < 26 ? String.fromCharCode(65 + index) : `A${String.fromCharCode(65 + index - 26)}` }
function resetReadingScroll() { readingBody.value?.scrollTo({ top: 0, behavior: 'auto' }) }
watch(sectionIndex, resetReadingScroll)
watch(archiveQuery, () => { archiveLimit.value = 80 })
const progress = ref({ phase: 'downloading', received: 0, total: null })
const percent = computed(() => progress.value.total > 0 ? Math.min(100, Math.floor(progress.value.received / progress.value.total * 100)) : null)
let disposed = false, stopListen, downloadId
const controller = new AbortController()
function playbackError() { error.value = '设备无法播放此格式，可保存原件后使用其他应用查看' }
async function cancel() {
  if (!downloadId || cancelling.value) return
  cancelling.value = true
  try { await remoteClient.cancelDownload(downloadId, token) }
  catch (failure) { if (!disposed) { saveError.value = failure.message; cancelling.value = false } }
}
async function save() {
  if (saving.value) return
  saving.value = true; cancelling.value = false; saveError.value = ''; saved.value = ''
  try {
    assertToken(token, 'remote')
    if (props.resource.kind === 'external') { await openSystemUrl(externalResourceUrl(props.resource.url)); if (!disposed) saved.value = '已交给系统打开，可在系统中保存原件'; return }
    downloadId = crypto.randomUUID(); progress.value = { phase: 'downloading', received: 0, total: info.value?.size }
    const result = await remoteClient.downloadResource(nativeInput.value, fileName.value, downloadId, token)
    if (!disposed) saved.value = result.data?.saved ? result.data.path ? `已保存：${result.data.path}` : '文件已保存' : '已取消保存'
  } catch (failure) { if (!disposed) { if (failure.code === 'DOWNLOAD_CANCELLED') saved.value = '已取消保存'; else saveError.value = failure.message } }
  finally { downloadId = null; if (!disposed) { saving.value = false; cancelling.value = false } }
}
async function load() {
  if (disposed) return
  loading.value = true; error.value = ''; media.value?.release(); media.value = null; text.value = null; sections.value = []; archive.value = null; notice.value = ''; htmlTicket.value = ''
  try {
    info.value = await remoteFileInfo(props.resource, token)
    if (disposed) return
    if (documentKind(fileName.value, info.value.mimeType) === 'html' && nativeInput.value) {
      try { const response = await remoteClient.htmlPreview(nativeInput.value, token); assertToken(token, 'remote'); if (disposed) return; htmlTicket.value = response.data.url }
      catch (failure) { assertToken(token, 'remote'); if (disposed) return; notice.value = `会话资源预览暂不可用：${failure.message}` }
    }
    const content = await prepareRemoteFile(props.resource, { ...info.value, name: fileName.value }, { signal: controller.signal })
    if (disposed) { content.media?.release(); return }
    assertToken(token, 'remote')
    kind.value = content.kind; text.value = content.text; media.value = content.media; sections.value = content.sections; archive.value = content.archive; notice.value = content.notice
    if (kind.value === 'pdf' && !nativeInput.value) { kind.value = 'binary'; notice.value = '在线 PDF 可通过系统打开；私有附件支持应用内分页阅读' }
    if (kind.value === 'pdf' && info.value.size > 512 * 1024 * 1024) { kind.value = 'binary'; notice.value = 'PDF 较大，请保存原件查看' }
  } catch (failure) { if (!disposed) { if (htmlTicket.value) { kind.value = 'html'; notice.value = '源码速览暂不可用，页面预览仍可浏览' } else error.value = failure.message } }
  finally { if (!disposed) loading.value = false }
}
function pauseMedia() { if (document.hidden) readingBody.value?.querySelectorAll('audio,video').forEach(element => element.pause()) }
onMounted(async () => {
  document.addEventListener('visibilitychange', pauseMedia)
  try { stopListen = await remoteClient.listen(event => { if (event.kind === 'download' && event.data?.id === downloadId) progress.value = event.data }); if (disposed) { stopListen(); return }; await load() }
  catch (failure) { if (!disposed) { error.value = failure.message; loading.value = false } }
})
onBeforeUnmount(() => { disposed = true; controller.abort(); stopListen?.(); media.value?.release(); document.removeEventListener('visibilitychange', pauseMedia); readingBody.value?.querySelectorAll('audio,video').forEach(element => { element.pause(); element.removeAttribute('src'); element.load() }); if (downloadId) remoteClient.cancelDownload(downloadId, token).catch(() => {}) })
</script>
