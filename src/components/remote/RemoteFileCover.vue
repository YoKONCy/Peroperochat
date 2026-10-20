<template>
  <span class="remote-file-cover" :data-tone="tone" aria-hidden="true">
    <svg class="file-paper" viewBox="0 0 56 68" fill="none">
      <path class="file-paper-shadow" d="M10 8h28l12 13v41H10Z" />
      <path class="file-paper-face" d="M6 3h29l13 13v43a4 4 0 0 1-4 4H10a4 4 0 0 1-4-4Z" />
      <path class="file-paper-fold" d="M35 3v10a3 3 0 0 0 3 3h10" />
      <path class="file-paper-lines" d="M15 25h17m-17 6h22m-22 6h12" />
      <path class="file-paper-star" d="m39 34 1.6 4.4L45 40l-4.4 1.6L39 46l-1.6-4.4L33 40l4.4-1.6Z" />
      <path class="file-paper-seal" d="M6 47h29l-3 6 3 6H6Z" />
    </svg>
    <small>{{ type }}</small>
  </span>
</template>
<script setup>
import { computed } from 'vue'
const props = defineProps({ name: String })
const type = computed(() => props.name?.includes('.') ? props.name.split('.').at(-1).slice(0, 6).toUpperCase() || 'FILE' : 'FILE')
const tone = computed(() => ['XLS', 'XLSX', 'CSV'].includes(type.value) ? 'green' : ['PDF', 'PPT', 'PPTX'].includes(type.value) ? 'rose' : ['MD', 'TXT', 'JSON', 'ZIP'].includes(type.value) ? 'purple' : 'blue')
</script>
<style scoped>
.remote-file-cover { width: 44px; height: 54px; transform: none; --file-color: #6a9fc4; --file-wash: #eaf5fc; position: relative; flex: none; display: inline-block; }
.remote-file-cover[data-tone='green'] { --file-color: #609f8b; --file-wash: #e9f6ef; }
.remote-file-cover[data-tone='purple'] { --file-color: #a08abf; --file-wash: #f0eafa; }
.remote-file-cover[data-tone='rose'] { --file-color: #bf819b; --file-wash: #faedf2; }
.file-paper { width: 100%; height: 100%; overflow: visible; }
.file-paper-shadow { fill: var(--file-color); opacity: .12; }
.file-paper-face { fill: var(--file-wash); stroke: var(--file-color); stroke-width: .8; }
.file-paper-fold { fill: color-mix(in srgb, var(--file-color) 18%, var(--file-wash)); stroke: var(--file-color); stroke-width: .8; }
.file-paper-lines { stroke: var(--file-color); stroke-width: 1.4; stroke-linecap: round; opacity: .5; }
.file-paper-star { fill: var(--file-color); opacity: .8; }
.file-paper-seal { fill: var(--file-color); }
.remote-file-cover > small { position: absolute; inset: auto auto 14% 16%; bottom: 14%; left: 16%; padding: 0; background: none; color: white; font: 700 var(--file-label-size, 6px)/1 ui-sans-serif, system-ui; letter-spacing: .04em; }
</style>
