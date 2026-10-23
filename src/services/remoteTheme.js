import { computed, reactive, ref, onBeforeUnmount, provide } from 'vue'
import { currentToken, assertToken } from '../stores/mode'
import { readPresentation, writePresentation } from './remotePresentation'
import { uiPaletteVariables } from '../utils/uiPalette'
import { APP_BACKGROUND, useAppBackground } from './appBackground'

export const REMOTE_THEME = Symbol('remote-theme')
export const remotePalettes = [
  { id: 'sky', name: '晴空蓝', light: ['#467fc5', '#eff8ff', '#96c7e8'], dark: ['#9fc9f0', '#162133', '#b9d8ed'] },
  { id: 'night', name: '月夜紫', light: ['#8966b7', '#f7f2ff', '#cb9dbb'], dark: ['#c7b3f0', '#211a30', '#d79bb8'] },
  { id: 'sakura', name: '樱花粉', light: ['#bb5c84', '#fff3f7', '#e6b4ca'], dark: ['#eeafcc', '#2a1d29', '#c98ba8'] },
  { id: 'mint', name: '薄荷青', light: ['#307e74', '#effaf6', '#8dcabb'], dark: ['#98d9c4', '#172923', '#b5cec5'] },
  { id: 'amber', name: '琥珀金', light: ['#9c7127', '#fff9eb', '#dfba75'], dark: ['#e6c58b', '#2b251a', '#d0a86a'] },
  { id: 'crimson', name: '绯红', light: ['#b74a5e', '#fff2f4', '#d78b9b'], dark: ['#f0a2b3', '#2d1c26', '#d18399'] },
  { id: 'silver', name: '银雾', light: ['#5c738e', '#f0f5fa', '#b0c2d3'], dark: ['#b9cddd', '#1e252e', '#a7b6cc'] },
  { id: 'peach', name: '桃橙', light: ['#b46643', '#fff5ee', '#e9b29d'], dark: ['#f1ba9d', '#2c221e', '#dc9c8e'] },
]
export const defaultCharacterAppearance = () => ({ expanded: false, model: null, palette: 'sky', cg: null, style: null })
export function builtinCg(agent) { return ['pero', 'nana'].includes(agent?.id) ? `/remote/${agent.id}-cg.png` : '' }
export function characterCg(agent, appearance) { return appearance?.cg || builtinCg(agent) }
export function useRemoteTheme(character, agentId) {
  const token = currentToken(), system = window.matchMedia('(prefers-color-scheme: dark)')
  const systemDark = ref(system.matches), ready = ref(false), saving = ref(false), error = ref('')
  const display = reactive({ theme: 'light', palette: 'standard', showCg: true, reduceMotion: false, roleStyle: false, fontSize: 14 })
  let disposed = false, saveVersion = 0
  const mode = computed(() => display.theme === 'system' ? systemDark.value ? 'dark' : 'light' : display.theme)
  const background = useAppBackground(agentId, mode, 'remote')
  provide(APP_BACKGROUND, background)
  const selectedStyle = computed(() => display.roleStyle ? character.style : null)
  const theme = computed(() => {
    const id = ({ violet: 'night', rose: 'sakura' })[display.palette] || display.palette
    const palette = remotePalettes.find(item => item.id === id) || remotePalettes[mode.value === 'dark' ? 1 : 0]
    const [accent, tint, secondary] = palette[mode.value]
    const style = selectedStyle.value
    const color = style?.[mode.value === 'dark' ? 'darkAccent' : 'lightAccent'] || accent
    const surfaceTint = mode.value === 'dark' && style?.darkAccent ? undefined : tint
    const paletteVariables = uiPaletteVariables(color, mode.value, surfaceTint)
    const backgroundTint = surfaceTint || paletteVariables['--ui-neutral-inset']
    return { mode: mode.value, reduceMotion: display.reduceMotion, backgroundActive: background.active, bubble: style?.bubble || 'standard', decoration: style?.decoration === 'standard' || !style?.decoration ? mode.value === 'dark' ? 'constellation' : 'ribbon' : style.decoration,
      style: { ...paletteVariables, ...background.style, '--remote-font-size': (display.fontSize || 14) + 'px', '--remote-chat-size': ((display.fontSize || 14) + 1) + 'px', '--accent': color, '--tint': backgroundTint, '--accent-secondary': style?.secondary || secondary, '--remote-accent': color, '--remote-tint': backgroundTint, '--room-accent': color } }
  })
  provide(REMOTE_THEME, theme)
  function systemChanged(event) { systemDark.value = event.matches }
  system.addEventListener('change', systemChanged)
  async function restore() {
    try { const saved = await readPresentation({ kind: 'appearance' }, token); assertToken(token, 'remote'); if (!disposed && saved) Object.assign(display, saved) }
    catch (failure) { if (!disposed) error.value = `显示偏好读取失败：${failure.message}` }
    finally { if (!disposed) ready.value = true }
  }
  async function save() {
    if (!ready.value) return
    const request = ++saveVersion
    saving.value = true; error.value = ''
    try { await writePresentation({ kind: 'appearance' }, { ...display }, token); assertToken(token, 'remote') }
    catch (failure) { if (!disposed && request === saveVersion) error.value = `显示偏好尚未保存：${failure.message}` }
    finally { if (!disposed && request === saveVersion) saving.value = false }
  }
  onBeforeUnmount(() => { disposed = true; system.removeEventListener('change', systemChanged) })
  return { display, theme, background, ready, saving, error, restore, save }
}
