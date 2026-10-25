<template>
  <div class="remote-prompt-studio phase2"><RemoteFeedback :message="error" /><RemoteFeedback :message="notice" tone="success" /><RemoteAction v-if="unresolved" icon="alert" tone="danger" @click="$emit('pending')">操作结果待核对</RemoteAction>
    <div v-if="workspace" class="prompt-draft-status"><RemoteIcon :name="hasChanges ? 'edit' : 'check'" /><span>{{ tr(hasChanges ? '草稿有修改' : '草稿已保存') }}<small>{{ tr('保存后发布，下轮对话生效') }}</small></span><span class="prompt-revision">{{ tr('修订 {0}', [workspace.revision]) }}</span></div><RemoteFeedback :message="workspace?.recoveryError" />
    <div class="remote-purpose-actions"><RemoteAction icon="check" tone="primary" :disabled="busy || unresolved || !bundle || !hasChanges" @click="run(save)">{{ t('保存草稿') }}</RemoteAction><RemoteAction icon="send" :disabled="busy || unresolved || !bundle" @click="dialog = 'publish'">{{ t('发布') }}</RemoteAction><RemoteAction icon="shield" tone="quiet" :disabled="busy || !bundle" @click="run(validate)">校验</RemoteAction></div>
    <nav class="remote-subnav" :aria-label="tr('提示词工作区')"><button v-for="item in panels" :key="item.id" type="button" :aria-current="panel === item.id ? 'page' : undefined" :class="{ selected: panel === item.id }" :disabled="busy" @click="switchPanel(item.id)"><RemoteIcon :name="item.icon" />{{ t(item.label) }}</button></nav>
    <template v-if="bundle && collection">
      <p class="prompt-panel-help">{{ tr(panelHelp[panel]) }}</p>
      <div class="prompt-filter-bar">
        <label class="remote-search"><RemoteIcon name="search" /><input v-model="search" :placeholder="tr('搜索名称或内容')" :aria-label="tr('搜索提示词条目')" /><button v-if="search" type="button" :aria-label="tr('清空搜索')" @click="search = ''"><RemoteIcon name="close" /></button></label>
        <RemoteSelect v-if="panel !== 'assignments'" v-model="origin" :label="tr('来源筛选')" :options="originOptions" />
      </div>
      <nav class="prompt-category-picker" :aria-label="tr('条目分类')"><button v-for="item in categoryOptions" :key="item.id" type="button" :class="{ selected: category === item.id }" :aria-pressed="category === item.id" @click="category = item.id"><RemoteIcon :name="item.icon" /><span>{{ tr(item.label) }}</span><small>{{ item.count }}</small></button></nav>
      <div class="prompt-list-heading"><span aria-live="polite">{{ tr('{0} 个条目', [entries.length]) }}</span><RemoteAction icon="plus" :disabled="busy || unresolved" @click="addEntry">{{ tr(addLabels[panel]) }}</RemoteAction></div>
      <section v-for="group in groupedEntries" :key="group.id" class="prompt-category-group" :data-category="group.id">
        <header><span><RemoteIcon :name="group.icon" />{{ tr(group.label) }}</span><small>{{ group.entries.length }}</small></header>
        <div class="prompt-entry-grid"><button v-for="entry in group.entries" :key="entry.index" type="button" class="prompt-entry-card" :disabled="busy" :class="{ 'is-disabled': entry.item.enabled === false }" @click="openEntry(entry.index)">
          <span class="prompt-entry-icon"><RemoteIcon :name="group.icon" /></span><span class="prompt-entry-copy"><strong>{{ entryTitle(entry.item, entry.index) }}</strong><small>{{ entryDescription(entry.item) }}</small><span class="prompt-entry-badges"><span v-if="panel !== 'assignments'">{{ tr(entry.item.builtin ? '内置' : '自定义') }}</span><span v-if="entry.item.enabled === false" class="is-muted">{{ t('停用') }}</span></span></span><RemoteIcon name="next" />
        </button></div>
      </section>
      <div v-if="!entries.length" class="prompt-empty"><RemoteIcon name="search" /><strong>{{ tr('没有找到条目') }}</strong><p>{{ tr('换个分类或关键词试试') }}</p><button v-if="search || category !== 'all' || origin !== 'all'" type="button" class="room-text-button" @click="search = ''; category = 'all'; origin = 'all'">{{ tr('清除筛选') }}</button></div>
    </template>
    <RoomDrawer v-if="currentEntry" variant="editor" :title="entryTitle(currentEntry, selectedIndex)" :subtitle="tr('提示词草稿')" :busy="busy" @close="selectedIndex = -1">
      <div class="prompt-entry-editor">
        <div class="prompt-editor-toolbar"><span>{{ tr(categoryOf(currentEntry).label) }}</span><nav v-if="panel === 'documents'" :aria-label="tr('正文显示方式')"><button type="button" :class="{ selected: editingPrompt }" :aria-pressed="editingPrompt" :disabled="busy" @click="editingPrompt = true"><RemoteIcon name="edit" />{{ t('编辑') }}</button><button type="button" :class="{ selected: !editingPrompt }" :aria-pressed="!editingPrompt" :disabled="busy" @click="editingPrompt = false"><RemoteIcon name="document" />{{ t('阅读') }}</button></nav></div>
        <div :key="panel + ':' + selectedIndex" class="prompt-editor-scroll">
          <template v-if="panel === 'documents'">
            <RemoteFields v-model="currentEntry" :fields="[field('name', 'text', { max: 100 })]" :disabled="busy" />
            <label v-if="editingPrompt" class="prompt-body-field"><span>{{ tr('提示词正文') }}<small>{{ tr('{0} 字', [currentEntry.body?.length || 0]) }}</small></span><textarea :value="currentEntry.body || ''" :aria-label="tr('提示词正文')" :disabled="busy" maxlength="200000" spellcheck="false" @input="updateBody($event.target.value)" /></label>
            <article v-else class="prompt-body-preview"><RemoteRichText v-if="currentEntry.body" :source="currentEntry.body" :node-id="'prompt-' + currentEntry.id" /><p v-else class="remote-help">{{ tr('正文还是空的，切到编辑开始写吧') }}</p></article>
            <details class="prompt-entry-metadata"><summary><RemoteIcon name="settings" /><span>{{ tr('条目设置') }}</span><RemoteIcon name="down" /></summary><p class="remote-help">{{ tr('标识用于框架引用，修改前请确认关联项') }}</p><RemoteFields v-model="currentEntry" :fields="documentMetadata" :disabled="busy" /></details>
          </template>
          <RemoteFormSections v-else v-model="currentEntry" :kind="'prompt-' + panel" :fields="entryFields" :disabled="busy" />
          <div class="prompt-editor-remove"><RemoteAction icon="trash" tone="danger" :disabled="busy || unresolved" @click="dialog = 'remove'">{{ tr('移除此条目') }}</RemoteAction></div>
        </div>
      </div>
      <template #footer="{ close }"><div class="prompt-editor-footer"><span>{{ tr(hasChanges ? '修改尚未保存' : '草稿已保存') }}</span><RemoteAction icon="back" tone="quiet" :disabled="busy" @click="close">{{ tr('返回列表') }}</RemoteAction><RemoteAction icon="check" tone="primary" :disabled="busy || unresolved || !hasChanges" @click="run(save)">{{ t('保存草稿') }}</RemoteAction></div></template>
    </RoomDrawer>
    <section v-if="panel === 'preview'" class="remote-manager-card"><h2>编译预览</h2><p class="remote-help">使用当前草稿编译，不会发布或调用模型。</p><RemoteFormSections v-model="previewInput" :fields="previewFields" :disabled="busy" /><RemoteAction icon="open" tone="primary" :disabled="busy || !bundle" @click="run(preview)">编译预览</RemoteAction><RemoteAction icon="download" :disabled="busy || !previewInput.frameworkId" @click="run(async () => { result = null; await downloadData(await read('prompts.export', { body: { bundle, frameworkId: previewInput.frameworkId } }), 'sillytavern-preset.json') })">导出酒馆预设</RemoteAction></section>
    <section v-if="panel === 'releases'"><p class="remote-help">恢复操作只改草稿，仍需单独发布。</p><div v-for="release in workspace?.releases || []" :key="release.id" class="remote-manager-card"><h3>{{ release.note || '未填写说明' }}</h3><p class="remote-help">{{ release.createdAt }} · {{ release.id }}</p><div class="remote-purpose-actions"><RemoteAction icon="open" :disabled="busy" @click="run(async () => { result = await read('prompts.release', { id: release.id }) })">查看版本</RemoteAction><RemoteAction icon="refresh" :disabled="busy || unresolved" @click="restoreId = release.id; dialog = 'restore'">恢复到草稿</RemoteAction></div></div><p v-if="!workspace?.releases?.length" class="room-empty">暂无发布记录</p></section>
    <section v-if="panel === 'variables'"><h2>模板变量</h2><RemoteFields v-model="variableScope" :fields="[field('scope')]" /><div class="remote-purpose-actions"><RemoteAction icon="open" :disabled="busy" @click="run(async () => { result = await read('prompts.variables', { params: variableScope }) })">读取状态</RemoteAction><RemoteAction icon="trash" tone="danger" :disabled="busy || unresolved" @click="dialog = 'reset'">重置此范围变量</RemoteAction></div><details class="remote-diagnostic-data"><summary>服务端变量定义</summary><RemoteValue :value="workspace?.variables" /></details></section>
    <details class="remote-secondary-operations"><summary><RemoteIcon name="folder" />导入、导出与重新读取</summary><div class="remote-purpose-actions"><RemoteAction icon="download" :disabled="busy || !bundle" @click="run(() => downloadData(bundle, 'infos-prompts.json'))">导出草稿</RemoteAction><RemoteAction icon="attach" :disabled="busy || !bundle" @click="importInput.click()">导入预设</RemoteAction><RemoteAction icon="refresh" :disabled="busy" @click="dialog = 'reload'">重新读取</RemoteAction><input ref="importInput" type="file" accept="application/json,.json" hidden @change="importPreset" /></div></details>
    <RoomDrawer v-if="dialog" :title="({ publish: '发布当前草稿？', restore: '恢复发布版本到草稿？', reload: '重新读取草稿？', reset: '重置模板变量？', remove: '从草稿移除此条目？' })[dialog]" :busy="busy" @close="dialog = null"><RemoteFeedback :message="error" /><p class="remote-help">{{ dialog === 'publish' ? '会先校验并保存当前修改，再发布到 infOS。' : dialog === 'reload' || dialog === 'restore' ? '当前未保存的编辑将被替换。' : dialog === 'remove' ? '仅修改本页草稿，保存并发布后生效。引用此条目的规则也需要同步调整。' : '指定范围的模板变量将被清空。' }}</p><label v-if="dialog === 'publish'" class="remote-field">发布说明<input v-model="releaseNote" maxlength="200" /></label><RemoteAction icon="check" :tone="['remove', 'reset'].includes(dialog) ? 'danger' : 'primary'" :disabled="busy || unresolved && !['reload', 'remove'].includes(dialog)" @click="confirm">确认</RemoteAction></RoomDrawer>
    <RoomDrawer v-if="result !== null" title="提示词结果" @close="result = null"><RemoteOperationResult operation="prompts" :value="result" /></RoomDrawer>
  </div>
</template>
<script setup>
import RemoteFeedback from './RemoteFeedback.vue'
import { ref, computed, onMounted, watch } from 'vue'
import RemoteRichText from './RemoteRichText.vue'
import { t } from '../../i18n/remote'
import RemoteSelect from './RemoteSelect.vue'
import { remoteState } from '../../stores/remote'
import RemoteFields from './RemoteFields.vue'
import RemoteValue from './RemoteValue.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteFormSections from './RemoteFormSections.vue'
import RemoteOperationResult from './RemoteOperationResult.vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import { field, downloadData, cloneData, arrayDefaults, inferFields } from '../../services/remoteManagement'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
const props = defineProps({ agentId: String })
defineEmits(['pending'])
const { read, write, run, guard, error, busy, unresolved } = useRemoteManagement()
const workspace = ref(null), bundle = ref(null), notice = ref(''), result = ref(null), dialog = ref(null), releaseNote = ref(''), restoreId = ref(''), importInput = ref(null)
const previewInput = ref({ agentId: props.agentId || '', channel: 'desktop', frameworkId: '', variables: {} }), variableScope = ref({ scope: '$global' })
const tr = (source, parameters = []) => t(source, parameters, 'prompts')
const panel = ref('documents'), selectedIndex = ref(-1), search = ref(''), category = ref('all'), origin = ref('all')
const editingPrompt = ref(true)
watch([panel, selectedIndex], () => { editingPrompt.value = true })
const panels = [{ id: 'documents', label: '文档', icon: 'document' }, { id: 'frameworks', label: '框架', icon: 'code' }, { id: 'assignments', label: '分配', icon: 'route' }, { id: 'preview', label: '预览', icon: 'open' }, { id: 'releases', label: '发布记录', icon: 'clock' }, { id: 'variables', label: '变量', icon: 'settings' }]
const panelHelp = { documents: '按用途整理提示词，点开即可编辑。', frameworks: '选择框架，调整正文组合与历史位置。', assignments: '选择适用范围，安排使用的框架。' }
const addLabels = { documents: '新建正文', frameworks: '新建框架', assignments: '新建分配' }
const categoryDefinitions = {
  documents: [{ id: 'slot', label: '对话槽位', icon: 'chat' }, { id: 'component', label: '复用组件', icon: 'code' }, { id: 'task', label: '任务提示', icon: 'tasks' }, { id: 'other', label: '其他', icon: 'folder' }],
  frameworks: [{ id: 'infos', label: 'infOS 框架', icon: 'ribbon' }, { id: 'sillytavern', label: '酒馆预设', icon: 'document' }, { id: 'custom', label: '自定义框架', icon: 'edit' }, { id: 'other', label: '其他', icon: 'folder' }],
  assignments: [{ id: 'global', label: '全局', icon: 'globe' }, { id: 'channel', label: '按频道', icon: 'chat' }, { id: 'agent', label: '按角色', icon: 'profile' }, { id: 'agent-channel', label: '角色与频道', icon: 'route' }],
}
const originOptions = computed(() => [{ value: 'all', label: tr('全部来源') }, { value: 'builtin', label: tr('内置') }, { value: 'custom', label: tr('自定义') }])
const collection = computed(() => ['documents', 'frameworks', 'assignments'].includes(panel.value) ? bundle.value?.[panel.value] : null)
const channelLabel = value => t(({ desktop: '单聊', group: '群聊', social: '社交频道' })[value] || value || '全部频道')
function entryTitle(item, index) { return item.name || (panel.value === 'assignments' ? `${remoteState.agents.find(agent => agent.id === item.agentId)?.name || item.agentId || tr('全局')} · ${channelLabel(item.channel)}` : item.id) || tr('条目 {0}', [index + 1]) }
function categoryOf(item) {
  const id = panel.value === 'documents' ? item.kind : panel.value === 'frameworks' ? item.format ?? (item.builtin ? 'infos' : 'custom') : item.agentId ? item.channel ? 'agent-channel' : 'agent' : item.channel ? 'channel' : 'global'
  return categoryDefinitions[panel.value]?.find(category => category.id === id) || categoryDefinitions[panel.value]?.find(category => category.id === 'other')
}
function entryDescription(item) {
  if (panel.value === 'documents') return String(item.body || '').slice(0, 700).replace(/\s+/g, ' ').trim().slice(0, 110) || tr('正文还是空的')
  if (panel.value === 'frameworks') return tr('{0} 个区块 · {1} 个绑定', [item.blocks?.length || 0, item.bindings?.length || 0])
  return bundle.value?.frameworks.find(framework => framework.id === item.frameworkId)?.name || item.frameworkId || tr('继承框架')
}
const matchingEntries = computed(() => {
  const keyword = search.value.trim().toLowerCase()
  return (collection.value || []).map((item, index) => ({ item, index })).filter(({ item, index }) =>
    (panel.value === 'assignments' || origin.value === 'all' || (item.builtin ? 'builtin' : 'custom') === origin.value) &&
    (!keyword || [entryTitle(item, index), item.id, item.body, item.agentId, item.channel, item.frameworkId].some(value => String(value || '').toLowerCase().includes(keyword))))
})
const categoryOptions = computed(() => [{ id: 'all', label: '全部', icon: 'folder', count: matchingEntries.value.length }, ...(categoryDefinitions[panel.value] || []).map(item => ({ ...item, count: matchingEntries.value.filter(entry => categoryOf(entry.item).id === item.id).length })).filter(item => item.id !== 'other' || item.count)])
const entries = computed(() => matchingEntries.value.filter(entry => category.value === 'all' || categoryOf(entry.item).id === category.value))
const groupedEntries = computed(() => (categoryDefinitions[panel.value] || []).map(group => ({ ...group, entries: entries.value.filter(entry => categoryOf(entry.item).id === group.id) })).filter(group => group.entries.length))
function switchPanel(id) { panel.value = id; selectedIndex.value = -1; search.value = ''; category.value = 'all'; origin.value = 'all' }
function openEntry(index) { selectedIndex.value = index }
const currentEntry = computed({ get: () => collection.value?.[selectedIndex.value] || null, set: value => { if (collection.value && selectedIndex.value >= 0) bundle.value = { ...bundle.value, [panel.value]: collection.value.map((item, index) => index === selectedIndex.value ? value : item) } } })
function updateBody(value) { currentEntry.value = { ...currentEntry.value, body: value } }
const documentMetadata = computed(() => [field('enabled', 'boolean'), field('kind', 'text', { choices: categoryDefinitions.documents.filter(item => item.id !== 'other').map(item => ({ value: item.id, label: tr(item.label) })), translateChoices: false }), field('role', 'text', { choices: ['system', 'user', 'assistant'] }), field('id', 'text', { max: 180 }), field('variables', 'array')])
const entryFields = computed(() => inferFields(currentEntry.value || {}).filter(item => !['builtin', 'builtinBodyHash'].includes(item.key)).map(item => item.key === 'frameworkId' ? { ...item, choices: [{ value: '', label: '继承框架' }, ...(bundle.value?.frameworks || []).map(framework => ({ value: framework.id, label: framework.name || framework.id }))] } : item.key === 'channel' ? { ...item, choices: [{ value: '', label: '全部频道' }, ...['desktop', 'group', 'social']] } : item))
function addEntry() {
  if (!collection.value || busy.value || unresolved.value) return
  const value = cloneData(arrayDefaults[panel.value])
  if ('id' in value) value.id = crypto.randomUUID()
  if (panel.value === 'documents' && ['slot', 'component', 'task'].includes(category.value)) value.kind = category.value
  if (panel.value === 'frameworks' && ['infos', 'sillytavern', 'custom'].includes(category.value)) value.format = category.value
  if (panel.value === 'assignments') { value.agentId = ['agent', 'agent-channel'].includes(category.value) ? props.agentId || '' : ''; value.channel = ['global', 'agent'].includes(category.value) ? '' : 'desktop' }
  bundle.value = { ...bundle.value, [panel.value]: [...collection.value, value] }
  // 新条目总能在返回列表后找到，来源筛选随之切到自定义。
  search.value = ''; origin.value = panel.value === 'assignments' ? 'all' : 'custom'; category.value = categoryOf(value).id
  selectedIndex.value = collection.value.length - 1
}
const hasChanges = computed(() => !!bundle.value && JSON.stringify(bundle.value) !== JSON.stringify(workspace.value?.draft))
defineExpose({ busy, hasChanges })
const previewFields = [field('agentId'), field('channel', 'text', { choices: ['desktop', 'group', 'social'] }), field('frameworkId', 'text', { help: '留空使用当前分配的框架' }), field('variables', 'object')]
async function load() { const value = await read('prompts.get'); if (!value?.draft || !Number.isInteger(value.revision)) throw new Error('提示词工作区响应不完整'); workspace.value = value; bundle.value = cloneData(value.draft) }
async function validate() { const issues = await read('prompts.validate', { body: { bundle: bundle.value } }); result.value = issues; return Array.isArray(issues) ? issues : issues?.issues || [] }
async function save() { const check = await read('prompts.validate', { body: { bundle: bundle.value } }); const issues = Array.isArray(check) ? check : check?.issues || []; if (issues.some(item => item.level === 'error')) { result.value = issues; throw new Error('草稿存在错误，请先处理校验结果') }; const value = await write('prompts.save', { body: { bundle: bundle.value, revision: workspace.value.revision } }); if (value?.draft && Number.isInteger(value.revision)) { workspace.value = value; bundle.value = cloneData(value.draft) } else await load(); notice.value = '草稿已保存，尚未发布' }
async function preview() { const body = { bundle: bundle.value, ...previewInput.value }; if (!body.frameworkId) delete body.frameworkId; result.value = await read('prompts.preview', { body }) }
async function confirm() { await run(async () => { if (dialog.value === 'remove') { bundle.value = { ...bundle.value, [panel.value]: collection.value.filter((_, index) => index !== selectedIndex.value) }; selectedIndex.value = -1; notice.value = '条目已从本页草稿移除，尚未保存' } else if (dialog.value === 'reload') { await load(); selectedIndex.value = -1 } else if (dialog.value === 'publish') { await save(); await write('prompts.publish', { body: { revision: workspace.value.revision, note: releaseNote.value } }); await load(); notice.value = '已发布，下轮调用生效' } else if (dialog.value === 'restore') { await write('prompts.restore', { body: { id: restoreId.value, revision: workspace.value.revision } }); await load(); notice.value = '已恢复到草稿，尚未发布' } else await write('prompts.reset_variables', { body: variableScope.value }); dialog.value = null }) }
async function importPreset(event) { const file = event.target.files?.[0]; event.target.value = ''; if (!file) return; await run(async () => { if (file.size > 1024 * 1024) throw new Error('预设请控制在 1 MiB 内'); const value = JSON.parse(await file.text()); guard(); if (Array.isArray(value?.documents) && Array.isArray(value?.frameworks) && Array.isArray(value?.assignments)) { bundle.value = value; notice.value = '已载入本页，保存并发布后生效' } else { const imported = await read('prompts.import', { body: { preset: value, name: file.name.replace(/\.json$/i, '') } }); const id = imported.framework?.id; if (!id) throw new Error('酒馆预设未返回框架'); bundle.value = { ...bundle.value, frameworks: [...bundle.value.frameworks.filter(item => item.id !== id), imported.framework] }; result.value = imported.issues; notice.value = '已导入框架到本页，保存并发布后生效' } }) }
onMounted(() => run(load))
</script>
