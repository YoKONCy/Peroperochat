<template>
  <header class="companion-page-header">
    <button class="room-icon-button page-back" :aria-label="backLabel" @click="$emit('back')"><RoomIcon name="back" /></button>
    <CompanionEmblem class="page-emblem" />
    <div class="companion-page-title"><span class="page-kicker">{{ activeAgent?.name || 'PeroperoChat' }} · 陪伴空间</span><h1>{{ title }}</h1><p v-if="subtitle">{{ subtitle }}</p></div>
    <div class="companion-page-actions"><slot /></div>
    <CharacterAvatar v-if="activeAgent" class="page-partner" :src="activeAgent.avatar" :label="activeAgent.name" />
  </header>
</template>

<script setup>
import RoomIcon from './RoomIcon.vue'
import CharacterAvatar from './CharacterAvatar.vue'
import CompanionEmblem from './CompanionEmblem.vue'
import { activeAgent } from '../../stores/agents'
defineProps({ title: { type: String, required: true }, subtitle: { type: String, default: '' }, backLabel: { type: String, default: '返回' } })
defineEmits(['back'])
</script>
