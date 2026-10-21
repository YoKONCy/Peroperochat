<template>
  <div class="remote-fields">
    <template v-for="field in fields" :key="field.key">
      <div v-if="field.type !== 'object' && field.type !== 'array'" class="remote-field" :class="{ 'is-switch': field.type === 'boolean' }">
        <span>{{ t(field.label || labels[field.key] || field.key) }}<button v-if="removable" type="button" class="remote-field-remove" :disabled="disabled" @click.prevent="removeKey(field.key)">移除字段</button><small v-if="field.help">{{ t(field.help) }}</small></span>
        <RoomSwitch v-if="field.type === 'boolean'" :model-value="!!modelValue?.[field.key]" :disabled="disabled" :label="field.label || field.key" @update:model-value="set(field.key, $event)" />
        <RemoteSelect v-else-if="field.choices" :model-value="String(modelValue?.[field.key] ?? '')" :label="t(field.label || labels[field.key] || field.key)" :options="options(field)" :disabled="disabled" @update:model-value="set(field.key, choice(field, $event))" />
        <textarea v-else-if="field.type === 'long'" :value="modelValue?.[field.key] ?? ''" :aria-label="t(field.label || labels[field.key] || field.key)" :disabled="disabled" :rows="field.rows || 4" :maxlength="field.max || 524288" @input="set(field.key, $event.target.value)" />
        <input v-else :aria-label="t(field.label || labels[field.key] || field.key)" :type="field.type === 'secret' ? 'password' : field.type === 'number' ? 'number' : 'text'" :value="modelValue?.[field.key] ?? ''" :disabled="disabled" :required="field.required" :min="field.min" :max="field.max" :step="field.type === 'number' ? 'any' : undefined" :autocomplete="field.type === 'secret' ? 'new-password' : 'off'" :placeholder="field.nullable ? '继承 / 不设置' : field.placeholder" @input="set(field.key, field.type === 'number' ? ($event.target.value === '' ? null : Number($event.target.value)) : field.nullable && $event.target.value === '' ? null : $event.target.value)" />
      </div>
      <details v-else class="remote-field-group" :open="field.open || undefined">
        <summary>{{ t(field.label || labels[field.key] || field.key) }}<small>{{ field.type === 'array' ? `${(modelValue?.[field.key] || []).length} 项` : '展开编辑' }}</small></summary>
        <template v-if="field.type === 'object'"><RemoteFields :model-value="modelValue?.[field.key] || {}" :fields="field.fields || infer(modelValue?.[field.key] || {})" :removable="field.dictionary !== false" :disabled="disabled" @update:model-value="set(field.key, $event)" /><div v-if="field.dictionary !== false" class="remote-dictionary-add"><input v-model="newKeys[field.key]" :disabled="disabled" aria-label="新字段名称" placeholder="新字段名称" /><button type="button" :disabled="disabled || !newKeys[field.key]?.trim()" @click="addKey(field.key)">添加字段</button></div></template>
        <template v-else><div v-for="(item, index) in modelValue?.[field.key] || []" :key="index" class="remote-array-item">
          <strong>{{ item?.name || item?.id || `第 ${index + 1} 项` }}</strong>
          <RemoteFields v-if="item && typeof item === 'object' && !Array.isArray(item)" :model-value="item" :fields="field.fields || infer(item)" :disabled="disabled" @update:model-value="replace(field.key, index, $event)" />
          <RemoteSelect v-else-if="field.itemChoices" :model-value="String(item)" :label="t(field.label || labels[field.key] || field.key)" :options="field.itemChoices.map(option => ({ ...option, value: String(option.value) }))" :disabled="disabled" @update:model-value="replace(field.key, index, typeof item === 'number' ? Number($event) : $event)" />
          <textarea v-else :value="typeof item === 'string' ? item : JSON.stringify(item)" :disabled="disabled" rows="2" @input="replace(field.key, index, typeof item === 'number' ? Number($event.target.value) : $event.target.value)" />
          <div class="remote-manager-actions"><button type="button" :disabled="disabled || index === 0" @click="move(field.key, index, -1)">上移</button><button type="button" :disabled="disabled || index === modelValue[field.key].length - 1" @click="move(field.key, index, 1)">下移</button><button type="button" :disabled="disabled" @click="remove(field.key, index)">移除</button></div>
        </div><button type="button" class="room-text-button" :disabled="disabled" @click="add(field)">添加一项</button></template>
      </details>
    </template>
  </div>
</template>
<script setup>
import RoomSwitch from '../companion/RoomSwitch.vue'
import RemoteSelect from './RemoteSelect.vue'
import { reactive } from 'vue'
import { t, remoteChoiceLabel } from '../../i18n/remote'
import { fieldLabels as baseLabels, inferFields as infer, arrayDefaults, cloneData } from '../../services/remoteManagement'
const labels = { ...baseLabels, gender: '性别', identity: '身份', appearance: '外貌', personality: '性格' }
const props = defineProps({ modelValue: { type: Object, default: () => ({}) }, fields: { type: Array, default: () => [] }, disabled: Boolean, removable: Boolean })
const emit = defineEmits(['update:modelValue'])
const newKeys = reactive({})
function set(key, value) { emit('update:modelValue', { ...props.modelValue, [key]: value }) }
function removeKey(key) { const value = { ...props.modelValue }; delete value[key]; emit('update:modelValue', value) }
function choice(field, value) { const item = field.choices.find(item => String(item.value ?? item) === value); return value === '' && field.nullable ? null : item?.value ?? value }
function options(field) { return [...(field.nullable ? [{ value: '', label: t('继承 / 不设置') }] : []), ...field.choices.map(item => ({ value: String(item.value ?? item), label: item.label ? (field.translateChoices === false ? item.label : t(item.label)) : item === '' ? t('全部 / 不限制') : remoteChoiceLabel(item.value ?? item), description: field.translateChoices === false ? item.description : item.description ? t(item.description) : undefined, disabled: item.disabled }))] }
function replace(key, index, value) { const list = [...props.modelValue[key]]; list[index] = value; set(key, list) }
function remove(key, index) { set(key, props.modelValue[key].filter((_, i) => i !== index)) }
function move(key, index, delta) { const list = [...props.modelValue[key]]; [list[index], list[index + delta]] = [list[index + delta], list[index]]; set(key, list) }
function add(field) { const fallback = field.itemDefault ?? arrayDefaults[field.key] ?? ''; set(field.key, [...(props.modelValue[field.key] || []), cloneData(fallback)]) }
function addKey(key) { const name = newKeys[key]?.trim(); if (!name || ['__proto__', 'constructor', 'prototype'].includes(name) || Object.hasOwn(props.modelValue[key] || {}, name)) return; set(key, { ...(props.modelValue[key] || {}), [name]: '' }); newKeys[key] = '' }
</script>
