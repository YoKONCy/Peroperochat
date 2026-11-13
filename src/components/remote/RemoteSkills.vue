<template>
  <section>
    <RemoteFeedback :message="error" />
    <RemoteFeedback :message="notice" tone="success" />
    <label class="remote-search"><RemoteIcon name="search" /><input v-model="search" :placeholder="tr('查找技能')" /></label>
    <div class="remote-purpose-actions"><RemoteAction icon="refresh" :disabled="busy" @click="run(load)">{{ t('刷新') }}</RemoteAction><RemoteAction icon="refresh" :disabled="blocked" @click="run(reload)">{{ tr('重新扫描技能') }}</RemoteAction></div>
    <p class="remote-help">{{ tr('在会话设置的 Skills 中选择要使用的技能。') }}</p>
    <article v-for="skill in visible" :key="skill.id" class="remote-manager-card">
      <h2>{{ skill.name || skill.id }}</h2><p>{{ skill.description }}</p>
      <p v-if="skill.compatibility" class="remote-help">{{ skill.compatibility }}</p>
      <div class="remote-purpose-actions"><RemoteAction icon="document" :disabled="busy" @click="run(() => inspect(skill))">{{ tr('指引与兼容性') }}</RemoteAction><RemoteAction icon="trash" tone="danger" :disabled="blocked" @click="removing = skill">{{ tr('删除用户技能') }}</RemoteAction></div>
    </article>
    <p v-if="!busy && !visible.length" class="room-empty">{{ tr('没有找到技能') }}</p>
    <details class="remote-secondary-operations"><summary>{{ tr('从 infOS 服务器导入') }}</summary>
      <form @submit.prevent="run(importSkill)"><label class="remote-field">{{ tr('服务器上的技能目录') }}<input v-model="sourcePath" maxlength="2000" :disabled="busy" required /></label><p class="remote-help">{{ tr('填写运行 infOS 的机器上包含 SKILL.md 的目录。') }}</p><RemoteAction icon="plus" type="submit" :disabled="blocked || !sourcePath.trim()">{{ t('导入') }}</RemoteAction></form>
    </details>
    <RemoteAction v-if="unresolved" icon="alert" @click="$emit('pending')">{{ tr('查看操作状态') }}</RemoteAction>
    <RoomDrawer v-if="selected" :title="selected.name || selected.id" :busy="busy" @close="selected = null">
      <RemoteFeedback :message="detailError" />
      <template v-if="report"><p>{{ tr(report.compatible ? '所需工具已就绪' : '部分依赖工具不可用') }}</p><p v-if="report.missingTools?.length">{{ tr('缺少工具') }} · {{ report.missingTools.join('、') }}</p><p v-for="warning in report.warnings || []" :key="warning" class="remote-help">{{ warning }}</p><details v-if="report.resources?.length"><summary>{{ tr('技能资源') }}</summary><ul><li v-for="file in report.resources" :key="file"><code>{{ file }}</code></li></ul></details></template>
      <RemoteRichText :source="content" :node-id="`skill:${selected.id}`" @resource="$emit('resource', $event)" />
    </RoomDrawer>
    <RoomDrawer v-if="removing" :title="tr('删除用户技能？')" :busy="busy" @close="removing = null"><p>{{ tr('将从 infOS 删除技能「{0}」。', [removing.name || removing.id]) }}</p><RemoteFeedback :message="error" /><RemoteAction icon="trash" tone="danger" :disabled="blocked" @click="run(remove)">{{ t('确认删除') }}</RemoteAction></RoomDrawer>
  </section>
</template>
<script setup>
import { computed, onMounted, ref } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteRichText from './RemoteRichText.vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { remoteState } from '../../stores/remote'
import { t } from '../../i18n/remote'

defineEmits(['pending', 'resource'])
const { read, write, run, guard, busy, writing, error, unresolved } = useRemoteManagement()
const tr = (source, values = []) => t(source, values, 'alignment')
const skills = ref([]), search = ref(''), sourcePath = ref(''), selected = ref(null), removing = ref(null)
const content = ref(''), report = ref(null), detailError = ref(''), notice = ref('')
const blocked = computed(() => busy.value || unresolved.value || remoteState.connection !== 'online')
const hasChanges = computed(() => !!sourcePath.value.trim())
const visible = computed(() => skills.value.filter(skill => `${skill.id} ${skill.name} ${skill.description} ${(skill.tags || []).join(' ')}`.toLowerCase().includes(search.value.trim().toLowerCase())))

async function load() {
  const value = await read('skills.list')
  if (!Array.isArray(value)) throw new Error(tr('技能配置响应不完整，请重新读取'))
  skills.value = value
}
async function reload() {
  await write('skills.reload'); guard(); await load()
}
async function inspect(skill) {
  const results = await Promise.allSettled([read('skills.get', { id: skill.id }), read('skills.compatibility', { id: skill.id })])
  guard()
  const [body, compatibility] = results
  if (body.status === 'rejected') throw body.reason
  selected.value = skill; content.value = body.value?.content || ''
  report.value = compatibility.status === 'fulfilled' ? compatibility.value : null
  detailError.value = compatibility.status === 'rejected' ? compatibility.reason.message : ''
}
async function importSkill() {
  await write('skills.import', { body: { sourcePath: sourcePath.value.trim() } })
  guard(); sourcePath.value = ''; await load(); notice.value = tr('技能已导入')
}
async function remove() {
  await write('skills.delete', { id: removing.value.id })
  guard(); removing.value = null; await load()
}
defineExpose({ busy, writing, hasChanges })
onMounted(() => run(load))
</script>
