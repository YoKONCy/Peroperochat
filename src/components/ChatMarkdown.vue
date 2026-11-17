<template><div ref="root" /></template>

<script setup>
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { createTextMeasurer, observeTextLayout, readTextStyle } from '../utils/textLayout'

// HTML 由调用方现有的 Markdown 净化器生成，链接和媒体事件继续向上冒泡。
const props = defineProps({ html: { type: String, default: '' } })
const emit = defineEmits(['resize'])
const root = ref(null)
const measurer = createTextMeasurer()
let markup = []
let styles = new WeakMap()
let heights = new WeakMap()
let width = 0
let frame = null
let observer = null
let stopEnvironment = null
const managedParagraphs = new WeakSet()
const inlineTags = new Set(['A', 'STRONG', 'B', 'EM', 'I', 'SPAN', 'U', 'S', 'DEL'])
const supportsContainment = CSS.supports('content-visibility', 'auto') && CSS.supports('contain-intrinsic-block-size', '1px')
let disposed = false

function nodeMarkup(node) {
  return node instanceof Element ? node.outerHTML : `${node.nodeType}:${node.textContent}`
}

function render() {
  if (!root.value) return
  const template = document.createElement('template')
  template.innerHTML = props.html
  const incoming = [...template.content.childNodes]
  const nextMarkup = incoming.map(nodeMarkup)
  let start = 0
  let end = markup.length
  let nextEnd = nextMarkup.length
  while (start < end && start < nextEnd && markup[start] === nextMarkup[start]) start++
  while (end > start && nextEnd > start && markup[end - 1] === nextMarkup[nextEnd - 1]) {
    end--
    nextEnd--
  }
  // 保留未变化的段落节点，流式尾段更新不会重建已完成的正文、图片和选区。
  const previous = [...root.value.childNodes]
  for (let index = start; index < end; index++) previous[index].remove()
  const before = root.value.childNodes[start] || null
  const replacement = document.createDocumentFragment()
  for (let index = start; index < nextEnd; index++) replacement.append(incoming[index])
  root.value.insertBefore(replacement, before)
  markup = nextMarkup
  schedule()
  emit('resize')
}

function textStyle(element) {
  if (!styles.has(element)) styles.set(element, readTextStyle(element))
  return styles.get(element)
}

function paragraphHeight(paragraph, availableWidth) {
  const style = textStyle(paragraph)
  const contentWidth = availableWidth - style.marginX - style.paddingX - style.borderX
  if (style.whiteSpace !== 'normal') return null
  for (const child of paragraph.querySelectorAll('*')) {
    if (!inlineTags.has(child.tagName) || child.matches('.katex, .remote-inline-math')) return null
    const childStyle = textStyle(child)
    if (childStyle.display !== 'inline' || childStyle.paddingX || childStyle.borderX) return null
  }
  if (!paragraph.children.length) return measurer.measure(paragraph.textContent, style, contentWidth)
  const items = []
  const walker = document.createTreeWalker(paragraph, NodeFilter.SHOW_TEXT)
  while (walker.nextNode()) {
    const parent = walker.currentNode.parentElement
    const runStyle = textStyle(parent)
    items.push({ text: walker.currentNode.textContent, font: runStyle.font, letterSpacing: runStyle.letterSpacing })
  }
  return measurer.measureInline(items, style, contentWidth)
}

function measure() {
  if (!supportsContainment || !(width > 0) || !root.value) return
  for (const paragraph of root.value.children) {
    if (!(paragraph instanceof HTMLElement) || paragraph.tagName !== 'P') continue
    if (paragraph.hasAttribute('style') && !managedParagraphs.has(paragraph)) continue
    if (heights.has(paragraph)) continue
    const height = paragraphHeight(paragraph, width)
    heights.set(paragraph, height)
    if (height == null) {
      if (managedParagraphs.has(paragraph)) {
        paragraph.style.removeProperty('content-visibility')
        paragraph.style.removeProperty('contain-intrinsic-block-size')
      }
      continue
    }
    // 浏览器负责可见正文的实际排版，预计算高度只为屏外段落占位。
    paragraph.style.contentVisibility = 'auto'
    paragraph.style.containIntrinsicBlockSize = `${height}px`
    managedParagraphs.add(paragraph)
  }
}

function schedule() {
  if (frame !== null || disposed) return
  frame = requestAnimationFrame(() => {
    frame = null
    if (!disposed) measure()
  })
}

function environmentChanged() {
  styles = new WeakMap()
  heights = new WeakMap()
  schedule()
}

watch(() => props.html, render, { flush: 'post' })
onMounted(() => {
  render()
  const style = textStyle(root.value)
  width = Math.max(0, root.value.clientWidth - style.paddingX)
  measure()
  if (typeof ResizeObserver !== 'undefined') {
    observer = new ResizeObserver(([entry]) => {
      const nextWidth = entry.contentRect.width
      if (nextWidth !== width) {
        width = nextWidth
        environmentChanged()
      }
      emit('resize')
    })
    observer.observe(root.value)
  }
  root.value.addEventListener('load', resized, true)
  root.value.addEventListener('error', resized, true)
  stopEnvironment = observeTextLayout(root.value, environmentChanged)
})
function resized() { emit('resize') }
onBeforeUnmount(() => {
  disposed = true
  observer?.disconnect()
  stopEnvironment?.()
  if (frame !== null) cancelAnimationFrame(frame)
  root.value?.removeEventListener('load', resized, true)
  root.value?.removeEventListener('error', resized, true)
})
</script>
