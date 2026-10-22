<template>
  <section class="remote-manager remote-management-shell phase-management-shell phase2">
    <header class="remote-manager-heading"><button v-if="section" class="room-icon-button" aria-label="返回管理首页" :disabled="busy" @click="askLeave"><RemoteIcon name="back" /></button><span><small>infOS · {{ t(section ? areaLabel : '管理') }}</small><h1>{{ t(section?.label || '管理') }}</h1></span><button v-if="section && !section.special" class="room-icon-button" aria-label="刷新" :disabled="busy" @click="requestReload"><RemoteIcon name="refresh" /></button></header>
    <RemoteFeedback v-if="!selected && !dialog" :message="error" /><RemoteFeedback :message="notice" tone="success" />
    <RemoteAction v-if="unresolved" icon="alert" tone="danger" @click="$emit('pending')">上次操作结果待核对</RemoteAction>
    <template v-if="!section">
      <RemoteManagementHome :agent="remoteState.agents.find(item => item.id === agentId)" :online="remoteState.connection === 'online'" @open="openDestination" />
    </template>
    <template v-else-if="section.special">
      <RemoteWorkbench v-if="section.id === 'workbench'" ref="special" :agent-id="agentId" :thread-id="threadId" @pending="$emit('pending')" @resource="$emit('resource', $event)" @context="$emit('context', $event)" />
      <RemotePromptStudio v-else-if="section.id === 'prompts'" ref="special" :agent-id="agentId" @pending="$emit('pending')" />

      <RemoteVoice v-else-if="section.id === 'voice'" ref="special" @pending="$emit('pending')" />
      <RemoteInsights v-else-if="['usage', 'runtime'].includes(section.id)" ref="special" :key="section.id" :kind="section.id" @thread="$emit('thread', $event)" />
    </template>
    <template v-else>
      <div v-if="['models', 'providers'].includes(section.id)" class="phase-model-nav"><nav class="phase-tabs"><button type="button" :class="{ selected: section.id === 'models' }" :disabled="busy" @click="openDestination('models')"><RemoteIcon name="model" />{{ t('模型') }}</button><button type="button" :class="{ selected: section.id === 'providers' }" :disabled="busy" @click="openDestination('providers')"><RemoteIcon name="connection" />{{ t('服务商连接') }}</button></nav><button v-if="section.id === 'models'" type="button" class="room-icon-button" :aria-label="t('模型分工')" :disabled="busy" @click="openDestination('assignments')"><RemoteIcon name="route" /></button></div>
      <label v-if="['agents', 'models', 'providers'].includes(section.id)" class="phase-search"><RemoteIcon name="search" /><input v-model="search" :placeholder="t('输入关键词查找')" :aria-label="t('输入关键词查找')" /></label><form v-if="section.filters" class="remote-manager-filter" @submit.prevent="page = 1; run(load)"><details><summary><RemoteIcon name="search" />筛选与查找</summary><RemoteFields v-model="filters" :fields="section.filters" :disabled="busy" /><RemoteAction icon="search" type="submit" :disabled="busy">应用筛选</RemoteAction></details></form>
      <div class="remote-purpose-actions"><RemoteAction v-if="section.id === 'memories' && selectionIds.length" icon="check" :disabled="busy || unresolved" @click="memoryBatch">处理已选 {{ selectionIds.length }} 条</RemoteAction><RemoteAction v-if="section.create" icon="plus" tone="primary" :disabled="busy || unresolved" @click="command(section.create, null)">{{ t(section.create.label) }}</RemoteAction><RemoteAction v-if="section.upload" icon="attach" :disabled="busy || unresolved" @click="replaceUpload = false; uploadInput.click()">上传文本文件</RemoteAction><RemoteAction v-for="action in primaryCommands" :key="action.op" v-bind="actionPresentation(action)" :disabled="busy || !action.read && unresolved" @click="command(action, null)">{{ t(action.label) }}</RemoteAction><input ref="uploadInput" type="file" :accept="section.id === 'agents' ? 'image/png' : '.txt,.md,.json,.csv,.html,.xml,.log'" hidden @change="upload" /></div>
      <details v-if="additionalCommands.length" class="remote-secondary-operations"><summary><RemoteIcon name="more" />更多操作<span>{{ additionalCommands.length }}</span></summary><div class="remote-purpose-actions"><RemoteAction v-for="action in additionalCommands" :key="action.op" v-bind="actionPresentation(action)" :disabled="busy || !action.read && unresolved" @click="command(action, null)">{{ t(action.label) }}</RemoteAction></div></details>
      <p v-if="busy" class="remote-help" role="status">正在处理…</p>
      <template v-if="section.config || section.singleton"><form v-if="editor" class="remote-management-editor" @submit.prevent="save"><RemoteSettingsForm v-model="editor" :kind="section.id" :record-id="selected ? recordId(selected) : undefined" :connection-dirty="hasChanges" @appearance="$emit('appearance')" @avatar="uploadInput.click()" :fields="section.config ? configEditorFields : section.fields || inferFields(editor)" :disabled="busy" /><footer class="remote-edit-footer phase-settings-save"><span>{{ t(hasChanges ? '有未保存的修改' : '已是最新设置') }}</span><RemoteAction icon="check" tone="primary" type="submit" :disabled="busy || unresolved || !hasChanges">{{ t(section.id === 'models' ? '保存模型' : '保存') }}</RemoteAction></footer></form><RemoteRecordView v-if="section.singleton && !editor" :kind="section.id" :record="data || {}" /></template>
      <template v-else-if="list"><p v-if="!list.length && !busy" class="room-empty">还没有记录</p><div class="remote-manager-records"><div v-for="(item, index) in list" :key="recordId(item) ?? index" class="remote-manager-record"><RoomSwitch v-if="section.id === 'memories'" :model-value="selectionIds.includes(String(recordId(item)))" :label="'选择第 ' + (index + 1) + ' 条记忆'" @update:model-value="toggleSelection(recordId(item), $event)" /><button class="remote-manager-record-link" :disabled="busy" @click="choose(item)"><RemoteAvatar v-if="section.id === 'agents'" :id="item.id" :label="item.name" fit="contain" /><BrandIcon v-else-if="section.id === 'models' || section.id === 'providers'" :brand="section.id === 'models' ? resolveModelBrand(item.modelId) || resolveProviderBrand(item.provider, item.apiBase) : resolveProviderBrand(item.brand, item.apiBase)" :size="25" :fallback="section.id === 'providers' ? 'provider' : 'model'" /><RemoteIcon v-else :name="section.icon" /><span><strong>{{ recordTitle(item) }}</strong><small>{{ item.description || item.narrative || item.modelId || item.instruction || item.status || '查看详情' }}</small><span v-if="section.id === 'models'" class="phase-model-meta"><span v-if="item.enableVision" class="phase-chip">{{ t('图片') }}</span><span v-if="item.enableAudioInput" class="phase-chip">{{ t('音频') }}</span><span v-if="item.reasoningEffort" class="phase-chip">{{ remoteChoiceLabel(item.reasoningEffort) }}</span><span v-if="item.isDefault" class="phase-chip green">{{ t('默认') }}</span></span><em v-if="item.status">{{ remoteStatus(item.status) }}</em></span><RemoteIcon name="next" /></button></div></div><div v-if="section.paged" class="remote-pagination"><button :disabled="busy || page <= 1" @click="page--; run(load)">上一页</button><span>第 {{ page }} 页<span v-if="data?.total != null"> · {{ data.total }} 条</span></span><button :disabled="busy || list.length < 30 || data?.total != null && page * 30 >= data.total" @click="page++; run(load)">下一页</button></div></template>
      <RemoteOperationResult v-else-if="data" :operation="section.id" :value="data" />
    </template>
    <RoomDrawer v-if="selected" :title="section.id === 'models' ? t('编辑模型') : recordTitle(selected)" :subtitle="section.id === 'models' ? '' : '记录详情'" :busy="busy" :before-close="beforeCloseRecord" @close="closeRecord"><RemoteFeedback :message="error" /><p v-if="unresolved" class="room-error">操作结果待核对，请先到通知查看。</p><RemoteRecordView v-if="!['agents', 'models', 'providers'].includes(section.id)" :kind="section.id" :record="record || selected" /><form v-if="section.fields" :class="{ 'model-config-form': section.id === 'models' }" @submit.prevent="save"><RemoteSettingsForm v-model="editor" :kind="section.id" :record-id="selected ? recordId(selected) : undefined" :connection-dirty="hasChanges" @appearance="$emit('appearance')" @avatar="uploadInput.click()" :fields="detailFields" :disabled="busy" /><footer class="remote-edit-footer phase-settings-save" :class="{ 'model-config-save': section.id === 'models', 'has-changes': hasChanges }"><span>{{ t(section.id === 'models' ? hasChanges ? '有修改' : '未修改' : hasChanges ? '有未保存的修改' : '修改后可保存') }}</span><RemoteAction icon="check" tone="primary" type="submit" :disabled="busy || unresolved || !hasChanges">{{ t(section.id === 'models' ? '保存模型' : '保存') }}</RemoteAction></footer></form><template v-else-if="section.id === 'logs'"><RemoteAction icon="chat" :disabled="record?.thread?.channel !== 'desktop'" @click="openLogThread">回到这个会话</RemoteAction><RemoteAction icon="download" :disabled="busy" @click="run(() => downloadData(record, 'infos-messages-page.json'))">导出当前页</RemoteAction><div v-for="message in record?.messages || []" :key="message.id" class="remote-log-message"><small>{{ message.role === 'user' ? '你' : '角色' }}</small><p>{{ message.content }}</p><RemoteAction icon="edit" :disabled="record?.thread?.channel !== 'desktop'" @click="logMessage = { ...message, messageId: String(message.id) }">消息操作</RemoteAction></div><div class="remote-purpose-actions"><RemoteAction icon="back" :disabled="busy || logPage <= 1" @click="loadLogs(logPage - 1)">上一页</RemoteAction><span>第 {{ logPage }} 页</span><RemoteAction icon="next" :disabled="busy || logPage * 30 >= (record?.total || 0)" @click="loadLogs(logPage + 1)">下一页</RemoteAction></div></template><div class="remote-purpose-actions" :class="{ 'model-record-actions': section.id === 'models' }"><RemoteAction v-for="action in recordActions" :key="action.op" v-bind="actionPresentation(action)" :disabled="busy || !action.read && unresolved" @click="command(action, selected)">{{ t(action.label) }}</RemoteAction><RemoteAction v-if="section.id === 'agents' || section.id === 'documents'" icon="attach" :disabled="busy || unresolved" @click="replaceUpload = section.id === 'documents'; uploadInput.click()">{{ section.id === 'agents' ? '更换头像' : '替换文档' }}</RemoteAction></div></RoomDrawer>
    <RoomDrawer v-if="dialog" :title="dialog.action.label" :busy="busy" :before-close="beforeCloseCommand" @close="closeCommand"><RemoteFeedback :message="error" /><p v-if="dialog.action.help" class="remote-help">{{ dialog.action.help }}</p><p v-if="dialog.action.confirm" class="remote-operation-scope"><RemoteIcon :name="actionPresentation(dialog.action).icon" />{{ dialog.action.label }}{{ dialog.item ? '：' + recordTitle(dialog.item) : '' }}。{{ actionPresentation(dialog.action).tone === 'danger' ? '会影响服务端数据，完成后不保证可以恢复。' : '会修改当前 infOS，其他客户端也将同步。' }}</p><form @submit.prevent="execute"><input v-if="dialog.action.importFile" type="file" accept="application/json,.json" @change="importConfig" /><RemoteSettingsForm v-model="commandBody" :kind="section.id" :fields="commandFields" :disabled="busy" /><RemoteAction :icon="actionPresentation(dialog.action).icon" :tone="dialog.action.read ? 'secondary' : actionPresentation(dialog.action).tone === 'danger' ? 'danger' : 'primary'" type="submit" :disabled="busy || !dialog.action.read && unresolved || dialog.action.phrase && commandBody.confirm !== dialog.action.phrase">{{ dialog.action.label }}</RemoteAction></form></RoomDrawer>
    <RoomDrawer v-if="result !== undefined" :title="resultTitle || '操作结果'" @close="result = undefined"><template v-if="sensitiveResult"><p class="remote-help">此授权信息只在当前页显示，请妥善保管。</p><pre class="remote-sensitive-result">{{ JSON.stringify(result, null, 2) }}</pre></template><RemoteOperationResult v-else :operation="resultOperation" :value="result" /></RoomDrawer>
    <RoomDrawer v-if="discarding" :title="t('修改还没有保存')" @close="cancelDiscard"><p>{{ t('离开后，这次未保存的修改会丢失。') }}</p><div class="remote-purpose-actions"><RemoteAction icon="back" @click="cancelDiscard">{{ t('继续编辑') }}</RemoteAction><RemoteAction v-if="leaveTarget === 'navigation' && editor && !dialog && !section?.special" icon="check" tone="primary" :disabled="busy" @click="saveAndLeave">{{ t('保存并继续') }}</RemoteAction><RemoteAction icon="trash" tone="danger" :disabled="busy" @click="discardChanges">{{ t('放弃修改') }}</RemoteAction></div></RoomDrawer>
    <RemoteMessageActions v-if="logMessage && selected" :thread-id="String(recordId(selected))" :agent-id="record?.thread?.agentId" :message="logMessage" @close="logMessage = null" @updated="loadLogs(logPage)" @pending="$emit('pending')" />
  </section>
</template>
<script setup>
import RemoteFeedback from './RemoteFeedback.vue'
import { computed, ref, watch, nextTick, onBeforeUnmount } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteAvatar from './RemoteAvatar.vue'
import RemoteSettingsForm from './RemoteSettingsForm.vue'
import RemoteManagementHome from './RemoteManagementHome.vue'
import RemoteInsights from './RemoteInsights.vue'
import { t, setRemoteLocale, remoteChoiceLabel } from '../../i18n/remote'
import RemoteRecordView from './RemoteRecordView.vue'
import RemoteOperationResult from './RemoteOperationResult.vue'
import { remoteStatus } from '../../utils/remoteMarkdown'
import { managementAreas, actionPresentation } from '../../services/remoteManagementPresentation'

import RemoteFields from './RemoteFields.vue'

import RemoteWorkbench from './RemoteWorkbench.vue'
import RemotePromptStudio from './RemotePromptStudio.vue'
import RemoteVoice from './RemoteVoice.vue'
import RemoteMessageActions from './RemoteMessageActions.vue'
import RoomSwitch from '../companion/RoomSwitch.vue'
import BrandIcon from '../companion/BrandIcon.vue'
import { resolveModelBrand, resolveProviderBrand } from '../../utils/modelBrands'
import { invalidateRemoteAvatar } from '../../services/remoteMedia'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { managementSections, configFields, inferFields, records, recordId, recordTitle, downloadData, cloneData } from '../../services/remoteManagement'
import { remoteClient } from '../../services/remote'
import { initializeRemote, refreshRemotePendingRequests, remoteState } from '../../stores/remote'
import '../../styles/remote-management.css'
import '../../styles/remote-interactions.css'
import '../../styles/remote-phase-two.css'
const props = defineProps({ agentId: String, threadId: String, initialSection: String })
const emit = defineEmits(['pending', 'resource', 'thread', 'context', 'section-change', 'ready', 'appearance'])
const { token, read, write, run, guard, error, busy, writing, unresolved, fail } = useRemoteManagement()
const section = ref(null), data = ref(null), selected = ref(null), record = ref(null), editor = ref(null), filters = ref({}), page = ref(1), search = ref(''), dialog = ref(null), commandBody = ref({}), result = ref(undefined), sensitiveResult = ref(false), notice = ref(''), uploadInput = ref(null)
watch(() => section.value?.id || null, value => emit('section-change', value))
const leaveTarget = ref('home'), commandBaseline = ref(''), resultTitle = ref(''), resultOperation = ref('')
const areaLabel = computed(() => managementAreas.find(group => group.ids.includes(section.value?.id))?.label || '管理')
const mobileTopCommands = computed(() => (section.value?.top || []).filter(action => ['providers.models'].includes(action.op)))
const primaryCommands = computed(() => mobileTopCommands.value.slice(0, 2))
const additionalCommands = computed(() => mobileTopCommands.value.slice(2))
const recordActions = computed(() => (section.value?.actions || []).filter(action => !['agents.save_companion', 'providers.models'].includes(action.op)))

const listSections = new Set(['agents', 'providers', 'models', 'mcp', 'skills', 'tools', 'memories', 'facts', 'documents', 'distributed', 'inbound', 'assets', 'logs', 'task-management', 'scheduler', 'config'])
const list = computed(() => { if (!listSections.has(section.value?.id)) return null; const values = records(data.value); if (!['agents', 'models', 'providers'].includes(section.value.id) || !search.value.trim()) return values; const keyword = search.value.trim().toLowerCase(); return values.filter(item => `${item.name || ''} ${item.description || ''} ${item.modelId || ''}`.toLowerCase().includes(keyword)) })
const special = ref(null), configModels = ref([]), baseline = ref('')
const logPage = ref(1), logMessage = ref(null), selectionIds = ref([])
const replaceUpload = ref(false)
const providerConnections = ref([])
let navigationDecision = null
const mobileSections = new Set(['agents', 'providers', 'models', 'assignments', 'user', 'voice', 'prompts', 'memories', 'facts', 'documents', 'workbench', 'scheduler', 'proactive', 'usage', 'runtime', 'assets', 'logs', 'task-management'])
function openDestination(id) { if (id === 'appearance') { emit('appearance'); return }; const item = managementSections.find(item => item.id === id); if (item && mobileSections.has(id)) open(item) }
const discarding = ref(false)
const hasChanges = computed(() => !!editor.value && JSON.stringify(editor.value) !== baseline.value || !!special.value?.hasChanges || !!dialog.value && JSON.stringify(commandBody.value) !== commandBaseline.value)
const configEditorFields = computed(() => configFields(editor.value).map(field => {
  if (section.value?.agentAssignments && (/^model\./.test(field.key) || field.key === 'multimodalRelay.modelConfigId')) return { ...field, translateChoices: false, label: field.key.startsWith('model.agent.') ? remoteState.agents.find(item => `model.agent.${item.id}` === field.key)?.name || field.key : field.key === 'model.main' ? t('默认聊天模型') : field.label, choices: [{ value: '', label: t('沿用默认模型') }, ...configModels.value.filter(item => field.key !== 'multimodalRelay.modelConfigId' || item.enableVision || String(item.id) === String(editor.value[field.key])).map(item => ({ value: String(item.id), label: `${item.name} · ${item.modelId}` }))] }
  if (field.key === 'user.language') return { ...field, choices: [{ value: 'zh-CN', label: '简体中文' }, { value: 'zh-TW', label: '繁體中文' }, { value: 'en-US', label: 'English' }] }
  if (/enabled|autoSave|soundEffects/.test(field.key)) return { ...field, help: undefined, choices: [{ value: 'true', label: '开启' }, { value: 'false', label: '关闭' }] }
  return field
}))
function enhanceField(field) {
  if (field.key === 'agentId') return { ...field, translateChoices: false, choices: [{ value: '', label: t('请选择角色') }, ...remoteState.agents.filter(item => !item.isUser).map(item => ({ value: item.id, label: item.name }))] }
  if (field.key === 'providerConnectionId') return { ...field, translateChoices: false, choices: providerConnections.value.map(item => ({ value: item.id, label: item.name, description: item.protocol })) }
  if (field.key === 'modelIds' && section.value?.id === 'models') { const choices = (list.value || []).filter(item => String(item.id) !== String(recordId(selected.value))).map(item => ({ value: Number(item.id), label: `${item.name} · ${item.modelId}` })); return { ...field, itemChoices: choices, itemDefault: choices[0]?.value ?? 0 } }
  return field
}
const commandFields = computed(() => (dialog.value?.action.fields || inferFields(commandBody.value)).map(enhanceField))
const detailFields = computed(() => (section.value?.fields || []).map(enhanceField))
async function prepareLeave() {
  if (writing.value || (special.value?.writing ?? special.value?.busy)) throw new Error(t('正在处理，请稍后再试。'))
  if (!hasChanges.value) return
  leaveTarget.value = 'navigation'; discarding.value = true
  const approved = await new Promise(resolve => { navigationDecision = resolve })
  if (!approved) throw Object.assign(new Error(''), { code: 'UI_CANCELLED' })
}
function cancelDiscard() { discarding.value = false; navigationDecision?.(false); navigationDecision = null }
async function saveAndLeave() { await save(); if (error.value || hasChanges.value) return; discarding.value = false; navigationDecision?.(true); navigationDecision = null }
onBeforeUnmount(() => { navigationDecision?.(false); navigationDecision = null })
function askLeave() { if (busy.value || special.value?.busy) { error.value = '当前管理操作尚未完成，请稍后再离开'; return }; leaveTarget.value = 'home'; if (hasChanges.value) discarding.value = true; else leave() }
function beforeCloseRecord() { if (hasChanges.value) { leaveTarget.value = 'record'; discarding.value = true; return false } return true }
function beforeCloseCommand() { if (JSON.stringify(commandBody.value) !== commandBaseline.value) { leaveTarget.value = 'command'; discarding.value = true; return false } return true }
function closeRecord() { if (busy.value) return; leaveTarget.value = 'record'; if (hasChanges.value) discarding.value = true; else { selected.value = null; record.value = null; editor.value = null } }
function closeCommand() { if (busy.value) return; leaveTarget.value = 'command'; if (JSON.stringify(commandBody.value) !== commandBaseline.value) discarding.value = true; else { dialog.value = null; commandBody.value = {} } }
function discardChanges() { discarding.value = false; if (leaveTarget.value === 'navigation') { navigationDecision?.(true); navigationDecision = null } else if (leaveTarget.value === 'home') leave(); else if (leaveTarget.value === 'record') { selected.value = null; record.value = null; editor.value = null } else { dialog.value = null; commandBody.value = {} } }
function requestReload() { if (hasChanges.value) { error.value = '还有未保存的编辑，请先保存，或返回管理首页并确认放弃'; return }; run(load) }
defineExpose({ prepareLeave, back: askLeave, openSection: openDestination })
function leave() { search.value = ''; section.value = null; data.value = null; editor.value = null; selected.value = null; record.value = null; dialog.value = null; commandBody.value = {}; result.value = undefined; configModels.value = []; providerConnections.value = []; error.value = ''; notice.value = ''; commandBaseline.value = ''; resultTitle.value = ''; resultOperation.value = '' }
async function open(item) { if (busy.value || !mobileSections.has(item.id)) return; leave(); section.value = ['usage', 'runtime'].includes(item.id) ? { ...item, special: true } : item; page.value = 1; filters.value = { ...item.params }; await run(load) }
async function load() {
  const item = section.value
  if (!item || item.special) return
  const params = { ...filters.value }
  for (const key of Object.keys(params)) if (params[key] == null || params[key] === '') delete params[key]
  if (item.agent) { if (!props.agentId) throw new Error('请先选择角色'); params.agentId = props.agentId }
  if (item.paged) Object.assign(params, { page: page.value, pageSize: 30 })
  if (item.config) { const keys = item.id === 'user' ? { 'owner.name': '', 'owner.persona': '', 'user.language': 'zh-CN' } : { ...item.keys }; if (item.agentAssignments) { for (const agent of remoteState.agents.filter(agent => !agent.isUser)) keys[`model.agent.${agent.id}`] = ''; configModels.value = await read('models.list') }; const values = await read('configs.batch', { body: { keys: Object.keys(keys) } }); editor.value = Object.fromEntries(Object.entries(keys).map(([key, value]) => [key, values?.[key] ?? value])); if (item.id === 'user') setRemoteLocale(editor.value['user.language']); baseline.value = JSON.stringify(editor.value); await nextTick(); emit('ready'); return }
  data.value = await read(item.read, { params }); await nextTick(); emit('ready')
  if (item.id === 'models') providerConnections.value = await read('providers.list')
  selectionIds.value = []
  if (item.singleton) editor.value = cloneData(item.pick ? data.value?.[item.pick] ?? data.value : data.value)
  baseline.value = JSON.stringify(editor.value)
}
function toggleSelection(id, checked) { const value = String(id); selectionIds.value = checked ? [...new Set([...selectionIds.value, value])] : selectionIds.value.filter(item => item !== value) }
async function memoryBatch() { await command({ label: '批量处理记忆', op: 'memories.batch', fields: [{ key: 'action', label: '操作', type: 'text', choices: [{ value: 'archive', label: '归档' }, { value: 'restore', label: '恢复' }, { value: 'delete', label: '删除' }] }], defaults: { ids: selectionIds.value, action: 'archive', deleteSource: false }, confirm: true }, null) }
async function loadLogs(page) { await run(async () => { record.value = await read('logs.get', { id: recordId(selected.value), params: { page, pageSize: 30 } }); logPage.value = page }) }
function openLogThread() { if (record.value?.thread?.channel !== 'desktop') return; emit('thread', { threadId: record.value.thread.id, agentId: record.value.thread.agentId, title: record.value.thread.title || '会话' }) }
function fillIntegration() {
  const op = dialog.value?.action.op, body = commandBody.value
  if (op === 'integrations.extension') { const value = data.value?.extensions?.[body.extension]; if (value) commandBody.value = { extension: body.extension, enabled: value.enabled, chatEnabled: value.chatEnabled, endpoint: value.endpoint, maxResults: value.maxResults, apiKey: '', clearApiKey: false } }
  if (op === 'integrations.feature') { const value = data.value?.features?.[body.feature]; if (value) commandBody.value = { feature: body.feature, enabled: value.enabled, selectedSourceId: value.selectedSourceId } }
}
watch(() => commandBody.value.extension, (value, old) => { if (value !== old && value) fillIntegration() })
watch(() => commandBody.value.feature, (value, old) => { if (value !== old && value) fillIntegration() })
async function choose(item) { await run(async () => { record.value = null; editor.value = null; logPage.value = 1; const value = section.value.detail ? await read(section.value.detail, { id: recordId(item), params: section.value.id === 'logs' ? { page: 1, pageSize: 30 } : {} }) : item; record.value = value; editor.value = cloneData(value); baseline.value = JSON.stringify(editor.value); selected.value = item }) }
async function save() { await run(async () => { notice.value = ''; const item = section.value; if (item.config) await write('configs.save_batch', { body: { items: Object.entries(editor.value).filter(([key, value]) => String(value) !== String(JSON.parse(baseline.value || '{}')[key] ?? '')).map(([key, value]) => ({ key, value: String(value) })) } }); else { const body = Object.fromEntries(item.fields.filter(field => editor.value[field.key] !== undefined && !(field.type === 'object' && editor.value[field.key] === null)).map(field => [field.key, editor.value[field.key]])); if (item.id === 'config') body.value = String(body.value ?? ''); await write(item.save, { id: item.singleton ? null : recordId(selected.value), body }) }; selected.value = null; record.value = null; if (!item.config && !item.singleton) editor.value = null; notice.value = t('已保存'); if (item.id === 'user') setRemoteLocale(editor.value['user.language']); await load(); if (item.id === 'agents') await initializeRemote() }) }
async function command(action, item) { await run(async () => { if (editor.value && JSON.stringify(editor.value) !== baseline.value && !action.read) throw new Error('请先保存或放弃当前编辑，再执行其他写操作'); error.value = ''; notice.value = ''; let body = cloneData(action.defaults); if ('agentId' in body && !body.agentId) body.agentId = props.agentId || ''; if ('targetThreadId' in body && !body.targetThreadId && props.threadId) body.targetThreadId = props.threadId; if ('threadId' in body && body.threadId === null && props.threadId) body.threadId = props.threadId; if (action.load) { const value = await read(action.load, { id: item ? recordId(item) : null }); body = action.transform ? action.transform(value) : action.pick ? value[action.pick] : value }; commandBody.value = cloneData(body); dialog.value = { action, item }; if (action.replaceSelected && item) commandBody.value = { ...commandBody.value, replaceId: String(recordId(item)), title: item.title || '' }; fillIntegration(); commandBaseline.value = JSON.stringify(commandBody.value); if (!action.confirm && action.fields?.length === 0) await executeInner() }) }
async function execute() { await run(executeInner) }
async function executeInner() {
  const { action, item } = dialog.value
  resultTitle.value = action.label; resultOperation.value = action.op
  if (action.phrase && commandBody.value.confirm !== action.phrase) throw new Error('确认短语不匹配')
  let body = cloneData(commandBody.value)
  const id = action.idField ? body[action.idField] : item ? recordId(item) : null
  if (action.idField) delete body[action.idField]
  if (action.agentBody) body.agentId = item?.agentId || props.agentId
  if (action.cleanEmpty) for (const key of Object.keys(body)) if (body[key] === '' || body[key] === null || Array.isArray(body[key]) && !body[key].length) delete body[key]
  const params = { ...action.params }; if (action.agent) params.agentId = props.agentId
  for (const key of action.paramFields || []) { params[key] = body[key]; delete body[key] }
  const value = await (action.read ? read : write)(action.op, { id, params, body: Object.keys(body).length ? body : null })
  if (action.download) await downloadData(value, action.op === 'agents.export' ? value.fileName || 'infos-agent.json' : 'infos-config.json')
  else if (action.read || action.sensitive) { sensitiveResult.value = !!action.sensitive; result.value = value ?? { message: '已完成' } }
  dialog.value = null; commandBody.value = {}
  if (!action.read) { selected.value = null; editor.value = null; record.value = null; notice.value = '请求已由 infOS 接收'; await load(); if (section.value.id === 'agents') await initializeRemote() }
}
async function importConfig(event) { await run(async () => { const file = event.target.files?.[0]; if (!file) return; if (file.size > 1024 * 1024) throw new Error('配置文件请控制在 1 MiB 内'); const data = JSON.parse(await file.text()); guard(); if (!data || typeof data !== 'object' || Array.isArray(data) || Object.values(data).some(value => typeof value !== 'string')) throw new Error('请使用 infOS 导出的配置文件'); commandBody.value = { ...commandBody.value, data } }) }
async function upload(event) { const file = event.target.files?.[0]; event.target.value = ''; if (!file) return; await run(async () => { if (!file.size || file.size > 8 * 1024 * 1024) throw new Error('请选择不超过 8 MiB 的文件'); const bytes = Array.from(new Uint8Array(await file.arrayBuffer())); guard(); try { await remoteClient.manageUpload(section.value.id === 'agents' ? 'avatar' : section.value.upload, selected.value ? String(recordId(selected.value)) : null, { name: file.name, mimeType: file.type || 'application/octet-stream', bytes }, { token, fields: section.value.id === 'documents' && replaceUpload.value && selected.value ? { replaceId: String(recordId(selected.value)), title: selected.value.title || file.name } : {} }); guard(); if (section.value.id === 'agents') invalidateRemoteAvatar(String(recordId(selected.value))); notice.value = '上传完成'; selected.value = null; editor.value = null; record.value = null; await load() } catch (failure) { fail(failure); throw failure } finally { await refreshRemotePendingRequests().catch(failure => { try { guard(); remoteState.pendingError = failure.message } catch { /* 页面关闭后不更新新连接。 */ } }) } }) }
watch(() => props.initialSection, value => { if (value && value !== section.value?.id) { const item = managementSections.find(item => item.id === value); if (item) open(item) } }, { immediate: true })
</script>
