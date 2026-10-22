<template>
  <div ref="editorRoot" class="model-editor" @invalid.capture="revealInvalid">
    <header class="model-editor-hero">
      <span class="model-editor-brand"><BrandIcon :brand="brand" :size="32" /></span>
      <div><small>{{ t(recordId ? '模型配置' : '添加模型') }}</small><h3>{{ modelValue.name || modelValue.modelId || t('为聊天选一个模型') }}</h3><p>{{ modelValue.modelId || t('从服务商列表选择，也可以手动填写') }}</p></div>
      <RemoteIcon class="model-editor-ornament" name="bow" />
    </header>
    <nav class="model-editor-tabs" :aria-label="t('模型配置分区')"><button v-for="item in tabs" :key="item.id" type="button" :class="{ selected: tab === item.id }" :aria-pressed="tab === item.id" :aria-controls="`${panelId}-${item.id}`" @click="changeTab(item.id)"><RemoteIcon :name="item.icon" />{{ t(item.label) }}</button></nav>

    <section v-show="tab === 'main'" :id="`${panelId}-main`" data-model-panel="main" class="model-editor-panel" :class="{ 'is-active': tab === 'main' }" :aria-label="t('模型')">
      <div class="model-editor-section model-identity-section"><header><span class="model-section-icon"><RemoteIcon name="model-chip" /></span><div><h4>{{ t('模型身份') }}</h4><p>{{ t('给它一个好认的名字') }}</p></div></header>
        <RemoteFields :model-value="modelValue" :fields="pick(['name', 'modelId']).map(field => ({ ...field, label: field.key === 'name' ? '显示名称' : '模型 ID', max: field.key === 'name' ? 100 : field.max }))" :disabled="disabled" @update:model-value="update"><template #modelId><RemoteModelCatalog kind="models" :model-value="modelValue" :record-id="recordId" :disabled="disabled" @select="selectModel" /></template></RemoteFields>
      </div>
      <button type="button" class="model-connection-link" @click="changeTab('connection')"><span class="model-connection-mark"><RemoteIcon name="connection" /></span><span><small>{{ t(sharedConnection ? '共享连接' : '独立连接') }}</small><strong>{{ connectionName }}</strong></span><RemoteIcon name="next" /></button>
      <div class="model-editor-glance"><span><RemoteIcon name="model-thought" />{{ t('思考') }}<strong>{{ reasoningLabel }}</strong></span><span><RemoteIcon name="pulse" />{{ t(modelValue.stream ? '流式回复' : '完整回复') }}</span><span v-if="modelValue.enableVision"><RemoteIcon name="image" />{{ t('图片理解') }}</span><span v-if="modelValue.enableAudioInput"><RemoteIcon name="audio" />{{ t('音频输入') }}</span></div>
      <RemoteFields v-if="has('isDefault')" :model-value="modelValue" :fields="pick(['isDefault'])" :disabled="disabled" @update:model-value="update" />
    </section>

    <section v-show="tab === 'response'" :id="`${panelId}-response`" data-model-panel="response" class="model-editor-panel" :class="{ 'is-active': tab === 'response' }" :aria-label="t('回复与能力')">
      <div v-if="has('reasoningEffort')" class="model-editor-section model-reasoning-section"><header><span class="model-section-icon"><RemoteIcon name="model-thought" /></span><div><h4>{{ t('思考强度') }}</h4><p>{{ t('默认时不额外指定强度') }}</p></div></header><div class="model-reasoning-options" role="radiogroup" :aria-label="t('思考强度')"><button v-for="choice in reasoningChoices" :key="choice.value ?? 'default'" type="button" role="radio" :aria-checked="(modelValue.reasoningEffort ?? null) === choice.value" :class="{ selected: (modelValue.reasoningEffort ?? null) === choice.value }" :disabled="disabled" @click="set('reasoningEffort', choice.value)">{{ choice.label }}</button></div></div>
      <div class="model-editor-section model-sampling-section"><header><span class="model-section-icon"><RemoteIcon name="sampling" /></span><div><h4>{{ t('回复参数') }}</h4><p>{{ t('留空沿用默认，按需微调') }}</p></div></header><div class="model-number-grid"><RemoteModelNumber v-for="item in visibleNumbers" :key="item.key" v-bind="item" :value="modelValue[item.key]" :disabled="disabled" @change="set(item.key, $event)" /></div></div>
      <div class="model-editor-section model-capability-section"><header><span class="model-section-icon"><RemoteIcon name="sparkle" /></span><div><h4>{{ t('输入与输出') }}</h4><p>{{ t('按模型实际支持的能力开启') }}</p></div></header><div class="model-capability-list"><label v-for="item in visibleCapabilities" :key="item.key" class="model-capability" :class="{ enabled: modelValue[item.key] }" :title="t(item.help)"><span class="model-capability-icon"><RemoteIcon :name="item.icon" /></span><span><strong>{{ t(item.label) }}</strong><small>{{ t(item.help) }}</small></span><RoomSwitch :model-value="!!modelValue[item.key]" :label="t(item.label)" :disabled="disabled" @update:model-value="set(item.key, $event)" /></label></div></div>
      <details v-if="pick(['maxReActTurns', 'reasoningDialect']).length" class="model-advanced"><summary><RemoteIcon name="settings" /><span>{{ t('更多回复设置') }}</span><RemoteIcon name="down" /></summary><RemoteFields :model-value="modelValue" :fields="pick(['maxReActTurns', 'reasoningDialect']).map(field => field.key === 'maxReActTurns' ? { ...field, min: 1, label: '工具调用轮数' } : field)" :disabled="disabled" @update:model-value="update" /></details>
    </section>

    <section v-show="tab === 'connection'" :id="`${panelId}-connection`" data-model-panel="connection" class="model-editor-panel" :class="{ 'is-active': tab === 'connection' }" :aria-label="t('连接')">
      <div class="model-editor-section model-source-section"><header><span class="model-section-icon"><RemoteIcon name="connection" /></span><div><h4>{{ t('服务商连接') }}</h4><p>{{ t('复用已保存的连接，或单独配置') }}</p></div></header><RemoteFields :model-value="modelValue" :fields="pick(['providerConnectionId']).map(field => ({ ...field, label: '选择连接' }))" :disabled="disabled" @update:model-value="update" /><RemoteFields v-if="modelValue.providerConnectionId" :model-value="modelValue" :fields="pick(['connectionOverride']).map(field => ({ ...field, label: '单独设置此模型的连接' }))" :disabled="disabled" @update:model-value="update" /><div v-if="sharedConnection" class="model-shared-note"><RemoteIcon name="shield" /><span>{{ t('使用服务商中保存的地址与密钥') }}</span></div></div>
      <div v-show="!sharedConnection" class="model-editor-section model-credentials-section"><header><span class="model-section-icon"><RemoteIcon name="shield" /></span><div><h4>{{ t('连接信息') }}</h4></div></header><RemoteFields :model-value="modelValue" :fields="pick(['provider', 'connectionProtocol', 'connectionAuthMode', 'apiBase', 'apiKey']).map(field => ({ ...field, label: ({ provider: '服务类型', connectionProtocol: 'API 协议', connectionAuthMode: '授权方式', apiBase: 'API 地址', apiKey: 'API 密钥' })[field.key] }))" :disabled="disabled" @update:model-value="update" /></div>
      <details v-if="pick(['wireApi', 'providerType', 'brand']).length" class="model-advanced"><summary><RemoteIcon name="settings" /><span>{{ t('接口与兼容') }}</span><RemoteIcon name="down" /></summary><RemoteFields :model-value="modelValue" :fields="pick(['wireApi', 'providerType', 'brand'])" :disabled="disabled" @update:model-value="update" /></details>
      <details v-if="remaining.length" class="model-advanced" :open="remaining.some(field => field.required) || undefined"><summary><RemoteIcon name="settings" /><span>{{ t('其他参数') }}</span><RemoteIcon name="down" /></summary><RemoteFields :model-value="modelValue" :fields="remaining" :disabled="disabled" @update:model-value="update" /></details>
    </section>
  </div>
</template>

<script setup>
import { computed, ref, useId, nextTick } from 'vue'
import BrandIcon from '../companion/BrandIcon.vue'
import RoomSwitch from '../companion/RoomSwitch.vue'
import RemoteFields from './RemoteFields.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteModelCatalog from './RemoteModelCatalog.vue'
import RemoteModelNumber from './RemoteModelNumber.vue'
import { resolveModelBrand, resolveProviderBrand } from '../../utils/modelBrands'
import { t, remoteChoiceLabel } from '../../i18n/remote'

const props = defineProps({ modelValue: { type: Object, required: true }, fields: { type: Array, default: () => [] }, recordId: [String, Number], disabled: Boolean })
const emit = defineEmits(['update:modelValue'])
const tab = ref(props.recordId ? 'response' : 'main'), panelId = `model-panel-${useId()}`
const editorRoot = ref(null)
const tabs = [{ id: 'main', label: '模型', icon: 'model-chip' }, { id: 'response', label: '调参', icon: 'sampling' }, { id: 'connection', label: '连接', icon: 'connection' }]
const brand = computed(() => resolveModelBrand(props.modelValue.modelId) || resolveProviderBrand(props.modelValue.provider, props.modelValue.apiBase))
const sharedConnection = computed(() => !!props.modelValue.providerConnectionId && !props.modelValue.connectionOverride)
const connectionName = computed(() => {
  const id = props.modelValue.providerConnectionId
  if (sharedConnection.value) return props.fields.find(field => field.key === 'providerConnectionId')?.choices?.find(choice => String(choice.value) === String(id))?.label || t('已保存的连接')
  if (props.modelValue.apiBase) { try { return new URL(props.modelValue.apiBase).host || t('自定义连接') } catch { return t('自定义连接') } }
  return props.modelValue.provider || t('选择服务商连接')
})
const reasoningLabel = computed(() => props.modelValue.reasoningEffort == null ? t('默认') : remoteChoiceLabel(props.modelValue.reasoningEffort))
const reasoningChoices = computed(() => [{ value: null, label: t('默认') }, ...(props.fields.find(field => field.key === 'reasoningEffort')?.choices || []).map(choice => {
  const value = typeof choice === 'object' ? choice.value : choice
  return { value, label: typeof choice === 'object' && choice.label ? t(choice.label) : remoteChoiceLabel(value) }
})])
const numbers = [
  { key: 'temperature', label: '温度', help: '较低更稳，较高更灵活', icon: 'sampling', min: 0, max: 2 },
  { key: 'topP', label: 'Top P', help: '控制用词的选择范围', icon: 'filter', min: 0, max: 1 },
  { key: 'maxTokens', label: '最大输出', help: '一次回复的长度上限', icon: 'reply-length', min: 1, unit: 'Token', integer: true },
  { key: 'contextWindowTokens', label: '上下文窗口', help: '可使用的完整上下文', icon: 'context-window', min: 1, unit: 'Token', integer: true },
]
const capabilities = [
  { key: 'enableVision', label: '图片理解', help: '允许模型读取图片', icon: 'image' },
  { key: 'enableAudioInput', label: '音频输入', help: '使用模型原生音频能力', icon: 'audio' },
  { key: 'stream', label: '流式回复', help: '边生成，边显示', icon: 'pulse' },
  { key: 'returnNativeReasoning', label: '显示思考', help: '请求模型返回思考摘要', icon: 'model-thought' },
]
const visibleNumbers = computed(() => numbers.filter(item => has(item.key)))
const visibleCapabilities = computed(() => capabilities.filter(item => has(item.key)))
const knownKeys = new Set(['name', 'modelId', 'isDefault', 'reasoningEffort', 'providerConnectionId', 'connectionOverride', 'provider', 'connectionProtocol', 'connectionAuthMode', 'apiBase', 'apiKey', 'maxReActTurns', 'reasoningDialect', 'wireApi', 'providerType', 'brand', ...numbers.map(item => item.key), ...capabilities.map(item => item.key)])
const remaining = computed(() => props.fields.filter(field => !knownKeys.has(field.key)))
function has(key) { return props.fields.some(field => field.key === key) }
function pick(keys) { return keys.flatMap(key => props.fields.filter(field => field.key === key)) }
function update(value) { emit('update:modelValue', value) }
function set(key, value) { update({ ...props.modelValue, [key]: value }) }
function selectModel(id) { update({ ...props.modelValue, modelId: id, ...(!props.modelValue.name?.trim() ? { name: id } : {}) }) }
async function changeTab(value) {
  if (tab.value === value) return
  tab.value = value
  await nextTick()
  const root = editorRoot.value, scroller = root?.closest('.room-drawer-content')
  if (scroller) scroller.scrollTop = Math.max(0, scroller.scrollTop + root.getBoundingClientRect().top - scroller.getBoundingClientRect().top)
}
async function revealInvalid(event) {
  const input = event.target, panel = input.closest('[data-model-panel]')
  // 浏览器校验遇到其他分区的必填项时，先切过去，避免焦点落在隐藏输入上。
  if (panel && (panel.dataset.modelPanel !== tab.value || input.closest('.model-advanced:not([open])'))) {
    event.preventDefault(); tab.value = panel.dataset.modelPanel
    const fold = input.closest('details'); if (fold) fold.open = true
    await nextTick(); input.focus()
  }
}
</script>

<style scoped>
.model-editor{--model-soft:color-mix(in srgb,var(--room-accent) 8%,var(--room-surface));min-width:0;padding-top:2px}
.model-editor-hero{position:relative;display:flex;align-items:center;gap:13px;min-height:99px;padding:17px 15px;margin-bottom:15px;border:1px solid color-mix(in srgb,var(--room-accent) 25%,var(--room-line));border-radius:20px 20px 20px 7px;background:linear-gradient(115deg,var(--model-soft),var(--room-surface));overflow:hidden}
.model-editor-brand{display:grid;place-items:center;flex:none;width:54px;height:54px;border:1px solid var(--room-line);border-radius:17px;background:var(--room-surface);box-shadow:0 4px 12px #23364b08}
.model-editor-hero>div{position:relative;z-index:1;flex:1;min-width:0}
.model-editor-hero small{font-size:10px;color:var(--room-accent);letter-spacing:.04em}
.model-editor-hero h3{font-size:19px;line-height:1.5;font-weight:650;margin:3px 0;overflow-wrap:anywhere;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden}
.model-editor-hero p{font-size:11px;color:var(--room-muted);margin:0;line-height:1.7;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.model-editor-ornament{position:absolute;width:83px;height:83px;right:-17px;top:-14px;transform:rotate(17deg);color:var(--room-accent);opacity:.07}
.model-editor-tabs{position:sticky;top:0;z-index:3;display:grid;grid-template-columns:1fr 1.3fr 1fr;gap:4px;padding:5px;border:1px solid var(--room-line);border-radius:14px;background:var(--room-subtle);margin-bottom:18px;box-shadow:0 3px 10px #24374c05}
.model-editor-tabs>button{display:flex;align-items:center;justify-content:center;gap:6px;min-height:42px;padding:8px 5px;border:1px solid transparent;border-radius:10px;background:transparent;color:var(--room-muted);font:inherit;font-size:12px;cursor:pointer;transition:background .16s,color .16s,transform .16s}
.model-editor-tabs>button>svg{width:17px;height:17px;flex:none}
.model-editor-tabs>button.selected{border-color:var(--room-line);background:var(--room-surface);color:var(--room-accent);font-weight:600;box-shadow:0 2px 6px #26384b08}
.model-editor-tabs>button:active{transform:scale(.97)}
.model-editor-tabs>button:focus-visible,.model-connection-link:focus-visible,.model-reasoning-options>button:focus-visible{outline:2px solid var(--room-accent);outline-offset:2px}
.model-editor-panel.is-active{animation:model-panel-in .16s ease}
.model-editor-section{margin:0 0 18px;padding:15px 14px;border:1px solid var(--room-line);border-radius:17px;background:var(--room-surface);min-width:0}
.model-editor-section>header{display:flex;align-items:center;gap:10px;margin-bottom:14px}
.model-editor-section>header>div{min-width:0;flex:1}
.model-section-icon{display:grid;place-items:center;width:33px;height:33px;flex:none;border-radius:10px;background:var(--model-soft);color:var(--room-accent)}
.model-section-icon>svg{width:21px;height:21px}
.model-editor-section h4{font-size:13px;line-height:1.5;font-weight:650;margin:0}
.model-editor-section header p{font-size:10px;line-height:1.6;color:var(--room-muted);margin:3px 0 0}
.model-editor-section :deep(.remote-fields){margin:0;gap:16px}
.model-editor-section :deep(.remote-field>span){font-size:11px;color:var(--room-muted)}
.model-editor-section :deep(.remote-field input),.model-editor-section :deep(.remote-select){min-height:46px;border-radius:11px;font-size:13px;background:var(--room-subtle)}
.model-editor-section :deep(input:user-invalid){border-color:var(--room-danger,#b6506b)}
.model-identity-section :deep(.remote-field input[required]){font-weight:500;color:var(--room-ink)}
.model-identity-section :deep(.remote-model-catalog){padding-top:2px;margin-top:2px}
.model-connection-link{display:flex;align-items:center;gap:11px;width:100%;min-height:75px;padding:12px 13px;border:1px dashed color-mix(in srgb,var(--room-accent) 35%,var(--room-line));border-radius:15px;background:var(--model-soft);color:var(--room-ink);font:inherit;text-align:left;cursor:pointer}
.model-connection-mark{display:grid;place-items:center;width:36px;height:36px;flex:none;border-radius:11px;background:var(--room-surface);color:var(--room-accent)}
.model-connection-mark>svg{width:20px;height:20px}
.model-connection-link>span:nth-child(2){flex:1;min-width:0}
.model-connection-link small{display:block;font-size:10px;color:var(--room-muted);margin-bottom:4px}
.model-connection-link strong{display:block;font-size:12px;font-weight:550;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.model-connection-link>svg{width:15px;height:15px;color:var(--room-accent);flex:none}
.model-editor-glance{display:flex;gap:10px 13px;flex-wrap:wrap;padding:13px 2px 3px;color:var(--room-muted);font-size:10px;line-height:1.6}
.model-editor-glance>span{display:inline-flex;align-items:center;gap:4px}
.model-editor-glance svg{width:14px;height:14px;color:var(--room-accent)}
.model-editor-glance strong{font-weight:500;color:var(--room-ink);margin-left:2px}
.model-reasoning-section{background:linear-gradient(140deg,color-mix(in srgb,var(--room-accent) 5%,var(--room-surface)),var(--room-surface))}
.model-reasoning-options{display:flex;flex-wrap:wrap;gap:7px}
.model-reasoning-options>button{min-height:38px;flex:1 0 22%;padding:7px;border:1px solid var(--room-line);border-radius:9px;background:var(--room-surface);color:var(--room-muted);font:inherit;font-size:11px;cursor:pointer;transition:background .15s,border-color .15s}
.model-reasoning-options>button.selected{color:var(--room-accent);border-color:var(--room-accent);background:var(--model-soft);font-weight:600;box-shadow:inset 0 -2px color-mix(in srgb,var(--room-accent) 30%,transparent)}
.model-number-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:9px}
.model-sampling-section{border:0;background:var(--room-subtle);padding:14px 10px 10px}
.model-sampling-section>header{padding:0 4px}
.model-capability-section{padding-bottom:5px}
.model-capability-list{display:grid}
.model-capability{display:flex;align-items:center;gap:10px;padding:10px 0;border-bottom:1px solid var(--room-line);cursor:pointer}
.model-capability:last-child{border:0}
.model-capability-icon{display:grid;place-items:center;flex:none;width:33px;height:33px;border-radius:10px;background:var(--room-subtle);color:var(--room-muted);transition:background .15s,color .15s}
.model-capability-icon>svg{width:20px;height:20px}
.model-capability.enabled .model-capability-icon{background:var(--model-soft);color:var(--room-accent)}
.model-capability>span:nth-child(2){flex:1;min-width:0}
.model-capability strong{display:block;font-size:12px;line-height:1.6;font-weight:550}
.model-capability small{display:block;color:var(--room-muted);font-size:10px;line-height:1.6;margin-top:3px}
.model-advanced{margin:0 0 13px;border:1px solid var(--room-line);border-radius:13px;min-width:0}
.model-advanced>summary{display:flex;align-items:center;gap:8px;min-height:47px;padding:10px 13px;list-style:none;font-size:11px;cursor:pointer;color:var(--room-muted)}
.model-advanced>summary>svg{width:16px;height:16px;flex:none}
.model-advanced>summary>span{flex:1}
.model-advanced>summary>svg:last-child{width:13px;transition:transform .16s}
.model-advanced[open]>summary>svg:last-child{transform:rotate(180deg)}
.model-advanced>:deep(.remote-fields){padding:4px 13px 14px;margin:0;gap:14px}
.model-shared-note{display:flex;align-items:center;gap:7px;padding:11px;margin-top:12px;border-radius:10px;background:var(--model-soft);color:var(--room-accent);font-size:11px;line-height:1.7}
.model-shared-note>svg{width:17px;height:17px;flex:none}
.model-source-section :deep(.remote-field.is-switch){margin-top:14px}
.model-editor button:disabled{opacity:.45;cursor:default}
@keyframes model-panel-in{from{opacity:.5;transform:translateY(4px)}to{opacity:1;transform:none}}
@media(prefers-reduced-motion:reduce){.model-editor-panel.is-active{animation:none}.model-editor-tabs>button,.model-reasoning-options>button,.model-capability-icon,.model-advanced>summary>svg:last-child{transition:none}}
:global([data-reduce-motion=true]) .model-editor-panel.is-active{animation:none}
:global([data-reduce-motion=true]) .model-editor-tabs>button,:global([data-reduce-motion=true]) .model-reasoning-options>button,:global([data-reduce-motion=true]) .model-capability-icon,:global([data-reduce-motion=true]) .model-advanced>summary>svg:last-child{transition:none}
</style>
