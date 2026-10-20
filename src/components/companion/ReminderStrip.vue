<template>
  <button type="button" class="reminder-strip" aria-label="查看和管理提醒" @click="$emit('open')">
    <span class="reminder-strip-icon"><RoomIcon name="bell" /><span class="reminder-strip-label" aria-hidden="true">提醒</span></span>
    <span v-if="error" class="reminder-strip-copy"><strong>提醒暂时无法读取</strong><small>点此重试与管理</small></span>
    <span v-else class="reminder-strip-copy"><strong>{{ entries[0]?.task }}</strong><small>{{ entries[0]?.deliveryError || entries[0]?.backgroundError ? '通知待处理 · 点此查看' : entries[0]?.time?.slice(5, 16) }}<span v-if="total > 1"> · 共 {{ total }} 项</span></small></span>
    <RoomIcon name="arrow" />
  </button>
</template>
<script setup>
import RoomIcon from './RoomIcon.vue'
defineProps({ entries: { type: Array, default: () => [] }, total: { type: Number, default: 0 }, error: { type: String, default: '' } })
defineEmits(['open'])
</script>
<style scoped>
.reminder-strip { position: absolute; z-index: 24; bottom: calc(var(--dock-height) + 12px); left: 14px; display: flex; align-items: center; gap: 10px; width: min(72vw, 300px); min-height: 50px; padding: 9px 12px; border: 1px solid var(--room-line); border-radius: 4px 12px 4px 4px; background: #fffffff2; text-align: left; box-shadow: 0 4px 14px #28354d0b; }
.reminder-strip-icon { display: grid; place-items: center; width: 28px; height: 30px; border-radius: 5px; color: var(--room-accent); background: var(--room-accent-soft); flex: none; }
.reminder-strip-icon svg { width: 17px; height: 17px; }
.reminder-strip-copy { display: flex; flex-direction: column; gap: 4px; flex: 1; min-width: 0; }
.reminder-strip strong { font-size: 11px; font-weight: 500; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
.reminder-strip small { font-size: 9px; color: var(--room-muted); }
.reminder-strip > svg { width: 12px; height: 12px; color: var(--room-muted); }
@media (min-width: 960px) { .reminder-strip { left: 28px; bottom: 28px; } }
</style>
