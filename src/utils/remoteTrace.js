import { remoteToolFallbacks } from '../constants/remoteToolCatalog'

// 这里只格式化服务端事实；不会执行参数、推测任务完成度或生成业务摘要。
export function traceText(value) {
  if (value === undefined || value === null) return ''
  return typeof value === 'string' ? value : JSON.stringify(value, null, 2)
}
export function traceJson(value) {
  if (value && typeof value === 'object') return value
  if (typeof value !== 'string' || value.length > 512 * 1024) return null
  try { return JSON.parse(value) } catch { return null }
}
export function traceDuration(value) {
  return typeof value === 'number' && Number.isFinite(value) && value >= 0 ? value < 1000 ? `${Math.round(value)} ms` : `${(value / 1000).toFixed(1)} s` : ''
}
export function traceRows(nodes) {
  const calls = new Map(), results = new Map()
  for (const node of nodes) {
    const id = node.props.callId
    if (typeof id !== 'string' || !id) continue
    const map = node.kind === 'tool-call' ? calls : node.kind === 'tool-result' ? results : null
    if (map) map.set(id, [...(map.get(id) || []), node])
  }
  const paired = new Map(), consumed = new Set()
  for (const [id, matches] of calls) {
    const outputs = results.get(id)
    // 身份重复时保留所有原始节点，避免吞掉结果或拼错调用。
    if (matches.length === 1 && outputs?.length === 1) { paired.set(matches[0], outputs[0]); consumed.add(outputs[0]) }
  }
  const rows = nodes.filter(node => !consumed.has(node)).map(node => ({ ...node, resultNode: paired.get(node), children: [] }))
  const uniqueCalls = new Map(rows.filter(row => row.kind === 'tool-call' && calls.get(row.props.callId)?.length === 1).map(row => [row.props.callId, row]))
  const grouped = new Set()
  for (const row of rows) {
    if (row.kind !== 'tool-call' || !row.props.parentCallId) continue
    const parent = uniqueCalls.get(row.props.parentCallId)
    // NIT 服务端禁止嵌套 run_script；这里只收纳明确关联到脚本的直接子调用。
    // 不根据 nit- 前缀或相邻位置推断父级，异常关系保持平铺，避免循环和丢记录。
    if (parent && parent !== row && parent.props.name === 'run_script' && !parent.props.parentCallId && row.props.name !== 'run_script' && uniqueCalls.get(row.props.callId) === row) {
      parent.children.push(row)
      grouped.add(row)
    } else {
      row.parentIssue = parent ? '父级关联异常，单独展示此调用' : '父级记录缺失或不唯一，单独展示此调用'
    }
  }
  return rows.filter(row => !grouped.has(row))
}

export function nestedTraceCounts(children = []) {
  const counts = { total: children.length, completed: 0, failed: 0, active: 0, unknown: 0 }
  for (const child of children) {
    if (child.resultNode?.props.isError || child.props.state === 'failed') counts.failed++
    else if (child.props.state === 'completed') counts.completed++
    else if (['assembling', 'calling'].includes(child.props.state)) counts.active++
    else counts.unknown++
  }
  return counts
}

const styles = {
  edit: ['edit', '文件变更', 'pink'], read: ['files', '读取', 'sky'], file: ['folder', '文件', 'sky'],
  search: ['search', '检索', 'purple'], terminal: ['terminal', '终端', 'green'], browser: ['globe', '浏览器', 'sky'],
  web: ['globe', '网页', 'sky'], script: ['code', '脚本', 'purple'], screen: ['image', '视觉', 'pink'],
  desktop: ['desktop', '桌面', 'purple'], reminder: ['clock', '提醒', 'orange'], task: ['tasks', '任务', 'green'],
  skill: ['diamond', '技能', 'purple'], social: ['chat', '社交', 'green'], stronghold: ['home', '据点', 'purple'],
  system: ['orbit', '系统', 'purple'], memory: ['memory', '记忆', 'purple'], generic: ['tool', '工具', 'purple'],
}
function fallbackStyle(name) {
  if (/^(query_facts|write_fact|supersede_fact|delete_fact|add_fact_|remove_fact_|.*event_note|search_knowledge)/.test(name)) return 'memory'
  if (/^(update_plan|manage_task|finish_task|suggest_next_step)$/.test(name)) return 'task'
  if (/^(read_image|take_screenshot|browser_screenshot|browser_page_image)$/.test(name)) return 'screen'
  if (/^(remote_)?terminal_/.test(name)) return 'terminal'
  if (name.startsWith('browser_')) return 'browser'
  if (['automation_execute', 'get_mouse_position'].includes(name)) return 'desktop'
  if (['expand_advanced_tools', 'update_state', 'update_flow_state', 'communicate_with_host', 'interact_with_social'].includes(name)) return 'system'
  const style = Object.hasOwn(remoteToolFallbacks, name) ? remoteToolFallbacks[name].style : null
  return Object.hasOwn(styles, style) ? style : 'generic'
}
export function toolPresentation(name = '', tools = []) {
  const declared = tools.find(tool => tool?.name === name)?.display || {}
  const fallback = Object.hasOwn(remoteToolFallbacks, name) ? remoteToolFallbacks[name] : {}
  const style = Object.hasOwn(styles, declared.style) ? declared.style : fallbackStyle(name)
  const [icon, category, color] = styles[style] || styles.generic
  return { label: typeof declared.label === 'string' && declared.label.trim() ? declared.label.trim() : fallback.label || '扩展工具',
    style, icon, category, color: ['pink', 'sky', 'purple', 'green', 'orange'].includes(declared.color) ? declared.color : color }
}

const fields = {
  edit: ['file_path', 'path', 'objectId', 'standardName'], read: ['file_path', 'path', 'dir_path'], file: ['file_path', 'path', 'dir_path', 'source'],
  search: ['query', 'pattern', 'keywords', 'object'], memory: ['query', 'keywords', 'object', 'standardName', 'objectId', 'targetId'],
  terminal: ['command', 'title', 'terminal_id', 'node_id'], browser: ['url', 'query', 'target', 'selector', 'action'],
  web: ['url', 'query'], script: ['script_path', 'path', 'script'], screen: ['file_path', 'file_paths', 'imageId'],
  reminder: ['content', 'reason', 'time'], task: ['title', 'action', 'taskId'], skill: ['skill_id', 'path'],
  social: ['action', 'group_id', 'user_id'], stronghold: ['room_name', 'request', 'key'],
  system: ['current_goal', 'app_name', 'target'], generic: ['title', 'path', 'query', 'url', 'action'],
}
export function toolTarget(args, style) {
  const value = traceJson(args)
  if (!value || Array.isArray(value)) return ''
  for (const key of fields[style] || fields.generic) {
    const text = value[key]
    if (typeof text === 'string' && text.trim()) return text.trim().slice(0, 200)
    if (Array.isArray(text) && text.every(item => typeof item === 'string')) return text.join(' · ').slice(0, 200)
  }
  return ''
}
function nonNegative(value) { return Number.isSafeInteger(value) && value >= 0 ? value : null }
function object(value) { return value && typeof value === 'object' && !Array.isArray(value) ? value : {} }
export function toolResultView(name, raw) {
  const parsed = traceJson(raw), outer = object(parsed)
  const wrapped = typeof outer.ok === 'boolean' && typeof outer.output === 'string'
  const payload = wrapped ? traceJson(outer.output) : parsed
  const data = object(payload), metadata = wrapped ? object(outer.metadata) : {}
  const output = wrapped ? outer.output : traceText(raw)
  const view = { output, facts: [], matches: [], steps: [], truncated: outer.truncated === true || metadata.truncated === true || data.truncated === true || data.previewTruncated === true }
  if (['code_search', 'glob_files', 'search_files'].includes(name)) {
    if (Array.isArray(data.matches)) {
      view.matches = data.matches.filter(item => item && typeof item.file === 'string').slice(0, 6).map(item => ({ file: item.file, line: nonNegative(item.line), content: typeof item.content === 'string' ? item.content.slice(0, 240) : '' }))
      const total = nonNegative(data.total), count = nonNegative(data.fileCount)
      if (total !== null) view.facts.push(`${total} 处匹配`)
      if (count !== null) view.facts.push(`${count} 个文件`)
    } else if (Array.isArray(payload) || Array.isArray(data.files)) {
      const entries = Array.isArray(payload) ? payload : data.files
      const files = entries.filter(item => typeof item === 'string')
      view.matches = files.slice(0, 6).map(file => ({ file }))
      const count = nonNegative(data.count) ?? (entries.every(item => typeof item === 'string') ? files.length : null)
      if (count !== null) view.facts.push(`${view.truncated ? '至少 ' : ''}${count} 个文件`)
      if (nonNegative(data.directoryCount) !== null) view.facts.push(`${data.directoryCount} 个目录`)
    } else if (data.kind === 'tool_result_audit') {
      if (nonNegative(data.fileCount) !== null) view.facts.push(`${data.fileCount} 个文件`)
      if (nonNegative(data.total) !== null) view.facts.push(`${data.total} 处匹配`)
      view.facts.push('历史记录仅保留摘要')
    }
  }
  if (/^(remote_)?terminal_/.test(name)) {
    const exitCode = metadata.exitCode ?? data.exitCode
    if (Number.isInteger(exitCode)) view.facts.push(`退出码 ${exitCode}`)
    if (typeof data.output === 'string') view.output = data.output
    else if (typeof data.stdout === 'string' || typeof data.stderr === 'string') view.output = [data.stdout, data.stderr].filter(item => typeof item === 'string').join('\n')
  }
  if (name === 'update_plan') {
    const plan = object(metadata.plan || data.plan)
    if (Array.isArray(plan.steps)) {
      const known = new Set(['pending', 'in_progress', 'completed', 'blocked'])
      const steps = plan.steps.filter(step => step && typeof step.title === 'string' && known.has(step.status))
      view.steps = steps.slice(0, 12)
      if (steps.length === plan.steps.length) view.facts.push(`已完成 ${steps.filter(step => step.status === 'completed').length} / ${steps.length} 个步骤`)
    }
  }
  if (name === 'set_reminder' && data.success === true && typeof data.reminder?.time === 'string') view.facts.push(data.reminder.time)
  if (name === 'query_event_notes' && Array.isArray(data.notes) && nonNegative(data.total) !== null) view.facts.push(`返回 ${data.total} 条事件记录`)
  if (['write_event_note', 'revise_event_note'].includes(name) && data.accepted === true && data.commit === 'after_assistant_persisted') view.facts.push('草稿已接收，随回复保存后提交')
  return view
}
