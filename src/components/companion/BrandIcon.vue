<template>
  <span class="brand-icon" :style="{ '--brand-size': `${size}px` }" :title="label" role="img" :aria-label="label">
    <template v-if="source && !failed">
      <span v-if="definition.mono" class="brand-mono" :style="maskStyle" aria-hidden="true" />
      <img :key="source" :src="source" :class="{ 'brand-probe': definition.mono }" alt="" draggable="false" decoding="async" @error="onImageError" />
    </template>
    <RoomIcon v-else :name="fallback === 'provider' ? 'globe' : 'model'" />
  </span>
</template>

<script setup>
import { computed, ref, watch } from 'vue'
import RoomIcon from './RoomIcon.vue'
import { modelBrandDefinition } from '../../utils/modelBrands'

const props = defineProps({ brand: { type: String, default: '' }, size: { type: Number, default: 20 }, fallback: { type: String, default: 'model' } })
const failed = ref(false)
const definition = computed(() => modelBrandDefinition(props.brand))
// 图片路径仅来自本地品牌清单，不使用 API 地址或模型 ID 拼接资源地址。
const source = computed(() => definition.value ? `${import.meta.env.BASE_URL}model-brands/${definition.value.asset}` : '')
const label = computed(() => definition.value?.label || (props.fallback === 'provider' ? '自定义服务' : '模型'))
const maskStyle = computed(() => ({ maskImage: `url("${source.value}")`, WebkitMaskImage: `url("${source.value}")` }))
function onImageError(event) { if (event.target.getAttribute('src') === source.value) failed.value = true }
watch(source, () => { failed.value = false })
</script>

<style scoped>
.brand-icon { position: relative; display: inline-flex; align-items: center; justify-content: center; width: var(--brand-size); height: var(--brand-size); flex: none; vertical-align: middle; color: var(--room-ink); }
.brand-icon > img, .brand-icon > svg, .brand-mono { display: block; width: 100%; height: 100%; object-fit: contain; }
.brand-mono { background: currentColor; mask: no-repeat center / contain; -webkit-mask: no-repeat center / contain; }
.brand-icon > .brand-probe { position: absolute; opacity: 0; pointer-events: none; }
</style>
