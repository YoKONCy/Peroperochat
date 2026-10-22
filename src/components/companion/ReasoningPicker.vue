<template>
  <button type="button" class="reasoning-picker remote-select" :class="{ 'is-compact': compact }" :data-effort="shown" :disabled="disabled" :aria-label="`${tr('思考强度')}：${selectionLabel}`" aria-haspopup="dialog" :aria-expanded="open" @click="open = true"><ReasoningIcon :level="shown" /><span>{{ compact && shown !== 'unknown' ? tr('思考 · {0}', [selectionLabel]) : selectionLabel }}</span><RemoteIcon name="down" /></button>
  <RoomDrawer v-if="open" :title="tr('这次，要想多深？')" @close="open = false"><div class="reasoning-options" :aria-label="tr('思考强度')"><button v-for="item in choices" :key="item.value" type="button" :data-effort="item.level" :aria-pressed="modelValue === item.value" :class="{ selected: modelValue === item.value }" @click="choose(item.value)"><ReasoningIcon :level="item.level" :running="open && modelValue === item.value" /><span><strong>{{ item.label }}</strong><small>{{ item.hint }}</small></span><RemoteIcon v-if="modelValue === item.value" name="check" /></button></div><p class="reasoning-picker-hint">{{ tr(inherit ? '自动会沿用默认配置，只显示当前模型支持的选项。' : '不额外指定时，不向模型发送思考强度参数。') }}</p></RoomDrawer>
</template>
<script setup>
import { computed, ref, watch } from 'vue'
import RoomDrawer from './RoomDrawer.vue'
import RemoteIcon from '../remote/RemoteIcon.vue'
import ReasoningIcon from './ReasoningIcon.vue'
import { reasoningLevel, reasoningLabels, reasoningHints } from '../../utils/reasoningAppearance'
import { t } from '../../i18n/remote'
const props = defineProps({ modelValue: { type: String, default: '' }, options: { type: Array, default: () => [] }, effective: { type: String, default: 'unknown' }, inherit: Boolean, compact: Boolean, disabled: Boolean })
const emit = defineEmits(['update:modelValue'])
const tr = (key, values) => t(key, values, 'expression')
const open = ref(false)
const shown = computed(() => props.modelValue ? reasoningLevel(props.modelValue) : props.inherit ? props.effective : 'default')
const selectionLabel = computed(() => tr(props.modelValue === 'minimal' ? '极低' : reasoningLabels[shown.value] || '思考'))
const choices = computed(() => props.options.map(item => {
  const value = String(item.value ?? ''), level = value ? reasoningLevel(value) : props.inherit ? props.effective : 'default'
  return { value, level, label: !value ? tr(props.inherit ? '自动' : '不额外指定') : value === 'minimal' ? tr('极低') : tr(reasoningLabels[level] || item.label), hint: !value && props.inherit ? tr('当前：{0}', [tr(reasoningLabels[props.effective] || '思考')]) : tr(reasoningHints[level] || '交给模型自行决定') }
}))
function choose(value) { if (props.disabled) return; emit('update:modelValue', value); open.value = false }
watch(() => props.disabled, value => { if (value) open.value = false })
</script>
