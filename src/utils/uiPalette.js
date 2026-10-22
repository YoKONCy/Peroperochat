// 主色跟随配色包；功能辅色在这里集中生成，页面只按用途取色。
const colors = {
  light: { page: '#f5f4f9', paper: '#ffffff', ink: '#36354c', muted: '#817e94', line: '#e3deec', shelf: '#ded9e8', rose: '#b8587d', mint: '#307f70', violet: '#8664b1', amber: '#a17536', blue: '#4d7fae' },
  dark: { page: '#211f2d', paper: '#302d3d', ink: '#f2edf7', muted: '#b3aabe', line: '#53495f', shelf: '#15131f', rose: '#f1a4ba', mint: '#8cd2b9', violet: '#c2abed', amber: '#edc182', blue: '#97c3ee' },
}
const washes = { rose: '#fbe3eb', mint: '#e0f3eb', violet: '#eee5fc', amber: '#fff0d7', blue: '#e2effa' }
const sections = { api: 'blue', model: 'blue', models: 'blue', providers: 'blue', agents: 'rose', role: 'rose', proactive: 'amber', appearance: 'violet', assignments: 'blue', prompts: 'violet', voice: 'rose', user: 'rose', usage: 'mint', runtime: 'mint', memory: 'violet', data: 'amber' }
export function uiSectionTone(id) { return sections[id] || 'blue' }
function mix(first, second, ratio) {
  const parts = value => value.slice(1).match(/../g).map(part => parseInt(part, 16))
  const a = parts(first), b = parts(second)
  return '#' + a.map((value, index) => Math.round(value * ratio + b[index] * (1 - ratio)).toString(16).padStart(2, '0')).join('')
}
export function uiPaletteVariables(accent, mode = 'light') {
  const dark = mode === 'dark', base = colors[dark ? 'dark' : 'light']
  // 角色样式包允许 CSS 颜色；交给浏览器混色，不解析或执行样式包内容。
  const primary = /^#[a-f\d]{6}$/i.test(accent || '') ? accent : null
  const tint = (color, ratio) => primary ? mix(color, primary, ratio) : `color-mix(in srgb, ${color} ${ratio * 100}%, ${accent})`
  const variables = Object.fromEntries(['page', 'paper', 'ink', 'muted', 'line', 'shelf'].map(key => [`--ui-neutral-${key}`, base[key]]))
  for (const role of Object.keys(washes)) {
    const color = tint(base[role], .92)
    variables[`--ui-${role}`] = color
    variables[`--ui-${role}-soft`] = dark ? `color-mix(in srgb, ${color} 20%, ${base.paper})` : tint(washes[role], .97)
    variables[`--ui-${role}-edge`] = `color-mix(in srgb, ${color} ${dark ? 50 : 30}%, ${base.paper})`
  }
  return variables
}
