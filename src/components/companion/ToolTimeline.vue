<template>
  <details v-if="items.length" class="tool-timeline" @pointerdown.stop @click.stop @contextmenu.stop @toggle="$emit('resize')">
    <summary><RoomIcon name="model" /><span>工具 · {{ items.length }} 项</span><small>{{ stateLabel }}</small><RoomIcon class="tool-timeline-chevron" name="down" /></summary>
    <ol>
      <li v-for="(item, index) in items" :key="item.id" :data-status="item.status">
        <span class="tool-timeline-node" aria-hidden="true">{{ index + 1 }}</span>
        <div><div class="tool-timeline-heading"><strong>{{ Object.hasOwn(names, item.name) ? names[item.name] : '工具调用' }}</strong><span>{{ Object.hasOwn(statuses, item.status) ? statuses[item.status] : '已记录' }}</span></div><p v-if="item.summary">{{ item.summary }}</p></div>
      </li>
    </ol>
  </details>
</template>
<script setup>
import { computed } from 'vue'
import RoomIcon from './RoomIcon.vue'
const props = defineProps({ items: { type: Array, default: () => [] } })
defineEmits(['resize'])
const names = { query_diary: '查日记', write_diary: '写日记', update_character: '更新角色', query_reminders: '查提醒', manage_reminder: '管理提醒' }
const statuses = { running: '进行中', staged: '待保存', completed: '完成', failed: '失败', not_applied: '未保存', interrupted: '已中断' }
const stateLabel = computed(() => {
  if (props.items.some(item => item.status === 'running')) return '进行中'
  if (props.items.some(item => item.status === 'staged')) return '待保存'
  return props.items.some(item => ['failed', 'not_applied', 'interrupted'].includes(item.status)) ? '有未完成' : '已完成'
})
</script>
<style scoped>
.tool-timeline { min-width: 0; margin-bottom: 10px; border: 1px solid var(--room-line); border-radius: 11px 11px 11px 4px; background: color-mix(in srgb, var(--room-accent-soft) 55%, transparent); }
.tool-timeline > summary { display: flex; align-items: center; gap: 6px; min-height: 38px; padding: 8px 9px; color: var(--room-accent); font-size: 11px; cursor: pointer; list-style: none; }
.tool-timeline > summary::-webkit-details-marker { display: none; }.tool-timeline > summary::marker { content: ''; }
.tool-timeline > summary > svg { width: 14px; height: 14px; }.tool-timeline > summary > span { white-space: nowrap; }
.tool-timeline > summary > small { margin-left: auto; color: var(--room-muted); font-size: 10px; white-space: nowrap; }
.tool-timeline-chevron { transition: transform .18s; }.tool-timeline[open] .tool-timeline-chevron { transform: rotate(180deg); }
.tool-timeline ol { list-style: none; margin: 0; padding: 2px 10px 10px; }
.tool-timeline li { position: relative; display: flex; gap: 8px; padding: 7px 0 9px; }
.tool-timeline li:not(:last-child)::before { content: ''; position: absolute; top: 27px; bottom: -7px; left: 9px; width: 1px; background: var(--room-line); }
.tool-timeline-node { position: relative; z-index: 1; display: grid; place-items: center; width: 19px; height: 19px; flex: none; border: 1px solid var(--room-accent-bright); border-radius: 50%; background: var(--room-surface); color: var(--room-accent); font-size: 9px; font-variant-numeric: tabular-nums; }
.tool-timeline li > div { flex: 1; min-width: 0; }.tool-timeline-heading { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 4px 8px; min-height: 19px; }
.tool-timeline-heading strong { color: var(--room-ink); font-size: 11px; font-weight: 500; }.tool-timeline-heading > span { color: var(--room-muted); font-size: 10px; }
.tool-timeline p { margin: 4px 0 0; color: var(--room-muted); font-size: 11px; line-height: 1.6; white-space: pre-wrap; overflow-wrap: anywhere; }
.tool-timeline li[data-status='failed'] .tool-timeline-heading > span, .tool-timeline li[data-status='failed'] p { color: var(--room-danger); }
.tool-timeline li[data-status='not_applied'] .tool-timeline-node, .tool-timeline li[data-status='interrupted'] .tool-timeline-node { border-style: dashed; color: var(--room-muted); }
</style>
