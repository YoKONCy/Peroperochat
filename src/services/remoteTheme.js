import { computed, reactive, ref, onBeforeUnmount, provide, watch } from 'vue'
import { currentToken, assertToken } from '../stores/mode'
import { readPresentation, writePresentation } from './remotePresentation'
import { journalVariables, setJournalChrome } from '../utils/uiPalette'
import { APP_BACKGROUND, useAppBackground } from './appBackground'

export const REMOTE_THEME = Symbol('remote-theme')
export const defaultCharacterAppearance = () => ({ expanded: false, model: null, palette: 'sky', cg: null, style: null })
export function builtinCg(agent) { return ['pero', 'nana'].includes(agent?.id) ? `/remote/${agent.id}-cg.png` : '' }
export function characterCg(agent, appearance) { return appearance?.cg || builtinCg(agent) }
export function useRemoteTheme(agentId) {
  const token = currentToken(), system = window.matchMedia('(prefers-color-scheme: dark)')
  const systemDark = ref(system.matches), ready = ref(false), saving = ref(false), error = ref('')
  const display = reactive({ theme: 'light', palette: 'standard', showCg: true, reduceMotion: false, roleStyle: false, fontSize: 14 })
  let disposed = false, saveVersion = 0
  const mode = computed(() => display.theme === 'system' ? systemDark.value ? 'dark' : 'light' : display.theme)
  watch(mode, setJournalChrome, { immediate: true })
  const background = useAppBackground(agentId, mode, 'remote')
  provide(APP_BACKGROUND, background)
  const theme = computed(() => ({
    mode: mode.value, reduceMotion: display.reduceMotion, backgroundActive: background.active, bubble: 'standard', decoration: 'ribbon',
    style: { ...journalVariables(mode.value), ...background.style, '--remote-font-size': (display.fontSize || 14) + 'px', '--remote-chat-size': ((display.fontSize || 14) + 1) + 'px' }
  }))
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
