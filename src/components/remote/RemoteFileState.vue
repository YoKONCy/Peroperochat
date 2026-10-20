<template>
  <div class="remote-file-state" :class="{ 'is-loading': loading }" role="status">
    <div class="remote-file-state-art" aria-hidden="true">
      <svg viewBox="0 0 160 120" fill="none"><ellipse cx="80" cy="102" rx="51" ry="7" /><path d="M47 35h63v60H47Z" /><path d="m55 24 58 8-8 61-58-8Z" /><path d="m30 43 3 8 8 3-8 3-3 8-3-8-8-3 8-3Zm100 24 2 6 6 2-6 2-2 6-2-6-6-2 6-2Z" /><circle cx="126" cy="32" r="2" /><path d="M45 21h5m-2.5-2.5v5" /></svg>
      <RemoteIcon :name="icon" />
    </div>
    <h3>{{ title }}</h3><p v-if="description">{{ description }}</p><slot />
  </div>
</template>
<script setup>
import RemoteIcon from './RemoteIcon.vue'
defineProps({ title: String, description: String, icon: { type: String, default: 'files' }, loading: Boolean })
</script>
