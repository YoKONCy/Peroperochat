import { computed, ref, shallowRef } from 'vue'
import { localInvoke } from '../services/localChat'
import { onModeInvalidated, currentToken, assertToken } from './mode'
import { roomThemes, themeVariables } from '../utils/uiPalette'

const defaults = { displayMode: 'illustration', palette: 'light', glass: 80, figureSize: 57, motion: true }
export const appearance = shallowRef(null)
export const appearanceBusy = ref(false)
export const appearanceError = ref('')
export const displayPreferences = computed(() => appearance.value || defaults)
let loading = null, generation = 0
onModeInvalidated(() => { generation++; loading = null; appearance.value = null; appearanceBusy.value = false; appearanceError.value = '' })

export async function loadAppearance() {
  if (appearance.value) { appearanceError.value = ''; return appearance.value }
  if (loading) return loading
  const version = generation
  const task = localInvoke('local_appearance').then(value => {
    if (version === generation) { appearance.value = { ...value, palette: value.palette === 'dark' ? 'dark' : 'light' }; appearanceError.value = '' }
    return value
  }).catch(error => { if (version === generation) appearanceError.value = error.message; throw error })
  loading = task
  try { return await task } finally { if (loading === task) loading = null }
}
export async function saveAppearance(patch) {
  if (appearanceBusy.value) throw new Error('显示设置正在保存，请稍候')
  const token = currentToken(), version = generation
  appearanceBusy.value = true
  try {
    assertToken(token, 'standalone')
    if (!appearance.value && !Object.keys(defaults).every(key => Object.hasOwn(patch, key))) await loadAppearance()
    assertToken(token, 'standalone')
    const result = await localInvoke('local_save_appearance', { appearance: { ...(appearance.value || defaults), ...patch } })
    assertToken(token, 'standalone')
    if (version === generation) { appearance.value = result; appearanceError.value = '' }
    return result
  } catch (error) { if (version === generation) appearanceError.value = error.message; throw error }
  finally { if (version === generation) appearanceBusy.value = false }
}

export function appearanceStyle() {
  const value = displayPreferences.value
  return { ...themeVariables(value.palette, roomThemes), '--local-glass': value.glass / 100, '--figure-ratio': value.figureSize / 100 }
}
export const LOCAL_APPEARANCE = Symbol('独立模式显示主题')
