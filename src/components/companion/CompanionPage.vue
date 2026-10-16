<template>
  <div class="companion-page" :style="viewportStyle">
    <CompanionHeader :title="title" :subtitle="subtitle" @back="$emit('back')"><slot name="actions" /></CompanionHeader>
    <main class="companion-page-body"><slot /></main>
  </div>
</template>

<script setup>
import CompanionHeader from './CompanionHeader.vue'
import { useRoomViewport } from '../../composables/useRoomViewport'
const { viewportStyle } = useRoomViewport()
defineProps({ title: { type: String, required: true }, subtitle: { type: String, default: '' } })
defineEmits(['back'])
</script>
