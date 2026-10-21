<template>
  <RoomDrawer title="语音输入" subtitle="转成文字，确认后放入输入框" :busy="busy" :before-close="beforeClose" @close="$emit('close')">
    <RemoteFeedback :message="error" /><RemoteAction v-if="status === null && !busy" icon="refresh" @click="loadStatus">重新读取语音服务</RemoteAction><p v-if="status === false" class="remote-notice">当前 infOS 尚未配置语音识别。<button @click="$emit('configure')">前往语音设置</button></p>
    <div class="remote-voice-capture" :class="{ recording }"><RemoteIcon name="audio" /><strong>{{ recording ? '正在听…' : busy ? '正在识别…' : '说完后，先看看文字' }}</strong><span>{{ recording ? '结束录音后开始转写' : '不会自动发送消息' }}</span></div>
    <div class="remote-purpose-actions"><RemoteAction :icon="recording ? 'stop' : 'audio'" :tone="recording ? 'danger' : 'primary'" :disabled="busy || !recording && (blocked || unknown || status !== true)" @click="recording ? stop() : start()">{{ recording ? '结束并转写' : '开始录音' }}</RemoteAction><RemoteAction icon="folder" :disabled="busy || recording || blocked || unknown || status !== true" @click="input.click()">选择音频</RemoteAction><input ref="input" type="file" accept="audio/*" hidden @change="upload" /></div>
    <label v-if="text" class="remote-field">识别文字<textarea v-model="text" rows="5" maxlength="131072" /></label><RemoteAction v-if="text" icon="send" tone="primary" :disabled="busy || recording || !text.trim()" @click="$emit('transcript', text)">加入输入框</RemoteAction>
    <button v-if="unknown" class="room-text-button" @click="$emit('pending')">识别请求结果待核对</button>
    <RoomDrawer v-if="discarding" title="丢弃识别文字？" @close="discarding = false"><p>文字还没有加入输入框。</p><RemoteAction icon="close" tone="danger" @click="$emit('close')">丢弃并关闭</RemoteAction></RoomDrawer>
  </RoomDrawer>
</template>
<script setup>
import RemoteFeedback from './RemoteFeedback.vue'
import { ref, onMounted, onBeforeUnmount } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteIcon from './RemoteIcon.vue'
import { remoteClient } from '../../services/remote'
import { refreshRemotePendingRequests } from '../../stores/remote'
import { currentToken, assertToken } from '../../stores/mode'
defineProps({ blocked: Boolean })
defineEmits(['close', 'transcript', 'configure', 'pending'])
const token = currentToken(), text = ref(''), error = ref(''), busy = ref(false), recording = ref(false), status = ref(null), input = ref(null), unknown = ref(false)
const discarding = ref(false)
function beforeClose() { if (text.value.trim()) { discarding.value = true; return false } return true }
let recorder, stream, timer, chunks = [], bytes = 0, cancelled = false, disposed = false
function release() { clearTimeout(timer); stream?.getTracks().forEach(track => track.stop()); stream = null; recording.value = false }
async function recognize(blob, name) {
  busy.value = true; error.value = ''
  try {
    if (!blob.size || blob.size > 8 * 1024 * 1024) throw new Error('请选择不超过 8 MiB 的音频')
    const data = Array.from(new Uint8Array(await blob.arrayBuffer())); assertToken(token, 'remote'); if (disposed) return
    const result = await remoteClient.manageUpload('asr', null, { name, mimeType: blob.type || 'audio/webm', bytes: data }, { token })
    if (!disposed) text.value = result.data?.text || ''
  } catch (failure) { if (!disposed) { error.value = failure.message; unknown.value = !!failure.requestId && !['not_sent', 'rejected', 'failed'].includes(failure.outcome) } }
  finally { if (!disposed) busy.value = false; await refreshRemotePendingRequests().catch(() => {}) }
}
async function start() {
  if (unknown.value || recording.value || busy.value) return
  try {
    assertToken(token, 'remote'); error.value = ''; busy.value = true; cancelled = false
    if (!navigator.mediaDevices?.getUserMedia || typeof MediaRecorder === 'undefined') throw new Error('此设备暂不支持录音，可选择音频文件')
    stream = await navigator.mediaDevices.getUserMedia({ audio: true }); assertToken(token, 'remote'); if (disposed) { release(); return }
    const type = ['audio/webm;codecs=opus', 'audio/mp4', 'audio/webm'].find(mime => MediaRecorder.isTypeSupported(mime))
    recorder = new MediaRecorder(stream, type ? { mimeType: type } : {}); chunks = []; bytes = 0
    recorder.ondataavailable = event => { if (event.data.size) { chunks.push(event.data); bytes += event.data.size; if (bytes > 8 * 1024 * 1024) { cancelled = true; error.value = '录音过长，请缩短后重录'; stop() } } }
    recorder.onerror = () => { cancelled = true; error.value = '录音中断，请重试'; stop() }
    recorder.onstop = () => { const blob = new Blob(chunks, { type: recorder.mimeType }); chunks = []; release(); if (!disposed && !cancelled) recognize(blob, blob.type.includes('mp4') ? 'recording.m4a' : 'recording.webm') }
    recorder.start(1000); recording.value = true; timer = setTimeout(stop, 120000)
  } catch (failure) { release(); if (!disposed) error.value = failure.message }
  finally { if (!disposed) busy.value = false }
}
function stop() { if (recorder?.state !== 'inactive') recorder?.stop(); release() }
function upload(event) { const file = event.target.files?.[0]; event.target.value = ''; if (file && !unknown.value) recognize(file, file.name) }
function visibility() { if (document.hidden) { cancelled = true; stop() } }
async function loadStatus() { try { const result = await remoteClient.manage({ operation: 'voice.status', id: null, params: {}, body: null }, { token }); if (!disposed) status.value = result.data?.asr?.available === true } catch (failure) { if (!disposed) error.value = failure.message } }
onMounted(() => { document.addEventListener('visibilitychange', visibility); loadStatus() })
onBeforeUnmount(() => { disposed = true; cancelled = true; stop(); document.removeEventListener('visibilitychange', visibility) })
</script>
