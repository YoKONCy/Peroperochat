<template><button :type="type" class="remote-purpose-action" :class="`is-${tone}`" :disabled="disabled || busy" :aria-busy="busy || undefined"><RemoteIcon :name="busy ? 'refresh' : icon" /><span><slot /></span></button></template>
<script setup>
import RemoteIcon from './RemoteIcon.vue'
defineProps({ icon: { type: String, default: 'next' }, tone: { type: String, default: 'secondary' }, type: { type: /** @type {import('vue').PropType<'button' | 'submit' | 'reset'>} */ (String), default: 'button' }, disabled: Boolean, busy: Boolean })
</script>
