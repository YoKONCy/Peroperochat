const functionNames = new Intl.Collator('en', { sensitivity: 'base', numeric: true })
const compareNames = (a, b) => functionNames.compare(a, b) || (a < b ? -1 : a > b ? 1 : 0)

// 系统身份、模式限制与手动选择分别保留；开关草稿不能覆盖服务端的模式限制。
export function toolListState(tool, disabledNames = null) {
  const system = tool.locked === true, modeBlocked = tool.modeAllowed === false
  const manualDisabled = !system && (disabledNames == null ? tool.userEnabled === false : disabledNames.has(tool.name))
  const enabled = !modeBlocked && !manualDisabled
  return {
    system, modeBlocked, manualDisabled, enabled,
    section: system ? 'system' : enabled ? 'enabled' : 'disabled',
    visual: system ? modeBlocked ? 'system-blocked' : 'system' : modeBlocked ? 'mode-disabled' : manualDisabled ? 'manual-disabled' : 'enabled',
    toggleable: !system && !modeBlocked,
  }
}

function toolFamily(tool) {
  const packageId = typeof tool.packageId === 'string' ? tool.packageId.trim() : ''
  if (packageId) return { key: `package:${packageId}`, label: packageId }
  const chain = tool.display?.signature?.chain
  if (typeof chain === 'string' && chain) return { key: `chain:${chain}`, label: chain }
  // 旧服务缺少包标识时，只合并明确的命名空间，不把 read_* 等同名前缀误认作同一工具。
  const namespace = tool.name.match(/^(.+?)(?:__|[.:])/)
  const knownFamily = tool.name.match(/^(remote_terminal|terminal|browser|stronghold|social|skill)_/)
  const family = namespace?.[1] || knownFamily?.[1]
  return family ? { key: `namespace:${family}`, label: family } : { key: `tool:${tool.name}`, label: '' }
}

export function groupToolList(tools = [], disabledNames = null, search = '') {
  const disabled = disabledNames == null ? null : new Set(disabledNames), query = search.trim().toLowerCase()
  const sections = [
    { id: 'system', label: '系统工具', icon: 'shield', groups: [], count: 0 },
    { id: 'enabled', label: '已启用', icon: 'check', groups: [], count: 0 },
    { id: 'disabled', label: '已停用', icon: 'tool-off', groups: [], count: 0 },
  ]
  const buckets = new Map(sections.map(section => [section.id, new Map()]))
  for (const tool of tools) {
    if (!tool || typeof tool.name !== 'string' || !tool.name) continue
    if (query && !`${tool.label || tool.display?.label || ''} ${tool.name} ${tool.description || ''} ${tool.packageId || ''}`.toLowerCase().includes(query)) continue
    const state = toolListState(tool, disabled), family = toolFamily(tool), groups = buckets.get(state.section)
    if (!groups.has(family.key)) groups.set(family.key, { ...family, tools: [] })
    groups.get(family.key).tools.push({ ...tool, listState: state })
  }
  for (const section of sections) {
    section.groups = [...buckets.get(section.id).values()]
    for (const group of section.groups) group.tools.sort((a, b) => compareNames(a.name, b.name))
    // 一组的位置由首个函数名决定，中文显示名称和工具包名称不影响字母顺序。
    section.groups.sort((a, b) => compareNames(a.tools[0].name, b.tools[0].name))
    section.count = section.groups.reduce((total, group) => total + group.tools.length, 0)
  }
  return sections.filter(section => section.count)
}
