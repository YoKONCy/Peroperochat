export const WHOLESOME_MODE_KEY = 'system.wholesomeMode'
export const ASSISTANT_PREFILL_KEY = 'system.assistantPrefill'
export const ANSWER_PREFERENCE_KEYS = [WHOLESOME_MODE_KEY, ASSISTANT_PREFILL_KEY]
export const MAX_ASSISTANT_PREFILL_LENGTH = 8000

// 与 infOS 的默认值一致；无效旧配置不能自动开启预填充，正文不裁剪空白或标签。
export function parseAnswerPreferences(values = {}) {
  let prefill = { enabled: false, content: '', mode: 'once' }
  try {
    const value = JSON.parse(values[ASSISTANT_PREFILL_KEY] ?? '')
    if (value && !Array.isArray(value) && typeof value.enabled === 'boolean' && typeof value.content === 'string' && value.content.length <= MAX_ASSISTANT_PREFILL_LENGTH && (value.mode === undefined || ['once', 'continuous'].includes(value.mode))) prefill = { enabled: value.enabled, content: value.content, mode: value.mode ?? 'once' }
  } catch { /* 尚未配置时保持关闭。 */ }
  return { wholesome: values[WHOLESOME_MODE_KEY] !== 'false', prefill }
}
export function changedAnswerPreferences(current, saved) {
  const items = []
  if (current.wholesome !== saved.wholesome) items.push({ key: WHOLESOME_MODE_KEY, value: String(current.wholesome) })
  if (JSON.stringify(current.prefill) !== JSON.stringify(saved.prefill)) items.push({ key: ASSISTANT_PREFILL_KEY, value: JSON.stringify(current.prefill) })
  return items
}
