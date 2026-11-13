<template>
  <section class="remote-thread-skills">
    <RemoteFeedback :message="error" />
    <RemoteFeedback :message="notice" tone="success" />
    <p class="remote-help">{{ tr('选中的 Skills 会将完整指引加入当前会话。工具权限仍由工具设置管理。') }}</p>
    <label class="remote-search"><RemoteIcon name="search" /><input v-model="search" :placeholder="tr('查找技能')" /></label>
    <article v-for="skill in visible" :key="skill.id" class="remote-manager-card">
      <header class="remote-purpose-actions"><strong>{{ skill.name || skill.id }}</strong><RoomSwitch :model-value="selected.includes(skill.id)" :label="skill.name || skill.id" :disabled="busy || unresolved || !online" @update:model-value="toggle(skill.id, $event)" /></header>
      <p>{{ skill.description }}</p>
    </article>
    <p v-if="data && !visible.length" class="room-empty">{{ tr('当前会话没有可选技能') }}</p>
    <div class="remote-purpose-actions">
      <RemoteAction icon="refresh" :disabled="busy" @click="run(load)">{{ t('刷新') }}</RemoteAction>
      <RemoteAction icon="check" tone="primary" :disabled="busy || unresolved || !online || !hasChanges" @click="run(save)">{{ tr('保存技能选择') }}</RemoteAction>
      <RemoteAction v-if="unresolved" icon="alert" @click="$emit('pending')">{{ tr('查看操作状态') }}</RemoteAction>
    </div>
  </section>
</template>
<script setup>
import { computed, onMounted, ref } from 'vue'
import RoomSwitch from '../companion/RoomSwitch.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import RemoteIcon from './RemoteIcon.vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { remoteState } from '../../stores/remote'
import { t } from '../../i18n/remote'

const props = defineProps({ threadId: { type: String, required: true } })
const emit = defineEmits(['updated', 'pending'])
const { read, write, run, guard, busy, error, unresolved } = useRemoteManagement()
const tr = (source, values = []) => t(source, values, 'alignment')
const data = ref(null), selected = ref([]), search = ref(''), notice = ref('')
const online = computed(() => remoteState.connection === 'online')
const visible = computed(() => (data.value?.skills || []).filter(skill => `${skill.name} ${skill.id} ${skill.description}`.toLowerCase().includes(search.value.trim().toLowerCase())))
const hasChanges = computed(() => JSON.stringify([...selected.value].sort()) !== JSON.stringify([...(data.value?.enabledSkills || [])].sort()))

function toggle(id, enabled) {
  selected.value = enabled ? [...new Set([...selected.value, id])] : selected.value.filter(value => value !== id)
}
function install(value) {
  if (value?.threadId !== props.threadId || !Array.isArray(value.skills) || !Array.isArray(value.enabledSkills)) throw new Error(tr('技能配置响应不完整，请重新读取'))
  data.value = value
  selected.value = [...value.enabledSkills]
}
async function load() {
  if (hasChanges.value) throw new Error(tr('请先保存技能选择，再重新读取'))
  install(await read('thread.skills', { id: props.threadId }))
}
async function save() {
  const value = await write('thread.save_skills', { id: props.threadId, body: { enabledSkills: [...selected.value] } })
  guard(); install(value)
  notice.value = tr('技能选择已保存，下次回复生效')
  emit('updated')
}
defineExpose({ busy, hasChanges })
onMounted(() => run(load))
</script>
