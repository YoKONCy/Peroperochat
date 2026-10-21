<template>
  <FeedbackMessage :message="text" :tone="tone" :duration="duration" :action-label="actionLabel" @dismiss="$emit('dismiss')" @action="$emit('action')" />
</template>
<script setup>
import { computed } from 'vue'
import FeedbackMessage from '../FeedbackMessage.vue'
import { remoteFeedbackMessage } from '../../services/remoteFeedback.js'
const props = defineProps({ message: [String, Object], tone: { type: String, default: 'error' }, duration: Number, actionLabel: String })
defineEmits(['dismiss', 'action'])
const text = computed(() => remoteFeedbackMessage(props.message))
</script>
