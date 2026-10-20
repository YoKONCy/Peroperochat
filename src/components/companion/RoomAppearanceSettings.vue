<template>
  <form class="room-appearance-settings" @submit.prevent="save">
    <p v-if="appearanceError" class="room-error" role="alert">{{ appearanceError }} <button type="button" class="room-text-button" @click="reload">重新读取</button></p>
    <fieldset :disabled="appearanceBusy || loading">
      <label class="appearance-label">角色显示</label>
      <div class="appearance-renderers" role="group" aria-label="角色显示方式">
        <button v-for="item in renderers" :key="item.id" type="button" :aria-pressed="draft.displayMode === item.id" @click="draft.displayMode = item.id"><RoomIcon :name="item.icon" /><strong>{{ item.name }}</strong><small>{{ item.note }}</small><RoomIcon v-if="draft.displayMode === item.id" class="appearance-check" name="check" /></button>
      </div>
      <label class="appearance-label">点缀配色<select v-model="draft.palette"><option value="auto">跟随角色</option><option v-for="(palette, id) in appearancePalettes" :key="id" :value="id">{{ palette.label }}</option></select></label>
      <label class="appearance-label">玻璃底色 <output>{{ draft.glass }}%</output><input v-model.number="draft.glass" type="range" min="60" max="94" /></label>
      <label class="appearance-label">静态立绘大小 <output>{{ draft.figureSize }}%</output><input v-model.number="draft.figureSize" type="range" min="44" max="64" /></label>
      <label class="appearance-motion"><span><strong>轻量交互动效</strong><small>系统减少动态效果时优先遵循系统设置</small></span><input v-model="draft.motion" type="checkbox" role="switch" /></label>
      <p class="appearance-note">静态立绘与完整 CG 随角色保存，可在角色管理中设置。Pio 为通用 Live2D 模型，服装选择仅对 Live2D 生效。</p>
      <button type="submit" class="room-primary-button" :disabled="!dirty && !appearanceError">{{ appearanceBusy ? '正在保存…' : '保存显示设置' }}</button>
    </fieldset>
  </form>
</template>
<script setup>
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import RoomIcon from './RoomIcon.vue'
import { appearance, appearanceBusy, appearanceError, appearancePalettes, displayPreferences, loadAppearance, saveAppearance } from '../../stores/appearance'
const draft = reactive({ ...displayPreferences.value })
const loading = ref(!appearance.value)
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
.appearance-label select { display: block; width: 100%; border: 1px solid var(--room-line); border-radius: 12px; background: var(--room-glass); color: inherit; padding: 12px; margin-top: 9px; }
.appearance-label input[type='range'] { width: 100%; display: block; margin-top: 13px; accent-color: var(--room-accent); }
.appearance-renderers { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin: -5px 0 23px; }
.appearance-renderers > button { position: relative; display: flex; flex-direction: column; align-items: flex-start; gap: 7px; text-align: left; border: 1px solid var(--room-line); background: var(--room-glass); border-radius: 16px; padding: 16px 12px; }
.appearance-renderers > button[aria-pressed='true'] { border-color: var(--room-accent); background: var(--room-accent-soft); }
.appearance-renderers strong { font-size: 13px; }.appearance-renderers small { font-size: 10px; line-height: 1.6; color: var(--room-muted); }
.appearance-renderers svg { color: var(--room-accent); }.appearance-renderers .appearance-check { position: absolute; right: 10px; top: 13px; width: 15px; }
.appearance-motion { display: flex; align-items: center; gap: 15px; justify-content: space-between; }.appearance-motion span { display: flex; flex-direction: column; gap: 6px; }.appearance-motion strong { font-size: 12px; font-weight: 500; }.appearance-motion small { font-size: 10px; color: var(--room-muted); }.appearance-motion input { accent-color: var(--room-accent); width: 18px; height: 18px; }
.appearance-note { margin: 22px 0; font-size: 11px; line-height: 1.8; color: var(--room-muted); }.room-primary-button { width: 100%; }
</style>
