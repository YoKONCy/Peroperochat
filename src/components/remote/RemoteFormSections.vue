<template>
  <div class="remote-form-sections"><details v-for="group in groups" :key="group.id" :open="group.primary || undefined" class="remote-form-section"><summary><RemoteIcon :name="group.icon" /><strong>{{ group.title }}</strong><small>{{ group.fields.length }} 项</small><RemoteIcon name="down" /></summary><RemoteFields :model-value="modelValue" :fields="group.fields" :disabled="disabled" @update:model-value="$emit('update:modelValue', $event)" /></details></div>
</template>
<script setup>
import { computed } from 'vue'
import RemoteFields from './RemoteFields.vue'
import RemoteIcon from './RemoteIcon.vue'
import { formSections } from '../../services/remoteManagementPresentation'
const props = defineProps({ modelValue: Object, fields: { type: Array, default: () => [] }, kind: String, disabled: Boolean })
defineEmits(['update:modelValue'])
const groups = computed(() => formSections(props.fields, props.kind))
</script>
