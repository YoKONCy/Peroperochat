<template>
  <details ref="root" class="soft-disclosure" :data-expanded="expanded" @toggle="nativeToggle">
    <summary :aria-expanded="expanded" @click.prevent="toggle"><slot name="summary" /></summary>
    <div ref="body" class="soft-disclosure-body" :inert="!expanded ? true : undefined"><div v-if="visited" class="soft-disclosure-content"><slot /></div></div>
  </details>
</template>
<script setup>
import { ref, nextTick, onBeforeUnmount, inject, watch } from 'vue'
import { REMOTE_THEME } from '../../services/remoteTheme'
import { LOCAL_APPEARANCE } from '../../stores/appearance'
const emit = defineEmits(['resize', 'change'])
const root = ref(null), body = ref(null), expanded = ref(false), visited = ref(false)
const remote = inject(REMOTE_THEME, null), local = inject(LOCAL_APPEARANCE, null)
const preference = window.matchMedia('(prefers-reduced-motion: reduce)')
let animation = null, version = 0, disposed = false, changing = false
const reduced = () => preference.matches || remote?.value?.reduceMotion || local?.value?.reduceMotion
function settle() { animation?.cancel(); animation = null; if (root.value) root.value.open = expanded.value; changing = false; emit('resize') }
async function toggle() {
  const request = ++version, height = root.value.open ? body.value.getBoundingClientRect().height : 0
  animation?.cancel(); animation = null
  changing = true
  expanded.value = !expanded.value; visited.value = true
  root.value.open = true
  emit('change', expanded.value)
  await nextTick()
  if (disposed || request !== version) return
  if (reduced() || !body.value.animate) { settle(); return }
  const current = body.value.animate([{ height: `${height}px`, opacity: expanded.value ? .5 : 1 }, { height: expanded.value ? `${body.value.scrollHeight}px` : '0px', opacity: expanded.value ? 1 : 0 }], { duration: expanded.value ? 220 : 170, easing: 'cubic-bezier(.2,.75,.25,1)' })
  animation = current
  current.onfinish = () => { if (animation === current) settle() }
}
// 浏览器查找可能直接展开详情，保持可访问状态和延迟内容一致。
function nativeToggle(event) { if (event.target !== root.value || changing || root.value.open === expanded.value) return; expanded.value = root.value.open; if (expanded.value) visited.value = true; emit('change', expanded.value); nextTick(() => emit('resize')) }
function motionChanged() { if (reduced()) { version++; settle() } }
preference.addEventListener('change', motionChanged)
watch(() => [remote?.value?.reduceMotion, local?.value?.reduceMotion], motionChanged)
onBeforeUnmount(() => { disposed = true; version++; animation?.cancel(); preference.removeEventListener('change', motionChanged) })
</script>
<style>
.soft-disclosure > summary { list-style: none; cursor: pointer; }
.soft-disclosure > summary::-webkit-details-marker { display: none; }
.soft-disclosure-body { min-width: 0; overflow: hidden; }
.soft-disclosure-content { min-width: 0; }
.soft-disclosure > summary:focus-visible { outline: 2px solid var(--room-accent); outline-offset: -3px; border-radius: inherit; }
</style>
