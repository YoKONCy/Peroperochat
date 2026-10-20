<template>
  <section class="remote-greeting" aria-label="角色迎接区">
    <div class="remote-greeting-copy"><span class="remote-eyebrow">连接的另一端</span><h2>{{ agent?.name || '你的伙伴' }}</h2><p>{{ state?.vibe || state?.mood || agent?.description || '从上次的对话，继续聊下去。' }}</p></div>
    <img v-if="image && !failed" class="remote-greeting-cg" :src="image" :alt="`${agent?.name || '角色'}的 CG`" @error="failed = true" />
    <RemoteAvatar v-else class="remote-greeting-portrait" :id="agent?.id" :label="agent?.name" />
    <button class="room-icon-button remote-greeting-collapse" aria-label="收起角色迎接区" @click="$emit('collapse')"><RoomIcon name="up" /></button>
  </section>
</template>
<script setup>
import { ref, watch } from 'vue'
import RoomIcon from './RemoteIcon.vue'
import RemoteAvatar from './RemoteAvatar.vue'
const props = defineProps({ agent: Object, state: Object, image: String })
defineEmits(['collapse'])
const failed = ref(false)
watch(() => props.image, () => { failed.value = false })
</script>
