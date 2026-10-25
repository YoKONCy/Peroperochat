<template>
  <section class="remote-tts-settings">
    <RemoteFeedback :message="error" /><RemoteAction v-if="unresolved" icon="alert" @click="$emit('pending')">{{ tr('去核对') }}</RemoteAction>
    <header class="tts-workspace-heading"><span><RemoteIcon name="volume" /><strong>{{ tr('声音预设') }}</strong></span><RoomSwitch v-if="workspace" v-model="workspace.enabled" :disabled="locked" :label="tr('启用朗读')" /></header>
    <button v-if="!workspace" type="button" class="room-secondary-button" :disabled="busy" @click="load">{{ tr(busy ? '正在读取…' : '重新读取声音配置') }}</button>
    <template v-else>
      <div class="tts-preset-picker"><RemoteSelect :model-value="selected" :label="tr('声音预设')" :options="presetOptions" :disabled="locked" @update:model-value="select" /><button type="button" class="room-icon-button" :disabled="locked" :aria-label="tr('新建声音预设')" @click="adding = true"><RemoteIcon name="plus" /></button></div>
      <nav class="phase-tabs"><button v-for="item in panels" :key="item.id" type="button" :class="{ selected: panel === item.id }" :disabled="locked" @click="panel = item.id"><RemoteIcon :name="item.icon" />{{ tr(item.label) }}</button></nav>
      <section v-if="preset && panel === 'voice'" class="phase-card tts-preset-card" data-ui-tone="rose">
        <header><span class="chromatic-tag">{{ adapterName(profile?.adapter) }}</span><button type="button" class="room-text-button" :disabled="locked || workspace.defaultPresetId === selected" @click="workspace.defaultPresetId = selected"><RemoteIcon name="check" />{{ tr(workspace.defaultPresetId === selected ? '默认声音' : '设为默认声音') }}</button></header>
        <RemoteFields :model-value="preset" :fields="voiceFields" :disabled="locked" @update:model-value="Object.assign(preset, $event)" />
        <template v-if="supportsReference"><h4>{{ tr('参考音频') }}</h4><p class="remote-help">{{ tr('上传 WAV 或 MP3，最多 7MB。') }}</p><div class="remote-purpose-actions"><button type="button" class="room-secondary-button" :disabled="locked" @click="referenceInput?.click()"><RemoteIcon name="attach" />{{ tr(preset.referenceAssetId ? '更换参考音频' : '上传参考音频') }}</button><button v-if="preset.referenceAssetId" type="button" class="room-text-button" :disabled="locked" @click="delete preset.referenceAssetId; referenceName = ''">{{ tr('移除') }}</button><span v-if="preset.referenceAssetId" class="chromatic-tag">{{ referenceName || tr('已选择参考音频') }}</span><input ref="referenceInput" type="file" accept="audio/wav,audio/x-wav,audio/mpeg,.wav,.mp3" hidden @change="uploadReference" /></div><RemoteFields :model-value="preset" :fields="[f('referenceText', 'long', { label: '参考音频对应的文字', max: 16000 })]" :disabled="locked" @update:model-value="Object.assign(preset, $event)" /></template>
        <details class="phase-fold"><summary><RemoteIcon name="settings" />{{ tr('扩展参数') }}<RemoteIcon name="down" /></summary><label class="tts-json-label">{{ tr('厂商扩展参数') }}<textarea v-model="extraText" rows="5" :disabled="locked" spellcheck="false" /></label></details>
      </section>
      <section v-if="preset && panel === 'connection'" class="phase-card" data-ui-tone="blue">
        <h3><RemoteIcon name="model" />{{ tr('模型与连接') }}</h3>
        <label class="tts-select-label">{{ tr('合成模型') }}<RemoteSelect :model-value="preset.modelId" :label="tr('合成模型')" :options="modelOptions" :disabled="locked" @update:model-value="changeModel" /></label>
        <RemoteFields v-if="model" :model-value="model" :fields="modelFields" :disabled="locked" @update:model-value="Object.assign(model, $event)" />
        <label v-if="model" class="tts-select-label">{{ tr('连接') }}<RemoteSelect :model-value="model.connectionId" :label="tr('连接')" :options="connectionOptions" :disabled="locked" @update:model-value="changeConnection" /></label>
        <p class="remote-help">{{ tr('共用模型或连接的声音，也会使用这里的修改。') }}</p>
        <RemoteFields v-if="connection" :model-value="connection" :fields="connectionFields" :disabled="locked" @update:model-value="Object.assign(connection, $event)" />
        <details class="phase-fold"><summary><RemoteIcon name="code" />{{ tr('高级连接设置') }}<RemoteIcon name="down" /></summary><label class="tts-json-label">{{ tr('请求头') }}<textarea v-model="headersText" rows="4" :disabled="locked" spellcheck="false" /></label><template v-if="profile?.adapter === 'custom'"><p class="remote-help">{{ tr('模板仅由 infOS 解释，支持 JSON、表单、XML 和流式响应。') }}</p><label class="tts-json-label">{{ tr('自定义 HTTP 协议') }}<textarea v-model="customText" rows="12" :disabled="locked" spellcheck="false" /></label></template></details>
      </section>
      <section v-if="panel === 'bindings'" class="phase-card" data-ui-tone="violet"><h3><RemoteIcon name="people" />{{ tr('角色声音') }}</h3><label class="tts-select-label">{{ tr('默认声音') }}<RemoteSelect v-model="workspace.defaultPresetId" :label="tr('默认声音')" :options="presetOptions" :disabled="locked" /></label><div v-for="agent in agents" :key="agent.id" class="tts-agent-binding"><RemoteAvatar :id="agent.id" :label="agent.name" /><div><strong>{{ agent.name }}</strong><RemoteSelect :model-value="workspace.bindings[agent.id] || ''" :label="agent.name" :options="bindingOptions" :disabled="locked" @update:model-value="bind(agent.id, $event)" /></div></div></section>
      <section class="phase-card tts-preview-card" data-ui-tone="mint"><h3><RemoteIcon name="volume" />{{ tr('试听') }}</h3><textarea v-model="testText" :aria-label="tr('试听文字')" :placeholder="tr('你好，很高兴与你见面。')" rows="2" maxlength="100000" :disabled="busy" @input="stopPreview" /><p v-if="hasChanges" class="remote-help">{{ tr('试听前会先保存当前修改。') }}</p><div class="remote-purpose-actions"><button type="button" class="room-primary-button" :disabled="busy || unresolved || !workspace.enabled" @click="preview"><RemoteIcon :name="previewActive && ['loading', 'playing'].includes(speech.status) ? 'stop' : 'volume'" />{{ tr(previewActive && speech.status === 'loading' ? '停止生成' : previewActive && speech.status === 'playing' ? '停止朗读' : '试听') }}</button></div></section>
      <footer class="remote-edit-footer phase-settings-save"><button type="button" class="room-text-button" :disabled="locked || workspace.presets.length < 2" @click="removing = true"><RemoteIcon name="trash" />{{ tr('删除预设') }}</button><RemoteAction icon="check" tone="primary" :disabled="locked || !hasChanges || unresolved" @click="save">{{ tr('保存声音配置') }}</RemoteAction></footer>
      <button type="button" class="room-text-button" :disabled="locked || hasChanges" @click="load"><RemoteIcon name="refresh" />{{ tr('刷新') }}</button>
    </template>
    <RoomDrawer v-if="adding" :title="tr('新建声音预设')" @close="adding = false"><RemoteSelect v-model="addAdapter" :label="tr('语音服务')" :options="adapterOptions" /><button type="button" class="room-primary-button" :disabled="locked || !addAdapter" @click="add">{{ tr('创建预设') }}</button></RoomDrawer>
    <RoomDrawer v-if="removing" :title="tr('删除这个声音预设？')" @close="removing = false"><p>{{ tr('使用它的角色将改为跟随默认声音。') }}</p><div class="remote-purpose-actions"><button type="button" class="room-secondary-button" @click="removing = false">{{ tr('取消') }}</button><button type="button" class="room-danger-button" :disabled="locked" @click="remove">{{ tr('删除预设') }}</button></div></RoomDrawer>
  </section>
</template>
<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { remoteClient } from '../../services/remote'
import { remoteState, refreshRemotePendingRequests } from '../../stores/remote'
import { remoteSpeech as speech, startRemoteSpeech, stopRemoteSpeech, replayRemoteSpeech } from '../../services/remoteSpeech'
import { ttsAdapterLabels, ttsChain, addTtsPreset, removeTtsPreset } from '../../utils/ttsWorkspace'
import { t } from '../../i18n/remote'
import { appMessage } from '../../services/appFeedback'
import RemoteFeedback from './RemoteFeedback.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteAvatar from './RemoteAvatar.vue'
import RemoteSelect from './RemoteSelect.vue'
import RemoteFields from './RemoteFields.vue'
import RoomSwitch from '../companion/RoomSwitch.vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
defineEmits(['pending'])
const { token, read, write, guard, run, busy, error, unresolved, fail } = useRemoteManagement()
const tr = key => t(key, [], 'tts')
const workspace = ref(null), adapters = ref([]), selected = ref(''), baseline = ref(''), panel = ref('voice'), adding = ref(false), removing = ref(false), addAdapter = ref(''), referenceInput = ref(null), referenceName = ref(''), testText = ref('')
const extraText = ref('{}'), headersText = ref('{}'), customText = ref('{}')
const previewId = `tts-preview:${crypto.randomUUID()}`, previewActive = computed(() => speech.id === previewId)
const synthesizing = computed(() => previewActive.value && speech.status === 'loading')
const locked = computed(() => busy.value || synthesizing.value || remoteState.connection !== 'online')
const chain = computed(() => ttsChain(workspace.value, selected.value)), preset = computed(() => chain.value.preset), model = computed(() => chain.value.model), profile = computed(() => chain.value.profile), connection = computed(() => chain.value.connection)
const adapter = computed(() => adapters.value.find(item => item.id === profile.value?.adapter))
const agents = computed(() => remoteState.agents.filter(item => !item.isUser))
const panels = [{ id: 'voice', label: '声音', icon: 'volume' }, { id: 'connection', label: '连接与模型', icon: 'model' }, { id: 'bindings', label: '角色绑定', icon: 'people' }]
const hasChanges = computed(() => !!workspace.value && (JSON.stringify(workspace.value) !== baseline.value || extraText.value !== JSON.stringify(preset.value?.extra || {}, null, 2) || headersText.value !== JSON.stringify(connection.value?.headers || {}, null, 2) || customText.value !== JSON.stringify(profile.value?.custom || {}, null, 2)))
const working = computed(() => busy.value || synthesizing.value)
defineExpose({ busy: working, hasChanges })
const presetOptions = computed(() => (workspace.value?.presets || []).map(item => ({ value: item.id, label: item.name, description: adapterName(ttsChain(workspace.value, item.id).profile?.adapter) })))
const bindingOptions = computed(() => [{ value: '', label: tr('跟随默认声音') }, ...presetOptions.value])
const adapterOptions = computed(() => adapters.value.map(item => ({ value: item.id, label: adapterName(item.id) })))
const modelOptions = computed(() => (workspace.value?.models || []).map(item => ({ value: item.id, label: item.modelId || 'Edge TTS', description: workspace.value.connections.find(connection => connection.id === item.connectionId)?.name })))
const connectionOptions = computed(() => (workspace.value?.connections || []).map(item => ({ value: item.id, label: item.name })))
const supportsReference = computed(() => profile.value?.adapter === 'custom' || profile.value?.adapter === 'mimo' && model.value?.modelId.endsWith('-voiceclone'))
const supportsInstructions = computed(() => adapter.value?.instructions && !(profile.value?.adapter === 'openai' && /^tts-1(?:-hd)?$/.test(model.value?.modelId || '')))
const f = (key, type = 'text', options = {}) => ({ key, type, ...options, label: tr(options.label || key) })
const voiceFields = computed(() => [f('name', 'text', { label: '预设名称', max: 200 }), f('voice', 'text', { label: '音色编号', max: 300 }), f('format', 'text', { label: '输出格式', choices: (adapter.value?.formats || []).map(value => ({ value, label: value.toUpperCase() })) }), ...(profile.value?.adapter === 'edge_tts' ? [f('rate', 'text', { label: '语速变化', max: 30 }), f('pitch', 'text', { label: '音调变化', max: 30 })] : [f('speed', 'number', { label: '语速', min: .25, max: 4 })]), f('language', 'text', { label: '语言', max: 80 }), ...(supportsInstructions.value ? [f('instructions', 'long', { label: '声音描述与表达方式', max: 8000 })] : []), ...(adapter.value?.emotion ? [f('emotion', 'text', { label: '情绪', max: 100 })] : [])])
const connectionFields = computed(() => [f('name', 'text', { label: '连接名称', max: 200 }), ...(profile.value?.adapter === 'edge_tts' ? [] : [f('apiBase', 'text', { label: 'API 地址', max: 4000 }), f('apiKey', 'secret', { label: 'API 密钥', max: 16000 })]), f('timeoutMs', 'number', { label: '超时（毫秒）', min: 1000, max: 300000 })])
const modelFields = computed(() => [...(profile.value?.adapter === 'edge_tts' ? [] : [f('modelId', 'text', { label: '模型编号', max: 300 })]), f('maxChars', 'number', { label: '每段文字上限', min: 1, max: 100000, nullable: true })])
function adapterName(id) { return tr(ttsAdapterLabels[id] || id || '') }
function editors() { extraText.value = JSON.stringify(preset.value?.extra || {}, null, 2); headersText.value = JSON.stringify(connection.value?.headers || {}, null, 2); customText.value = JSON.stringify(profile.value?.custom || {}, null, 2); referenceName.value = '' }
function object(text) { const value = JSON.parse(text); if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error(); return value }
function commit() {
  try {
    const extra = object(extraText.value), headers = object(headersText.value), custom = profile.value?.adapter === 'custom' ? object(customText.value) : null
    if (Object.values(headers).some(value => typeof value !== 'string')) throw new Error()
    if (preset.value) preset.value.extra = extra
    if (connection.value) connection.value.headers = headers
    if (custom) profile.value.custom = custom
    for (const item of workspace.value.models) if (item.maxChars == null) delete item.maxChars
  } catch { throw new Error(tr('高级参数格式有误，请检查后再保存。')) }
}
function select(id) { try { commit(); stopPreview(); selected.value = id; editors() } catch (failure) { error.value = failure.message } }
function changeModel(id) { try { commit(); stopPreview(); preset.value.modelId = id; const formats = adapters.value.find(item => item.id === ttsChain(workspace.value, selected.value).profile?.adapter)?.formats; if (formats?.length && !formats.includes(preset.value.format)) preset.value.format = formats[0]; editors() } catch (failure) { error.value = failure.message } }
function changeConnection(id) { try { commit(); model.value.connectionId = id; editors() } catch (failure) { error.value = failure.message } }
function bind(id, value) { if (value) workspace.value.bindings[id] = value; else delete workspace.value.bindings[id] }
function add() { try { commit(); const descriptor = adapters.value.find(item => item.id === addAdapter.value); if (!descriptor) return; selected.value = addTtsPreset(workspace.value, descriptor, adapterName(descriptor.id)); editors(); adding.value = false } catch (failure) { error.value = failure.message } }
function remove() { stopPreview(); removeTtsPreset(workspace.value, selected.value); selected.value = workspace.value.defaultPresetId; editors(); removing.value = false }
async function load() { await run(async () => {
  let config, descriptions
  try { [config, descriptions] = await Promise.all([read('voice.tts_config'), read('voice.tts_adapters')]) }
  catch (failure) { if (failure.code === 'REMOTE_NOT_FOUND') throw new Error(tr('请更新 infOS 后再使用声音配置。')); throw failure }
  if (config?.version !== 1 || !['presets', 'connections', 'profiles', 'models'].every(key => Array.isArray(config[key])) || !config.bindings || !Array.isArray(descriptions)) throw new Error(tr('请更新 infOS 后再使用声音配置。'))
  workspace.value = config; adapters.value = descriptions; baseline.value = JSON.stringify(config); selected.value = config.presets.some(item => item.id === selected.value) ? selected.value : config.defaultPresetId; addAdapter.value = descriptions[0]?.id || ''; editors()
}) }
async function save() {
  let saved = false
  await run(async () => { commit(); const result = await write('voice.save_tts_config', { body: workspace.value }); stopPreview(); workspace.value = result; baseline.value = JSON.stringify(result); editors(); appMessage.success(tr('声音配置已保存')); saved = true })
  return saved
}
function stopPreview() { stopRemoteSpeech(previewId) }
async function preview() {
  if (previewActive.value && ['loading', 'playing'].includes(speech.status)) { stopPreview(); return }
  if (hasChanges.value && !await save()) return
  try { if (previewActive.value && speech.status === 'ready') await replayRemoteSpeech(); else await startRemoteSpeech({ text: testText.value.trim() || tr('你好，很高兴与你见面。'), presetId: selected.value }, previewId) }
  catch (failure) { if (failure.code !== 'CANCELLED' && failure.code !== 'MODE_INACTIVE') error.value = tr(failure.message); if (failure.outcome === 'unknown') { fail(failure); await refreshRemotePendingRequests().catch(() => {}) } }
}
async function uploadReference(event) {
  const file = event.target.files?.[0]; event.target.value = ''; if (!file || locked.value || !preset.value) return
  const target = preset.value
  await run(async () => { if (!file.size || file.size > 7 * 1024 * 1024) throw new Error(tr('请选择不超过 7MB 的 WAV 或 MP3 参考音频')); const bytes = Array.from(new Uint8Array(await file.arrayBuffer())); guard(); try { const result = await remoteClient.manageUpload('tts_reference', null, { name: file.name, mimeType: file.type || 'application/octet-stream', bytes }, { token }); guard(); target.referenceAssetId = result.data.assetId; referenceName.value = result.data.name || file.name } catch (failure) { if (failure.outcome === 'unknown') fail(failure); throw failure } finally { await refreshRemotePendingRequests().catch(() => {}) } })
}
onMounted(load)
onBeforeUnmount(() => { stopPreview(); workspace.value = null; extraText.value = ''; headersText.value = ''; customText.value = '' })
</script>
