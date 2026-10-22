<template><svg :class="['remote-icon', { 'svg-art': artwork, ornament: artwork && ['ribbon', 'constellation'].includes(name) }]" :viewBox="box" fill="none" stroke="currentColor" stroke-width="1.65" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false" v-html="markup" /></template>
<script setup>
import { computed } from 'vue'
import { softIcon } from '../../constants/softIcons'
import { remoteIconPaths, remoteArtwork } from '../../constants/remoteArtwork'
import { conversationIconPaths } from '../../constants/conversationIcons'
import { workspaceIcons } from '../../constants/workspaceIcons'
const props = defineProps({ name: { type: String, default: 'chat' }, artwork: Boolean })
const aliases = { arrow: 'next', document: 'files', diary: 'memory', journal: 'memory', tools: 'tool', volume: 'audio', constellation: 'orbit', heart: 'bow', model: 'tasks', sliders: 'settings', help: 'alert', connection: 'globe', remote: 'wifi', wardrobe: 'folder' }
const extras = { bell: 'M18 9a6 6 0 0 0-12 0v5l-2 4h16l-2-4ZM10 22h4M12 3V1', plus: 'M12 5v14M5 12h14', stop: 'M6 6h12v12H6Z', trash: 'M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7M14 10v7', edit: 'm14 5 5 5M4 20l1-6L16 3a2 2 0 0 1 3 0l2 2a2 2 0 0 1 0 3L10 19Z', up: 'm6 15 6-6 6 6', profile: 'M16 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0ZM4 21v-2a8 8 0 0 1 16 0v2' }
const box = computed(() => props.artwork ? remoteArtwork[props.name]?.box || '0 0 100 44' : '0 0 24 24')
// 这里只渲染编译进应用的图标常量；角色样式包不能提供 HTML 或 SVG 源码。
const richIcons = { diagram: '<rect x="8" y="2" width="8" height="6" rx="2"/><rect x="2" y="16" width="7" height="6" rx="2"/><rect x="15" y="16" width="7" height="6" rx="2"/><path d="M12 8v4m-7 4v-4h14v4"/>', formula: '<path d="M5 3h14M5 3l8 9-8 9h14M3 12h5"/>', play: '<path class="icon-wash" d="m8 4 12 8-12 8Z"/><path d="m8 4 12 8-12 8Z"/>', pause: '<rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/>', table: '<rect x="3" y="4" width="18" height="16" rx="3"/><path d="M3 10h18M10 4v16m0-5h11"/>' }
const markup = computed(() => props.artwork ? remoteArtwork[props.name]?.paths || '' : softIcon(props.name) || workspaceIcons[props.name] || conversationIconPaths[props.name] || richIcons[props.name] || remoteIconPaths[aliases[props.name] || props.name] || `<path d="${extras[props.name] || extras.profile}"/>`)
</script>
