<template>
  <div><RemoteFeedback :message="error" /><RemoteFeedback :message="notice" tone="success" /><RemoteAction v-if="unresolved" icon="alert" tone="danger" @click="$emit('pending')">上次请求结果待核对</RemoteAction>
    <div v-if="status" class="remote-service-status"><article><h3><RemoteIcon name="audio" />语音识别</h3><p>{{ status.asr?.available ? '可用' : '尚不可用' }}</p></article><article><h3><RemoteIcon name="volume" />文字朗读</h3><p>{{ tr('由声音预设决定') }}</p></article></div>
    <nav class="remote-subnav" aria-label="语音服务"><button v-for="item in panels" :key="item.id" :class="{ selected: panel === item.id }" :disabled="recording || busy || ttsSettings?.busy" @click="panel = item.id"><RemoteIcon :name="item.icon" />{{ tr(item.label) }}</button></nav>
    <section v-if="panel === 'asr'" class="remote-manager-card"><h2>{{ recording ? '正在录音…' : '语音识别' }}</h2><p class="remote-help">录音或上传音频，由当前 infOS 识别。不会自动发送消息。</p><div class="remote-purpose-actions"><RemoteAction :icon="recording ? 'stop' : 'audio'" :tone="recording ? 'danger' : 'primary'" :disabled="busy || unresolved || !status?.asr?.available" @click="recording ? stopRecording() : startRecording()">{{ recording ? '结束并识别' : '开始录音' }}</RemoteAction><RemoteAction icon="attach" :disabled="busy || recording || unresolved || !status?.asr?.available" @click="audioInput.click()">上传音频</RemoteAction><input ref="audioInput" type="file" accept="audio/*" hidden @change="importAudio" /></div><textarea v-if="transcript" v-model="transcript" class="remote-code-editor" rows="4" aria-label="识别结果" /><RemoteAction v-if="transcript" icon="copy" @click="copy">复制文字</RemoteAction></section>
    <RemoteTtsSettings ref="ttsSettings" v-show="panel === 'tts'" @pending="$emit('pending')" />
    <section v-if="panel === 'settings'"><p class="remote-help">配置当前 infOS 的识别服务。唤醒词属于服务端设置，不会让手机后台持续录音。</p><form @submit.prevent="save"><RemoteFormSections v-model="settings" kind="voice" :fields="settingsFields" :disabled="busy" /><footer class="remote-edit-footer"><span>{{ asrHasChanges ? '有未保存的修改' : '当前服务端设置' }}</span><RemoteAction icon="check" tone="primary" type="submit" :disabled="busy || unresolved || !asrHasChanges">保存语音设置</RemoteAction></footer></form></section>
    <RemoteAction icon="refresh" tone="quiet" :disabled="busy || recording || hasChanges" @click="run(load)">刷新服务状态与设置</RemoteAction>
  </div>
</template>
<script setup>
import RemoteFeedback from './RemoteFeedback.vue'
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { configFields } from '../../services/remoteManagement'
import { remoteClient } from '../../services/remote'
import { refreshRemotePendingRequests, remoteState } from '../../stores/remote'
import RemoteTtsSettings from './RemoteTtsSettings.vue'
import { t } from '../../i18n/remote'
import RemoteAction from './RemoteAction.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteFormSections from './RemoteFormSections.vue'
defineEmits(['pending'])
const { token, read, write, run, guard, busy, error, unresolved, fail } = useRemoteManagement()
const status = ref(null), settings = ref({}), notice = ref(''), recording = ref(false), transcript = ref(''), ttsSettings = ref(null), audioInput = ref(null)
const panel = ref('asr'), panels = [{ id: 'asr', label: '识别', icon: 'audio' }, { id: 'tts', label: '朗读', icon: 'volume' }, { id: 'settings', label: '识别设置', icon: 'settings' }]
const tr = key => t(key, [], 'tts')
const settingsFields = computed(() => configFields(settings.value).map(item => /\.enabled$/.test(item.key) ? { ...item, choices: [{ value: 'true', label: '开启' }, { value: 'false', label: '关闭' }], help: undefined } : item))
const defaults = { 'asr.enabled': 'false', 'asr.apiBase': '', 'asr.apiKey': '', 'asr.model': 'whisper-1', 'asr.language': 'zh', 'asr.wakeWord': '', 'asr.sensitivity': '0.5' }
let recorder = null, stream = null, chunks = [], stopTimer = null, disposed = false, recordingError = false, recordingBytes = 0
const baseline = ref('')
const asrHasChanges = computed(() => !!baseline.value && JSON.stringify(settings.value) !== baseline.value)
const hasChanges = computed(() => recording.value || ttsSettings.value?.hasChanges || asrHasChanges.value)
const working = computed(() => busy.value || recording.value || ttsSettings.value?.busy)
defineExpose({ busy: working, hasChanges })
function releaseStream() { clearTimeout(stopTimer); stream?.getTracks().forEach(track => track.stop()); stream = null; recording.value = false }
async function load() { const values = await Promise.all([read('voice.status'), read('configs.batch', { body: { keys: Object.keys(defaults) } })]); status.value = values[0]; settings.value = Object.fromEntries(Object.entries(defaults).map(([key, value]) => [key, values[1]?.[key] ?? value])); baseline.value = JSON.stringify(settings.value) }
async function save() { await run(async () => { await write('configs.save_batch', { body: { items: Object.entries(settings.value).map(([key, value]) => ({ key, value: String(value) })) } }); notice.value = '语音设置已保存'; await load() }) }
async function mediaRequest(callback) { try { const result = await callback(); guard(); return result.data } catch (failure) { fail(failure); throw failure } finally { await refreshRemotePendingRequests().catch(failure => { try { guard(); remoteState.pendingError = failure.message } catch { /* 页面关闭后不更新新连接。 */ } }) } }
async function recognize(blob, name = 'recording.webm') { await run(async () => { if (!blob.size || blob.size > 8 * 1024 * 1024) throw new Error('音频为空或超过 8 MiB，请缩短录音'); const bytes = Array.from(new Uint8Array(await blob.arrayBuffer())); guard(); const data = await mediaRequest(() => remoteClient.manageUpload('asr', null, { name, mimeType: blob.type || 'audio/webm', bytes }, { fields: settings.value['asr.language'] ? { language: settings.value['asr.language'] } : {}, token })); transcript.value = data.text || '' }) }
async function startRecording() { await run(async () => { if (!navigator.mediaDevices?.getUserMedia || typeof MediaRecorder === 'undefined') throw new Error('当前设备不支持网页录音，可使用上传音频识别'); stream = await navigator.mediaDevices.getUserMedia({ audio: true }); try { guard(); chunks = []; recordingBytes = 0; recordingError = false; const mimeType = ['audio/webm;codecs=opus', 'audio/mp4', 'audio/webm'].find(type => MediaRecorder.isTypeSupported(type)); recorder = new MediaRecorder(stream, mimeType ? { mimeType } : {}); recorder.ondataavailable = event => { if (event.data.size) { chunks.push(event.data); recordingBytes += event.data.size; if (recordingBytes > 8 * 1024 * 1024) { recordingError = true; error.value = '录音已超过 8 MiB，请缩短后重录'; stopRecording() } } }; recorder.onerror = () => { recordingError = true; error.value = '录音中断，请重新录制或上传音频'; releaseStream() }; recorder.onstop = () => { const blob = new Blob(chunks, { type: recorder.mimeType || 'audio/webm' }); chunks = []; releaseStream(); if (!disposed && !recordingError) recognize(blob, blob.type.includes('mp4') ? 'recording.m4a' : 'recording.webm') }; recorder.start(1000); recording.value = true; stopTimer = setTimeout(stopRecording, 120000) } catch (failure) { releaseStream(); throw failure } }) }
function stopRecording() { if (recorder && recorder.state !== 'inactive') recorder.stop(); releaseStream() }
async function importAudio(event) { const file = event.target.files?.[0]; event.target.value = ''; if (file) await recognize(file, file.name) }
async function copy() { try { await navigator.clipboard.writeText(transcript.value); notice.value = '已复制识别文字' } catch { error.value = '复制未完成，可以长按选择文字' } }
function visibility() { if (document.hidden && recording.value) { recordingError = true; stopRecording(); error.value = '录音因页面进入后台而取消' } }
onMounted(() => { run(load); document.addEventListener('visibilitychange', visibility) })
onBeforeUnmount(() => { disposed = true; recordingError = true; stopRecording(); document.removeEventListener('visibilitychange', visibility) })
</script>
