<template>
  <section class="phase-card owner-avatar-settings">
    <div class="owner-avatar-heading"><RemoteUserAvatar :label="avatar.name || t('你')" /><div><h3>{{ t('用户头像') }}</h3><p>{{ t('和当前 infOS 的其他客户端同步。') }}</p></div></div>
    <RemoteFeedback :message="error" />
    <p v-if="!avatar.available" class="remote-help">{{ t('当前 infOS 暂不支持用户头像，请更新后再试。') }}</p>
    <div class="remote-purpose-actions"><button type="button" class="room-primary-button" :disabled="locked || !avatar.available" @click="fileInput?.click()"><RemoteIcon name="image" />{{ t('更换头像') }}</button><button type="button" class="room-text-button" :disabled="locked || !avatar.version" @click="confirmReset = true">{{ t('恢复默认头像') }}</button><button type="button" class="room-text-button" :disabled="locked" @click="refresh"><RemoteIcon name="refresh" />{{ t('刷新') }}</button></div>
    <input ref="fileInput" type="file" accept="image/png,image/jpeg,image/webp" hidden @change="choose" />
    <p v-if="reading" class="remote-help" role="status">{{ t('正在读取图片…') }}</p>
    <RoomDrawer v-if="image" :title="t('调整用户头像')" :busy="working || avatar.busy" @close="close">
      <RemoteFeedback :message="error" />
      <div ref="viewport" class="owner-crop-stage" role="application" tabindex="0" :aria-label="t('拖动图片调整位置，也可以使用方向键。')" @pointerdown="startDrag" @pointermove="moveDrag" @pointerup="endDrag" @pointercancel="endDrag" @lostpointercapture="drag = null" @keydown="moveKey"><img :src="imageUrl" :style="cropStyle" alt="" draggable="false" /><div class="owner-crop-mask" /></div>
      <p class="remote-help">{{ t('拖动图片调整位置，也可以使用方向键。') }}</p>
      <label class="owner-crop-zoom"><span>{{ t('缩放') }}</span><input :value="crop.zoom" type="range" min="1" max="3" step="0.01" :aria-label="t('缩放')" :disabled="locked" @input="update({ zoom: Number($event.target.value) })" /><output>{{ Math.round(crop.zoom * 100) }}%</output></label>
      <template #footer><div class="owner-crop-footer"><button type="button" class="room-text-button" :disabled="locked" @click="update({ zoom: 1, x: 0, y: 0 })">{{ t('居中') }}</button><span /><button type="button" class="room-secondary-button" :disabled="working || avatar.busy" @click="close">{{ t('取消') }}</button><button type="button" class="room-primary-button" :disabled="locked" @click="save">{{ t(working || avatar.busy ? '正在保存…' : '保存头像') }}</button></div></template>
    </RoomDrawer>
    <RoomDrawer v-if="confirmReset" :title="t('恢复默认头像？')" :busy="working || avatar.busy" @close="confirmReset = false"><RemoteFeedback :message="error" /><div class="remote-purpose-actions"><button type="button" class="room-secondary-button" :disabled="working || avatar.busy" @click="confirmReset = false">{{ t('取消') }}</button><button type="button" class="room-primary-button" :disabled="locked" @click="reset">{{ t('恢复默认头像') }}</button></div></RoomDrawer>
  </section>
</template>
<script setup>
import { computed, ref, shallowRef, onMounted, onBeforeUnmount } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteUserAvatar from './RemoteUserAvatar.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import { remoteUserAvatar as avatar, refreshRemoteUserAvatar, saveRemoteUserAvatar } from '../../services/remoteUserAvatar'
import { clampAvatarCrop, avatarCropRect, avatarCropStyle } from '../../utils/userAvatarCrop'
import { currentToken, assertToken } from '../../stores/mode'
import { remoteState, refreshRemotePendingRequests } from '../../stores/remote'
import { appMessage } from '../../services/appFeedback'
import { t as translate } from '../../i18n/remote'
const props = defineProps({ disabled: Boolean })
const emit = defineEmits(['pending'])
const t = key => translate(key, [], 'user-avatar')
const token = currentToken(), fileInput = ref(null), viewport = ref(null), image = shallowRef(null), imageUrl = ref(''), reading = ref(false), working = ref(false), error = ref(''), confirmReset = ref(false)
const crop = ref({ width: 1, height: 1, zoom: 1, x: 0, y: 0 })
const cropStyle = computed(() => avatarCropStyle(crop.value))
const locked = computed(() => props.disabled || working.value || reading.value || avatar.busy || remoteState.connection !== 'online')
let disposed = false, version = 0, drag = null
function guard(request = version) { try { assertToken(token, 'remote'); return !disposed && request === version } catch { return false } }
function close() { if (working.value || avatar.busy) return; version++; image.value = null; drag = null; if (imageUrl.value) URL.revokeObjectURL(imageUrl.value); imageUrl.value = ''; error.value = '' }
function update(value) { crop.value = clampAvatarCrop({ ...crop.value, ...value }) }
async function fail(failure) {
  if (!guard()) return
  error.value = t(failure instanceof DOMException ? '图片无法读取，请重新选择' : failure.message || '操作失败，请重试')
  if (failure.outcome === 'unknown') { await refreshRemotePendingRequests().catch(() => {}); if (guard()) { working.value = false; close(); confirmReset.value = false; emit('pending') } }
}
async function refresh() { try { await refreshRemoteUserAvatar(true); if (guard()) error.value = '' } catch (failure) { await fail(failure) } }
async function choose(event) {
  const file = event.target.files?.[0]; event.target.value = ''
  if (!file || locked.value) return
  close(); const request = ++version; reading.value = true
  let url = ''
  try {
    if (!['image/png', 'image/jpeg', 'image/webp'].includes(file.type)) throw new Error('请选择 PNG、JPEG 或 WebP 图片')
    if (!file.size || file.size > 15 * 1024 * 1024) throw new Error('请选择不超过 15MB 的图片')
    url = URL.createObjectURL(file); const loaded = new Image(); loaded.src = url; await loaded.decode()
    if (!loaded.naturalWidth || !loaded.naturalHeight || loaded.naturalWidth * loaded.naturalHeight > 64_000_000) throw new Error('图片尺寸过大，请选择较小图片')
    if (!guard(request)) return
    crop.value = { width: loaded.naturalWidth, height: loaded.naturalHeight, zoom: 1, x: 0, y: 0 }
    image.value = loaded; imageUrl.value = url; url = ''
  } catch (failure) { if (!disposed && request === version) await fail(failure) }
  finally { if (url) URL.revokeObjectURL(url); if (!disposed && request === version) reading.value = false }
}
async function save() {
  if (locked.value || !image.value) return
  working.value = true; error.value = ''
  try {
    const canvas = document.createElement('canvas'); canvas.width = canvas.height = 512
    const context = canvas.getContext('2d'); if (!context) throw new Error('裁切失败，请重试')
    const rect = avatarCropRect(crop.value); context.drawImage(image.value, rect.x, rect.y, rect.size, rect.size, 0, 0, 512, 512)
    const blob = await new Promise(resolve => canvas.toBlob(resolve, 'image/png'))
    if (!blob) throw new Error('裁切失败，请重试')
    if (!guard()) return
    await saveRemoteUserAvatar(blob)
    if (guard()) { working.value = false; close(); appMessage.success(t('头像已更新')) }
  } catch (failure) { await fail(failure) }
  finally { if (!disposed) working.value = false }
}
async function reset() {
  if (locked.value) return
  working.value = true; error.value = ''
  try { await saveRemoteUserAvatar(); if (guard()) { confirmReset.value = false; appMessage.success(t('已恢复默认头像')) } }
  catch (failure) { await fail(failure) }
  finally { if (!disposed) working.value = false }
}
function startDrag(event) { if (locked.value || event.button !== 0) return; event.preventDefault(); event.currentTarget.focus({ preventScroll: true }); event.currentTarget.setPointerCapture(event.pointerId); drag = { id: event.pointerId, x: event.clientX, y: event.clientY, px: crop.value.x, py: crop.value.y, size: viewport.value.clientWidth } }
function moveDrag(event) { if (drag?.id === event.pointerId && !locked.value) update({ x: drag.px + (event.clientX - drag.x) / drag.size, y: drag.py + (event.clientY - drag.y) / drag.size }) }
function endDrag(event) { if (drag?.id !== event.pointerId) return; drag = null; if (event.currentTarget.hasPointerCapture(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId) }
function moveKey(event) { const delta = { ArrowLeft: [-.01, 0], ArrowRight: [.01, 0], ArrowUp: [0, -.01], ArrowDown: [0, .01] }[event.key]; if (delta && !locked.value) { event.preventDefault(); update({ x: crop.value.x + delta[0], y: crop.value.y + delta[1] }) } }
onMounted(() => refreshRemoteUserAvatar().catch(() => {}))
onBeforeUnmount(() => { disposed = true; version++; if (imageUrl.value) URL.revokeObjectURL(imageUrl.value) })
</script>
<style scoped>
.owner-avatar-heading { display: flex; gap: 15px; align-items: center; margin-bottom: 15px; }
.owner-avatar-heading > .character-avatar { width: 66px; height: 66px; flex: none; border: 2px solid var(--ui-paper); border-radius: 22px; box-shadow: 0 0 0 2px var(--ui-contour), var(--ui-shadow); }
.owner-avatar-heading h3 { margin: 0 0 6px; font-size: var(--ui-text); }
.owner-avatar-heading p { margin: 0; font-size: var(--ui-caption); line-height: 1.7; color: var(--ui-muted); }
.owner-crop-stage { width: min(240px, 100%); aspect-ratio: 1; position: relative; overflow: hidden; margin: 15px auto; border: 2px solid var(--ui-accent); border-radius: 23px; background: var(--ui-inset); cursor: grab; touch-action: none; }
.owner-crop-stage:active { cursor: grabbing; }
.owner-crop-stage > img { position: absolute; max-width: none; transform: translate(-50%, -50%); pointer-events: none; user-select: none; }
.owner-crop-mask { position: absolute; inset: 0; border-radius: 50%; box-shadow: 0 0 0 90px color-mix(in srgb, var(--ui-inset) 65%, transparent); border: 1px dashed var(--ui-accent); pointer-events: none; }
.owner-crop-zoom { display: flex; align-items: center; gap: 12px; margin: 18px 0; font-size: var(--ui-label); }
.owner-crop-zoom input { flex: 1; min-width: 0; }
.owner-crop-zoom output { min-width: 40px; color: var(--ui-muted); font-size: var(--ui-caption); }
.owner-crop-footer { display: flex; align-items: center; gap: 10px; }
.owner-crop-footer > span { flex: 1; }
.owner-avatar-settings button, .owner-crop-footer > button { display: inline-flex; align-items: center; justify-content: center; gap: 6px; }
.owner-avatar-settings button > svg { width: 16px; height: 16px; }
</style>
