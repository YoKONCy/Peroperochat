<template>
  <RoomDrawer :title="`${agent.name} 的衣柜`" :subtitle="displayMode === 'live2d' ? '服装细节预览 · 点选查看上身效果' : '服装设置保留给 Live2D，静态立绘不换装'" :tone="tone" variant="wardrobe" @close="$emit('close')">
    <button v-if="displayMode !== 'live2d'" class="room-wardrobe-display room-secondary-button" type="button" @click="$emit('display', 'live2d')"><RoomIcon name="model" />切换到 Live2D 查看服装</button>
    <FeedbackMessage :message="error" action-label="重试" @action="load" /><button v-if="error" class="room-text-button" @click="load"><RoomIcon name="refresh" />重新打开衣柜</button>
    <p v-if="loading" class="room-empty" role="status">正在打开衣柜…</p>
    <div v-else class="outfit-grid">
      <button v-for="outfit in outfits" :key="outfit.id" :disabled="changing" :aria-pressed="String(selected) === String(textureId(outfit))" :class="{ selected: String(selected) === String(textureId(outfit)) }" @click="choose(outfit)">
        <OutfitThumbnail :texture="outfit.texturePath" :label="outfit.name" /><span class="outfit-name">{{ outfit.name }}</span><span v-if="String(selected) === String(textureId(outfit))" class="outfit-check"><RoomIcon name="check" /></span>
      </button>
    </div>
    <template #footer><button class="room-text-button" :disabled="changing" @click="switchModel">切换角色模型</button></template>
  </RoomDrawer>
</template>

<script setup>
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { appMessage as ElMessage } from '../../services/appFeedback'
import FeedbackMessage from '../FeedbackMessage.vue'
import RoomDrawer from './RoomDrawer.vue'
import RoomIcon from './RoomIcon.vue'
import OutfitThumbnail from './OutfitThumbnail.vue'
import { localInvoke } from '../../services/localChat'
import { getActiveAgentId } from '../../stores/agents'
const props = defineProps({ agent: { type: Object, required: true }, tone: { type: String, default: 'pero' }, texture: { type: [String, Number], default: '' }, displayMode: { type: String, default: 'live2d' } })
const emit = defineEmits(['close', 'display', 'updated'])
const outfits = ref([])
const loading = ref(true)
const changing = ref(false)
const error = ref('')
const selected = ref(props.texture)
watch(() => props.texture, value => { selected.value = value })
let disposed = false
let controller = null
function textureId(outfit) { return Number(outfit.id.match(/index_(\d+)\.json$/)?.[1] || 0) }
async function load() {
  controller?.abort(); controller = new AbortController(); loading.value = true; error.value = ''
  try {
    const response = await fetch('/live2d-widget/model/potion-Maker-Pio/outfits.json', { signal: controller.signal })
    if (!response.ok) throw new Error('衣柜暂时没有打开')
    const result = await response.json()
    if (!Array.isArray(result)) throw new Error('服装目录格式无效')
    if (!disposed) outfits.value = result
  } catch (failure) { if (!disposed && failure.name !== 'AbortError') error.value = failure.message }
  finally { if (!disposed) loading.value = false }
}
async function choose(outfit) {
  if (changing.value) return
  changing.value = true
  try {
    const agentId = props.agent.id, texture = textureId(outfit)
    await localInvoke('local_save_model_preferences', { agentId, modelId: 0, textureId: texture })
    if (disposed || agentId !== getActiveAgentId()) return
    selected.value = texture; emit('updated')
    if (props.displayMode === 'live2d' && window.PPC_LIVE2D_ENABLED && window.WaifuWidget) await window.WaifuWidget.loadModel(0, texture, `换上${outfit.name}啦`, false)
  } catch (failure) { if (!disposed) ElMessage.error(failure.message) }
  finally { if (!disposed) changing.value = false }
}
async function switchModel() {
  if (changing.value) return
  changing.value = true
  try { if (props.displayMode !== 'live2d') { emit('display', 'live2d'); return }; if (!window.WaifuWidget) throw new Error('角色还在准备中'); await window.WaifuWidget.loadOtherModel() }
  catch (failure) { if (!disposed) ElMessage.error(failure.message) }
  finally { if (!disposed) changing.value = false }
}
onMounted(load)
onBeforeUnmount(() => { disposed = true; controller?.abort() })
</script>

<style scoped>
.room-wardrobe-display { display: flex; align-items: center; justify-content: center; gap: 8px; width: 100%; margin-bottom: 14px; }
.outfit-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
.outfit-grid > button { border: 1px solid var(--room-line); background: var(--room-surface); border-radius: 3px 10px 3px 3px; overflow: hidden; text-align: left; padding: 0; display: flex; flex-direction: column; position: relative; font-size: 12px; line-height: 1.5; }
.outfit-grid > button.selected { box-shadow: inset 2px 0 var(--room-accent); border-color: var(--room-accent); background: var(--room-accent-soft); }
.outfit-name { display: flex; align-items: center; min-height: 40px; width: 100%; padding: 9px 11px; font-size: 11px; }
.outfit-grid .outfit-check { display: grid; place-items: center; position: absolute; top: 7px; right: 7px; width: 21px; height: 21px; border: 2px solid white; border-radius: 50%; color: white; background: var(--room-accent); }
.outfit-check svg { width: 13px; height: 13px; }
.outfit-check { animation: room-check-in .3s var(--room-spring); }
.outfit-grid > button { transition: border-color .2s, background .2s, transform .22s var(--room-spring); }
@media (hover: hover) { .outfit-grid > button:hover:not(:disabled) { transform: translateY(-2px); border-color: var(--room-accent-bright); } }
</style>
