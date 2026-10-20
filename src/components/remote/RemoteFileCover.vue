<template><span class="remote-file-cover" :data-tone="tone"><RemoteIcon :name="`cover-${tone}`" artwork /><small>{{ type }}</small></span></template>
<script setup>
import { computed } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
const props = defineProps({ name: String })
const type = computed(() => props.name?.split('.').at(-1)?.slice(0, 6).toUpperCase() || 'FILE')
const tone = computed(() => ['XLS', 'XLSX', 'CSV'].includes(type.value) ? 'green' : ['MD', 'TXT', 'JSON'].includes(type.value) ? 'purple' : 'blue')
</script>
