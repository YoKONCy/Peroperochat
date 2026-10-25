import test from 'node:test'
import assert from 'node:assert/strict'
import { groupToolList, toolListState } from '../src/utils/toolList.js'

test('系统身份优先排序，同时保留模式限制；手动选择不能绕过模式禁用', () => {
  const disabled = new Set(['system_tool', 'manual_tool', 'both_tool'])
  const system = toolListState({ name: 'system_tool', locked: true }, disabled)
  assert.equal(system.section, 'system')
  assert.equal(system.enabled, true)
  assert.equal(system.toggleable, false)
  const blockedSystem = toolListState({ name: 'system_tool', locked: true, modeAllowed: false }, disabled)
  assert.equal(blockedSystem.section, 'system')
  assert.equal(blockedSystem.visual, 'system-blocked')
  assert.equal(blockedSystem.enabled, false)
  assert.equal(toolListState({ name: 'manual_tool' }, disabled).visual, 'manual-disabled')
  const both = toolListState({ name: 'both_tool', modeAllowed: false }, disabled)
  assert.equal(both.visual, 'mode-disabled')
  assert.equal(both.manualDisabled, true)
  assert.equal(both.toggleable, false)
  assert.equal(toolListState({ name: 'mode_tool', userEnabled: true, modeAllowed: false }, new Set()).enabled, false)
})

test('状态优先，包内子工具相邻；组与单项以函数名排序，不使用显示名称排序', () => {
  const tools = [
    { name: 'z_system', locked: true },
    { name: 'z_write', label: 'A', packageId: 'same-package' },
    { name: 'c_tool' },
    { name: 'b_read', label: 'Z', packageId: 'same-package' },
    { name: 'e_mode', modeAllowed: false },
    { name: 'a_tool' },
    { name: 'd_manual' },
    { name: 'a_system', locked: true },
  ]
  const snapshot = JSON.stringify(tools)
  const sections = groupToolList(tools, ['d_manual'])
  assert.deepEqual(sections.map(section => section.id), ['system', 'enabled', 'disabled'])
  assert.deepEqual(sections.map(section => section.groups.flatMap(group => group.tools.map(tool => tool.name))), [
    ['a_system', 'z_system'], ['a_tool', 'b_read', 'z_write', 'c_tool'], ['d_manual', 'e_mode'],
  ])
  assert.deepEqual(sections.map(section => section.count), [2, 4, 2])
  assert.equal(JSON.stringify(tools), snapshot)
})

test('开关草稿实时改变分区，服务端旧 enabled 不会把刚启用的工具继续当作停用', () => {
  const tool = { name: 'draft_tool', userEnabled: false, enabled: false, modeAllowed: true }
  assert.equal(groupToolList([tool])[0].id, 'disabled')
  assert.equal(groupToolList([tool], [tool.name])[0].id, 'disabled')
  assert.equal(groupToolList([tool], [])[0].id, 'enabled')
  assert.equal(groupToolList([{ ...tool, modeAllowed: false }], [])[0].id, 'disabled')
})

test('MCP 按服务端包归组；旧清单只按明确命名空间合并，不把相同动词当作工具组', () => {
  const sections = groupToolList([
    { name: 'mcp_server_with_underscores_b', packageId: 'mcp:server_with_underscores' },
    { name: 'mcp_server_with_underscores_a', packageId: 'mcp:server_with_underscores' },
    { name: 'read_file' }, { name: 'read_memory' },
    { name: 'browser_open' }, { name: 'browser_click' },
  ])
  const groups = sections[0].groups
  assert.equal(groups.find(group => group.label === 'mcp:server_with_underscores').tools.length, 2)
  assert.equal(groups.find(group => group.label === 'browser').tools.length, 2)
  assert.equal(groups.filter(group => group.key.startsWith('tool:read_')).length, 2)
})

test('按函数、标签、说明和包名查找仍保留状态分组与稳定顺序', () => {
  const tools = [{ name: 'z_child', packageId: 'demo-kit', label: '测试乙' }, { name: 'a_child', packageId: 'demo-kit', label: '测试甲' }, { name: 'b_other', description: '说明关键词' }]
  assert.deepEqual(groupToolList(tools, ['z_child'], 'demo-kit').map(section => section.id), ['enabled', 'disabled'])
  assert.equal(groupToolList(tools, [], '测试甲')[0].groups[0].tools[0].name, 'a_child')
  assert.equal(groupToolList(tools, [], '关键词')[0].groups[0].tools[0].name, 'b_other')
  assert.deepEqual(groupToolList(tools, [], 'not-present'), [])
})
