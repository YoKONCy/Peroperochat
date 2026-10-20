<template>
  <nav class="file-breadcrumbs" aria-label="文件位置">
    <button :disabled="disabled || path === '.'" :aria-current="path === '.' ? 'location' : undefined" @click="$emit('navigate', '.')"><RemoteIcon name="folder" />会话文件</button>
    <template v-for="(part, index) in parts" :key="part.path"><RemoteIcon name="next" /><button :disabled="disabled || index === parts.length - 1" :aria-current="index === parts.length - 1 ? 'location' : undefined" :title="part.name" @click="$emit('navigate', part.path)">{{ part.name }}</button></template>
  </nav>
</template>
<script setup>
import { computed } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
const props = defineProps({ path: { type: String, default: '.' }, disabled: Boolean })
defineEmits(['navigate'])
const parts = computed(() => {
  const names = props.path.split('/').filter(part => part && part !== '.')
  return names.map((name, index) => ({ name, path: `${props.path.startsWith('/') ? '/' : ''}${names.slice(0, index + 1).join('/')}` }))
})
</script>
