<template>
  <button class="journal-photo" type="button" :aria-label="label" @click="expanded = true"><img :src="portrait" alt="" width="52" height="50"><i aria-hidden="true">Pero</i></button>
  <RoomDrawer v-if="expanded" title="Pero" @close="expanded = false"><img class="journal-portrait-print" :src="portrait" alt="Pero" width="384" height="384"></RoomDrawer>
</template>
<script setup>
import { ref } from 'vue'
import portrait from '../../assets/journal/pero-sketch.webp'
import RoomDrawer from './RoomDrawer.vue'
defineProps({ label: { type: String, default: 'Pero' } })
const expanded = ref(false)
</script>
