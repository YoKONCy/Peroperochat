<template><button type="button" class="remote-speech-button" :class="{ 'is-playing': active && speech.status === 'playing' }" :disabled="disabled || !text?.trim()" :aria-label="label" :title="label" @click="toggle"><RemoteIcon :name="active && ['loading', 'playing'].includes(speech.status) ? 'stop' : 'volume'" /><span v-if="showLabel">{{ label }}</span><i v-if="active && speech.status === 'playing'" aria-hidden="true"><b /><b /><b /></i></button></template>
<script setup>
import { computed, watch, onBeforeUnmount } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
import { remoteSpeech as speech, startRemoteSpeech, stopRemoteSpeech, replayRemoteSpeech } from '../../services/remoteSpeech'
import { appMessage } from '../../services/appFeedback'
import { refreshRemotePendingRequests } from '../../stores/remote'
import { t } from '../../i18n/remote'
const props = defineProps({ text: String, agentId: String, presetId: String, id: { type: String, required: true }, disabled: Boolean, showLabel: Boolean })
const emit = defineEmits(['pending'])
const active = computed(() => speech.id === props.id)
const label = computed(() => t(active.value && speech.status === 'loading' ? '停止生成' : active.value && speech.status === 'playing' ? '停止朗读' : active.value && speech.status === 'ready' ? '再次播放' : '朗读', [], 'tts'))
async function toggle() {
  if (props.disabled) return
  if (active.value && ['loading', 'playing'].includes(speech.status)) { stopRemoteSpeech(props.id); return }
  try {
    if (active.value && speech.status === 'ready') await replayRemoteSpeech()
    else await startRemoteSpeech({ text: props.text, ...(props.agentId ? { agentId: props.agentId } : {}), ...(props.presetId ? { presetId: props.presetId } : {}) }, props.id)
  } catch (failure) {
    if (['CANCELLED', 'MODE_INACTIVE'].includes(failure.code)) return
    appMessage.error(t(failure.name === 'NotAllowedError' ? '请再点一次播放，或检查声音权限。' : failure.name === 'NotSupportedError' || failure.name === 'EncodingError' ? '这个音频暂时无法播放，请更换输出格式。' : failure.message, [], 'tts'))
    if (failure.outcome === 'unknown') { await refreshRemotePendingRequests().catch(() => {}); emit('pending') }
  }
}
onBeforeUnmount(() => stopRemoteSpeech(props.id))
watch(() => [props.text, props.agentId, props.presetId], () => stopRemoteSpeech(props.id))
</script>
