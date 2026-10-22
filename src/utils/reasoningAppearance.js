export const reasoningLevels = ['off', 'low', 'medium', 'high', 'xhigh', 'max']
export function reasoningLevel(value) {
  if (value == null || value === '') return 'default'
  if (value === 'none') return 'off'
  if (value === 'minimal') return 'low'
  return reasoningLevels.includes(value) ? value : 'unknown'
}
export const reasoningLabels = { default: '不额外指定', unknown: '思考', off: '关闭', low: '低', medium: '中', high: '高', xhigh: '极高', max: '最高' }
export const reasoningHints = { default: '交给模型自行决定', unknown: '强度暂未记录', off: '直接回应', low: '轻轻想一下', medium: '多想一小步', high: '仔细想一想', xhigh: '再多推敲一点', max: '充分展开思考' }
// effective 已由 infOS 按会话、角色和默认模型解析，并包含协议档位归一化。
export function effectiveReasoning(settings) {
  return settings?.effective && Object.prototype.hasOwnProperty.call(settings.effective, 'reasoningEffort') ? reasoningLevel(settings.effective.reasoningEffort) : 'unknown'
}
export function recordedReasoning(props) {
  return props && Object.prototype.hasOwnProperty.call(props, 'reasoningEffort') ? reasoningLevel(props.reasoningEffort) : 'unknown'
}
