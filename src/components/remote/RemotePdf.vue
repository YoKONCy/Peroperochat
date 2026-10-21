<template>
  <section class="remote-pdf" :class="{ 'is-compact': compact }" :inert="compact ? true : undefined" aria-label="PDF 预览">
    <p v-if="unlock && compact" class="remote-help"><RemoteIcon name="shield" />文档已加密，展开后输入密码</p>
    <form v-if="unlock && !compact" class="file-unlock-card" @submit.prevent="submitPassword">
      <RemoteIcon name="shield" /><h3>这份文件上了锁</h3><p>{{ passwordWrong ? '密码没有对上，再试一次吧' : '输入文档密码，继续阅读' }}</p>
      <label class="file-sr-only" for="remote-pdf-password">文档密码</label><input id="remote-pdf-password" v-model="password" type="password" autocomplete="off" placeholder="文档密码" />
      <button class="file-button primary" type="submit">打开文档<RemoteIcon name="next" /></button>
    </form>
    <RemoteFileState v-if="error" title="这一页暂时打不开" :description="error" icon="files" />
    <RemoteFileState v-else-if="!pages && !unlock" title="正在展开文档…" description="先读这一页，其他的慢慢来" loading />
    <div v-show="pages && !error" ref="viewport" class="remote-pdf-canvas" tabindex="0" aria-label="PDF 页面阅读区" :aria-busy="busy">
      <span v-if="busy" class="file-page-loading" role="status">正在绘制…</span>
      <canvas ref="canvas" :aria-label="`PDF 第 ${page} 页`" />
    </div>
    <nav v-if="pages && !error && !compact" class="file-pdf-controls" aria-label="PDF 阅读控制">
      <button aria-label="上一页" :disabled="page <= 1 || busy" @click="page--"><RemoteIcon name="back" /></button>
      <label class="file-page-counter"><span class="file-sr-only">跳转到页码</span><input :value="page" inputmode="numeric" pattern="[0-9]*" :disabled="busy" aria-label="当前页，输入页码跳转" @change="jumpTo" @keydown.enter.prevent="jumpTo" /><span>/ {{ pages }}</span></label>
      <button aria-label="下一页" :disabled="page >= pages || busy" @click="page++"><RemoteIcon name="next" /></button>
      <i />
      <button :disabled="busy || zoom <= .75" aria-label="缩小页面" @click="zoom -= .25"><RemoteIcon name="zoom-out" /></button>
      <button class="file-fit-page" :disabled="busy" :aria-label="`当前缩放 ${Math.round(zoom * 100)}%，点击适应宽度`" @click="zoom = 1">{{ zoom === 1 ? '适宽' : `${Math.round(zoom * 100)}%` }}</button>
      <button :disabled="busy || zoom >= 2" aria-label="放大页面" @click="zoom += .25"><RemoteIcon name="zoom-in" /></button>
    </nav>
    <p v-if="pages && !error && !compact" class="file-reading-caption">{{ pages }} 页文档 · 输入页码可跳转</p>
  </section>
</template>
<script setup>
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { remoteClient } from '../../services/remote'
import { currentToken, assertToken } from '../../stores/mode'
import RemoteIcon from './RemoteIcon.vue'
import RemoteFileState from './RemoteFileState.vue'

const props = defineProps({ resource: Object, info: Object, compact: Boolean })
const emit = defineEmits(['page-change', 'ready'])
const canvas = ref(null), viewport = ref(null), page = ref(1), pages = ref(0), zoom = ref(1), busy = ref(true), error = ref('')
const unlock = ref(false), password = ref(''), passwordWrong = ref(false)
const token = currentToken()
let task, pdf, render, observer, timer, disposed = false, rendering = 0, updatePassword
function submitPassword() { updatePassword?.(password.value); password.value = ''; unlock.value = false; busy.value = true }
function jumpTo(event) { const value = event.target.value.trim(); if (/^\d+$/.test(value)) page.value = Math.max(1, Math.min(pages.value, Number(value))); event.target.value = String(page.value); if (event.type === 'keydown') event.target.blur() }
async function draw() {
  if (!pdf || disposed) return
  const serial = ++rendering
  const previous = render
  previous?.cancel(); busy.value = true; error.value = ''
  try {
    await previous?.promise.catch(() => {})
    if (disposed || serial !== rendering) return
    const pdfPage = await pdf.getPage(page.value)
    if (disposed || serial !== rendering) return
    const base = pdfPage.getViewport({ scale: 1 }), width = Math.max(160, (viewport.value?.clientWidth || 320) - 32)
    const screen = pdfPage.getViewport({ scale: width / base.width * zoom.value })
    // 每次只保留一页画布，限制像素总量，避免手机上的超大扫描页耗尽内存。
    const density = Math.min(window.devicePixelRatio || 1, props.compact ? 1 : 2, Math.sqrt((props.compact ? 700_000 : 4_000_000) / (screen.width * screen.height)))
    const target = canvas.value
    target.width = Math.max(1, Math.floor(screen.width * density)); target.height = Math.max(1, Math.floor(screen.height * density))
    target.style.width = `${screen.width}px`; target.style.height = `${screen.height}px`
    render = pdfPage.render({ canvas: target, canvasContext: target.getContext('2d'), viewport: screen, transform: [density, 0, 0, density, 0, 0] })
    await render.promise; pdfPage.cleanup()
  } catch (failure) { if (!disposed && serial === rendering && failure.name !== 'RenderingCancelledException') error.value = '此页无法显示，可下载原件查看' }
  finally { if (!disposed && serial === rendering) busy.value = false }
}
onMounted(async () => {
  try {
    // 旧版移动 WebView 也能使用 PDF 引擎所需的 Promise 接口。
    Promise.withResolvers ||= function () { let resolve, reject; const promise = new Promise((yes, no) => { resolve = yes; reject = no }); return { promise, resolve, reject } }
    const [engine, worker] = await Promise.all([import('pdfjs-dist/legacy/build/pdf.mjs'), import('pdfjs-dist/legacy/build/pdf.worker.min.mjs?url')])
    if (disposed) return
    assertToken(token, 'remote'); engine.GlobalWorkerOptions.workerSrc = worker.default
    const assets = {
      ...import.meta.glob('/node_modules/pdfjs-dist/cmaps/*.bcmap', { eager: true, exhaustive: true, query: '?url', import: 'default' }),
      ...import.meta.glob('/node_modules/pdfjs-dist/standard_fonts/*', { eager: true, exhaustive: true, query: '?url', import: 'default' }),
      ...import.meta.glob('/node_modules/pdfjs-dist/wasm/*.wasm', { eager: true, exhaustive: true, query: '?url', import: 'default' }),
    }
    class LocalBinaryData {
      async fetch({ filename }) {
        const path = Object.keys(assets).find(key => key.endsWith(`/${filename}`))
        if (!path) throw new Error('缺少 PDF 字体或解码资源')
        const response = await fetch(assets[path]); if (!response.ok) throw new Error('PDF 资源读取失败')
        return new Uint8Array(await response.arrayBuffer())
      }
    }
    class NativeRange extends engine.PDFDataRangeTransport {
      requestDataRange(begin, end) {
        remoteClient.resource(props.resource, { range: { start: begin, end: end - 1 }, token }).then(result => {
          if (disposed) return
          if (result.contentRange !== `bytes ${begin}-${end - 1}/${props.info.size}` || result.bytes.length !== end - begin) throw new Error('PDF 内容范围已改变')
          this.onDataRange(begin, new Uint8Array(result.bytes))
        }).catch(() => {
          if (disposed) return
          error.value = 'PDF 读取中断或文件已变化，请重新打开；也可下载原件'
          busy.value = false; task?.destroy().catch(() => {})
        })
      }
    }
    const input = { disableAutoFetch: true, disableStream: true, rangeChunkSize: 512 * 1024, useWorkerFetch: false, BinaryDataFactory: LocalBinaryData, enableXfa: false, maxImageSize: 16_000_000, canvasMaxAreaInBytes: 16_000_000, isImageDecoderSupported: false, verbosity: 0 }
    if (props.info.ranges && props.info.size > 0) input.range = new NativeRange(props.info.size, new Uint8Array(), true)
    else {
      if (props.info.size > 8 * 1024 * 1024) throw new Error('服务端不支持分段预览，请下载原件查看')
      input.data = new Uint8Array((await remoteClient.resource(props.resource, { token })).bytes)
    }
    if (disposed) return
    task = engine.getDocument(input)
    task.onPassword = (callback, reason) => { updatePassword = callback; passwordWrong.value = reason === 2; unlock.value = true; busy.value = false }
    pdf = await task.promise
    if (disposed) { await pdf.destroy(); return }
    pages.value = pdf.numPages; emit('ready', pdf.numPages); await draw()
    if (disposed) return
    observer = new ResizeObserver(() => { clearTimeout(timer); timer = setTimeout(draw, 180) }); observer.observe(viewport.value)
  } catch { if (!disposed) { error.value ||= 'PDF 无法预览，可下载原件查看'; busy.value = false } }
})
watch([page, zoom], draw)
watch(page, () => emit('page-change'))
onBeforeUnmount(() => { disposed = true; rendering++; clearTimeout(timer); observer?.disconnect(); render?.cancel(); task?.destroy().catch(() => {}); password.value = '' })
</script>
