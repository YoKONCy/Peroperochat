<template>
  <section class="remote-greeting" aria-label="角色迎接区">
    <div class="remote-greeting-copy"><span class="remote-eyebrow">连接的另一端</span><h2>{{ agent?.name || '你的伙伴' }}</h2><p>{{ state?.vibe || state?.mood || agent?.description || '从上次的对话，继续聊下去。' }}</p><button class="remote-text-link" @click="$emit('profile')">查看角色 <RoomIcon name="arrow" /></button></div>
    <iframe v-if="model !== null && visible" :key="model" class="remote-live2d" :src="`/remote-character.html?model=${model}`" title="本机角色形象" sandbox="allow-scripts allow-same-origin" tabindex="-1" />
    <RemoteAvatar v-else class="remote-greeting-portrait" :id="agent?.id" :label="agent?.name" />
    <button class="room-icon-button remote-greeting-collapse" aria-label="收起角色迎接区" @click="$emit('collapse')"><RoomIcon name="up" /></button>
  </section>
</template>
<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'
import RoomIcon from '../companion/RoomIcon.vue'
import RemoteAvatar from './RemoteAvatar.vue'
defineProps({ agent: Object, state: Object, model: { type: Number, default: null } })
defineEmits(['collapse', 'profile'])
const visible = ref(!document.hidden)
function visibility() { visible.value = !document.hidden }
onMounted(() => document.addEventListener('visibilitychange', visibility))
onBeforeUnmount(() => document.removeEventListener('visibilitychange', visibility))
</script>
