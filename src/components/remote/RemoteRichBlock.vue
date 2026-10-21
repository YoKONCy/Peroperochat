<template>
  <div ref="root" class="remote-rich-block" :class="{ 'is-expanded': expanded }" :data-kind="block.kind" :data-format="block.format" :style="{ '--diagram-scale': diagramScale }">
    <RemoteResourceCard v-if="block.kind === 'resource'" :value="{ source: block.source, name: block.name }" :context="context" @open="$emit('resource', $event)" @resize="$emit('resize')" />
    <div v-else-if="block.kind === 'markdown'" class="remote-rich-prose" @click="activateLink">
      <div v-if="html" class="markdown-body" v-html="html" />
      <pre v-else class="remote-rich-raw">{{ block.source }}</pre>
      <button v-if="!expanded && (block.source.length > 1800 || containsTable)" type="button" class="remote-rich-read-more" @click="open = true"><RemoteIcon name="expand" />{{ containsTable ? '展开表格' : '展开阅读' }}</button>
    </div>
    <section v-else class="remote-rich-panel" :class="`is-${block.kind}`">
      <header><span class="remote-rich-caption"><i><RemoteIcon :name="panelIcon" /></i><span>{{ title }}</span></span><div class="remote-rich-actions">
        <button v-if="block.kind === 'code'" type="button" aria-label="自动换行" :aria-pressed="wrap" @click="wrap = !wrap"><RemoteIcon name="wrap" /><span>换行</span></button>
        <template v-if="expanded && block.kind === 'diagram'"><button type="button" aria-label="缩小图表" :disabled="diagramScale <= .6" @click="diagramScale = Math.max(.6, diagramScale - .2)"><RemoteIcon name="zoom-out" /></button><button type="button" aria-label="适应图表" @click="diagramScale = 1">{{ Math.round(diagramScale * 100) }}%</button><button type="button" aria-label="放大图表" :disabled="diagramScale >= 2.4" @click="diagramScale = Math.min(2.4, diagramScale + .2)"><RemoteIcon name="zoom-in" /></button></template>
        <button v-if="['web', 'diagram', 'formula'].includes(block.kind)" type="button" :aria-label="showSource ? '查看预览' : '查看源码'" :aria-pressed="showSource" @click="showSource = !showSource"><RemoteIcon :name="showSource ? panelIcon : 'code'" /><span>{{ showSource ? '预览' : '源码' }}</span></button>
        <button v-if="block.kind === 'web' && block.previewUrl && block.source" type="button" :aria-label="useInlinePreview ? '使用完整预览' : '使用简易预览'" :aria-pressed="useInlinePreview" @click="useInlinePreview = !useInlinePreview"><RemoteIcon name="desktop" /><span>{{ useInlinePreview ? '完整预览' : '简易预览' }}</span></button>
        <button type="button" :aria-label="copied || '复制原文'" @click="copy"><RemoteIcon name="copy" /><span>{{ copied || '复制' }}</span></button>
        <button v-if="!expanded" type="button" aria-label="展开阅读" @click="open = true"><RemoteIcon name="expand" /></button>
      </div></header>
      <pre v-if="showSource || block.kind === 'code'" class="remote-rich-code" :class="{ wrapped: wrap }" tabindex="0"><code v-if="block.kind === 'code' && html" v-html="html" /><code v-else>{{ block.source }}</code></pre>
      <template v-else-if="block.kind === 'web'">
        <iframe v-if="visible && !pageHidden && (webDocument || previewUrl)" ref="frame" :src="previewUrl || undefined" :srcdoc="previewUrl ? undefined : webDocument" sandbox="allow-scripts" referrerpolicy="no-referrer" :title="block.language === 'svg' ? 'SVG 预览' : 'HTML 预览'" :style="{ height: `${expanded ? 600 : webHeight}px` }" @load="$emit('resize')" />
        <div v-else class="remote-rich-placeholder"><RemoteIcon name="desktop" />{{ pageHidden ? '返回后继续预览' : '滑到这里时展开预览' }}</div>
        <footer><RemoteIcon name="shield" />安全预览<button v-if="block.previewUrl" type="button" @click="$emit('refresh')"><RemoteIcon name="refresh" />刷新</button></footer>
        <p v-if="previewUrl && block.source" class="remote-web-hint">没有显示？试试简易预览。</p>
      </template>
      <div v-else-if="html" class="remote-rich-canvas" :class="{ 'is-formula': block.kind === 'formula' }" tabindex="0" :aria-label="title" v-html="html" />
      <div v-else-if="loading" class="remote-rich-placeholder" role="status"><RemoteIcon name="clock" />正在排版…</div>
      <div v-else class="remote-rich-render-error"><p><RemoteIcon name="alert" />{{ failure || '暂时无法排版，原文仍然保留' }}</p><pre>{{ block.source }}</pre></div>
    </section>
    <p v-if="failure && block.kind === 'markdown'" class="remote-rich-incomplete" role="alert">{{ failure }}</p>
    <RoomDrawer v-if="open" title="展开阅读" :subtitle="title" variant="file" @close="open = false"><RemoteRichBlock :block="block" :context="context" expanded @resource="$emit('resource', $event)" /></RoomDrawer>
  </div>
</template>
<script setup>
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RemoteResourceCard from './RemoteResourceCard.vue'
import { renderRemoteMarkdown, embeddedRemoteDocument } from '../../utils/remoteRichContent'
import { highlightRemoteCode, remoteMath, remoteDiagram } from '../../services/remoteRichEngines'
import { resolveRemoteResource } from '../../services/remoteResources'
import { currentToken, assertToken } from '../../stores/mode'
import { openRemoteLink } from '../../utils/remoteMarkdown'
const props = defineProps({ block: { type: Object, required: true }, context: Object, expanded: Boolean })
const emit = defineEmits(['resource', 'resize', 'refresh'])
const root = ref(null), frame = ref(null), html = ref(''), wrap = ref(false), showSource = ref(false), open = ref(false), copied = ref(''), failure = ref(''), loading = ref(false), visible = ref(false), pageHidden = ref(document.hidden), webHeight = ref(280)
const diagramScale = ref(1)
const useInlinePreview = ref(false)
const previewUrl = computed(() => useInlinePreview.value ? '' : props.block.previewUrl)
const containsTable = computed(() => /^\s*\|.*\|/m.test(props.block.source))
const channel = crypto.randomUUID(), token = currentToken()
let generation = 0, observer, copyTimer, disposed = false, prepared = ''
const title = computed(() => ({ code: String(props.block.language || 'TEXT').toUpperCase(), web: props.block.language === 'svg' ? 'SVG 预览' : 'HTML 预览', diagram: '图表', formula: '公式', markdown: '消息正文' }[props.block.kind] || '内容'))
const panelIcon = computed(() => ({ code: 'code', web: 'desktop', diagram: 'diagram', formula: 'formula' }[props.block.kind] || 'files'))
const webDocument = computed(() => props.block.kind === 'web' && !previewUrl.value ? embeddedRemoteDocument(props.block.source, channel) : '')
async function inlineMath(markup) {
  const fragment = document.createElement('template'); fragment.innerHTML = markup
  const walker = document.createTreeWalker(fragment.content, NodeFilter.SHOW_TEXT), nodes = []
  while (walker.nextNode()) if (!walker.currentNode.parentElement?.closest('code,pre,a')) nodes.push(walker.currentNode)
  let count = 0
  for (const node of nodes) {
    const source = node.data, pattern = /(?<![\\$])\$(?!\$)([^\n$]+?)(?<!\\)\$(?!\$)/g
    if (!pattern.test(source)) continue
    pattern.lastIndex = 0
    const replacement = document.createDocumentFragment(); let offset = 0
    for (const match of source.matchAll(pattern)) {
      if (++count > 80) break
      replacement.append(source.slice(offset, match.index))
      try { const container = document.createElement('span'); container.className = 'remote-inline-math'; container.innerHTML = await remoteMath(match[1], false); replacement.append(container) } catch { replacement.append(match[0]) }
      offset = match.index + match[0].length
    }
    replacement.append(source.slice(offset)); node.replaceWith(replacement)
  }
  return fragment.innerHTML
}
async function prepare() {
  if (!visible.value || pageHidden.value || disposed) return
  const block = props.block, key = `${block.kind}:${block.source}`
  if (prepared === key) return
  const serial = ++generation; loading.value = true; failure.value = ''; html.value = ''
  try {
    assertToken(token, 'remote')
    let result = ''
    if (block.kind === 'markdown') result = await inlineMath(renderRemoteMarkdown(block.source))
    else if (block.kind === 'code') result = await highlightRemoteCode(block.source, block.language)
    else if (block.kind === 'formula') result = await remoteMath(block.source)
    else if (block.kind === 'diagram') result = await remoteDiagram(block.source)
    if (disposed || serial !== generation) return
    assertToken(token, 'remote'); html.value = result || ''; prepared = key
  } catch { if (!disposed && serial === generation) failure.value = '这段内容暂时无法排版，原文仍然保留' }
  finally { if (!disposed && serial === generation) { loading.value = false; emit('resize') } }
}
async function copy() { try { await navigator.clipboard.writeText(props.block.source); copied.value = '已复制' } catch { copied.value = '请选取复制' }; clearTimeout(copyTimer); copyTimer = setTimeout(() => { copied.value = '' }, 2000) }
async function activateLink(event) {
  const link = event.target.closest('a')
  if (!link || !event.currentTarget.contains(link)) return
  event.preventDefault(); event.stopPropagation()
  try {
    if (link.dataset.resourceSource) emit('resource', await resolveRemoteResource(link.dataset.resourceSource, props.context, token))
    else await openRemoteLink(event)
  } catch (error) { failure.value = error.message }
}
function message(event) { if (event.source === frame.value?.contentWindow && event.data?.type === 'ppc-preview-size' && event.data.channel === channel && Number.isFinite(event.data.height)) { webHeight.value = Math.min(720, Math.max(120, event.data.height)); emit('resize') } }
function visibility() { pageHidden.value = document.hidden; if (!document.hidden) prepare() }
watch(() => [props.block.source, props.block.kind], () => { prepared = ''; generation++; prepare() })
onMounted(() => {
  if (typeof IntersectionObserver === 'function') { observer = new IntersectionObserver(entries => { visible.value = entries.some(item => item.isIntersecting); if (visible.value) prepare() }, { rootMargin: '250px' }); observer.observe(root.value) }
  else { visible.value = true; prepare() }
  window.addEventListener('message', message); document.addEventListener('visibilitychange', visibility)
})
onBeforeUnmount(() => { disposed = true; generation++; observer?.disconnect(); clearTimeout(copyTimer); window.removeEventListener('message', message); document.removeEventListener('visibilitychange', visibility) })
</script>
