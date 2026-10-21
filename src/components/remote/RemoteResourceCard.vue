<template>
  <article ref="root" class="remote-file-card" :class="[`is-${kind}`, { 'is-loading': loading }]" :aria-busy="loading">
    <div v-if="mediaKinds.has(kind)" class="remote-inline-media" :class="`is-${kind}`">
      <button v-if="kind === 'image' && preview?.media && !playbackFailed" type="button" class="remote-inline-picture" :aria-label="`放大 ${name}`" @click="open"><img :src="preview.media.url" :alt="name" loading="lazy" @load="$emit('resize')" @error="playbackFailed = true" /><span class="remote-media-expand"><RemoteIcon name="expand" /></span></button>
      <video v-else-if="kind === 'video' && preview?.media && !playbackFailed" ref="player" :src="preview.media.url" controls playsinline preload="metadata" @loadedmetadata="$emit('resize')" @error="playbackFailed = true" />
      <div v-else-if="kind === 'audio' && preview?.media && !playbackFailed" class="remote-inline-audio"><span aria-hidden="true"><RemoteIcon name="audio" /><i /><i /><i /><i /><i /></span><audio ref="player" :src="preview.media.url" controls preload="metadata" @error="playbackFailed = true" /></div>
      <button v-else type="button" class="remote-media-load" :disabled="loading" @click="loadMedia"><RemoteIcon :name="kind === 'image' ? 'image' : kind === 'video' ? 'play' : 'audio'" /><strong>{{ loading ? '正在展开…' : playbackFailed ? '此格式需保存后播放' : kind === 'image' ? '展开图片' : '加载并播放' }}</strong><small>{{ info ? formatFileSize(info.size) : '连接后读取文件' }}</small></button>
    </div>
    <button v-else type="button" class="remote-file-miniature" :aria-label="`打开 ${name}`" @click="open">
      <RemotePdf v-if="kind === 'pdf' && resolved && info && pdfReady && inView && !hidden" :resource="nativeResource(resolved)" :info="info" compact @ready="pageCount = $event" @page-change="$emit('resize')" />
      <iframe v-else-if="webPreview && inView && !hidden" :srcdoc="webPreview" sandbox="allow-scripts" referrerpolicy="no-referrer" tabindex="-1" title="文件内容缩略预览" />
      <div v-else-if="snippet" class="remote-file-paper"><span>{{ kind === 'markdown' ? 'MARKDOWN' : remoteFileLabel(kind) }}</span><p>{{ snippet }}</p></div>
      <div v-else-if="preview?.sections?.[0]?.rows" class="remote-file-mini-table"><div v-for="(row, index) in preview.sections[0].rows.slice(0, 5)" :key="index"><span v-for="(cell, column) in row.slice(0, 3)" :key="column">{{ cell || ' ' }}</span></div></div>
      <div v-else class="remote-file-mini-empty"><RemoteFileCover :name="name" /><span>{{ loading ? '正在读取小窗…' : failure ? '预览暂不可用' : remoteFileLabel(kind) }}</span><small>{{ failure ? '点击查看详情与原件' : '点击展开完整查看' }}</small></div>
      <span class="remote-file-window-corner"><RemoteIcon name="open" /></span>
    </button>
    <div class="remote-file-card-meta"><RemoteFileCover :name="name" /><button type="button" :title="name" @click="open"><strong>{{ name }}</strong><small>{{ remoteFileLabel(kind) }}<template v-if="info"> · {{ formatFileSize(info.size) }}</template><template v-if="pageCount"> · {{ pageCount }} 页</template></small></button><button type="button" class="remote-file-open" :aria-label="`展开 ${name}`" @click="open"><RemoteIcon name="expand" /></button></div>
    <p v-if="value?.caption" class="remote-file-card-caption">{{ value.caption }}</p>
    <p v-if="failure || playbackFailed" class="remote-file-card-error" role="status"><RemoteIcon name="alert" />{{ failure || '设备暂不支持此媒体格式，可展开后保存原件' }}<button v-if="failure" type="button" :disabled="loading" @click="load">重试</button></p>
  </article>
</template>
<script setup>
import { ref, computed, watch, onMounted, onBeforeUnmount, defineAsyncComponent } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteFileCover from './RemoteFileCover.vue'
import { resolveRemoteResource, nativeResource } from '../../services/remoteResources'
import { remoteFileInfo, prepareRemoteFile, mediaKinds, remoteFileLabel } from '../../services/remoteFilePreview'
import { documentKind, formatFileSize } from '../../services/remoteDocuments'
import { embeddedRemoteDocument } from '../../utils/remoteRichContent'
import { currentToken, assertToken } from '../../stores/mode'
const RemotePdf = defineAsyncComponent(() => import('./RemotePdf.vue'))
const props = defineProps({ value: [Object, String], context: Object })
const emit = defineEmits(['open', 'resize'])
const root = ref(null), player = ref(null), resolved = ref(null), info = ref(null), preview = ref(null), loading = ref(false), failure = ref(''), playbackFailed = ref(false), pdfReady = ref(false), pageCount = ref(0)
const inView = ref(false), hidden = ref(document.hidden)
const name = computed(() => info.value?.name || resolved.value?.name || props.value?.name || (typeof props.value === 'string' ? props.value.split('/').pop() : props.value?.source?.split('/').pop()) || '会话文件')
const kind = computed(() => preview.value?.kind || documentKind(name.value, info.value?.mimeType || props.value?.mimeType || ''))
const snippet = computed(() => (preview.value?.sections?.[0]?.text || preview.value?.text || '').slice(0, 700))
const webPreview = computed(() => preview.value?.text && ['html', 'svg'].includes(kind.value) && !preview.value.notice ? embeddedRemoteDocument(preview.value.text, channel) : '')
const token = currentToken(), channel = crypto.randomUUID()
let observer, controller, disposed = false, serial = 0, started = false, releaseTimer
function release() { preview.value?.media?.release(); preview.value = null; controller?.abort(); player.value?.pause() }
async function load(forceMedia = false) {
  if (loading.value || disposed || document.hidden) return
  const request = ++serial; controller?.abort(); controller = new AbortController(); const signal = controller.signal
  loading.value = true; failure.value = ''; playbackFailed.value = false
  try {
    const file = resolved.value || await resolveRemoteResource(props.value, props.context, token)
    assertToken(token, 'remote'); if (disposed || request !== serial) return
    if (file.kind === 'knowledge') { resolved.value = file; return }
    resolved.value = file
    const meta = info.value || await remoteFileInfo(file, token)
    assertToken(token, 'remote'); if (disposed || request !== serial) return
    info.value = meta
    const format = documentKind(meta.name || file.name, meta.mimeType.split(';')[0].toLowerCase())
    if (file.kind === 'external' && mediaKinds.has(format)) { preview.value = { kind: format, media: { url: file.url, release() {} } }; return }
    if (format === 'pdf') { pdfReady.value = file.kind !== 'external' && meta.size > 0 && meta.size < 512 * 1024 * 1024; return }
    if (mediaKinds.has(format) && (format !== 'image' || meta.size > 8 * 1024 * 1024) && !forceMedia) return
    const content = await prepareRemoteFile(file, meta, { compact: !forceMedia, signal })
    if (disposed || request !== serial || signal.aborted) { content.media?.release(); return }
    assertToken(token, 'remote'); preview.value?.media?.release(); preview.value = content
  } catch (error) { if (!disposed && request === serial && !signal.aborted) failure.value = error.message }
  finally { if (!disposed && request === serial) { loading.value = false; emit('resize') } }
}
const loadMedia = () => load(true)
async function open() {
  try { const file = resolved.value || await resolveRemoteResource(props.value, props.context, token); assertToken(token, 'remote'); if (disposed) return; player.value?.pause(); emit('open', { ...file, name: name.value }) }
  catch (error) { if (!disposed) failure.value = error.message }
}
function visibility() { hidden.value = document.hidden; if (document.hidden) player.value?.pause(); else if (!started && inView.value) { started = true; load() } }
watch(() => JSON.stringify([props.value, props.context]), () => { serial++; release(); info.value = null; resolved.value = null; pdfReady.value = false; pageCount.value = 0; loading.value = false; if (started && inView.value) load(); else started = false })
onMounted(() => {
  const update = entries => {
    inView.value = entries.some(entry => entry.isIntersecting); clearTimeout(releaseTimer)
    if (inView.value) { if (!started && !document.hidden) { started = true; load() } }
    else {
      player.value?.pause()
      releaseTimer = setTimeout(() => { if (!inView.value && !disposed) { serial++; release(); loading.value = false; started = false } }, 3000)
    }
  }
  if (typeof IntersectionObserver === 'function') { observer = new IntersectionObserver(update, { rootMargin: '120px' }); observer.observe(root.value) }
  else update([{ isIntersecting: true }])
  document.addEventListener('visibilitychange', visibility)
})
onBeforeUnmount(() => { disposed = true; serial++; observer?.disconnect(); clearTimeout(releaseTimer); document.removeEventListener('visibilitychange', visibility); release() })
</script>
