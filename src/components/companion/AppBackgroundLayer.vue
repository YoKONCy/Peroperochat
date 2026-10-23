<template>
  <div v-if="background?.source" class="app-background-layer" aria-hidden="true"><img :key="background.source" :src="background.source" alt="" decoding="async" @error="failed" /><i /></div>
</template>
<script setup>
const props = defineProps({ background: Object })
function failed(event) {
  // 图片移除或解码异常时停止透景，保留正常可读的默认界面。
  if (event.target.getAttribute('src') === props.background?.source) props.background.imageFailed()
}
</script>
