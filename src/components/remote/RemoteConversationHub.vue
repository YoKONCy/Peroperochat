<template>
  <section class="phase2 phase-conversation-hub" :class="{ 'has-open-room': roomId }">
    <RemoteFeedback v-if="active" :message="error" />
    <template v-if="!roomId"><header class="phase-hub-heading"><span><small>infOS</small><h1>{{ t('会话') }}</h1></span><RemoteIcon name="bow" /><button class="room-icon-button" :aria-label="t(mode === 'solo' ? '新建会话' : '新房间')" @click="mode === 'solo' ? $emit('create') : stronghold?.configure('create')"><RemoteIcon name="plus" /></button></header><nav class="phase-tabs"><button :class="{ selected: mode === 'solo' }" :aria-pressed="mode === 'solo'" @click="switchMode('solo')"><RemoteIcon name="chat" />{{ t('单聊') }}</button><button :class="{ selected: mode === 'group' }" :aria-pressed="mode === 'group'" @click="switchMode('group')"><RemoteIcon name="people" />{{ t('群聊') }}</button></nav></template>
    <Transition name="phase-pane"><div v-show="mode === 'solo'" class="phase-solo-list"><slot name="single" /></div></Transition>
    <Transition name="phase-pane"><RemoteStronghold v-if="visitedGroup" v-show="mode === 'group'" ref="stronghold" :active="active && mode === 'group'" :agent-id="agentId" @room-change="roomId = $event" @pending="$emit('pending')" @resource="$emit('resource', $event)" /></Transition>
  </section>
</template>
<script setup>
import { ref } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteStronghold from './RemoteStronghold.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import { t } from '../../i18n/remote'
defineProps({ active: Boolean, agentId: String })
const emit = defineEmits(['pending', 'resource', 'create', 'single'])
const mode = ref('solo'), roomId = ref(null), stronghold = ref(null), visitedGroup = ref(false), error = ref(''), switching = ref(false)
async function switchMode(value) { if (mode.value === value || switching.value) return; switching.value = true; try { await stronghold.value?.prepareLeave?.(); mode.value = value; if (value === 'group') visitedGroup.value = true; else emit('single') } catch (failure) { if (failure.code !== 'UI_CANCELLED') error.value = failure.message } finally { switching.value = false } }
async function prepareLeave() { await stronghold.value?.prepareLeave?.() }
async function back() { if (roomId.value) await stronghold.value?.back?.(); else await switchMode('solo') }
async function showList() { if (roomId.value) await stronghold.value?.back?.() }
defineExpose({ prepareLeave, back, roomId, showList })
</script>
