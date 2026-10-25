<template><time v-if="duration !== null" class="run-elapsed" aria-live="off" :datetime="`PT${Math.floor(duration / 1000)}S`">{{ prefix }}{{ prefix ? ' ' : '' }}{{ elapsedLabel(duration) }}</time></template>
<script setup>
import { computed, ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { runTimestamp, elapsedLabel } from '../../utils/runTiming.js'
const props = defineProps({ startedAt: [Number, String], durationMs: Number, running: { type: Boolean, default: true }, prefix: { type: String, default: '' } })
const fallback = Date.now(), now = ref(fallback)
const duration = computed(() => props.running ? Math.max(0, now.value - (runTimestamp(props.startedAt) ?? fallback)) : Number.isFinite(props.durationMs) && props.durationMs >= 0 ? props.durationMs : null)
let timer = null
function synchronize() {
  clearInterval(timer); timer = null; now.value = Date.now()
  // 后台不按秒唤醒；恢复时直接用时间戳补上全部经过时间。
  if (props.running && !document.hidden) timer = setInterval(() => { now.value = Date.now() }, 1000)
}
watch(() => [props.running, props.startedAt], synchronize)
onMounted(() => { synchronize(); document.addEventListener('visibilitychange', synchronize); window.addEventListener('pageshow', synchronize) })
onBeforeUnmount(() => { clearInterval(timer); document.removeEventListener('visibilitychange', synchronize); window.removeEventListener('pageshow', synchronize) })
</script>
<style scoped>
.run-elapsed { flex: none; font-size: 11px; font-weight: 450; line-height: 1.5; font-variant-numeric: tabular-nums; white-space: nowrap; color: var(--room-muted); }
</style>
