<template>
  <svg ref="root" class="reasoning-icon" :class="{ 'is-thinking': animate }" :data-effort="level" viewBox="0 0 32 32" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
    <template v-if="['off', 'default', 'unknown'].includes(level)"><path class="reasoning-wash" d="M7 23a6 6 0 0 1-1-12A9 9 0 0 1 23 10a7 7 0 0 1 1 13Z" /><path d="M7 23a6 6 0 0 1-1-12A9 9 0 0 1 23 10a7 7 0 0 1 1 13Z" /><path v-if="level === 'off'" d="M12 16h8" /><path v-else-if="level === 'default'" class="reasoning-core" d="M12 17a4 4 0 1 1 7 2m0-4v4h-4" /><circle v-else cx="16" cy="16" r="2" class="reasoning-core" /></template>
    <g v-else-if="['low', 'medium'].includes(level)" class="reasoning-core"><path class="reasoning-wash" d="M16 24C5 24 5 13 7 9c6 0 10 7 9 15Z" /><path d="M16 27V16M16 24C5 24 5 13 7 9c6 0 10 7 9 15Z" /><path v-if="level === 'medium'" d="M16 19C16 10 21 6 27 5c1 8-5 14-11 14Z" /><circle cx="25" cy="23" r="1" /></g>
    <template v-else><circle class="reasoning-wash" cx="16" cy="16" r="13" /><path class="reasoning-core" d="m16 9 2 5 5 2-5 2-2 5-2-5-5-2 5-2Z" /><g class="reasoning-orbit"><ellipse cx="16" cy="16" rx="14" ry="9" transform="rotate(-35 16 16)" opacity=".55" /><ellipse v-if="level !== 'high'" cx="16" cy="16" rx="14" ry="9" transform="rotate(35 16 16)" opacity=".35" /></g><g class="reasoning-spark"><path d="m26 3 1 4 4 1-4 1-1 4-1-4-4-1 4-1Z" /><circle cx="5" cy="26" r="1" fill="currentColor" stroke="none" /></g><template v-if="level === 'max'"><circle cx="7" cy="5" r="1.6" fill="currentColor" stroke="none" /><circle cx="26" cy="27" r="1.4" fill="currentColor" stroke="none" /></template></template>
  </svg>
</template>
<script setup>
import { computed, ref, watch, onMounted, onBeforeUnmount } from 'vue'
const props = defineProps({ level: { type: String, default: 'unknown' }, running: Boolean })
const root = ref(null), visible = ref(false), foreground = ref(!document.hidden)
const animate = computed(() => props.running && props.level !== 'off' && visible.value && foreground.value)
let observer = null
function visibility() { foreground.value = !document.hidden }
function observe() {
  observer?.disconnect(); observer = null; visible.value = false
  if (!props.running || !root.value) return
  if (!globalThis.IntersectionObserver) { visible.value = true; return }
  observer = new IntersectionObserver(entries => { visible.value = entries[0]?.isIntersecting || false }); observer.observe(root.value)
}
watch(() => props.running, observe)
onMounted(() => { document.addEventListener('visibilitychange', visibility); observe() })
onBeforeUnmount(() => { observer?.disconnect(); document.removeEventListener('visibilitychange', visibility) })
</script>
