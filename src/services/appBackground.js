import { computed, reactive, shallowReactive, unref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { assertToken, currentToken, onModeInvalidated } from '../stores/mode'
import { normalizeError } from './localChatClient'

export const APP_BACKGROUND = Symbol('本机背景与材质')
export const backgroundPresets = [
  { id: 'clear', label: '通透', opacity: .95, blur: 0, overlayOpacity: .08, surfaceOpacity: .68, surfaceBlur: 12 },
  { id: 'balanced', label: '均衡', opacity: .95, blur: 0, overlayOpacity: .18, surfaceOpacity: .8, surfaceBlur: 12 },
  { id: 'focus', label: '专注', opacity: .75, blur: 5, overlayOpacity: .38, surfaceOpacity: .94, surfaceBlur: 8 },
]
export function defaultBackgroundSettings(inherit = false) {
  const material = { opacity: .95, blur: 0, brightness: 1, saturation: 1, contrast: 1, overlayOpacity: .18, surfaceOpacity: .8, surfaceBlur: 12, positionX: 50, positionY: 50, fit: 'cover' }
  return { inherit, enabled: false, light: { ...material }, dark: { ...material } }
}
export function backgroundVariables(material) {
  const panel = material.surfaceOpacity, reader = Math.max(.9, panel)
  const fill = (color, alpha) => `color-mix(in srgb, var(${color}) ${alpha * 100}%, transparent)`
  return {
    '--bg-image-opacity': String(material.opacity), '--bg-image-blur': `${material.blur}px`,
    '--bg-brightness': String(material.brightness), '--bg-saturation': String(material.saturation), '--bg-contrast': String(material.contrast),
    '--bg-overlay': String(material.overlayOpacity), '--bg-position': `${material.positionX}% ${material.positionY}%`, '--bg-fit': material.fit,
    '--bg-material-blur': `${material.surfaceBlur}px`, '--bg-panel-fill': fill('--ui-paper', panel),
    '--bg-shell-fill': fill('--ui-raised', Math.max(.5, panel - .15)), '--bg-reading-fill': fill('--ui-paper', reader),
    '--bg-field-fill': fill('--ui-inset', .35), '--bg-raised-fill': fill('--ui-raised', Math.max(.94, panel)),
    '--bg-inset-reading-fill': fill('--ui-paper', Math.max(.12, (reader - panel) / Math.max(.01, 1 - panel))),
  }
}
const records = shallowReactive(new Map()), pending = new Map()
let generation = 0, writes = Promise.resolve()
const scopeKey = scope => JSON.stringify(scope)
const cacheKey = (token, scope) => `${token.installationId}:${token.epoch}:${token.target.profileId || 'standalone'}:${scopeKey(scope)}`
onModeInvalidated(() => {
  generation++
  for (const record of records.values()) if (record.url) URL.revokeObjectURL(record.url)
  records.clear(); pending.clear()
})
async function command(name, args, token) {
  assertToken(token)
  try { const result = await invoke(name, { ...args, token }); assertToken(token); return result }
  catch (failure) { throw normalizeError(failure) }
}
async function load(scope, token, force = false) {
  const key = cacheKey(token, scope)
  if (!force && records.has(key)) return records.get(key)
  if (pending.has(key)) {
    if (!force) return pending.get(key)
    await pending.get(key).catch(() => {})
  }
  const version = generation
  const task = (async () => {
    const meta = await command('appearance_background_read', { scope }, token)
    const previous = records.get(key)
    let url = '', imageError = false
    try {
      if (meta.hasImage && !meta.settingsRecovered) {
        if (previous?.revision === meta.revision && previous.url) url = previous.url
        else {
          const bytes = await command('appearance_background_image', { scope, revision: meta.revision }, token)
          url = URL.createObjectURL(new Blob([bytes instanceof ArrayBuffer ? bytes : new Uint8Array(bytes)], { type: meta.mime }))
          const image = new Image(); image.src = url
          await image.decode()
        }
      }
      assertToken(token)
      if (version !== generation) throw new Error('运行模式已经切换')
    } catch (failure) {
      if (url && url !== previous?.url) URL.revokeObjectURL(url)
      assertToken(token)
      if (version !== generation) throw failure
      url = ''; imageError = true
    }
    const record = { ...meta, url, imageError: imageError || meta.settingsRecovered && meta.hasImage }
    records.set(key, record)
    if (previous?.url && previous.url !== url) setTimeout(() => URL.revokeObjectURL(previous.url), 500)
    return record
  })()
  pending.set(key, task)
  try { return await task } finally { if (pending.get(key) === task) pending.delete(key) }
}
export function useAppBackground(agentId, scheme, expectedMode) {
  const token = computed(() => { try { const value = currentToken(); return !expectedMode || value.target.mode === expectedMode ? value : null } catch { return null } })
  const appScope = { kind: 'app' }, characterScope = computed(() => unref(agentId) ? { kind: 'character', agentId: String(unref(agentId)) } : null)
  const record = scope => token.value && scope ? records.get(cacheKey(token.value, scope)) : null
  const effective = computed(() => { const role = record(characterScope.value); return role && !role.settings.inherit ? role : record(appScope) })
  const roomReady = reactive({ source: '', defaultSource: '', ready: false, visible: false })
  const error = reactive({ message: '' })
  const material = computed(() => (effective.value?.settings || defaultBackgroundSettings())[unref(scheme) === 'dark' ? 'dark' : 'light'])
  const enabled = computed(() => effective.value?.settings.enabled === true)
  const source = computed(() => enabled.value ? effective.value?.url || (record(appScope)?.settings.enabled ? record(appScope)?.url : '') || '' : '')
  const active = computed(() => enabled.value && (roomReady.visible ? roomReady.ready : !!source.value))
  async function reload(scope = appScope, force = false) {
    if (!token.value) throw new Error('请等待当前模式准备完成')
    return load(scope, token.value, force)
  }
  async function save(input) {
    const captured = token.value
    if (!captured) throw new Error('请等待当前模式准备完成')
    // 参数取快照；图片字节已经只读，不反复序列化大数组阻塞界面。
    const snapshot = { ...input, scope: { ...input.scope }, settings: JSON.parse(JSON.stringify(input.settings)), image: input.image ? { ...input.image } : null }
    const task = writes.catch(() => {}).then(async () => {
      await command('appearance_background_save', { input: snapshot }, captured)
      return load(snapshot.scope, captured, true)
    })
    writes = task
    return task
  }
  watch(() => [token.value?.epoch, token.value?.target.profileId, unref(agentId)], async () => {
    error.message = ''; roomReady.source = ''; roomReady.ready = false
    if (!token.value) return
    const captured = token.value
    try {
      await Promise.all([load(appScope, captured), characterScope.value ? load(characterScope.value, captured) : null])
      if (token.value === captured) {
        const keep = new Set([cacheKey(captured, appScope), characterScope.value ? cacheKey(captured, characterScope.value) : ''])
        // 只保留当前应用和角色的解码图像，切换许多角色也不累积大图内存。
        for (const [key, value] of records) if (!keep.has(key)) { if (value.url) URL.revokeObjectURL(value.url); records.delete(key) }
      }
    }
    catch { if (token.value === captured) error.message = '背景暂时无法读取，已使用默认样式' }
  }, { immediate: true })
  function imageFailed() {
    const failedUrl = source.value
    for (const [key, value] of records) if (value.url && value.url === failedUrl) { URL.revokeObjectURL(value.url); records.set(key, { ...value, url: '', imageError: true }) }
  }
  return reactive({ agentId, scheme, expectedMode, source, active, material, effective, roomReady, error,
    style: computed(() => backgroundVariables(material.value)),
    characterImage: computed(() => { const role = record(characterScope.value); return role && !role.settings.inherit && role.settings.enabled ? role.url : '' }),
    load: reload, save, imageFailed,
  })
}
