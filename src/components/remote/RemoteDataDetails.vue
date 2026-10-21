<template><details class="remote-diagnostic-data" @toggle="open = $event.currentTarget.open"><summary><RemoteIcon name="code" />{{ label }}</summary><RemoteValue v-if="open" :value="value" /></details></template>
<script setup>
import { ref } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteValue from './RemoteValue.vue'
defineProps({ value: { default: null }, label: { type: String, default: '详细数据' } })
const open = ref(false)
</script>
