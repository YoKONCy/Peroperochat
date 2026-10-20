<template><svg :class="['remote-icon', { 'svg-art': artwork, ornament: artwork && ['ribbon', 'constellation'].includes(name) }]" :viewBox="box" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false" v-html="markup" /></template>
<script setup>
import { computed } from 'vue'
import { remoteIconPaths, remoteArtwork } from '../../constants/remoteArtwork'
const props = defineProps({ name: { type: String, default: 'chat' }, artwork: Boolean })
const aliases = { arrow: 'next', document: 'files', diary: 'memory', journal: 'memory', tools: 'tool', volume: 'audio', constellation: 'orbit', heart: 'bow', model: 'tasks', sliders: 'settings', help: 'alert', connection: 'globe', remote: 'wifi', wardrobe: 'folder' }
const extras = { bell: 'M18 9a6 6 0 0 0-12 0v5l-2 4h16l-2-4ZM10 22h4M12 3V1', plus: 'M12 5v14M5 12h14', stop: 'M6 6h12v12H6Z', trash: 'M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7M14 10v7', edit: 'm14 5 5 5M4 20l1-6L16 3a2 2 0 0 1 3 0l2 2a2 2 0 0 1 0 3L10 19Z', up: 'm6 15 6-6 6 6', profile: 'M16 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0ZM4 21v-2a8 8 0 0 1 16 0v2' }
const box = computed(() => props.artwork ? remoteArtwork[props.name]?.box || '0 0 100 44' : '0 0 24 24')
// 这里只渲染编译进应用的图标常量；角色样式包不能提供 HTML 或 SVG 源码。
const markup = computed(() => props.artwork ? remoteArtwork[props.name]?.paths || '' : remoteIconPaths[aliases[props.name] || props.name] || `<path d="${extras[props.name] || extras.profile}"/>`)
</script>
