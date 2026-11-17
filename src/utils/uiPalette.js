


// 手帐的纸面、墨色、批注与状态色按整套主题选取。
export const journalThemes = {
  light: { label: '标准浅色', page: '#fbf7ef', paper: '#fffdf7', inset: '#f1ebe0', raised: '#fffdf7', ink: '#302923', muted: '#706456', line: '#d9cfc0', shelf: '#d5c7b9', accent: '#344f91', tint: '#e6ebf5', on: '#fffdf7', secondary: '#a45d73',
    roles: { blue: ['#344f91','#e6ebf5'], violet: ['#775578','#f1e7ed'], rose: ['#92465e','#f5e4e6'], mint: ['#386553','#e1ebe1'], amber: ['#89501c','#f8e8cc'] } },
  dark: { label: '标准暗色', page: '#222430', paper: '#2b2d39', inset: '#252733', raised: '#343541', ink: '#f2e9d9', muted: '#c4b9ad', line: '#49444a', shelf: '#191a23', accent: '#aec1ff', tint: '#353d54', on: '#232b43', secondary: '#dfa4b5',
    roles: { blue: ['#aec1ff','#353d54'], violet: ['#d8b4d6','#3c3342'], rose: ['#e9adbc','#46343e'], mint: ['#acd8bc','#303e39'], amber: ['#eabe82','#483a2e'] } },
}
export function setJournalChrome(mode) {
  const palette = journalThemes[mode] || journalThemes.light
  document.documentElement.style.colorScheme = mode === 'dark' ? 'dark' : 'light'
  document.documentElement.style.setProperty('--journal-window', palette.page)
  document.querySelector('meta[name="theme-color"]')?.setAttribute('content', palette.page)
}
export function journalVariables(mode) {
  const p = journalThemes[mode] || journalThemes.light
  const vars = Object.fromEntries(['page','paper','inset','raised','ink','muted','line','shelf'].map(key => ['--ui-neutral-' + key, p[key]]))
  for (const [role, [ink, wash]] of Object.entries(p.roles)) {
    vars['--ui-' + role] = ink
    vars['--ui-' + role + '-soft'] = wash
    vars['--ui-' + role + '-edge'] = `color-mix(in srgb, ${ink} 32%, ${p.paper})`
  }
  return { ...vars, '--ui-neutral-on-accent': p.on, '--accent': p.accent, '--accent-secondary': p.secondary, '--tint': p.tint,
    '--remote-accent': p.accent, '--remote-tint': p.tint, '--room-accent': p.accent, '--local-accent': p.accent, '--local-wash': p.tint, '--local-ink': p.ink,
    '--journal-rule': mode === 'dark' ? '#75685e' : '#b9a996' }
}
