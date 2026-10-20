<template>
  <div><p v-if="error" class="room-error">{{ error }}</p><p v-if="notice" class="remote-notice">{{ notice }}</p><p v-if="unresolved" class="room-error">操作结果待核对。<button @click="$emit('pending')">去核对</button></p>
    <div class="remote-manager-actions"><button :disabled="busy" @click="dialog = 'reload'">重新读取</button><button :disabled="busy || !bundle" @click="run(validate)">校验草稿</button><button :disabled="busy || unresolved || !bundle" @click="run(save)">保存草稿</button><button :disabled="busy || unresolved || !bundle" @click="dialog = 'publish'">发布</button><button :disabled="busy || !bundle" @click="run(() => downloadData(bundle, 'infos-prompts.json'))">导出草稿</button><button :disabled="busy || !bundle" @click="importInput.click()">导入预设</button><input ref="importInput" type="file" accept="application/json,.json" hidden @change="importPreset" /></div>
    <p v-if="workspace" class="remote-help">修订 {{ workspace.revision }} · 当前发布 {{ workspace.activeReleaseId || '尚未发布' }}。草稿保存后仍需发布，下轮调用生效。</p><p v-if="workspace?.recoveryError" class="room-error">{{ workspace.recoveryError }}</p>
    <RemoteFields v-if="bundle" v-model="bundle" :fields="bundleFields" :disabled="busy" />
    <details v-if="workspace"><summary>服务端模板变量</summary><RemoteValue :value="workspace.variables" /></details>
    <div class="remote-manager-card"><h2>编译预览</h2><RemoteFields v-model="previewInput" :fields="previewFields" :disabled="busy" /><button class="room-secondary-button" :disabled="busy || !bundle" @click="run(preview)">预览编译结果</button></div>
    <details><summary>发布记录</summary><div class="remote-manager-records"><div v-for="release in workspace?.releases || []" :key="release.id" class="remote-manager-record"><span><strong>{{ release.note || release.id }}</strong><small>{{ release.createdAt }}</small></span><button @click="run(async () => { result = await read('prompts.release', { id: release.id }) })">查看</button><button :disabled="busy || unresolved" @click="restoreId = release.id; dialog = 'restore'">恢复</button></div></div></details>
    <div class="remote-manager-card"><h2>模板变量状态</h2><RemoteFields v-model="variableScope" :fields="[field('scope')]" /><div class="remote-manager-actions"><button :disabled="busy" @click="run(async () => { result = await read('prompts.variables', { params: variableScope }) })">读取变量</button><button :disabled="busy || unresolved" @click="dialog = 'reset'">重置变量</button><button :disabled="busy || !previewInput.frameworkId" @click="run(async () => { await downloadData(await read('prompts.export', { body: { bundle, frameworkId: previewInput.frameworkId } }), 'sillytavern-preset.json') })">导出酒馆预设</button></div></div>
    <RoomDrawer v-if="dialog" :title="({ publish: '发布当前草稿？', restore: '恢复发布版本到草稿？', reload: '重新读取草稿？', reset: '重置模板变量？' })[dialog]" :busy="busy" @close="dialog = null"><p v-if="error" class="room-error">{{ error }}</p><p class="remote-help">{{ dialog === 'publish' ? '会先校验并保存当前修改，再发布到 infOS。' : dialog === 'reload' || dialog === 'restore' ? '当前未保存的编辑将被替换。' : '指定范围的模板变量将被清空。' }}</p><label v-if="dialog === 'publish'" class="remote-field">发布说明<input v-model="releaseNote" maxlength="200" /></label><button class="room-primary-button" :disabled="busy || unresolved && dialog !== 'reload'" @click="confirm">确认</button></RoomDrawer>
    <RoomDrawer v-if="result !== null" title="提示词结果" @close="result = null"><RemoteValue :value="result" /></RoomDrawer>
  </div>
</template>
<script setup>
import { ref, computed, onMounted } from 'vue'
import RemoteFields from './RemoteFields.vue'
import RemoteValue from './RemoteValue.vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import { field, downloadData, cloneData } from '../../services/remoteManagement'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
const props = defineProps({ agentId: String })
defineEmits(['pending'])
const { read, write, run, guard, error, busy, unresolved } = useRemoteManagement()
const workspace = ref(null), bundle = ref(null), notice = ref(''), result = ref(null), dialog = ref(null), releaseNote = ref(''), restoreId = ref(''), importInput = ref(null)
const previewInput = ref({ agentId: props.agentId || '', channel: 'desktop', frameworkId: '', variables: {} }), variableScope = ref({ scope: '$global' })
const bundleFields = ['documents', 'frameworks', 'assignments'].map(key => field(key, 'array', { open: key === 'documents' }))
const hasChanges = computed(() => !!bundle.value && JSON.stringify(bundle.value) !== JSON.stringify(workspace.value?.draft))
defineExpose({ busy, hasChanges })
const previewFields = [field('agentId'), field('channel', 'text', { choices: ['desktop', 'group', 'social'] }), field('frameworkId', 'text', { help: '留空使用当前分配的框架' }), field('variables', 'object')]
async function load() { const value = await read('prompts.get'); if (!value?.draft || !Number.isInteger(value.revision)) throw new Error('提示词工作区响应不完整'); workspace.value = value; bundle.value = cloneData(value.draft) }
async function validate() { const issues = await read('prompts.validate', { body: { bundle: bundle.value } }); result.value = issues; return Array.isArray(issues) ? issues : issues?.issues || [] }
async function save() { const check = await read('prompts.validate', { body: { bundle: bundle.value } }); const issues = Array.isArray(check) ? check : check?.issues || []; if (issues.some(item => item.level === 'error')) { result.value = issues; throw new Error('草稿存在错误，请先处理校验结果') }; const value = await write('prompts.save', { body: { bundle: bundle.value, revision: workspace.value.revision } }); if (value?.draft && Number.isInteger(value.revision)) { workspace.value = value; bundle.value = cloneData(value.draft) } else await load(); notice.value = '草稿已保存，尚未发布' }
async function preview() { const body = { bundle: bundle.value, ...previewInput.value }; if (!body.frameworkId) delete body.frameworkId; result.value = await read('prompts.preview', { body }) }
async function confirm() { await run(async () => { if (dialog.value === 'reload') await load(); else if (dialog.value === 'publish') { await save(); await write('prompts.publish', { body: { revision: workspace.value.revision, note: releaseNote.value } }); await load(); notice.value = '已发布，下轮调用生效' } else if (dialog.value === 'restore') { await write('prompts.restore', { body: { id: restoreId.value, revision: workspace.value.revision } }); await load(); notice.value = '已恢复到草稿，尚未发布' } else await write('prompts.reset_variables', { body: variableScope.value }); dialog.value = null }) }
async function importPreset(event) { const file = event.target.files?.[0]; event.target.value = ''; if (!file) return; await run(async () => { if (file.size > 1024 * 1024) throw new Error('预设请控制在 1 MiB 内'); const value = JSON.parse(await file.text()); guard(); if (Array.isArray(value?.documents) && Array.isArray(value?.frameworks) && Array.isArray(value?.assignments)) { bundle.value = value; notice.value = '已载入本页，保存并发布后生效' } else { const imported = await read('prompts.import', { body: { preset: value, name: file.name.replace(/\.json$/i, '') } }); const id = imported.framework?.id; if (!id) throw new Error('酒馆预设未返回框架'); bundle.value = { ...bundle.value, frameworks: [...bundle.value.frameworks.filter(item => item.id !== id), imported.framework] }; result.value = imported.issues; notice.value = '已导入框架到本页，保存并发布后生效' } }) }
onMounted(() => run(load))
</script>
