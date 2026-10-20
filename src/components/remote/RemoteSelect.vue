<template>
  <button type="button" class="remote-select" :disabled="disabled" :aria-label="`${label}：${selected?.label || placeholder}`" aria-haspopup="dialog" :aria-expanded="open" @click="open = true"><span>{{ selected?.label || placeholder }}</span><RemoteIcon name="down" /></button>
  <RoomDrawer v-if="open" :title="label" @close="open = false"><label v-if="options.length > 8" class="remote-search"><RemoteIcon name="search" /><input v-model="search" type="search" placeholder="查找选项" :aria-label="`查找${label}`" /></label><div class="remote-select-options"><button v-for="item in filtered" :key="String(item.value)" type="button" :aria-pressed="modelValue === item.value" :class="{ selected: modelValue === item.value }" @click="choose(item.value)"><span>{{ item.label }}<small v-if="item.description">{{ item.description }}</small></span><RemoteIcon v-if="modelValue === item.value" name="check" /></button><p v-if="!filtered.length" class="room-empty">没有匹配选项</p></div></RoomDrawer>
</template>
<script setup>
import { ref, computed, watch } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RemoteIcon from './RemoteIcon.vue'
const props = defineProps({ modelValue: { type: String, default: '' }, label: { type: String, default: '选择选项' }, placeholder: { type: String, default: '请选择' }, options: { type: Array, default: () => [] }, disabled: Boolean })
const emit = defineEmits(['update:modelValue'])
const open = ref(false), search = ref('')
const selected = computed(() => props.options.find(item => item.value === props.modelValue))
const filtered = computed(() => props.options.filter(item => `${item.label} ${item.description || ''}`.toLowerCase().includes(search.value.trim().toLowerCase())))
function choose(value) { if (props.disabled) return; emit('update:modelValue', value); open.value = false }
watch(open, () => { search.value = '' })
watch(() => props.disabled, value => { if (value) open.value = false })
</script>
<style scoped>
.remote-select { display:flex; align-items:center; justify-content:space-between; gap:12px; width:100%; min-height:44px; padding:11px 12px; border:1px solid var(--room-line); border-radius:12px 12px 12px 4px; background:var(--room-surface); color:var(--room-ink); text-align:left; font:inherit; }
.remote-select > span { flex:1; min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }.remote-select svg { width:15px; height:15px; color:var(--room-accent); }.remote-select:disabled { opacity:.45; }.remote-select-options { display:grid; gap:6px; }.remote-select-options > button { display:flex; align-items:center; gap:12px; min-height:48px; padding:12px; border:1px solid transparent; border-radius:12px 12px 12px 4px; background:transparent; color:var(--room-ink); text-align:left; font:inherit; font-size:13px; }.remote-select-options > button.selected { border-color:var(--room-line); background:var(--room-subtle); color:var(--room-accent); }.remote-select-options > button > span { flex:1; min-width:0; overflow-wrap:anywhere; }.remote-select-options small { display:block; color:var(--room-muted); font-size:11px; margin-top:5px; }.remote-select-options svg { width:18px; height:18px; color:var(--room-accent); }
</style>
