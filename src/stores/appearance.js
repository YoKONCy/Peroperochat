import { computed, ref, shallowRef } from 'vue'
import { localInvoke } from '../services/localChat'
import { onModeInvalidated, currentToken, assertToken } from './mode'

const defaults = { displayMode: 'illustration', palette: 'auto', glass: 80, figureSize: 57, motion: true }
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
    if (version === generation) { appearance.value = value; appearanceError.value = '' }
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

export const appearancePalettes = {
  sky: { label: '晴空蓝', accent: '#467fc5', wash: '#eaf4ff', ink: '#2b3c56' },
  sakura: { label: '樱花粉', accent: '#b85d85', wash: '#fff0f5', ink: '#53384b' },
  mint: { label: '薄荷青', accent: '#398b7b', wash: '#eaf8f2', ink: '#2c4945' },
  violet: { label: '月夜紫', accent: '#8967ba', wash: '#f3edfc', ink: '#443756' },
}
export function appearanceStyle(agentId) {
  const value = displayPreferences.value
  const palette = appearancePalettes[value.palette === 'auto' ? agentId === 'nana' ? 'violet' : 'sky' : value.palette] || appearancePalettes.sky
  return { '--local-accent': palette.accent, '--local-wash': palette.wash, '--local-ink': palette.ink, '--local-glass': value.glass / 100, '--figure-ratio': value.figureSize / 100 }
}
export const LOCAL_APPEARANCE = Symbol('独立模式显示主题')
