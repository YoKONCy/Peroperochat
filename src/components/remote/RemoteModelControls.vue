<template>
  <div class="remote-model-controls" :aria-busy="loading || saving">
    <div class="remote-model-choice"><BrandIcon :brand="resolveModelBrand(settings?.effective?.modelId)" :size="19" fallback="model" /><RemoteSelect :model-value="String(settings?.modelConfigId ?? '')" label="本会话模型" :placeholder="loading ? '读取模型…' : '选择模型'" :options="modelOptions" :disabled="blocked" @update:model-value="selectModel" /></div>
    <div class="remote-effort-choice"><RemoteIcon name="brain" /><RemoteSelect :model-value="settings?.reasoningEffort || ''" label="思考强度" :options="settings?.effortOptions || []" :disabled="blocked" @update:model-value="selectEffort" /></div>
    <button v-if="error" class="room-text-button" type="button" @click="load"><RemoteIcon name="refresh" />重试配置</button>
    <p v-if="error" class="remote-controls-error" role="alert">{{ error }}</p>
    <button v-if="unresolved" class="room-text-button" type="button" @click="$emit('pending')">保存结果待核对</button>
  </div>
</template>
<script setup>
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import RemoteSelect from './RemoteSelect.vue'
import RemoteIcon from './RemoteIcon.vue'
import BrandIcon from '../companion/BrandIcon.vue'
import { resolveModelBrand } from '../../utils/modelBrands'
import { remoteClient } from '../../services/remote'
import { manageRemote } from '../../stores/remote'
import { currentToken, assertToken, sameToken } from '../../stores/mode'
const props = defineProps({ threadId: String, disabled: Boolean })
const emit = defineEmits(['changed', 'capabilities', 'pending', 'busy'])
const token = currentToken(), settings = ref(null), loading = ref(false), saving = ref(false), error = ref(''), unresolved = ref(false)
let serial = 0, disposed = false
const blocked = computed(() => props.disabled || loading.value || saving.value || unresolved.value || !settings.value)
const modelOptions = computed(() => [{ value: '', label: `全局主模型${!settings.value?.modelConfigId && settings.value?.effective?.name ? ` · ${settings.value.effective.name}` : ''}` }, ...(settings.value?.models || []).map(item => ({ value: String(item.id), label: item.name, description: item.modelId }))])
async function load() {
  if (saving.value || !props.threadId || props.disabled) return
  const request = ++serial, id = props.threadId; loading.value = true; error.value = ''
  try { const data = (await remoteClient.modelControls(id, token)).data; assertToken(token, 'remote'); if (!disposed && request === serial && id === props.threadId) { settings.value = data; emit('capabilities', data) } }
  catch (failure) { if (!disposed && request === serial) error.value = failure.message }
  finally { if (!disposed && request === serial) loading.value = false }
}
async function save(modelConfigId, reasoningEffort) {
  if (blocked.value) return
  const id = props.threadId; saving.value = true; error.value = ''; emit('busy', true)
  try {
    assertToken(token, 'remote')
    await manageRemote({ operation: 'thread.save_models', id, params: {}, body: { modelConfigId, reasoningEffort } }, { token })
    assertToken(token, 'remote')
    if (!disposed && id === props.threadId) { settings.value = null; emit('changed'); window.dispatchEvent(new CustomEvent('ppc:thread-model-changed', { detail: { threadId: id, token } })) }
  } catch (failure) { if (!disposed) { error.value = failure.message; unresolved.value = !['not_sent', 'rejected', 'failed'].includes(failure.outcome); if (unresolved.value) emit('pending') } }
  finally { if (!disposed) { saving.value = false; emit('busy', false); if (!unresolved.value) await load() } }
}
function selectModel(value) { const id = value ? Number(value) : null; if (id !== settings.value.modelConfigId) save(id, settings.value.reasoningEffort ?? null) }
function selectEffort(value) { save(settings.value.modelConfigId ?? null, value || null) }
function refresh(event) { if (!document.hidden && (!event?.detail || event.detail.threadId === props.threadId && sameToken(event.detail.token, token))) load() }
watch(() => props.threadId, () => { serial++; settings.value = null; unresolved.value = false; load() }, { immediate: true })
watch(() => props.disabled, value => { if (!value) load() })
onMounted(() => { window.addEventListener('focus', refresh); window.addEventListener('ppc:thread-model-changed', refresh) })
onBeforeUnmount(() => { disposed = true; serial++; emit('busy', false); window.removeEventListener('focus', refresh); window.removeEventListener('ppc:thread-model-changed', refresh) })
</script>
