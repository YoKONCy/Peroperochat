<template>
  <div class="phase2 phase-settings-form">
    <template v-if="kind === 'assignments'">
      <section class="phase-assignment-main"><small><RemoteIcon name="model" />{{ t('默认聊天模型') }}</small><h3>{{ currentModel || t('沿用默认模型') }}</h3><p>{{ t('没有单独指定时，就由它来接话。') }}</p><RemoteFields :model-value="modelValue" :fields="pick(['model.main'])" :disabled="disabled" @update:model-value="update" /></section>
      <h3 class="phase-section-title"><RemoteIcon name="people" />{{ t('角色单独设置') }}<small>{{ t('可选') }}</small></h3>
      <section class="phase-card"><div v-for="field in agentFields" :key="field.key" class="phase-agent-assignment"><RemoteAvatar :id="field.key.slice('model.agent.'.length)" :label="agentName(field.key)" fit="contain" /><div><strong>{{ agentName(field.key) }}</strong><small v-if="!modelValue[field.key]">{{ t('当前沿用：{0}', [currentModel || t('沿用默认模型')]) }}</small><RemoteFields :model-value="modelValue" :fields="[{ ...field, label: t('模型'), help: undefined }]" :disabled="disabled" @update:model-value="update" /></div></div></section>
      <section class="phase-card"><h3><RemoteIcon name="image" />{{ t('视觉理解') }}</h3><RemoteFields :model-value="modelValue" :fields="pick(['multimodalRelay.enabled', 'multimodalRelay.modelConfigId', 'multimodalRelay.detail'])" :disabled="disabled" @update:model-value="update" /></section>
      <details class="phase-fold"><summary><RemoteIcon name="leaf" />{{ t('后台整理与其他分工') }}<RemoteIcon name="down" /></summary><RemoteFields :model-value="modelValue" :fields="fields.filter(field => field.key.startsWith('model.task.') || field.key.startsWith('embedding.') || field.key.startsWith('memory.'))" :disabled="disabled" @update:model-value="update" /></details>
    </template>
    <template v-else-if="kind === 'proactive'">
      <p class="phase-context"><RemoteIcon name="people" />{{ t('这是所有已启用角色共用的陪伴方式。') }}</p>
      <div class="phase-modes" role="radiogroup" :aria-label="t('主动陪伴')"><button v-for="mode in modes" :key="mode.value" type="button" role="radio" :aria-checked="modelValue.mode === mode.value" :disabled="disabled" @click="set('mode', mode.value)"><span><RemoteIcon :name="mode.icon" /></span><div><strong>{{ t(mode.label) }}</strong><small>{{ t(mode.description) }}</small></div><i /></button></div>
      <section v-if="modelValue.mode !== 'off'" class="phase-card"><h3><RemoteIcon name="clock" />{{ t(modelValue.mode === 'autonomous' ? '行动的节奏' : '联系的节奏') }}</h3><RemoteFields :model-value="modelValue" :fields="pick([modelValue.mode === 'autonomous' ? 'autonomousIntervalMinutes' : 'companionIntervalMinutes', 'maxDailyRuns']).map((field, index) => ({ ...field, label: t(index ? '每天最多' : '检查间隔') }))" :disabled="disabled" @update:model-value="update" /></section>
      <RemoteFields v-if="modelValue.mode === 'autonomous'" :model-value="modelValue" :fields="pick(['goal']).map(field => ({ ...field, label: t('你希望它关注什么') }))" :disabled="disabled" @update:model-value="update" />
      <section v-if="modelValue.mode === 'off'" class="phase-card phase-quiet-card"><RemoteIcon name="moon" /><h3>{{ t('先安静地陪着你') }}</h3><p>{{ t('聊天和已有的定时安排不受影响。') }}</p></section>
      <details v-if="modelValue.mode !== 'off'" class="phase-fold"><summary><RemoteIcon name="globe" />{{ t('时区') }}<RemoteIcon name="down" /></summary><RemoteFields :model-value="modelValue" :fields="pick(['timeZone'])" :disabled="disabled" @update:model-value="update" /></details>
    </template>
    <template v-else-if="kind === 'user'">
      <section class="phase-card"><h3><RemoteIcon name="profile" />{{ t('个人资料') }}</h3><RemoteFields :model-value="modelValue" :fields="pick(['owner.name', 'owner.persona'])" :disabled="disabled" @update:model-value="update" /></section>
      <section class="phase-card"><h3><RemoteIcon name="language" />{{ t('infOS 界面语言') }}</h3><RemoteFields :model-value="modelValue" :fields="pick(['user.language'])" :disabled="disabled" @update:model-value="update" /><p class="phase-context"><RemoteIcon name="globe" />{{ t('与当前 infOS 同步，也会影响连接它的其他客户端。') }}</p></section>
      <button type="button" class="phase-destination-button" @click="$emit('appearance')"><RemoteIcon name="palette" /><span>{{ t('调整本机外观') }}<small>{{ t('仅用于这台设备') }}</small></span><RemoteIcon name="next" /></button>
    </template>
    <RemoteModelEditor v-else-if="kind === 'models' && fields.some(field => field.key === 'modelId')" :model-value="modelValue" :fields="fields" :record-id="recordId" :disabled="disabled" @update:model-value="update" />
    <template v-else-if="['agents', 'providers'].includes(kind) && tabs.length">
      <header v-if="kind === 'agents'" class="phase-profile-hero"><button type="button" class="phase-profile-avatar" :aria-label="t('更换头像')" :disabled="disabled || !recordId" @click="$emit('avatar')"><RemoteAvatar :id="recordId" :label="modelValue.name" fit="contain" /><span v-if="recordId"><RemoteIcon name="image" /></span></button><div><h3>{{ modelValue.name }}</h3><p>{{ modelValue.description }}</p></div></header>
      <nav v-if="tabs.length > 1" class="phase-tabs" :aria-label="t(kind === 'agents' ? '角色资料' : '模型与连接')"><button v-for="item in tabs" :key="item.id" type="button" :aria-pressed="tab === item.id" :class="{ selected: tab === item.id }" @click="tab = item.id"><RemoteIcon :name="item.icon" />{{ t(item.label) }}</button></nav>
      <Transition name="phase-pane" mode="out-in"><section :key="tab" class="phase-card"><RemoteFields :model-value="modelValue" :fields="tabFields" :disabled="disabled" @update:model-value="update" /><RemoteModelCatalog v-if="kind === 'providers'" kind="providers" :model-value="modelValue" :record-id="recordId" :connection-dirty="connectionDirty" :disabled="disabled" /></section></Transition>
      <details v-if="remaining.length" class="phase-fold" :open="remaining.some(field => field.required) || undefined"><summary><RemoteIcon name="settings" />{{ t('其他参数') }}<RemoteIcon name="down" /></summary><RemoteFields :model-value="modelValue" :fields="remaining" :disabled="disabled" @update:model-value="update" /></details>
    </template>
    <RemoteFormSections v-else :model-value="modelValue" :kind="kind" :fields="fields" :disabled="disabled" @update:model-value="update" />
  </div>
</template>
<script setup>
import { computed, ref, watch } from 'vue'
import RemoteAvatar from './RemoteAvatar.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteFields from './RemoteFields.vue'
import RemoteModelCatalog from './RemoteModelCatalog.vue'
import RemoteModelEditor from './RemoteModelEditor.vue'
import RemoteFormSections from './RemoteFormSections.vue'
import { remoteState } from '../../stores/remote'
import { t } from '../../i18n/remote'
const props = defineProps({ modelValue: Object, fields: { type: Array, default: () => [] }, kind: String, recordId: [String, Number], connectionDirty: Boolean, disabled: Boolean })
const emit = defineEmits(['update:modelValue', 'appearance', 'avatar'])
const tab = ref('main')
watch(() => [props.kind, props.recordId], () => { tab.value = 'main' })
function update(value) { emit('update:modelValue', value) }
function set(key, value) { update({ ...props.modelValue, [key]: value }) }
function pick(keys) { return props.fields.filter(field => keys.includes(field.key)) }
const currentModel = computed(() => props.fields.find(field => field.key === 'model.main')?.choices?.find(option => String(option.value) === String(props.modelValue?.['model.main']))?.label)
const agentFields = computed(() => props.fields.filter(field => field.key.startsWith('model.agent.')))
function agentName(key) { return remoteState.agents.find(agent => `model.agent.${agent.id}` === key)?.name || key.slice('model.agent.'.length) }
const modes = [
  { value: 'off', label: '关闭', icon: 'moon', description: '暂停主动联系，定时安排继续运行。' },
  { value: 'companion', label: '智能陪伴', icon: 'heart', description: '适时来聊几句，没回应时减少打扰。' },
  { value: 'autonomous', label: '自主行动', icon: 'sparkle', description: '主动整理信息，推进你关心的事。' },
]
const definitions = computed(() => props.kind === 'agents' ? [
  { id: 'main', label: '身份', icon: 'profile', keys: ['id', 'name', 'description', 'ownerAppellation', 'publicProfile'] },
  { id: 'persona', label: '人设', icon: 'memory', keys: ['systemPrompt', 'channelPatches'] },
  { id: 'interaction', label: '互动', icon: 'chat', keys: ['waifuTexts'] },
] : props.kind === 'providers' ? [
  { id: 'main', label: '连接', icon: 'connection', keys: ['name', 'brand', 'protocol', 'authMode', 'apiBase', 'apiKey'] },
] : [])
const tabs = computed(() => definitions.value.filter(item => pick(item.keys).length))
const tabFields = computed(() => {
  const fields = pick(tabs.value.find(item => item.id === tab.value)?.keys || tabs.value[0]?.keys || [])
  return fields.map(field => field.key === 'providerConnectionId' ? { ...field, label: t('服务商连接') } : field)
})
const remaining = computed(() => props.fields.filter(field => !definitions.value.some(item => item.keys.includes(field.key))))
</script>
