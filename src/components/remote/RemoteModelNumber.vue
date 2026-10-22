<template>
  <div class="model-number" :data-ui-tone="tone" :class="{ 'is-custom': value != null, 'is-long': String(value ?? '').length > 5 }">
    <div class="model-number-heading"><RemoteIcon :name="icon" /><label :for="inputId">{{ t(label) }}</label><button v-if="!required" type="button" :disabled="disabled || value == null" :aria-label="t('将{0}恢复默认', [t(label)])" @click="$emit('change', null)">{{ t(value == null ? '默认' : '重置') }}</button></div>
    <div class="model-number-value"><input :id="inputId" :value="value ?? ''" :disabled="disabled" :required="required" type="number" :inputmode="integer ? 'numeric' : 'decimal'" :min="min" :max="max" :step="integer ? 1 : 'any'" :placeholder="t('默认')" @input="change" /><span v-if="unit">{{ unit }}</span></div>
    <small>{{ t(help) }}</small>
  </div>
</template>
<script setup>
import { useId } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
import { t } from '../../i18n/remote'
defineProps({ value: Number, label: String, help: String, icon: String, unit: String, min: Number, max: Number, integer: Boolean, disabled: Boolean, required: Boolean, tone: { type: String, default: 'blue' } })
const emit = defineEmits(['change'])
const inputId = `model-number-${useId()}`
function change(event) {
  if (event.target.validity.badInput) return
  const value = event.target.value === '' ? null : Number(event.target.value)
  if (value === null || Number.isFinite(value)) emit('change', value)
}
</script>
<style scoped>
.model-number{min-width:0;padding:13px 12px 11px;border:1px solid var(--room-line);border-radius:15px;background:var(--room-surface);transition:border-color .15s,background .15s}
.model-number.is-custom{border-color:color-mix(in srgb,var(--room-accent) 32%,var(--room-line));background:color-mix(in srgb,var(--room-accent) 3%,var(--room-surface))}
.model-number-heading{display:flex;align-items:center;gap:6px;min-width:0}
.model-number-heading>svg{width:16px;height:16px;flex:none;color:var(--room-accent)}
.model-number-heading>label{flex:1;min-width:0;font-size:11px;color:var(--room-ink);font-weight:550}
.model-number-heading>button{flex:none;min-height:32px;padding:3px 5px;border:0;border-radius:6px;background:transparent;color:var(--room-accent);font:inherit;font-size:10px;cursor:pointer}
.model-number-heading>button:disabled{color:var(--room-muted);cursor:default}
.model-number-value{display:flex;align-items:baseline;gap:4px;min-width:0;margin:4px 0 7px}
.model-number-value>input{width:100%;min-width:0;min-height:44px;padding:2px 0;border:0;border-radius:4px;background:transparent;color:var(--room-ink);font:inherit;font-size:23px;font-weight:600;font-variant-numeric:tabular-nums;outline:none;appearance:textfield;-moz-appearance:textfield}
.model-number-value>input::-webkit-inner-spin-button,.model-number-value>input::-webkit-outer-spin-button{-webkit-appearance:none;margin:0}
.model-number-value>input::placeholder{color:var(--room-muted);font-size:19px;font-weight:400;opacity:.65}
.model-number-value>input:focus-visible{box-shadow:0 2px var(--room-accent)}
.model-number-value>span{font-size:10px;color:var(--room-muted);flex:none}
.model-number>small{display:block;font-size:10px;line-height:1.6;color:var(--room-muted)}
.model-number:focus-within{border-color:var(--room-accent)}
.model-number-heading>button:focus-visible{outline:2px solid var(--room-accent);outline-offset:1px}
@media(max-width:359px){.model-number{padding:10px}.model-number-value>input{font-size:20px}.model-number-heading{gap:4px}}
@media(prefers-reduced-motion:reduce){.model-number{transition:none}}
:global([data-reduce-motion=true]) .model-number{transition:none}
</style>
