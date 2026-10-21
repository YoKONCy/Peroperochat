// 页面层级与字段分区只影响展示，不推断服务端权限或修改请求语义。
export const managementAreas = [
  { id: 'daily', label: '常用配置', icon: 'settings', description: '连接、模型与角色', ids: ['providers', 'models', 'assignments', 'agents', 'user', 'voice'] },
  { id: 'content', label: '内容与资料', icon: 'memory', description: '提示词、记忆与知识', ids: ['prompts', 'memories', 'facts', 'documents', 'assets', 'logs'] },
  { id: 'work', label: '工作台', icon: 'folder', description: '文件、执行与陪伴安排', ids: ['workbench', 'task-management', 'rooms', 'scheduler', 'proactive', 'tarot'] },
  { id: 'advanced', label: '高级管理', icon: 'shield', description: '扩展、实例与维护', ids: ['mcp', 'skills', 'tools', 'integrations', 'inbound', 'access', 'distributed', 'usage', 'runtime', 'maintenance', 'config', 'reset'] },
]
export const sectionDescriptions = {
  providers: '维护连接地址与凭据，供多个模型共用。', models: '选择连接、声明能力并配置生成参数。', assignments: '安排全局、角色与视觉转述使用的模型。', agents: '编辑身份、公开资料、人设与互动方式。', user: '你的个人资料与 infOS 服务端偏好。', voice: '配置识别与朗读服务，或进行单次试听。',
  prompts: '编辑草稿、检查编译结果，再发布到服务端。', memories: '按内容查找记忆，查看来源与归档状态。', facts: '查看事实来源，处理不再适用的记录。', documents: '导入知识资料、查看解析结果与检索正文。', assets: '查看资源索引与模板。', logs: '查阅历史会话，回到原会话继续操作。',
  'task-management': '查看执行状态与关联会话，管理后台任务。', scheduler: '安排时间、执行内容与重复规则。', proactive: '调整主动陪伴方式、频率与每日上限。', access: '管理当前实例的手机连接与配对设备。', maintenance: '查看索引状态并执行维护任务。', config: '原始配置与迁移工具，日常设置优先使用专用页面。', reset: '清空与重置操作会影响服务端数据，请谨慎使用。',
}
export function actionPresentation(action) {
  const op = action.op || ''
  if (/delete|reset|factory|clear_flow|retract|cancel|kill|rollback/.test(op)) return { icon: /reset|rollback/.test(op) ? 'refresh' : 'trash', tone: 'danger', category: 'danger' }
  if (action.read) return { icon: /export/.test(op) ? 'download' : /search/.test(op) ? 'search' : /status|usage|info/.test(op) ? 'tasks' : 'open', tone: 'quiet', category: 'read' }
  return { icon: /save|publish|accept/.test(op) ? 'check' : /create|import|invite/.test(op) ? 'plus' : /connect|probe/.test(op) ? 'connection' : /retry|reload|reindex|restore/.test(op) ? 'refresh' : 'settings', tone: 'secondary', category: 'write' }
}
export function formSections(fields, kind) {
  const definitions = kind === 'agents' ? [
    ['identity', '身份与称呼', 'profile', ['name', 'description', 'ownerAppellation', 'publicProfile'], true],
    ['persona', '人设与频道提示', 'memory', ['systemPrompt', 'channelPatches'], true],
    ['interaction', '互动与台词', 'chat', ['waifuTexts', 'isEnabled'], false],
  ] : ['models', 'providers'].includes(kind) ? [
    ['identity', kind === 'models' ? '模型身份' : '连接身份', 'model', ['name', 'modelId', 'brand', 'provider', 'isDefault'], true],
    ['connection', '连接与认证', 'connection', ['providerConnectionId', 'apiBase', 'apiKey', 'protocol', 'authMode', 'connectionOverride', 'connectionProtocol', 'connectionAuthMode'], true],
    ['capabilities', '输入与响应能力', 'image', ['enableVision', 'enableAudioInput', 'returnNativeReasoning', 'stream', 'wireApi', 'reasoningDialect'], false],
    ['generation', '生成参数', 'sliders', ['reasoningEffort', 'temperature', 'topP', 'maxTokens', 'contextWindowTokens', 'maxReActTurns'], false],
  ] : kind === 'voice' ? [['asr', '识别服务', 'audio', fields.filter(field => field.key.startsWith('asr.') && !/wakeWord|sensitivity/.test(field.key)).map(field => field.key), true], ['tts', '朗读服务', 'volume', fields.filter(field => field.key.startsWith('tts.')).map(field => field.key), true], ['wake', '服务端唤醒', 'bell', ['asr.wakeWord', 'asr.sensitivity'], false]]
    : kind === 'mcp' ? [['connection', '服务连接', 'connection', ['name', 'type', 'command', 'url'], true], ['runtime', '运行参数', 'terminal', ['args', 'env', 'enabled'], false]]
    : kind === 'proactive' ? [['policy', '陪伴方式', 'heart', ['mode', 'goal'], true], ['schedule', '时间与频率', 'bell', ['timeZone', 'companionIntervalMinutes', 'autonomousIntervalMinutes', 'maxDailyRuns'], true]]
      : kind === 'user' ? [['identity', '关于你', 'profile', ['owner.name', 'owner.persona'], true], ['preferences', '服务端偏好', 'settings', ['user.language', 'ui.fontSize', 'ui.motionLevel', 'ui.soundEffects', 'system.logLevel', 'system.autoSave'], false]]
        : [['main', '内容与设置', 'edit', fields.filter(field => !/apiKey|token|secret|scope|revision|hash|id$/i.test(field.key) || field.required).map(field => field.key), true]]
  const used = new Set(), groups = []
  for (const [id, title, icon, keys, primary] of definitions) { const items = fields.filter(field => keys.includes(field.key)); if (items.length) { items.forEach(item => used.add(item.key)); groups.push({ id, title, icon, fields: items, primary }) } }
  const remaining = fields.filter(field => !used.has(field.key)); if (remaining.length) groups.push({ id: 'advanced', title: '更多设置', icon: 'settings', fields: remaining, primary: remaining.some(field => field.required) })
  return groups
}
