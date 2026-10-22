<template>
  <div class="remote-model-catalog">
    <RemoteFeedback :message="error" @dismiss="error = ''" />
    <div class="model-catalog-toolbar"><RemoteAction icon="refresh" :busy="fetching" :disabled="disabled || !online || providerNeedsSave" @click="fetchModels">{{ t('获取模型列表') }}</RemoteAction><small v-if="loaded && models.length">{{ t('{0} 个可用模型', [models.length]) }}</small></div>
    <p v-if="providerNeedsSave" class="model-catalog-hint">{{ t('先保存连接，再获取模型列表。') }}</p>
    <template v-if="models.length">
      <RemoteSelect v-if="kind === 'models'" :model-value="String(modelValue.modelId || '')" :display-label="modelValue.modelId || undefined" :label="t('选择模型')" :placeholder="t('从列表选择模型')" :options="models.map(id => ({ value: id, label: id }))" :disabled="disabled || fetching" @update:model-value="$emit('select', $event)" />
      <details v-else class="model-catalog-results"><summary>{{ t('查看可用模型') }}<RemoteIcon name="down" /></summary><label class="model-catalog-search"><RemoteIcon name="search" /><input v-model="search" type="search" :placeholder="t('搜索模型')" :aria-label="t('搜索模型')" /></label><ul><li v-for="id in visibleModels" :key="id">{{ id }}</li></ul><p v-if="!visibleModels.length" class="model-catalog-hint">{{ t('没有找到对应内容') }}</p></details>
    </template>
    <p v-else-if="loaded" class="model-catalog-hint">{{ t('列表为空，可手动填写模型 ID。') }}</p>
  </div>
</template>

<script setup>
import { computed, ref, watch, onBeforeUnmount } from 'vue'
import RemoteAction from './RemoteAction.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteSelect from './RemoteSelect.vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { remoteState } from '../../stores/remote'
import { t } from '../../i18n/remote'

const props = defineProps({ kind: String, modelValue: { type: Object, default: () => ({}) }, recordId: [String, Number], connectionDirty: Boolean, disabled: Boolean })
defineEmits(['select'])
const { read, guard } = useRemoteManagement()
const fetching = ref(false), loaded = ref(false), error = ref(''), models = ref([]), search = ref('')
const online = computed(() => remoteState.connection === 'online')
const providerNeedsSave = computed(() => props.kind === 'providers' && (!props.recordId || props.connectionDirty))
const visibleModels = computed(() => models.value.filter(id => id.toLowerCase().includes(search.value.trim().toLowerCase())))
let generation = 0, disposed = false
// 只观察连接参数；选中模型或修改名称不使已经获取的列表消失，密钥也不进入本机存储。
const connectionKey = computed(() => JSON.stringify([props.kind, props.recordId, props.connectionDirty,
  ...['providerConnectionId', 'connectionOverride', 'provider', 'connectionProtocol', 'connectionAuthMode', 'apiBase', 'apiKey'].map(key => props.modelValue[key])]))
watch(connectionKey, () => { generation++; models.value = []; loaded.value = false; error.value = ''; search.value = '' }, { flush: 'sync' })
onBeforeUnmount(() => { disposed = true; generation++ })

async function fetchModels() {
  if (fetching.value || props.disabled || !online.value || providerNeedsSave.value) return
  const request = generation, form = { ...props.modelValue }
  fetching.value = true; loaded.value = false; models.value = []; error.value = ''
  try {
    let values
    if (props.kind === 'providers' || form.providerConnectionId && !form.connectionOverride) {
      values = await read('providers.models', { id: props.kind === 'providers' ? props.recordId : form.providerConnectionId })
    } else {
      const body = { provider: form.provider || 'openai', apiKey: form.connectionAuthMode === 'none' ? '' : form.apiKey || '', connectionOverride: !!form.connectionOverride }
      if (form.apiBase?.trim()) body.apiBase = form.apiBase.trim()
      if (form.connectionProtocol) body.protocol = form.connectionProtocol
      values = await read('models.remote', { body })
    }
    guard()
    if (disposed || request !== generation) return
    if (!Array.isArray(values) || values.some(id => typeof id !== 'string')) throw new Error(t('暂时没能读取模型列表，请重试。'))
    models.value = [...new Set(values.map(id => id.trim()).filter(Boolean))]
    loaded.value = true
  } catch (failure) { if (!disposed && request === generation) error.value = failure.message }
  finally { if (!disposed) fetching.value = false }
}
</script>

<style scoped>
.remote-model-catalog{min-width:0;margin-top:9px}
.model-catalog-toolbar{display:flex;align-items:center;gap:10px;flex-wrap:wrap}
.model-catalog-toolbar>small{font-size:11px;color:var(--room-muted)}
.model-catalog-hint{font-size:11px;line-height:1.7;color:var(--room-muted);margin:8px 0}
.remote-model-catalog>.remote-select{margin-top:9px}
.model-catalog-results{margin-top:12px;padding:12px;border:1px solid var(--room-line);border-radius:12px;background:var(--room-subtle)}
.model-catalog-results>summary{display:flex;align-items:center;justify-content:space-between;font-size:12px;color:var(--room-accent);cursor:pointer;list-style:none;min-height:28px}
.model-catalog-results>summary>svg{width:14px;height:14px}
.model-catalog-search{display:flex;align-items:center;gap:8px;margin-top:10px}
.model-catalog-search>svg{width:16px;height:16px;flex:none;color:var(--room-muted)}
.model-catalog-search>input{min-width:0;width:100%;min-height:40px;font:inherit;font-size:12px;padding:8px 10px;border:1px solid var(--room-line);border-radius:9px;color:var(--room-ink);background:var(--room-surface)}
.model-catalog-results>ul{max-height:240px;overflow:auto;overscroll-behavior:contain;list-style:none;padding:0;margin:10px 0 0}
.model-catalog-results li{padding:8px 2px;font-size:12px;line-height:1.6;overflow-wrap:anywhere;border-bottom:1px solid var(--room-line)}
.model-catalog-results li:last-child{border:0}
</style>
