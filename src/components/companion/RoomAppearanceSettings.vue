<template>
  <form class="room-appearance-settings" @submit.prevent="save">
    <FeedbackMessage :message="appearanceError" action-label="重试" @action="reload" /><button v-if="appearanceError" type="button" class="room-text-button" @click="reload">重新加载显示偏好</button>
    <fieldset :disabled="appearanceBusy || loading">
      <label class="appearance-label">角色显示</label>
      <div class="appearance-renderers" role="group" aria-label="角色显示方式">
        <button v-for="item in renderers" :key="item.id" type="button" :aria-pressed="draft.displayMode === item.id" @click="draft.displayMode = item.id"><RoomIcon :name="item.icon" /><strong>{{ item.name }}</strong><small>{{ item.note }}</small><RoomIcon v-if="draft.displayMode === item.id" class="appearance-check" name="check" /></button>
      </div>
      <JournalThemePicker local :model-value="draft.palette" @update:model-value="draft.palette = $event" />
      <label class="appearance-label">静态立绘大小 <output>{{ draft.figureSize }}%</output><input v-model.number="draft.figureSize" type="range" min="44" max="64" :style="{ '--range-progress': `${(draft.figureSize - 44) / 20 * 100}%` }" /></label>
      <label class="appearance-motion"><span><strong>轻量交互动效</strong><small>系统减少动态效果时优先遵循系统设置</small></span><RoomSwitch v-model="draft.motion" label="轻量交互动效" /></label>
      <p class="appearance-note">默认房间随时间变化。可在角色资料里更换立绘和背景。</p>
      <button type="submit" class="room-primary-button" :disabled="!dirty && !appearanceError">{{ appearanceBusy ? '正在保存…' : '保存显示设置' }}</button>
    </fieldset>
    <BackgroundEditor :agent="activeAgent" :room-image="roomImage" />
  </form>
</template>
<script setup>
import { computed, onMounted, reactive, ref, watch, inject } from 'vue'
import { appMessage as ElMessage } from '../../services/appFeedback'
import FeedbackMessage from '../FeedbackMessage.vue'
import RoomIcon from './RoomIcon.vue'
import JournalThemePicker from './JournalThemePicker.vue'
import RoomSwitch from './RoomSwitch.vue'
import BackgroundEditor from './BackgroundEditor.vue'
import { activeAgent } from '../../stores/agents'
import { APP_BACKGROUND } from '../../services/appBackground'
import { appearance, appearanceBusy, appearanceError, displayPreferences, loadAppearance, saveAppearance } from '../../stores/appearance'
const draft = reactive({ ...displayPreferences.value })
const loading = ref(!appearance.value)
const background = inject(APP_BACKGROUND, null)
const roomImage = computed(() => background?.roomReady.defaultSource || activeAgent.value?.room_background || (activeAgent.value?.id === 'nana' ? '/companion/moonlit-room.png' : '/companion/sunlit-room.png'))

const renderers = [{ id: 'illustration', icon: 'image', name: '静态立绘', note: '角色专属立绘或 CG' }, { id: 'live2d', icon: 'model', name: 'Live2D', note: '通用 Pio · 支持换装' }]
const dirty = computed(() => Object.keys(draft).some(key => draft[key] !== displayPreferences.value[key]))
watch(displayPreferences, value => Object.assign(draft, value))
async function reload() {
  loading.value = true
  try { await loadAppearance() }
  catch {
    // 读取错误已由共享状态记录，交由顶部提示提供重试入口。
  } finally { loading.value = false }
}
async function save() { try { await saveAppearance({ ...draft }); ElMessage.success('显示设置已保存') } catch (error) { ElMessage.error(error.message) } }
onMounted(() => { if (loading.value) reload() })
</script>
<style scoped>
fieldset { padding: 0; margin: 0; border: 0; min-width: 0; }
.appearance-label { display: block; margin: 0 0 19px; font-size: 12px; color: var(--room-ink); }
.appearance-label output { float: right; color: var(--room-muted); }
.appearance-label .choice-picker { margin-top: 9px; }
.appearance-label input[type='range'] { width: 100%; display: block; margin-top: 6px; }
.appearance-renderers { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin: -5px 0 23px; }
.appearance-renderers > button { position: relative; display: flex; flex-direction: column; align-items: flex-start; gap: 7px; text-align: left; border: 1px solid var(--room-line); background: var(--room-glass); border-radius: 16px; padding: 16px 12px; }
.appearance-renderers > button[aria-pressed='true'] { border-color: var(--room-accent); background: var(--room-accent-soft); }
.appearance-renderers strong { font-size: 13px; }.appearance-renderers small { font-size: 10px; line-height: 1.6; color: var(--room-muted); }
.appearance-renderers svg { color: var(--room-accent); }.appearance-renderers .appearance-check { position: absolute; right: 10px; top: 13px; width: 15px; }
.appearance-motion { display: flex; align-items: center; gap: 15px; justify-content: space-between; }.appearance-motion > span:not(.room-switch) { display: flex; flex-direction: column; gap: 6px; }.appearance-motion strong { font-size: 12px; font-weight: 500; }.appearance-motion small { font-size: 10px; color: var(--room-muted); }
.appearance-note { margin: 22px 0; font-size: 11px; line-height: 1.8; color: var(--room-muted); }.room-primary-button { width: 100%; }
</style>
