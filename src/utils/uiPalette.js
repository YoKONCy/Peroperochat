


// 手帐的纸面、墨色、批注与状态色按整套主题选取。
export const journalThemes = {
  light: { label: '标准浅色', page: '#fbf7ef', paper: '#fffdf7', inset: '#f1ebe0', raised: '#fffdf7', ink: '#302923', muted: '#706456', line: '#d9cfc0', shelf: '#d5c7b9', accent: '#344f91', tint: '#e6ebf5', on: '#fffdf7', secondary: '#a45d73',
    roles: { blue: ['#344f91','#e6ebf5'], violet: ['#775578','#f1e7ed'], rose: ['#92465e','#f5e4e6'], mint: ['#386553','#e1ebe1'], amber: ['#89501c','#f8e8cc'] } },
  dark: { label: '标准暗色', page: '#222430', paper: '#2b2d39', inset: '#252733', raised: '#343541', ink: '#f2e9d9', muted: '#c4b9ad', line: '#49444a', shelf: '#191a23', accent: '#aec1ff', tint: '#353d54', on: '#232b43', secondary: '#dfa4b5',
    roles: { blue: ['#aec1ff','#353d54'], violet: ['#d8b4d6','#3c3342'], rose: ['#e9adbc','#46343e'], mint: ['#acd8bc','#303e39'], amber: ['#eabe82','#483a2e'] } },
}
export const roomThemes = {
  light: {
    label: '标准浅色', page: '#f3f6fc', paper: '#ffffff', inset: '#edf1fa', raised: '#fafbff',
    ink: '#27334f', muted: '#596781', line: '#d9e2f0', shelf: '#cbd8eb',
    accent: '#485fb0', tint: '#e7edff', on: '#ffffff', secondary: '#aa527e',
    roles: { blue: ['#3a66a3', '#e4efff'], violet: ['#7257a4', '#f0eafb'], rose: ['#9b4774', '#fce9f2'], mint: ['#276d69', '#def3ef'], amber: ['#875c26', '#fff1d8'] },
  },
  dark: {
    label: '标准暗色', page: '#161d31', paper: '#222c44', inset: '#1b243a', raised: '#29344d',
    ink: '#ecf1ff', muted: '#b2bfd8', line: '#394662', shelf: '#0f1525',
    accent: '#aabfff', tint: '#323f68', on: '#182341', secondary: '#efa3c8',
    roles: { blue: ['#adcaff', '#2b405e'], violet: ['#c9b4f3', '#3c3455'], rose: ['#efafcf', '#493149'], mint: ['#96d7ca', '#263f42'], amber: ['#ecc58a', '#443b35'] },
  },
}

/** @param {string} mode @param {typeof roomThemes} themes */
export function setThemeChrome(mode, themes) {
  const palette = mode === 'dark' ? themes.dark : themes.light
  document.documentElement.style.colorScheme = mode === 'dark' ? 'dark' : 'light'
  document.documentElement.style.setProperty('--journal-window', palette.page)
  document.querySelector('meta[name="theme-color"]')?.setAttribute('content', palette.page)
}
/** @param {string} mode @param {typeof roomThemes} themes */
export function themeVariables(mode, themes) {
  const p = mode === 'dark' ? themes.dark : themes.light
  const vars = Object.fromEntries(['page','paper','inset','raised','ink','muted','line','shelf'].map(key => ['--ui-neutral-' + key, p[key]]))
  for (const [role, [ink, wash]] of Object.entries(p.roles)) {
    vars['--ui-' + role] = ink
    vars['--ui-' + role + '-soft'] = wash
    vars['--ui-' + role + '-edge'] = `color-mix(in srgb, ${ink} 32%, ${p.paper})`
  }
  return { ...vars, '--ui-neutral-on-accent': p.on, '--accent': p.accent, '--accent-secondary': p.secondary, '--tint': p.tint,
    '--remote-accent': p.accent, '--remote-tint': p.tint, '--room-accent': p.accent, '--local-accent': p.accent, '--local-wash': p.tint, '--local-ink': p.ink,
  }
}
