<template>
  <span ref="root" class="outfit-thumbnail">
    <svg v-if="visible && source && !failed" viewBox="115 0 280 390" role="img" :aria-label="`${label}的服装细节`">
      <image :href="source" width="1024" height="1024" transform="translate(0 610) rotate(-90)" @error="failed = true" />
    </svg>
    <RoomIcon v-else name="hanger" />
  </span>
</template>
<script setup>
import { computed, onMounted, onBeforeUnmount, ref, watch } from 'vue'
import RoomIcon from './RoomIcon.vue'
const props = defineProps({ texture: { type: String, default: '' }, label: { type: String, default: '' } })
const root = ref(null), visible = ref(false), failed = ref(false)
// Pio 贴图中的衣服主体横放在固定区域，裁出并转正；不展示头发、眼睛等拆件。
const source = computed(() => /^textures\/[a-zA-Z0-9_-]+\.png$/.test(props.texture) ? `/live2d-widget/model/potion-Maker-Pio/${props.texture}` : '')
let observer = null
watch(source, () => { failed.value = false })
onMounted(() => {
  if (!window.IntersectionObserver) { visible.value = true; return }
  observer = new IntersectionObserver(items => { if (items.some(item => item.isIntersecting)) { visible.value = true; observer.disconnect() } }, { rootMargin: '100px' })
  observer.observe(root.value)
})
onBeforeUnmount(() => observer?.disconnect())
</script>
<style scoped>
.outfit-thumbnail { display: grid; place-items: center; width: 100%; aspect-ratio: 1.12; overflow: hidden; background: radial-gradient(ellipse at 50% 35%, white, var(--room-accent-soft)); border-bottom: 1px solid var(--room-line); }
.outfit-thumbnail > svg[role='img'] { display: block; width: 74%; height: 94%; overflow: hidden; filter: drop-shadow(0 3px 3px #28354d19); }
.outfit-thumbnail > svg:not([role='img']) { width: 26px; height: 26px; color: var(--room-accent); opacity: .6; }
</style>
