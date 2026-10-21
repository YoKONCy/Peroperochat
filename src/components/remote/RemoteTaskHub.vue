<template><section class="phase2 phase-task-hub"><header class="phase-hub-heading"><span><small>infOS</small><h1>{{ t('任务与安排') }}</h1></span><RemoteIcon name="calendar" /></header><nav class="phase-tabs"><button :class="{ selected: view === 'tasks' }" :aria-pressed="view === 'tasks'" @click="switchView('tasks')"><RemoteIcon name="tasks" />{{ t('执行任务') }}</button><button :class="{ selected: view === 'reminders' }" :aria-pressed="view === 'reminders'" @click="switchView('reminders')"><RemoteIcon name="bell" />{{ t('定时安排') }}</button></nav><RemoteCollection v-if="visitedTasks" v-show="view === 'tasks'" kind="tasks" :active="active && view === 'tasks'" :agent-id="agentId" :thread-id="threadId" @manage="$emit('manage', $event)" @detail="$emit('detail', $event)" @thread="$emit('thread', $event)" @pending="$emit('pending', $event)" @create-task="$emit('create-task')" /><RemoteReminders v-if="visitedReminders" v-show="view === 'reminders'" ref="reminders" :active="active && view === 'reminders'" :agent-id="agentId" @pending="$emit('pending')" /><RemoteFeedback v-if="active" :message="error" /></section></template>
<script setup>
import { ref } from 'vue'
import RemoteCollection from './RemoteCollection.vue'
import RemoteReminders from './RemoteReminders.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import { t } from '../../i18n/remote'
defineProps({ active: Boolean, agentId: String, threadId: String })
defineEmits(['manage', 'detail', 'thread', 'pending', 'create-task'])
const view = ref('tasks'), reminders = ref(null), visitedTasks = ref(true), visitedReminders = ref(false), error = ref(''), switching = ref(false)
async function prepareLeave() { await reminders.value?.prepareLeave?.() }
async function switchView(value) { if (view.value === value || switching.value) return; switching.value = true; try { await prepareLeave(); view.value = value; if (value === 'tasks') visitedTasks.value = true; else visitedReminders.value = true } catch (failure) { if (failure.code !== 'UI_CANCELLED') error.value = failure.message } finally { switching.value = false } }
defineExpose({ prepareLeave })
</script>
