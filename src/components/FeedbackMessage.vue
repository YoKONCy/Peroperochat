<template><!-- 气泡由全局浮层展示，不占据页面空间。 --></template>
<script setup>
import { watch, inject, onBeforeUnmount } from 'vue'
import { showFeedback } from '../services/appFeedback'
import { REMOTE_THEME } from '../services/remoteTheme'
import { LOCAL_APPEARANCE } from '../stores/appearance'
const props = defineProps({ message: [String, Object], tone: { type: String, default: 'error' }, duration: Number, actionLabel: String })
const emit = defineEmits(['dismiss', 'action'])
const remote = inject(REMOTE_THEME, null), local = inject(LOCAL_APPEARANCE, null)
let handle = null
watch(() => [typeof props.message === 'string' ? props.message : props.message?.message, props.tone], () => {
  handle?.close()
  const message = typeof props.message === 'string' ? props.message : props.message?.message
  if (!message) return
  handle = showFeedback({ message, tone: props.tone, duration: props.duration, theme: () => remote?.value || local?.value, remote: !!remote, actionLabel: props.actionLabel, onAction: () => emit('action'), onDismiss: () => emit('dismiss') })
}, { immediate: true })
onBeforeUnmount(() => handle?.close())
</script>
