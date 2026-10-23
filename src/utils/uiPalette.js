// 主色跟随配色包；功能辅色在这里集中生成，页面只按用途取色。
const colors = {
  light: { page: '#f5f4f9', paper: '#ffffff', ink: '#36354c', muted: '#817e94', line: '#e3deec', shelf: '#ded9e8', rose: '#b8587d', mint: '#307f70', violet: '#8664b1', amber: '#a17536', blue: '#4d7fae' },
  dark: { ink: '#e9ebf1', muted: '#a8afbd', shelf: '#03050b', rose: '#e9a5be', mint: '#8ed0b7', violet: '#bfa8ea', amber: '#e3bf83', blue: '#96bfe7' },
}
const washes = { rose: '#fbe3eb', mint: '#e0f3eb', violet: '#eee5fc', amber: '#fff0d7', blue: '#e2effa' }
const sections = { api: 'blue', model: 'blue', models: 'blue', providers: 'blue', agents: 'rose', role: 'rose', proactive: 'amber', appearance: 'violet', assignments: 'blue', prompts: 'violet', voice: 'rose', user: 'rose', usage: 'mint', runtime: 'mint', memory: 'violet', data: 'amber' }
export function uiSectionTone(id) { return sections[id] || 'blue' }
function mix(first, second, ratio) {
  if (![first, second].every(value => /^#[a-f\d]{6}$/i.test(value || ''))) {
    return `color-mix(in srgb, ${first} ${ratio * 100}%, ${second})`
  }
  const parts = value => value.slice(1).match(/../g).map(part => parseInt(part, 16))
  const a = parts(first), b = parts(second)
  return '#' + a.map((value, index) => Math.round(value * ratio + b[index] * (1 - ratio)).toString(16).padStart(2, '0')).join('')
}
export function uiPaletteVariables(accent, mode = 'light', paletteTint) {
  const dark = mode === 'dark', base = { ...colors[dark ? 'dark' : 'light'] }
  // 页面、内嵌区、卡片与浮层分别定色，不再让全部暗色配色共用紫灰底。
  if (dark) {
    const night = paletteTint || mix(accent, '#10131b', .16)
    Object.assign(base, {
      page: mix(night, '#080b12', .67), inset: mix(night, '#10131b', .85),
      paper: mix(night, '#3e4350', .65), raised: mix(night, '#4f5663', .53),
      on: mix(night, '#070a10', .38),
    })
    base.line = mix(base.ink, base.paper, .09)
  }
  // 角色样式包允许 CSS 颜色；交给浏览器混色，不解析或执行样式包内容。
  const tint = (color, ratio) => mix(color, accent, ratio)
  const variables = Object.fromEntries(['page', 'paper', 'ink', 'muted', 'line', 'shelf'].map(key => [`--ui-neutral-${key}`, base[key]]))
  variables['--ui-neutral-inset'] = base.inset || base.paper
  variables['--ui-neutral-raised'] = base.raised || base.paper
  variables['--ui-neutral-on-accent'] = base.on || '#ffffff'
  for (const role of Object.keys(washes)) {
    const color = tint(base[role], dark ? .94 : .92)
    variables[`--ui-${role}`] = color
    variables[`--ui-${role}-soft`] = dark ? mix(color, base.paper, .12) : tint(washes[role], .97)
    variables[`--ui-${role}-edge`] = mix(color, base.paper, dark ? .29 : .3)
  }
  return variables
}
