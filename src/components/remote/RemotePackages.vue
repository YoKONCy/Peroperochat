<template>
  <section>
    <RemoteFeedback :message="error" /><RemoteFeedback :message="panelError" />
    <p class="remote-help">{{ tr('导入插件 ZIP 包，查看权限后启用。更新插件可再次导入。') }}</p>
    <div class="remote-purpose-actions"><RemoteAction icon="attach" :disabled="blocked" @click="input.click()">{{ tr('导入插件包') }}</RemoteAction><RemoteAction icon="refresh" :disabled="busy" @click="run(load)">{{ t('刷新') }}</RemoteAction><input ref="input" type="file" accept=".zip,application/zip" hidden @change="upload" /></div>
    <label class="remote-search"><RemoteIcon name="search" /><input v-model="search" :placeholder="tr('查找插件')" /></label>
    <article v-for="item in visible" :key="item.manifest.packageId" class="remote-manager-card">
      <h2>{{ item.manifest.name }} <small>{{ item.manifest.version }}</small></h2><p>{{ item.manifest.description }}</p>
      <p class="remote-help">{{ tr(stateLabels[item.state] || item.state) }} · {{ item.manifest.packageId }}</p>
      <RemoteFeedback :message="item.error" />
      <div class="remote-purpose-actions">
        <RemoteAction :icon="item.enabled ? 'stop' : 'check'" :disabled="blocked" @click="toggle(item)">{{ tr(item.enabled ? '停用插件' : '启用插件') }}</RemoteAction>
        <RemoteAction icon="refresh" :disabled="blocked" @click="run(() => act(item, 'reload'))">{{ t('重新加载') }}</RemoteAction>
        <RemoteAction icon="shield" :disabled="blocked" @click="run(() => act(item, 'review'))">{{ tr('重新审核') }}</RemoteAction>
        <RemoteAction v-if="item.previousVersion" icon="back" :disabled="blocked" @click="confirmation = { item, action: 'rollback' }">{{ tr('回退到上一版') }}</RemoteAction>
        <RemoteAction icon="trash" tone="danger" :disabled="blocked" @click="confirmation = { item, action: 'delete' }">{{ tr('卸载插件') }}</RemoteAction>
      </div>
      <section v-for="panel in panels.filter(panel => panel.packageId === item.manifest.packageId)" :key="panel.id" class="remote-manager-card">
        <h3>{{ panel.title }}</h3><p v-if="panel.text">{{ panel.text }}</p><dl><div v-for="(field, index) in panel.fields || []" :key="index"><dt>{{ field.label }}</dt><dd>{{ field.value }}</dd></div></dl>
        <RemoteAction v-if="panel.settings?.kind === 'tool-groups'" icon="tools" :disabled="blocked" @click="settingsPanel = panel">{{ tr('插件工具与设置') }}</RemoteAction>
        <div class="remote-purpose-actions"><RemoteAction v-for="action in panel.actions || []" :key="action.id" :disabled="blocked" @click="run(() => panelAction(panel, action))">{{ action.label }}</RemoteAction></div>
      </section>
    </article>
    <p v-if="!busy && !visible.length" class="room-empty">{{ tr('没有找到插件') }}</p>
    <RemoteAction v-if="unresolved" icon="alert" @click="$emit('pending')">{{ tr('查看操作状态') }}</RemoteAction>
    <RoomDrawer v-if="authorization" :title="tr('授权并启用插件')" :subtitle="`${authorization.manifest.name} ${authorization.manifest.version}`" :busy="busy" @close="authorization = null">
      <RemoteFeedback :message="error" /><p>{{ tr('此版本申请以下权限') }}</p><ul><li v-for="permission in authorization.requestedPermissions" :key="permission">{{ tr(permissionLabels[permission] || permission) }}</li></ul><p v-if="!authorization.requestedPermissions.length">{{ tr('此版本没有额外权限请求') }}</p>
      <RemoteAction icon="check" tone="primary" :disabled="blocked" @click="run(authorize)">{{ tr('授权并启用') }}</RemoteAction>
    </RoomDrawer>
    <RoomDrawer v-if="confirmation" :title="tr(confirmation.action === 'delete' ? '卸载插件？' : '回退插件版本？')" :busy="busy" @close="confirmation = null"><RemoteFeedback :message="error" /><p>{{ confirmation.item.manifest.name }}</p><RemoteAction icon="check" tone="danger" :disabled="blocked" @click="run(confirm)">{{ t('确认') }}</RemoteAction></RoomDrawer>
    <RemotePackageTools v-if="settingsPanel" ref="settingsEditor" :panel="settingsPanel" @close="settingsPanel = null" @pending="$emit('pending')" />
    <RoomDrawer v-if="result !== undefined" :title="tr('插件返回结果')" @close="result = undefined"><RemoteValue :value="result" /></RoomDrawer>
  </section>
</template>
<script setup>
import { computed, onMounted, ref } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemotePackageTools from './RemotePackageTools.vue'
import RemoteValue from './RemoteValue.vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { remoteClient } from '../../services/remote'
import { refreshRemotePendingRequests, remoteState } from '../../stores/remote'
import { t } from '../../i18n/remote'

defineEmits(['pending'])
const { token, read, write, run, guard, busy, writing, error, unresolved, fail } = useRemoteManagement()
const tr = (source, values = []) => t(source, values, 'alignment')
const items = ref([]), panels = ref([]), input = ref(null), search = ref(''), panelError = ref('')
const authorization = ref(null), confirmation = ref(null), settingsPanel = ref(null), settingsEditor = ref(null), result = ref(undefined)
const blocked = computed(() => busy.value || unresolved.value || remoteState.connection !== 'online')
const hasChanges = computed(() => !!settingsEditor.value?.hasChanges)
const active = computed(() => busy.value || !!settingsEditor.value?.busy)
const visible = computed(() => items.value.filter(item => `${item.manifest.name} ${item.manifest.packageId} ${item.manifest.description}`.toLowerCase().includes(search.value.trim().toLowerCase())))
const stateLabels = { installed: '未启用', inactive: '已停用', active: '运行中', activating: '启动中', failed: '启动失败' }
const permissionLabels = {
  'runtime:execute': '运行插件代码', 'filesystem:read': '读取文件', 'filesystem:write': '写入文件',
  'network:local': '访问本地网络', 'network:internet': '访问互联网', 'process:spawn': '创建进程',
  'database:read': '读取数据', 'database:write': '写入数据', 'system:info': '读取系统信息',
}
async function load() {
  const value = await read('packages.list')
  if (!Array.isArray(value) || value.some(item => !item.manifest?.packageId)) throw new Error(tr('插件列表响应不完整，请重新读取'))
  items.value = value
  if (authorization.value) authorization.value = value.find(item => item.manifest.packageId === authorization.value.manifest.packageId) || null
  try { panels.value = await read('packages.panels', { id: 'integrations' }); panelError.value = '' }
  catch (failure) { panels.value = []; panelError.value = failure.message }
}
async function act(item, action, body = undefined) {
  try { await write(`packages.${action}`, { id: item.manifest.packageId, body }) }
  catch (failure) { await load().catch(() => {}); throw failure }
  guard(); await load()
}
function toggle(item) {
  if (!item.enabled && !item.authorized) authorization.value = item
  else run(() => act(item, 'enabled', { enabled: !item.enabled }))
}
async function authorize() {
  const item = authorization.value
  try { await write('packages.authorize', { id: item.manifest.packageId, body: { digest: item.digest, permissions: [...item.requestedPermissions] } }) }
  catch (failure) { await load().catch(() => {}); throw failure }
  guard(); await act(item, 'enabled', { enabled: true }); authorization.value = null
}
async function confirm() { await act(confirmation.value.item, confirmation.value.action); confirmation.value = null }
async function panelAction(panel, action) {
  result.value = await write('packages.action', { body: { panelId: panel.id, actionId: action.id } })
  guard(); await load()
}
async function upload(event) {
  const file = event.target.files?.[0]; event.target.value = ''
  if (!file || blocked.value) return
  await run(async () => {
    if (!file.size || file.size > 20 * 1024 * 1024) throw new Error(tr('请选择不超过 20 MiB 的插件 ZIP 包'))
    const bytes = Array.from(new Uint8Array(await file.arrayBuffer())); guard()
    try {
      await remoteClient.manageUpload('package', null, { name: file.name, mimeType: 'application/zip', bytes }, { token })
      guard(); await load()
    } catch (failure) { fail(failure); throw failure }
    finally { refreshRemotePendingRequests().catch(() => {}) }
  })
}
defineExpose({ busy: active, writing: computed(() => writing.value || !!settingsEditor.value?.busy), hasChanges })
onMounted(() => run(load))
</script>
