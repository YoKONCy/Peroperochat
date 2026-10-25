<template><section class="phase2 phase-task-hub"><header class="phase-hub-heading"><span><small>infOS</small><h1>{{ tr('任务与安排') }}</h1></span><RemoteIcon name="calendar" /></header><nav class="phase-tabs"><button :class="{ selected: view === 'tasks' }" :aria-pressed="view === 'tasks'" @click="switchView('tasks')"><RemoteIcon name="tasks" />{{ tr('执行任务') }}</button><button :class="{ selected: view === 'reminders' }" :aria-pressed="view === 'reminders'" @click="switchView('reminders')"><RemoteIcon name="bell" />{{ tr('定时安排') }}</button></nav><RemoteTaskCenter v-if="visitedTasks" v-show="view === 'tasks'" ref="tasks" :active="active && view === 'tasks'" @detail="$emit('detail', $event)" @pending="$emit('pending')" @create-task="$emit('create-task')" /><RemoteReminders v-if="visitedReminders" v-show="view === 'reminders'" ref="reminders" :active="active && view === 'reminders'" :agent-id="agentId" @pending="$emit('pending')" /><RemoteFeedback v-if="active" :message="error" /></section></template>
<script setup>
import { ref } from 'vue'
import RemoteTaskCenter from './RemoteTaskCenter.vue'
import RemoteReminders from './RemoteReminders.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import { t } from '../../i18n/remote'
defineProps({ active: Boolean, agentId: String, threadId: String })
defineEmits(['manage', 'detail', 'thread', 'pending', 'create-task'])
const tr = key => t(key, [], 'tasks')
const view = ref('tasks'), tasks = ref(null), reminders = ref(null), visitedTasks = ref(true), visitedReminders = ref(false), error = ref(''), switching = ref(false)
async function prepareLeave() { await tasks.value?.prepareLeave?.(); await reminders.value?.prepareLeave?.() }
async function switchView(value) { if (view.value === value || switching.value) return; switching.value = true; try { await prepareLeave(); view.value = value; if (value === 'tasks') visitedTasks.value = true; else visitedReminders.value = true } catch (failure) { if (failure.code !== 'UI_CANCELLED') error.value = failure.message } finally { switching.value = false } }
defineExpose({ prepareLeave })
</script>
