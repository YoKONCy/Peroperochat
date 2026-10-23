<template>
  <section v-if="background" class="background-editor" :aria-label="tr('背景与材质')">
    <header class="background-editor-heading"><RoomIcon name="image" /><div><h3>{{ tr('背景与材质') }}</h3><p>{{ tr('留一点风景，给喜欢的日常。') }}</p></div></header>
    <div class="background-scope" role="group" :aria-label="tr('设置范围')"><button type="button" :class="{ selected: scope.kind === 'app' }" :disabled="busy" @click="changeScope('app')">{{ tr('整个应用') }}</button><button v-if="agent?.id" type="button" :class="{ selected: scope.kind === 'character' }" :disabled="busy" @click="changeScope('character')">{{ tr('{0}的背景', [agent.name]) }}</button></div>
    <p class="background-scope-note">{{ tr(background.expectedMode === 'remote' ? '只保存在这台手机，不改变电脑上的外观。' : '默认房间继续跟随角色与时间，应用背景用于其他页面。') }}</p>
    <FeedbackMessage :message="error" />
    <button v-if="error && !original" type="button" class="room-text-button" :disabled="loading || busy" @click="load">{{ tr('重试') }}</button>
    <p v-if="original?.settingsRecovered" class="background-small-note">{{ tr('已恢复默认预览，保存后即可重新使用。') }}</p>
    <div v-if="scope.kind === 'character'" class="background-inherit"><span>{{ tr('沿用应用设置') }}</span><RoomSwitch :model-value="draft.inherit" :label="tr('沿用应用设置')" :disabled="busy || loading || !original" @update:model-value="changeInheritance" /></div>
    <fieldset :disabled="busy || loading || !original || scope.kind === 'character' && draft.inherit">
      <div class="background-preview-wrap">
        <div class="background-preview companion-surface remote-themed" data-ui="soft" :data-remote-theme="previewScheme" :data-background="previewActive ? 'active' : undefined" :style="previewStyle" @pointerdown="startDrag" @pointermove="moveDrag" @pointerup="endDrag" @pointercancel="endDrag" @lostpointercapture="drag = null">
          <div v-if="previewImage" class="background-preview-image" aria-hidden="true"><img :src="previewImage" alt="" /><i /></div>
          <div class="background-preview-ui"><header><span>{{ agent?.name || 'PeroperoChat' }}</span><RoomIcon name="bow" /></header><template v-if="previewPage === 'chat'"><div class="preview-message-user">{{ tr('今天也想慢慢来。') }}</div><div class="preview-message-assistant"><strong>{{ agent?.name || 'PeroperoChat' }}</strong><p>{{ tr('好呀，把时间留给喜欢的事。') }}</p></div><div class="preview-input"><RoomIcon name="plus" /><span>{{ tr('说说你的想法…') }}</span><RoomIcon name="send" /></div></template><template v-else><div class="preview-card-grid"><div data-ui-tone="rose"><RoomIcon name="profile" /><strong>{{ tr('角色') }}</strong></div><div data-ui-tone="blue"><RoomIcon name="model" /><strong>{{ tr('模型') }}</strong></div></div><div class="preview-management-row"><RoomIcon name="palette" /><span>{{ tr('显示与主题') }}</span><RoomIcon name="arrow" /></div></template></div>
        </div>
        <div class="background-preview-controls"><div><button type="button" :aria-pressed="previewPage === 'chat'" @click="previewPage = 'chat'">{{ tr('聊天') }}</button><button type="button" :aria-pressed="previewPage === 'manage'" @click="previewPage = 'manage'">{{ tr('管理') }}</button></div><div v-if="background.expectedMode === 'remote'"><button type="button" :aria-pressed="previewScheme === 'light'" :aria-label="tr('浅色')" @click="previewScheme = 'light'"><RoomIcon name="sun" /></button><button type="button" :aria-pressed="previewScheme === 'dark'" :aria-label="tr('暗色')" @click="previewScheme = 'dark'"><RoomIcon name="moon" /></button></div></div>
      </div>
      <div class="background-image-actions"><button type="button" class="room-secondary-button" @click="fileInput?.click()"><RoomIcon name="image" />{{ tr(previewImage ? '更换图片' : '选择背景图片') }}</button><button v-if="previewImage" type="button" class="room-text-button" @click="removeImage">{{ tr('移除图片') }}</button><input ref="fileInput" type="file" accept="image/png,image/jpeg,image/webp" hidden @change="importImage" /></div>
      <p v-if="previewImage" class="background-small-note">{{ tr('拖动预览调整图片位置。') }}</p>
      <p v-if="original?.imageError" class="background-small-note">{{ tr('原背景暂时无法显示，可以重新选图。') }}</p>
      <div class="background-enable"><span>{{ tr('启用背景与材质') }}</span><RoomSwitch v-model="draft.enabled" :label="tr('启用背景与材质')" /></div>
      <div class="background-presets" role="group" :aria-label="tr('快捷效果')"><button v-for="preset in backgroundPresets" :key="preset.id" type="button" :class="{ selected: isPreset(preset) }" @click="applyPreset(preset)"><RoomIcon :name="preset.id === 'clear' ? 'image' : preset.id === 'balanced' ? 'palette' : 'diary'" /><strong>{{ tr(preset.label) }}</strong></button></div>
      <div class="background-sliders"><label v-for="item in commonControls" :key="item.key"><span>{{ tr(item.label) }}<output>{{ formatValue(item) }}</output></span><input :value="item.key === 'surfaceOpacity' ? 1 - material[item.key] : material[item.key]" type="range" :min="item.min" :max="item.max" :step="item.step" :aria-label="tr(item.label)" @input="material[item.key] = item.key === 'surfaceOpacity' ? 1 - Number($event.target.value) : Number($event.target.value); draft.enabled = true" /></label></div>
      <details class="background-more"><summary><RoomIcon name="settings" />{{ tr('更多调整') }}<RoomIcon name="down" /></summary><div class="background-fit" role="group" :aria-label="tr('图片显示方式')"><button type="button" :aria-pressed="material.fit === 'cover'" @click="material.fit = 'cover'; draft.enabled = true">{{ tr('填满背景') }}</button><button type="button" :aria-pressed="material.fit === 'contain'" @click="material.fit = 'contain'; draft.enabled = true">{{ tr('完整显示') }}</button></div><div class="background-sliders"><label v-for="item in advancedControls" :key="item.key"><span>{{ tr(item.label) }}<output>{{ formatValue(item) }}</output></span><input v-model.number="material[item.key]" type="range" :min="item.min" :max="item.max" :step="item.step" :aria-label="tr(item.label)" @input="draft.enabled = true" /></label></div></details>
    </fieldset>
    <div class="background-editor-footer"><button type="button" class="room-text-button" :disabled="busy || loading || !original" @click="reset">{{ tr('恢复默认') }}</button><span /><button type="button" class="room-secondary-button" :disabled="!dirty || busy" @click="cancel">{{ tr('取消') }}</button><button type="button" class="room-primary-button" :disabled="!dirty || busy || loading" @click="save">{{ tr(busy ? '正在保存…' : '保存背景') }}</button></div>
    <div v-if="switchTo" class="background-switch-confirm" role="alert"><p>{{ tr('先保存或取消当前调整，再切换范围。') }}</p><button type="button" class="room-text-button" @click="cancel(); changeScope(switchTo)">{{ tr('放弃调整并切换') }}</button><button type="button" class="room-text-button" @click="switchTo = null">{{ tr('继续调整') }}</button></div>
  </section>
</template>
<script setup>
import { computed, inject, onBeforeUnmount, onMounted, reactive, ref, shallowRef, watch } from 'vue'
import { APP_BACKGROUND, backgroundPresets, backgroundVariables, defaultBackgroundSettings } from '../../services/appBackground'
import { uiPaletteVariables } from '../../utils/uiPalette'
import { remotePalettes } from '../../services/remoteTheme'
import { appearancePalettes, displayPreferences } from '../../stores/appearance'
import { t } from '../../i18n/remote'
import { appMessage } from '../../services/appFeedback'
import RoomIcon from './RoomIcon.vue'
import RoomSwitch from './RoomSwitch.vue'
import FeedbackMessage from '../FeedbackMessage.vue'
const props = defineProps({ agent: Object, palette: { type: String, default: 'standard' }, roomImage: String })
const background = inject(APP_BACKGROUND, null)
const tr = (key, values) => t(key, values, 'background')
const scope = ref({ kind: 'app' }), original = ref(null), globalOriginal = ref(null), draft = reactive(defaultBackgroundSettings())
const loading = ref(true), busy = ref(false), error = ref(''), fileInput = ref(null), pendingImage = shallowRef(null), removed = ref(false), temporaryUrl = ref(''), switchTo = ref(null)
const previewScheme = ref(background?.scheme === 'dark' ? 'dark' : 'light'), previewPage = ref('chat')
let disposed = false, request = 0, drag = null
const material = computed(() => draft[previewScheme.value])
const dirty = computed(() => !!original.value && (JSON.stringify(draft) !== JSON.stringify(original.value.settings) || !!pendingImage.value || removed.value || original.value.settingsRecovered))
const previewSettings = computed(() => scope.value.kind === 'character' && draft.inherit ? globalOriginal.value?.settings || draft : draft)
const previewImage = computed(() => (draft.inherit && scope.value.kind === 'character' ? globalOriginal.value?.url : temporaryUrl.value || (!removed.value ? original.value?.url : '') || (scope.value.kind === 'character' ? globalOriginal.value?.url : '')) || (background?.expectedMode === 'standalone' ? props.roomImage || '' : ''))
const previewActive = computed(() => previewSettings.value.enabled && (!!previewImage.value || background?.expectedMode === 'standalone'))
const previewStyle = computed(() => {
  const id = ({ violet: 'night', rose: 'sakura' })[props.palette] || props.palette
  const pack = remotePalettes.find(item => item.id === id) || remotePalettes[previewScheme.value === 'dark' ? 1 : 0]
  let [accent, tint, secondary] = pack[previewScheme.value]
  if (background?.expectedMode === 'standalone') {
    const local = displayPreferences.value.palette
    accent = (appearancePalettes[local === 'auto' ? props.agent?.id === 'nana' ? 'violet' : 'sky' : local] || appearancePalettes.sky).accent
  }
  return { ...uiPaletteVariables(accent, previewScheme.value, tint), '--accent': accent, '--room-accent': accent, '--accent-secondary': secondary, ...backgroundVariables(previewSettings.value[previewScheme.value]) }
})
const commonControls = [{ key:'brightness',label:'背景明暗',min:.6,max:1.4,step:.02 },{ key:'blur',label:'背景模糊',min:0,max:24,step:1 },{ key:'surfaceOpacity',label:'界面通透度',min:0,max:.45,step:.01 }]
const advancedControls = [{key:'opacity',label:'图片显现',min:0,max:1,step:.01},{key:'saturation',label:'饱和度',min:0,max:1.6,step:.02},{key:'contrast',label:'对比度',min:.6,max:1.4,step:.02},{key:'overlayOpacity',label:'背景遮罩',min:0,max:.85,step:.01},{key:'surfaceBlur',label:'玻璃模糊',min:0,max:20,step:1},{key:'positionX',label:'水平位置',min:0,max:100,step:1},{key:'positionY',label:'垂直位置',min:0,max:100,step:1}]
function formatValue(item) { const value = material.value[item.key]; return item.key === 'surfaceOpacity' ? `${Math.round((1 - value) * 100)}%` : item.key.toLowerCase().includes('blur') ? `${value} px` : item.key.startsWith('position') ? `${value}%` : `${Math.round(value * 100)}%` }
function clearTemporary() { if (temporaryUrl.value) URL.revokeObjectURL(temporaryUrl.value); temporaryUrl.value = ''; pendingImage.value = null }
function cancel() { if (original.value) Object.assign(draft, JSON.parse(JSON.stringify(original.value.settings))); clearTemporary(); removed.value = !!original.value?.settingsRecovered && original.value.hasImage; error.value = '' }
function reset() { Object.assign(draft, defaultBackgroundSettings(scope.value.kind === 'character')); clearTemporary(); removed.value = !!original.value?.hasImage }
function changeInheritance(inherit) {
  if (!inherit && draft.inherit && globalOriginal.value) {
    // 首次独立调整从应用材质开始，已有的角色参数切回时仍保留。
    const defaults = defaultBackgroundSettings()
    if (JSON.stringify(draft.light) === JSON.stringify(defaults.light) && JSON.stringify(draft.dark) === JSON.stringify(defaults.dark)) Object.assign(draft, JSON.parse(JSON.stringify(globalOriginal.value.settings)))
  }
  draft.inherit = inherit
}
async function load() {
  if (!background || busy.value) return
  const version = ++request; loading.value = true; error.value = ''
  try { const [value, global] = await Promise.all([background.load(scope.value, true), background.load({kind:'app'})]); if (!disposed && version === request) { original.value = value; globalOriginal.value = global; cancel() } }
  catch (failure) { if (!disposed && version === request) error.value = tr(failure.message) }
  finally { if (!disposed && version === request) loading.value = false }
}
async function changeScope(kind) {
  if (busy.value || loading.value) return
  if (kind === scope.value.kind) { switchTo.value = null; return }
  if (dirty.value) { switchTo.value = kind; return }
  switchTo.value = null
  scope.value = kind === 'character' && props.agent?.id ? { kind, agentId: props.agent.id } : { kind: 'app' }
  original.value = null; clearTemporary(); removed.value = false; await load()
}
function applyPreset(preset) { const { id, label, ...values } = preset; Object.assign(material.value, values, { brightness: 1, saturation: 1, contrast: 1 }); draft.enabled = true }
function isPreset(preset) { return draft.enabled && ['opacity','blur','overlayOpacity','surfaceOpacity','surfaceBlur'].every(key => Math.abs(material.value[key] - preset[key]) < .001) }
function removeImage() { clearTemporary(); removed.value = !!original.value?.hasImage }
async function importImage(event) {
  const file = event.target.files?.[0]; event.target.value = ''; if (!file || busy.value) return
  busy.value = true; error.value = ''; const version = request
  let source = ''
  try {
    if (!['image/png','image/jpeg','image/webp'].includes(file.type)) throw new Error('请选择 PNG、JPEG 或 WebP 图片')
    if (file.size > 32 * 1024 * 1024) throw new Error('图片太大了，请换一张小一些的图片')
    source = URL.createObjectURL(file); const image = new Image(); image.src = source; await image.decode()
    if (image.naturalWidth * image.naturalHeight > 64_000_000) throw new Error('图片太大了，请换一张小一些的图片')
    const ratio = Math.min(1, 1920 / Math.max(image.naturalWidth, image.naturalHeight)), canvas = document.createElement('canvas')
    canvas.width = Math.max(1,Math.round(image.naturalWidth * ratio)); canvas.height = Math.max(1,Math.round(image.naturalHeight * ratio))
    const context = canvas.getContext('2d'); if (!context) throw new Error('暂时无法处理图片，请再试一次')
    context.drawImage(image, 0, 0, canvas.width, canvas.height)
    const blob = await new Promise(resolve => canvas.toBlob(resolve,'image/webp',.88))
    if (!blob || blob.size > 8 * 1024 * 1024) throw new Error('图片太大了，请换一张小一些的图片')
    const bytes = Array.from(new Uint8Array(await blob.arrayBuffer()))
    if (!disposed && version === request) { clearTemporary(); pendingImage.value = { mime: blob.type, bytes }; temporaryUrl.value = URL.createObjectURL(blob); removed.value = false; draft.enabled = true; draft.inherit = false }
  } catch (failure) { if (!disposed && version === request) error.value = tr(failure instanceof DOMException ? '图片无法打开，请换一张试试' : failure.message || '图片无法打开，请换一张试试') }
  finally { if (source) URL.revokeObjectURL(source); if (!disposed) busy.value = false }
}
async function save() {
  if (!background || !original.value || busy.value || !dirty.value) return
  busy.value = true; error.value = ''; const version = request
  try { const value = await background.save({ scope: scope.value, expectedRevision: original.value.revision, settings: draft, image: pendingImage.value, removeImage: removed.value }); if (!disposed && version === request) { original.value = value; cancel(); appMessage.success(tr('背景已保存')) } }
  catch (failure) { if (!disposed && version === request) error.value = tr(failure.message) }
  finally { if (!disposed) busy.value = false }
}
function startDrag(event) { if (!previewImage.value || event.button !== 0 || busy.value || loading.value || scope.value.kind === 'character' && draft.inherit) return; event.preventDefault(); drag = { id:event.pointerId,x:event.clientX,y:event.clientY,px:material.value.positionX,py:material.value.positionY,rect:event.currentTarget.getBoundingClientRect() }; event.currentTarget.setPointerCapture(event.pointerId) }
function moveDrag(event) { if (!drag || drag.id !== event.pointerId) return; material.value.positionX = Math.round(Math.max(0,Math.min(100,drag.px - (event.clientX-drag.x)/drag.rect.width*100))); material.value.positionY = Math.round(Math.max(0,Math.min(100,drag.py - (event.clientY-drag.y)/drag.rect.height*100))); draft.enabled = true }
function endDrag(event) { if (drag?.id === event.pointerId) { drag = null; if (event.currentTarget.hasPointerCapture(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId) } }
watch(() => [props.agent?.id, busy.value], () => { if (!busy.value && scope.value.kind === 'character' && scope.value.agentId !== props.agent?.id) { request++; scope.value = {kind:'app'}; original.value = null; clearTemporary(); load() } })
onMounted(load)
onBeforeUnmount(() => { disposed = true; request++; clearTemporary() })
</script>
