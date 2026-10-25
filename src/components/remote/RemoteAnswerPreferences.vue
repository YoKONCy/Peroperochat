<template>
  <RoomDrawer :title="tr(prefillOnly ? '预填充模式' : '回答偏好')" :subtitle="tr('与当前 infOS 的其他客户端同步')" :busy="saving" :before-close="beforeClose" @close="$emit('close')">
    <RemoteFeedback :message="tr(error)" /><RemoteFeedback :message="tr(notice)" tone="success" />
    <RemoteAction v-if="unresolved" icon="alert" tone="danger" @click="$emit('pending')">{{ tr('核对保存结果') }}</RemoteAction>
    <div class="answer-preferences">
      <p class="answer-scope"><RemoteIcon name="globe" />{{ tr('对当前 infOS 的所有角色生效') }}</p>
      <p v-if="!settings && busy" class="room-empty" role="status">{{ tr('正在读取设置…') }}</p>
      <RemoteAction v-if="!settings && !busy" icon="refresh" :disabled="!online" @click="run(load)">{{ tr('重新读取') }}</RemoteAction>
      <template v-if="settings">
        <section v-if="!prefillOnly" class="answer-preference-card answer-wholesome">
          <header><span class="answer-setting-icon"><RemoteIcon name="shield" /></span><div><h3>{{ tr('健全模式') }}</h3><p>{{ tr('启用后会引导模型，避免生成限制级内容。') }}</p></div><RoomSwitch v-model="settings.wholesome" :disabled="locked" :label="tr('健全模式')" /></header>
        </section>
        <section class="answer-preference-card answer-prefill">
          <header><span class="answer-setting-icon"><RemoteIcon name="edit" /></span><div><h3>{{ tr('预填充模式') }}</h3><p>{{ tr('给回答写个开头，让模型接着生成。') }}</p></div><RoomSwitch v-model="settings.prefill.enabled" :disabled="locked" :label="tr('预填充模式')" /></header>
          <div class="answer-prefill-content">
            <nav class="answer-prefill-modes" :aria-label="tr('填充方式')"><button v-for="mode in modes" :key="mode.id" type="button" :class="{ selected: settings.prefill.mode === mode.id }" :aria-pressed="settings.prefill.mode === mode.id" :disabled="locked" @click="settings.prefill.mode = mode.id"><RemoteIcon :name="mode.icon" /><span>{{ tr(mode.label) }}</span><RemoteIcon v-if="settings.prefill.mode === mode.id" name="check" /></button></nav>
            <p class="answer-mode-description">{{ tr(settings.prefill.mode === 'continuous' ? '每轮生成都填充，包括使用工具后的继续回答。' : '每次回复只在首次生成时填充。') }}</p>
            <label class="answer-prefill-editor"><span>{{ tr('预填充内容') }}<small>{{ settings.prefill.content.length }} / {{ MAX_ASSISTANT_PREFILL_LENGTH }}</small></span><textarea v-model="settings.prefill.content" :disabled="locked" :maxlength="MAX_ASSISTANT_PREFILL_LENGTH" :placeholder="tr('填写模型继续生成前的回答开头')" :aria-label="tr('预填充内容')" rows="5" spellcheck="false" /></label>
            <p class="answer-prefill-note"><RemoteIcon :name="settings.prefill.enabled && !settings.prefill.content ? 'alert' : 'document'" />{{ tr(settings.prefill.enabled && !settings.prefill.content ? '还没有填写开头，暂不填充。' : settings.prefill.enabled ? '由 infOS 应用到新一轮回复，正文保持原样。' : '关闭时保留正文，稍后可以继续使用。') }}</p>
          </div>
        </section>
        <button type="button" class="answer-reload room-text-button" :disabled="busy || unresolved || !online" @click="requestReload"><RemoteIcon name="refresh" />{{ tr('重新读取') }}</button>
      </template>
    </div>
    <template #footer="{ close }"><div class="answer-settings-footer"><span>{{ tr(!online ? '连接后可修改' : hasChanges ? '有未保存的修改' : !settings ? '正在读取设置…' : '已与 infOS 同步') }}</span><RemoteAction icon="close" tone="quiet" :disabled="saving" @click="close">{{ tr('关闭') }}</RemoteAction><RemoteAction icon="check" tone="primary" :disabled="locked || !hasChanges" @click="run(save)">{{ tr('保存') }}</RemoteAction></div></template>
    <RoomDrawer v-if="leaving" :title="tr(leaveIntent === 'reload' ? '重新读取回答偏好？' : '还有未保存的修改')" :busy="busy" @close="leaving = false"><p class="remote-help">{{ tr(leaveIntent === 'reload' ? '重新读取会替换这次尚未保存的修改。' : '保存后会同步到 infOS，也可以放弃这次修改。') }}</p><div class="remote-purpose-actions"><RemoteAction icon="edit" :disabled="busy" @click="leaving = false">{{ tr('继续编辑') }}</RemoteAction><RemoteAction :icon="leaveIntent === 'reload' ? 'refresh' : 'close'" tone="danger" :disabled="busy || leaveIntent === 'reload' && !online" @click="discard">{{ tr(leaveIntent === 'reload' ? '放弃并重新读取' : '放弃并关闭') }}</RemoteAction><RemoteAction v-if="leaveIntent !== 'reload'" icon="check" tone="primary" :disabled="locked" @click="run(saveAndClose)">{{ tr('保存并关闭') }}</RemoteAction></div></RoomDrawer>
  </RoomDrawer>
</template>
<script setup>
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RoomSwitch from '../companion/RoomSwitch.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { remoteState } from '../../stores/remote'
import { t } from '../../i18n/remote'
import { ANSWER_PREFERENCE_KEYS, MAX_ASSISTANT_PREFILL_LENGTH, WHOLESOME_MODE_KEY, parseAnswerPreferences, changedAnswerPreferences } from '../../utils/answerPreferences'
defineProps({ prefillOnly: Boolean })
const emit = defineEmits(['close', 'saved', 'pending'])
const tr = (text, parameters = []) => t(text, parameters, 'answer-preferences')
const { token, read, write, run, guard, busy, error, unresolved } = useRemoteManagement()
const settings = ref(null), baseline = ref(null), notice = ref(''), leaving = ref(false), leaveIntent = ref('close'), saving = ref(false)
const online = computed(() => remoteState.connection === 'online')
const hasChanges = computed(() => !!settings.value && !!baseline.value && changedAnswerPreferences(settings.value, baseline.value).length > 0)
const locked = computed(() => !online.value || !settings.value || busy.value || unresolved.value)
const modes = [{ id: 'once', label: '单次填充', icon: 'edit' }, { id: 'continuous', label: '持续填充', icon: 'refresh' }]
function install(values) { const value = parseAnswerPreferences(values); settings.value = value; baseline.value = JSON.parse(JSON.stringify(value)) }
async function readSettings() {
  const values = await read('configs.batch', { body: { keys: ANSWER_PREFERENCE_KEYS } }); guard()
  if (!values || !ANSWER_PREFERENCE_KEYS.every(key => Object.hasOwn(values, key) && (values[key] === null || typeof values[key] === 'string'))) throw new Error(tr('暂时无法读取回答偏好，请稍后重试。'))
  return values
}
async function load() {
  if (!online.value) return
  const values = await readSettings(); guard()
  install(values)
}
async function save() {
  saving.value = true
  try { await persistSettings() } finally { saving.value = false }
}
async function persistSettings() {
  if (!settings.value || !baseline.value || !online.value || unresolved.value) throw new Error(tr('连接恢复后再保存吧。'))
  const snapshot = JSON.parse(JSON.stringify(settings.value)), items = changedAnswerPreferences(snapshot, baseline.value)
  if (!items.length) return
  if (snapshot.prefill.content.length > MAX_ASSISTANT_PREFILL_LENGTH) throw new Error(tr('预填充内容最多 8000 个字符。'))
  const fresh = parseAnswerPreferences(await readSettings()); guard()
  const remoteChanges = changedAnswerPreferences(fresh, baseline.value)
  if (remoteChanges.some(change => items.some(item => item.key === change.key && item.value !== change.value))) throw new Error(tr('这项设置刚在其他设备改过，请重新读取后再保存。'))
  await write('configs.save_batch', { body: { items } }); guard()
  // 回执成功后立即更新本页基线；后续读取失败不能诱导重复保存。
  settings.value = { wholesome: items.some(item => item.key === WHOLESOME_MODE_KEY) ? snapshot.wholesome : fresh.wholesome, prefill: items.some(item => item.key !== WHOLESOME_MODE_KEY) ? snapshot.prefill : fresh.prefill }
  baseline.value = JSON.parse(JSON.stringify(settings.value)); notice.value = '已保存，新一轮回复生效。'
  emit('saved')
  window.dispatchEvent(new CustomEvent('ppc:answer-preferences-changed', { detail: { token } }))
  try { await load() } catch { guard(); /* 保存已经成功，保留已确认的设置，等待下次读取。 */ }
}
function beforeClose() { if (hasChanges.value) { leaveIntent.value = 'close'; leaving.value = true; return false } return true }
function requestReload() { if (hasChanges.value) { leaveIntent.value = 'reload'; leaving.value = true; return }; notice.value = ''; run(load) }
function discard() { if (leaveIntent.value === 'reload') run(async () => { await load(); leaving.value = false; notice.value = '' }); else emit('close') }
async function saveAndClose() { await save(); emit('close') }
async function prepareLeave() { if (busy.value) throw new Error(tr('正在处理，请稍后。')); if (hasChanges.value) { leaveIntent.value = 'close'; leaving.value = true; throw Object.assign(new Error(''), { code: 'UI_CANCELLED' }) } }
function refresh() { if (!document.hidden && online.value && !hasChanges.value && !busy.value && !unresolved.value) run(load) }
watch(online, value => { if (value && !hasChanges.value && !busy.value) run(load) })
onMounted(() => { run(load); document.addEventListener('visibilitychange', refresh) })
onBeforeUnmount(() => document.removeEventListener('visibilitychange', refresh))
defineExpose({ busy, hasChanges, prepareLeave })
</script>
