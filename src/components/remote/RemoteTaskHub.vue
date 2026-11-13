<template>
  <section class="phase2 phase-task-hub task-list-first">
    <header class="phase-hub-heading">
      <span><h1>{{ tr('任务与安排') }}</h1></span>
      <div v-if="view !== 'reminders'" class="task-heading-actions">
        <button type="button" class="room-icon-button" :disabled="tasks?.loading" :aria-label="tr('刷新')" @click="tasks?.reload()"><RemoteIcon name="refresh" /></button>
        <button type="button" class="room-icon-button" :disabled="!online" :aria-label="tr('新任务')" @click="$emit('create-task')"><RemoteIcon name="plus" /></button>
      </div>
    </header>
    <div class="task-list-navigation">
      <nav class="task-view-chips" :aria-label="tr('任务分类')">
        <button v-for="item in views" :key="item.id" type="button" :class="{ selected: view === item.id }" :aria-pressed="view === item.id" :disabled="switching" @click="switchView(item.id)">
          {{ tr(item.label) }}<span v-if="view === item.id && item.id !== 'reminders' && tasks?.count !== null && tasks?.count !== undefined">{{ tasks.count }}</span>
        </button>
      </nav>
      <button v-if="view !== 'reminders'" type="button" class="task-filter-trigger" :class="{ 'is-filtered': tasks?.filterCount }" :aria-label="tr('筛选任务')" @click="tasks?.openFilters()"><RemoteIcon name="filter" /><span>{{ tr('筛选') }}</span><small v-if="tasks?.filterCount">{{ tasks.filterCount }}</small></button>
    </div>
    <RemoteTaskCenter v-show="view !== 'reminders'" ref="tasks" :view="taskView" :active="active && view !== 'reminders'" @detail="$emit('detail', $event)" @pending="$emit('pending')" />
    <RemoteReminders v-if="visitedReminders" v-show="view === 'reminders'" ref="reminders" :active="active && view === 'reminders'" :agent-id="agentId" @pending="$emit('pending')" />
    <RemoteFeedback v-if="active" :message="error" />
  </section>
</template>
<script setup>
import { computed, ref } from 'vue'
import RemoteTaskCenter from './RemoteTaskCenter.vue'
import RemoteReminders from './RemoteReminders.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import { t } from '../../i18n/remote'
import { remoteState } from '../../stores/remote'
defineProps({ active: Boolean, agentId: String, threadId: String })
defineEmits(['manage', 'detail', 'thread', 'pending', 'create-task'])
const tr = key => t(key, [], 'tasks')
const view = ref('active'), taskView = ref('active'), tasks = ref(null), reminders = ref(null), visitedReminders = ref(false), error = ref(''), switching = ref(false)
const online = computed(() => remoteState.connection === 'online')
const views = [{ id: 'active', label: '进行中' }, { id: 'history', label: '历史记录' }, { id: 'resident', label: '系统常驻' }, { id: 'reminders', label: '定时安排' }]
async function prepareLeave() { await tasks.value?.prepareLeave?.(); await reminders.value?.prepareLeave?.() }
async function switchView(value) {
  if (view.value === value || switching.value) return
  switching.value = true; error.value = ''
  try {
    await prepareLeave()
    if (value === 'reminders') visitedReminders.value = true
    else taskView.value = value
    view.value = value
  } catch (failure) { if (failure.code !== 'UI_CANCELLED') error.value = failure.message }
  finally { switching.value = false }
}
defineExpose({ prepareLeave })
</script>
