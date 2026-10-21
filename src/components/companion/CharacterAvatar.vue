<template>
  <span class="character-avatar" :class="{ 'is-empty': !source || failed, 'shows-full-image': fit === 'contain' }">
    <img v-if="source && !failed" :key="source" :src="source" :alt="label" decoding="async" @error="failed = true" />
    <RoomIcon v-else name="profile" :aria-label="label || '角色头像'" />
  </span>
</template>

<script setup>
import { computed, ref, watch } from 'vue'
import RoomIcon from './RoomIcon.vue'
const props = defineProps({ src: { type: String, default: '' }, label: { type: String, default: '' }, fit: { type: String, default: 'cover' } })
const source = computed(() => typeof props.src === 'string' ? props.src.trim() : '')
const failed = ref(false)
watch(source, () => { failed.value = false })
</script>

<style scoped>
:where(.character-avatar) { display: inline-grid; place-items: center; width: 42px; height: 42px; flex: none; overflow: hidden; background: var(--room-accent-soft); color: var(--room-accent); }
.character-avatar img { display: block; width: 100%; height: 100%; object-fit: cover; object-position: 50% 37%; }
.character-avatar.shows-full-image img { object-fit: contain; object-position:center; }
.character-avatar.is-empty svg { width: 42%; height: 42%; }
</style>
