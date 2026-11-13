<template>
  <RoomDrawer :title="panel.title" :busy="busy" :before-close="beforeClose" @close="$emit('close')">
    <RemoteFeedback :message="error" /><RemoteFeedback :message="notice" tone="success" />
    <form v-if="draft && snapshot" @submit.prevent="run(save)">
      <label v-for="field in snapshot.fields" :key="field.id" class="remote-field">{{ field.label }}<input v-model="draft.values[field.id]" :placeholder="field.placeholder" :disabled="busy" autocomplete="off" /></label>
      <section v-for="group in draft.groups" :key="group.id" class="remote-manager-card">
        <h3>{{ group.name }}</h3><p>{{ group.description }}</p>
        <RemoteSelect v-model="group.state" :label="tr('加载方式')" :options="states" :disabled="busy" />
        <label v-for="id in group.tools" :key="id" class="remote-field"><span>{{ toolName(id) }}</span><RoomSwitch :model-value="!draft.disabledTools.includes(id)" :label="toolName(id)" :disabled="busy" @update:model-value="toggle(id, $event)" /></label>
      </section>
      <section v-if="ungrouped.length" class="remote-manager-card"><h3>{{ tr('未分组工具') }}</h3><RemoteSelect v-model="draft.ungroupedState" :label="tr('加载方式')" :options="states" :disabled="busy" /><label v-for="tool in ungrouped" :key="tool.id" class="remote-field"><span>{{ tool.title || tool.id }}</span><RoomSwitch :model-value="!draft.disabledTools.includes(tool.id)" :label="tool.title || tool.id" :disabled="busy" @update:model-value="toggle(tool.id, $event)" /></label></section>
      <div class="remote-purpose-actions"><RemoteAction icon="check" tone="primary" type="submit" :disabled="busy || unresolved || !hasChanges">{{ t('保存') }}</RemoteAction><RemoteAction v-if="unresolved" icon="alert" @click="$emit('pending')">{{ tr('查看操作状态') }}</RemoteAction></div>
    </form>
    <p v-if="leaving" class="remote-inline-confirm">{{ tr('设置尚未保存，确认放弃？') }}<button type="button" @click="$emit('close')">{{ t('放弃修改') }}</button><button type="button" @click="leaving = false">{{ t('继续编辑') }}</button></p>
  </RoomDrawer>
</template>
<script setup>
import { computed, onMounted, ref } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RoomSwitch from '../companion/RoomSwitch.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import RemoteSelect from './RemoteSelect.vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { t } from '../../i18n/remote'

const props = defineProps({ panel: { type: Object, required: true } })
defineEmits(['close', 'pending'])
const { read, write, run, guard, busy, error, unresolved } = useRemoteManagement()
const tr = (source, values = []) => t(source, values, 'alignment')
const snapshot = ref(null), draft = ref(null), baseline = ref(''), notice = ref(''), leaving = ref(false)
const states = computed(() => [{ value: 'pinned', label: tr('常驻') }, { value: 'ondemand', label: tr('按需') }, { value: 'off', label: t('关闭') }])
const hasChanges = computed(() => !!draft.value && JSON.stringify(draft.value) !== baseline.value)
const ungrouped = computed(() => {
  const assigned = new Set((draft.value?.groups || []).flatMap(group => group.tools))
  return (snapshot.value?.tools || []).filter(tool => !assigned.has(tool.id))
})
function toolName(id) { return snapshot.value?.tools.find(tool => tool.id === id)?.title || id }
function toggle(id, enabled) { draft.value.disabledTools = enabled ? draft.value.disabledTools.filter(value => value !== id) : [...new Set([...draft.value.disabledTools, id])] }
function install(value) {
  if (!Number.isInteger(value?.revision) || !Array.isArray(value.groups) || !Array.isArray(value.tools) || !Array.isArray(value.disabledTools) || !value.values || !Array.isArray(value.fields)) throw new Error(tr('插件设置响应不完整，请重新读取'))
  snapshot.value = value
  draft.value = JSON.parse(JSON.stringify({ revision: value.revision, values: value.values, groups: value.groups, ungroupedState: value.ungroupedState, disabledTools: value.disabledTools }))
  baseline.value = JSON.stringify(draft.value)
}
async function load() { install(await read('packages.settings', { body: { panelId: props.panel.id, actionId: 'settings.load' } })) }
async function save() {
  const value = await write('packages.action', { body: { panelId: props.panel.id, actionId: 'settings.save', payload: JSON.parse(JSON.stringify(draft.value)) } })
  guard(); install(value)
  notice.value = tr('插件设置已保存，下次回复生效')
}
function beforeClose() { if (hasChanges.value) { leaving.value = true; return false } return true }
defineExpose({ busy, hasChanges })
onMounted(() => run(load))
</script>
