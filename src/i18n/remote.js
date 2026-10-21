import { ref } from 'vue'

// 与 infOS 使用相同的语言编号、中文查询键和编号占位符；词库按用途拆分。
const resources = import.meta.glob('../locales/remote/*/*.json', { import: 'default' })
const catalogs = {}
const loading = new Map()
let localeRequest = 0
export const remoteLocale = ref('zh-CN')
const catalogVersion = ref(0)
export function normalizeRemoteLocale(value) {
  const name = String(value || '').toLowerCase().replaceAll('_', '-')
  return /^zh-(tw|hk|mo|hant)/.test(name) ? 'zh-TW' : name.startsWith('en') ? 'en-US' : 'zh-CN'
}
export function setRemoteLocale(value) {
  const language = normalizeRemoteLocale(value), request = ++localeRequest
  // 简中直接使用查询原文，其他语言按需装载，避免初屏解析三套完整词库。
  if (language === 'zh-CN' || catalogs[language]) { remoteLocale.value = language; return }
  if (!loading.has(language)) loading.set(language, Promise.all(Object.entries(resources).filter(([path]) => path.includes(`/${language}/`)).map(async ([path, read]) => [path.split('/').at(-1).replace('.json', ''), await read()])).then(entries => { catalogs[language] = Object.fromEntries(entries); catalogVersion.value++ }).catch(() => { loading.delete(language) }))
  loading.get(language).then(() => { if (request === localeRequest && catalogs[language]) remoteLocale.value = language })
}
export function t(source, parameters = [], scope = 'workspace') {
  source = String(source ?? '')
  void catalogVersion.value
  const language = catalogs[remoteLocale.value] || {}
  const text = language[scope]?.[source] ?? ['workspace', 'feedback', 'settings', 'stronghold', 'usage', 'common', 'navigation'].map(name => language[name]?.[source]).find(value => value !== undefined) ?? source
  return text.replace(/\{(\d+)\}/g, (token, index) => parameters[Number(index)] == null ? token : String(parameters[Number(index)]))
}
const choices = {
  off: '关闭', companion: '智能陪伴', autonomous: '自主行动', full: '标准', reduced: '减少动态',
  today: '今天', '7d': '近 7 天', '30d': '近 30 天', low: '低', medium: '中', high: '高', xhigh: '极高', max: '最高',
  queued: '排队中', running: '执行中', paused: '已暂停', waiting_input: '等待回复', waiting: '等待中', blocked: '受阻', completed: '已完成', failed: '失败', cancelled: '已取消',
  active: '使用中', archived: '已归档', background: '后台整理', eventAt: '发生时间', createdAt: '创建时间', importance: '重要度', desc: '从新到旧', asc: '从旧到新',
  text: '文本', url: '网页', auto: '自动', native: '原生视觉', relay: '图片转述', api_key: 'API 密钥', none: '无需密钥',
  chat_completions: 'Chat Completions', responses: 'Responses', standard: '标准', detailed: '详细', brief: '简洁',
  zh: '中文', 'zh-CN': '简体中文', 'zh-TW': '繁體中文', 'en-US': 'English',
  pending: '待执行', dispatching: '提交中', dispatched: '已提交', component: '组件', template: '模板', preset: '预设', system: '系统', user: '用户', assistant: '角色', desktop: '单聊', group: '群聊', social: '社交频道',
  custom: '自定义', generic: '通用', openai: 'OpenAI', anthropic: 'Anthropic', gemini: 'Gemini', deepseek: 'DeepSeek', openrouter: 'OpenRouter',
}
export function remoteChoiceLabel(value) { return choices[value] ? t(choices[value]) : String(value ?? '') }
export function remoteNumber(value, options = {}) { return value != null && Number.isFinite(Number(value)) ? Number(value).toLocaleString(remoteLocale.value, options) : '—' }
export function remoteDate(value, options = {}) { const date = new Date(value); return value && Number.isFinite(date.getTime()) ? date.toLocaleString(remoteLocale.value, options) : '—' }
