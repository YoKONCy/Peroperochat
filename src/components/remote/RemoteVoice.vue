<template>
  <div><p v-if="error" class="room-error">{{ error }}</p><p v-if="notice" class="remote-notice">{{ notice }}</p><p v-if="unresolved" class="room-error">上次请求结果待核对。<button @click="$emit('pending')">去核对</button></p><RemoteValue v-if="status" :value="status" />
    <div class="remote-manager-card"><h2>语音识别</h2><p class="remote-help">录音或上传音频，由当前 infOS 的识别服务处理。</p><div class="remote-manager-actions"><button :disabled="busy || unresolved" @click="recording ? stopRecording() : startRecording()">{{ recording ? '结束录音并识别' : '开始录音' }}</button><button :disabled="busy || recording || unresolved" @click="audioInput.click()">上传音频识别</button><input ref="audioInput" type="file" accept="audio/*" hidden @change="importAudio" /></div><textarea v-if="transcript" v-model="transcript" class="remote-code-editor" rows="4" aria-label="识别结果" /><button v-if="transcript" class="room-text-button" @click="copy">复制识别文字</button></div>
    <div class="remote-manager-card"><h2>文字朗读</h2><RemoteFields v-model="ttsInput" :fields="ttsFields" :disabled="busy" /><button class="room-primary-button" :disabled="busy || unresolved || !ttsInput.text.trim()" @click="synthesize">生成语音</button><audio v-if="audioUrl" ref="player" :src="audioUrl" controls preload="metadata" /></div>
    <details><summary>服务端语音设置</summary><p class="remote-help">这里调整当前 infOS 的语音服务。唤醒词设置属于服务端，不会让手机后台持续录音。</p><form @submit.prevent="save"><RemoteFields v-model="settings" :fields="configFields(settings)" :disabled="busy" /><button class="room-primary-button" :disabled="busy || unresolved">保存语音设置</button></form></details>
  </div>
</template>
<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { field, configFields } from '../../services/remoteManagement'
import { remoteClient } from '../../services/remote'
import { refreshRemotePendingRequests, remoteState } from '../../stores/remote'
import RemoteFields from './RemoteFields.vue'
import RemoteValue from './RemoteValue.vue'
defineEmits(['pending'])
const { token, read, write, run, guard, busy, error, unresolved, fail } = useRemoteManagement()
const status = ref(null), settings = ref({}), notice = ref(''), recording = ref(false), transcript = ref(''), audioUrl = ref(null), player = ref(null), audioInput = ref(null)
const ttsInput = ref({ text: '', voice: '', speed: 1 }), ttsFields = [field('text', 'long', { max: 4096 }), field('voice', 'text', { help: '留空使用 infOS 默认音色' }), field('speed', 'number', { min: .25, max: 4 })]
const defaults = { 'tts.enabled': 'true', 'tts.provider': 'edge', 'tts.voice': 'zh-CN-XiaoxiaoNeural', 'tts.speed': '1', 'tts.pitch': '1', 'tts.rate': '+0%', 'tts.edgePitch': '+0Hz', 'tts.apiBase': '', 'tts.apiKey': '', 'asr.enabled': 'false', 'asr.apiBase': '', 'asr.apiKey': '', 'asr.model': 'whisper-1', 'asr.language': 'zh', 'asr.wakeWord': '', 'asr.sensitivity': '0.5' }
let recorder = null, stream = null, chunks = [], stopTimer = null, disposed = false, recordingError = false, recordingBytes = 0
const baseline = ref('')
const hasChanges = computed(() => recording.value || !!baseline.value && JSON.stringify(settings.value) !== baseline.value)
const working = computed(() => busy.value || recording.value)
defineExpose({ busy: working, hasChanges })
function releaseStream() { clearTimeout(stopTimer); stream?.getTracks().forEach(track => track.stop()); stream = null; recording.value = false }
async function load() { const values = await Promise.all([read('voice.status'), read('configs.batch', { body: { keys: Object.keys(defaults) } })]); status.value = values[0]; settings.value = Object.fromEntries(Object.entries(defaults).map(([key, value]) => [key, values[1]?.[key] ?? value])); baseline.value = JSON.stringify(settings.value) }
async function save() { await run(async () => { await write('configs.save_batch', { body: { items: Object.entries(settings.value).map(([key, value]) => ({ key, value: String(value) })) } }); notice.value = '语音设置已保存'; await load() }) }
async function mediaRequest(callback) { try { const result = await callback(); guard(); return result.data } catch (failure) { fail(failure); throw failure } finally { await refreshRemotePendingRequests().catch(failure => { try { guard(); remoteState.pendingError = failure.message } catch { /* 页面关闭后不更新新连接。 */ } }) } }
async function recognize(blob, name = 'recording.webm') { await run(async () => { if (!blob.size || blob.size > 8 * 1024 * 1024) throw new Error('音频为空或超过 8 MiB，请缩短录音'); const bytes = Array.from(new Uint8Array(await blob.arrayBuffer())); guard(); const data = await mediaRequest(() => remoteClient.manageUpload('asr', null, { name, mimeType: blob.type || 'audio/webm', bytes }, { fields: settings.value['asr.language'] ? { language: settings.value['asr.language'] } : {}, token })); transcript.value = data.text || '' }) }
async function startRecording() { await run(async () => { if (!navigator.mediaDevices?.getUserMedia || typeof MediaRecorder === 'undefined') throw new Error('当前设备不支持网页录音，可使用上传音频识别'); stream = await navigator.mediaDevices.getUserMedia({ audio: true }); try { guard(); chunks = []; recordingBytes = 0; recordingError = false; const mimeType = ['audio/webm;codecs=opus', 'audio/mp4', 'audio/webm'].find(type => MediaRecorder.isTypeSupported(type)); recorder = new MediaRecorder(stream, mimeType ? { mimeType } : {}); recorder.ondataavailable = event => { if (event.data.size) { chunks.push(event.data); recordingBytes += event.data.size; if (recordingBytes > 8 * 1024 * 1024) { recordingError = true; error.value = '录音已超过 8 MiB，请缩短后重录'; stopRecording() } } }; recorder.onerror = () => { recordingError = true; error.value = '录音中断，请重新录制或上传音频'; releaseStream() }; recorder.onstop = () => { const blob = new Blob(chunks, { type: recorder.mimeType || 'audio/webm' }); chunks = []; releaseStream(); if (!disposed && !recordingError) recognize(blob, blob.type.includes('mp4') ? 'recording.m4a' : 'recording.webm') }; recorder.start(1000); recording.value = true; stopTimer = setTimeout(stopRecording, 120000) } catch (failure) { releaseStream(); throw failure } }) }
function stopRecording() { if (recorder && recorder.state !== 'inactive') recorder.stop(); releaseStream() }
async function importAudio(event) { const file = event.target.files?.[0]; event.target.value = ''; if (file) await recognize(file, file.name) }
async function synthesize() { await run(async () => { const data = await mediaRequest(() => remoteClient.tts({ text: ttsInput.value.text, voice: ttsInput.value.voice || null, speed: ttsInput.value.speed }, { token })); if (audioUrl.value) URL.revokeObjectURL(audioUrl.value); audioUrl.value = URL.createObjectURL(new Blob([new Uint8Array(data.bytes)], { type: data.mimeType })) }) }
async function copy() { try { await navigator.clipboard.writeText(transcript.value); notice.value = '已复制识别文字' } catch { error.value = '复制未完成，可以长按选择文字' } }
function visibility() { if (document.hidden && recording.value) { recordingError = true; stopRecording(); error.value = '录音因页面进入后台而取消' } if (document.hidden) player.value?.pause() }
onMounted(() => { run(load); document.addEventListener('visibilitychange', visibility) })
onBeforeUnmount(() => { disposed = true; recordingError = true; stopRecording(); player.value?.pause(); if (audioUrl.value) URL.revokeObjectURL(audioUrl.value); document.removeEventListener('visibilitychange', visibility) })
</script>
