<template><details class="expression-reasoning" :data-effort="effort"><summary><span class="expression-thought-symbol"><ReasoningIcon :level="effort" :running="running" /></span><strong>{{ tr(running ? '正在思考' : '思考过程') }}<small v-if="effort !== 'unknown'" class="expression-effort-label">{{ tr(reasoningLabels[effort]) }}</small></strong><RoomIcon class="soft-fold-chevron" name="down" /></summary><div class="expression-reasoning-body markdown-body" v-html="render(source)" /></details></template>
<script setup>
import { computed } from 'vue'
import ReasoningIcon from './ReasoningIcon.vue'
import RoomIcon from './RoomIcon.vue'
import { roomMarkdown } from '../../utils/roomMarkdown'
import { recordedReasoning, reasoningLabels } from '../../utils/reasoningAppearance'
import { createTextRenderCache } from '../../utils/textRenderCache'
import { t } from '../../i18n/remote'
const props = defineProps({ source: { type: String, default: '' }, running: Boolean, generation: Object })
const tr = key => t(key, [], 'expression')
const effort = computed(() => recordedReasoning(props.generation))
const render = createTextRenderCache(roomMarkdown)
</script>
