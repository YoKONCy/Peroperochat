<template>
  <svg class="room-accent-mark" :data-kind="kind" viewBox="0 0 40 24" fill="none" stroke="currentColor" stroke-width="1.25" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
    <g v-if="kind === 'orbit'">
      <path d="M4 16c5-8 19-13 29-10M8 19c8-1 17-6 22-12" opacity=".5" />
      <path d="m22 5 1.8 4.2L28 11l-4.2 1.8L22 17l-1.8-4.2L16 11l4.2-1.8Z" fill="currentColor" fill-opacity=".12" />
      <circle cx="8" cy="10" r="1.3" fill="currentColor" stroke="none" /><path d="M33 15v4m-2-2h4" />
    </g>
    <g v-else>
      <path d="m16 3 2.3 6.7L25 12l-6.7 2.3L16 21l-2.3-6.7L7 12l6.7-2.3Z" fill="currentColor" fill-opacity=".1" />
      <path d="m30 3 1.1 3.9L35 8l-3.9 1.1L30 13l-1.1-3.9L25 8l3.9-1.1Z" />
      <circle cx="32" cy="19" r="1.2" fill="currentColor" stroke="none" /><path d="M3 5h2m-1-1v2" opacity=".55" />
    </g>
  </svg>
</template>
<script setup>
defineProps({ kind: { type: String, default: 'sparkle' } })
</script>
<style scoped>
.room-accent-mark { display: inline-block; width: 32px; height: 20px; flex: none; color: var(--room-accent); pointer-events: none; }
</style>
