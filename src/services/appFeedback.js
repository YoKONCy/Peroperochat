import { shallowRef } from 'vue'

export const activeFeedback = shallowRef(null)
export const feedbackSurface = shallowRef(null)
const pending = [], recent = new Map(), surfaces = []
let sequence = 0, timer = null, deadline = 0, remaining = 0
const holds = new Set()

function stopTimer() { clearTimeout(timer); timer = null }
function schedule() {
  stopTimer()
  if (!activeFeedback.value || holds.size) return
  deadline = Date.now() + remaining
  const id = activeFeedback.value.id
  timer = setTimeout(() => closeFeedback(id, true), remaining)
}
function advance() {
  if (activeFeedback.value) return
  // 过时的小提醒直接略过，避免操作结束很久后才陆续冒出来。
  while (pending.length && Date.now() - pending[0].createdAt > 8000) pending.shift()
  if (!pending.length) return
  const next = pending.shift()
  activeFeedback.value = next
  recent.set(next.key, Date.now())
  remaining = next.duration
  schedule()
}
export function holdFeedback(reason) {
  if (!holds.size && timer !== null) remaining = Math.max(0, deadline - Date.now())
  holds.add(reason); stopTimer()
}
export function resumeFeedback(reason) { if (holds.delete(reason)) schedule() }
export function closeFeedback(id, notify = false, allowRepeat = false) {
  if (id == null) return
  const index = pending.findIndex(item => item.id === id)
  if (index !== -1) { pending.splice(index, 1); return }
  const current = activeFeedback.value
  if (current?.id !== id) return
  if (allowRepeat) recent.delete(current.key)
  stopTimer(); activeFeedback.value = null
  holds.delete('pointer'); holds.delete('focus')
  try { if (notify) current.onDismiss?.() } finally { advance() }
}
export function clearFeedback() {
  stopTimer(); activeFeedback.value = null; pending.length = 0; recent.clear()
  holds.delete('pointer'); holds.delete('focus')
}

export function showFeedback(options) {
  const message = String(options.message || '').trim()
  if (!message) return { close() {} }
  const tone = ['error', 'warning', 'success', 'info'].includes(options.tone) ? options.tone : 'info'
  const key = `${tone}:${message}`
  const now = Date.now(), cooldown = tone === 'error' || tone === 'warning' ? 30000 : 1800
  for (const [entry, seen] of recent) if (now - seen > 30000) recent.delete(entry)
  // 重复刷新不续期、不堆叠；普通页面提示只保留少量等待项。
  if (activeFeedback.value?.key === key || pending.some(item => item.key === key) || now - (recent.get(key) ?? -Infinity) < cooldown) return { close() {} }
  const requested = Number(options.duration)
  const duration = Math.min(8000, Math.max(2200, Number.isFinite(requested) && requested > 0 ? requested : tone === 'success' ? 2800 : 4800))
  const item = { ...options, message, tone, key, id: ++sequence, duration, createdAt: now }
  if (pending.length >= 3) {
    const replace = pending.findIndex(entry => entry.tone === 'info' || entry.tone === 'success')
    if (replace === -1 && (tone === 'info' || tone === 'success')) return { close() {} }
    pending.splice(replace === -1 ? 0 : replace, 1)
  }
  pending.push(item); advance()
  return { close: () => closeFeedback(item.id) }
}

// 与旧页面的调用参数兼容，所有普通消息都经由同一个轻气泡层显示。
export const appMessage = Object.fromEntries(['success', 'info', 'warning', 'error'].map(tone => [tone, value => showFeedback({ ...(typeof value === 'string' ? { message: value } : value), tone })]))

// 抽屉内的气泡仍属于当前对话框，键盘导航和读屏可以访问重试、关闭按钮。
export function registerFeedbackSurface(element) {
  surfaces.push(element); feedbackSurface.value = element
  return () => {
    const index = surfaces.indexOf(element)
    if (index !== -1) surfaces.splice(index, 1)
    feedbackSurface.value = surfaces.at(-1) || null
  }
}
