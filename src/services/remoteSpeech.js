import { reactive } from 'vue'
import { remoteClient } from './remote'
import { currentToken, assertToken, onModeInvalidated } from '../stores/mode'

export const remoteSpeech = reactive({ id: '', status: 'idle', url: '', mimeType: '', error: '' })
let current = null, revision = 0, observing = false
export function stopRemoteSpeech(id) {
  if (id && id !== remoteSpeech.id) return
  revision++
  const previous = current; current = null
  if (previous) {
    try { previous.source?.stop() } catch { /* 已结束的音源不影响清理其他播放资源。 */ }
    previous.audio?.pause(); previous.context?.close().catch(() => {})
    if (previous.pending) remoteClient.cancelTts(previous.requestId, previous.token).catch(() => {})
  }
  if (remoteSpeech.url) URL.revokeObjectURL(remoteSpeech.url)
  Object.assign(remoteSpeech, { id: '', status: 'idle', url: '', mimeType: '', error: '' })
}
onModeInvalidated(() => stopRemoteSpeech())
export async function replayRemoteSpeech() {
  const entry = current, request = revision
  if (!entry || !remoteSpeech.url) return
  assertToken(entry.token, 'remote')
  if (entry.context && entry.buffer) {
    await entry.context.resume()
    if (request !== revision) return
    const source = entry.context.createBufferSource(); entry.source = source
    source.buffer = entry.buffer; source.connect(entry.context.destination)
    source.onended = () => { if (request === revision) remoteSpeech.status = 'ready' }
    source.start()
  } else {
    entry.audio ||= new Audio(remoteSpeech.url)
    entry.audio.onended = () => { if (request === revision) remoteSpeech.status = 'ready' }
    await entry.audio.play()
    if (request !== revision) { entry.audio.pause(); return }
  }
  remoteSpeech.status = 'playing'
}
export async function startRemoteSpeech(input, id) {
  stopRemoteSpeech()
  const token = currentToken(); assertToken(token, 'remote')
  const request = revision, entry = { token, requestId: crypto.randomUUID(), pending: false, context: null, source: null, audio: null, buffer: null }
  current = entry
  Object.assign(remoteSpeech, { id, status: 'loading', error: '' })
  if (!observing) { observing = true; document.addEventListener('visibilitychange', () => { if (document.hidden) stopRemoteSpeech() }) }
  try {
    const AudioContext = window.AudioContext || window.webkitAudioContext
    if (AudioContext) { entry.context = new AudioContext(); await entry.context.resume() }
    assertToken(token, 'remote'); if (request !== revision) return
    entry.pending = true
    const result = await remoteClient.tts(input, { requestId: entry.requestId, token })
    entry.pending = false
    assertToken(token, 'remote'); if (request !== revision) return
    const data = result.data, bytes = data.bytes
    remoteSpeech.url = URL.createObjectURL(new Blob([bytes], { type: data.mimeType })); remoteSpeech.mimeType = data.mimeType
    if (entry.context) {
      try { entry.buffer = await entry.context.decodeAudioData(bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength)) }
      catch { await entry.context.close(); entry.context = null }
    }
    if (request !== revision) return
    remoteSpeech.status = 'ready'
    // 系统阻止自动播放时保留音频，用户再次点播放即可，不重复调用合成。
    try { await replayRemoteSpeech() }
    catch (failure) { if (request === revision && failure.name !== 'NotAllowedError') throw failure }
  } catch (failure) {
    entry.pending = false
    if (request === revision) { remoteSpeech.status = remoteSpeech.url ? 'ready' : 'idle'; remoteSpeech.error = failure.message; throw failure }
  }
}
