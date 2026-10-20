<template>
  <span class="room-switch">
    <input type="checkbox" role="switch" :checked="modelValue" :disabled="disabled" :aria-label="label || undefined" @change="$emit('update:modelValue', $event.target.checked)" />
    <span class="room-switch-track" aria-hidden="true" />
  </span>
</template>
<script setup>
defineProps({ modelValue: Boolean, disabled: Boolean, label: { type: String, default: '' } })
defineEmits(['update:modelValue'])
</script>
<style scoped>
.room-switch { position: relative; display: inline-grid; place-items: center; flex: none; width: 44px; height: 44px; vertical-align: middle; }
.room-switch > input { appearance: none; -webkit-appearance: none; position: absolute; inset: 0; width: 100%; height: 100%; min-height: 0; margin: 0; padding: 0; opacity: 0; cursor: pointer; }
.room-switch-track { display: block; width: 38px; height: 24px; padding: 3px; border: 1px solid var(--room-line); border-radius: 15px 15px 15px 7px; background: color-mix(in srgb, var(--room-muted) 25%, var(--room-surface)); pointer-events: none; transition: background .18s, border-color .18s; }
.room-switch-track::after { content: ''; display: block; width: 16px; height: 16px; border-radius: 50%; background: white; box-shadow: 0 1px 3px #28354d1a; transition: transform .2s var(--room-spring); }
.room-switch > input:checked + .room-switch-track { background: var(--room-accent); border-color: var(--room-accent); }
.room-switch > input:checked + .room-switch-track::after { transform: translateX(14px); }
.room-switch > input:focus-visible + .room-switch-track { outline: 2px solid var(--room-accent); outline-offset: 3px; }
.room-switch > input:disabled { cursor: default; }.room-switch > input:disabled + .room-switch-track { opacity: .45; }
@media (forced-colors: active) { .room-switch-track { background: Canvas; border-color: ButtonText; }.room-switch-track::after { background: ButtonText; }.room-switch > input:checked + .room-switch-track { background: Highlight; }.room-switch > input:checked + .room-switch-track::after { background: HighlightText; } }
</style>
