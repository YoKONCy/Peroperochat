<template><div ref="root" class="remote-stream-tail" :class="{ 'is-receiving': active }" :aria-label="active ? '正在接收的正文' : '未完成的原文'" /></template>
<script setup>
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { appendAsideText } from '../../utils/parenthetical'
const props = defineProps({ source: { type: String, default: '' }, active: Boolean })
const emit = defineEmits(['resize'])
const root = ref(null)
let received = '', frame = null, stream = { node: null, depth: 0, start: 0 }, plain = false
function flush() {
  frame = null
  if (!root.value) return
  const source = props.source
  const nextPlain = /[`$<\[\\]/.test(source)
  if (!source.startsWith(received) || nextPlain !== plain) { root.value.replaceChildren(); stream = { node: null, depth: 0, start: 0 }; received = '' }
  plain = nextPlain
  // 直接追加文本节点，稳定区域的选区与 DOM 不随增量替换。
  const delta = source.slice(received.length)
  if (plain) { if (!stream.node) { stream.node = document.createTextNode(''); root.value.append(stream.node) }; stream.node.appendData(delta) }
  else appendAsideText(root.value, stream, delta)
  received = source; emit('resize')
}
function schedule() { if (frame === null && !document.hidden) frame = requestAnimationFrame(flush) }
watch(() => props.source, schedule)
onMounted(() => { flush(); document.addEventListener('visibilitychange', schedule) })
onBeforeUnmount(() => { if (frame !== null) cancelAnimationFrame(frame); document.removeEventListener('visibilitychange', schedule) })
</script>
