<template><div ref="root" class="remote-stream-tail" :class="{ 'is-receiving': active }" :aria-label="active ? '正在接收的正文' : '未完成的原文'" /></template>
<script setup>
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
const props = defineProps({ source: { type: String, default: '' }, active: Boolean })
const emit = defineEmits(['resize'])
const root = ref(null)
let received = '', frame = null, textNode = null
function flush() {
  frame = null
  if (!root.value) return
  const source = props.source
  if (!source.startsWith(received) || !textNode) { root.value.replaceChildren(); textNode = document.createTextNode(''); root.value.append(textNode); received = '' }
  // 直接追加文本节点，稳定区域的选区与 DOM 不随增量替换。
  textNode.appendData(source.slice(received.length)); received = source; emit('resize')
}
function schedule() { if (frame === null && !document.hidden) frame = requestAnimationFrame(flush) }
watch(() => props.source, schedule)
onMounted(() => { flush(); document.addEventListener('visibilitychange', schedule) })
onBeforeUnmount(() => { if (frame !== null) cancelAnimationFrame(frame); document.removeEventListener('visibilitychange', schedule) })
</script>
