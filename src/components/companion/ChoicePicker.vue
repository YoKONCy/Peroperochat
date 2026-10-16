<template>
  <button type="button" class="choice-picker" :disabled="disabled" aria-haspopup="dialog" :aria-expanded="open" :aria-label="`${label}：${displayLabel || selected?.label || modelValue || placeholder}`" @click="open = true">
    <slot name="leading" :option="selected" /><span>{{ displayLabel || selected?.shortLabel || selected?.label || modelValue || placeholder }}</span><RoomIcon class="choice-chevron" name="down" />
  </button>
  <RoomDrawer v-if="open" :title="label" :tone="tone" @close="open = false">
    <input v-if="options.length > 8" v-model="search" class="control-input choice-search" type="search" placeholder="搜索选项" :aria-label="`搜索${label}`" />
    <div class="choice-options"><button v-for="option in filtered" :key="option.value" type="button" :aria-pressed="modelValue === option.value" :class="{ selected: modelValue === option.value }" @click="choose(option.value)"><slot name="option-leading" :option="option" /><span class="choice-option-label"><strong>{{ option.label }}</strong><small v-if="option.description">{{ option.description }}</small></span><RoomIcon v-if="modelValue === option.value" class="choice-check" name="check" /></button></div>
    <p v-if="!filtered.length" class="room-empty">没有匹配选项</p>
  </RoomDrawer>
</template>

<script setup>
import { computed, ref, watch } from 'vue'
import { activeAgentId } from '../../stores/agents'
import RoomDrawer from './RoomDrawer.vue'
import RoomIcon from './RoomIcon.vue'
const props = defineProps({ modelValue: { type: String, default: '' }, label: { type: String, required: true }, displayLabel: { type: String, default: '' }, placeholder: { type: String, default: '请选择' }, options: { type: Array, required: true }, disabled: Boolean })
const emit = defineEmits(['update:modelValue', 'change'])
const open = ref(false)
const search = ref('')
const tone = computed(() => activeAgentId.value === 'nana' ? 'nana' : 'pero')
const selected = computed(() => props.options.find(option => option.value === props.modelValue))
const filtered = computed(() => props.options.filter(option => `${option.label} ${option.description || ''}`.toLowerCase().includes(search.value.trim().toLowerCase())))
function choose(value) { if (props.disabled) return; emit('update:modelValue', value); emit('change', value); open.value = false }
watch(open, () => { search.value = '' })
watch(() => props.disabled, value => { if (value) open.value = false })
</script>

<style scoped>
.choice-picker { display: flex; align-items: center; justify-content: space-between; gap: 8px; width: 100%; min-height: 40px; padding: 10px 12px; border: 1px solid var(--room-line); border-radius: 5px; background: var(--room-subtle); color: var(--room-ink); text-align: left; }
.choice-picker > span:not(.brand-icon) { flex: 1; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; font-size: 13px; }
.choice-chevron { width: 13px; height: 13px; color: var(--room-accent); transition: transform .22s var(--room-ease); }
.choice-picker[aria-expanded='true'] .choice-chevron { transform: rotate(180deg); }
.choice-picker:hover { border-color: var(--room-accent); }
.choice-search { margin: 0 0 12px; }
.choice-options { display: flex; flex-direction: column; }
.choice-options button { display: flex; align-items: center; justify-content: space-between; gap: 14px; min-height: 48px; border: 0; border-bottom: 1px solid var(--room-line); padding: 13px 4px; background: transparent; text-align: left; }
.choice-option-label { display: flex; flex: 1; min-width: 0; flex-direction: column; gap: 6px; overflow-wrap: anywhere; }
.choice-options strong { font-size: 13px; font-weight: 500; }
.choice-options small { color: var(--room-muted); font-size: 11px; line-height: 1.6; }
.choice-options button.selected { color: var(--room-accent); }
.choice-check { width: 17px; height: 17px; animation: room-check-in .26s var(--room-spring); }
@media (hover: hover) { .choice-options button:hover { background: var(--room-subtle); } }
</style>
