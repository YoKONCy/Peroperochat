<template>
  <RoomDrawer :title="`${agent.name} 的提醒`" subtitle="把要记得的小事交给我" :tone="tone" :busy="busy" @close="$emit('close')">
    <p class="reminder-runtime-hint">{{ !runtimeReady ? '正在读取系统提醒状态…' : runtime.background ? '已接入安卓系统提醒，关闭界面后可由系统触发；系统强制停止会中断调度。' : '此平台在应用运行时检查并发送通知。' }} 时间使用设备本地时区。</p>
    <div v-if="runtime.background && !runtime.exact" class="reminder-permission"><span>当前使用非精确提醒，系统可能延迟触发。</span><button class="room-text-button" :disabled="requesting" @click="enablePrecision">允许准时提醒</button></div>
    <FeedbackMessage :message="runtime.syncError || runtimeError" />
    <div v-if="!permission" class="reminder-permission"><span>开启系统通知，才能在到点时收到提醒。</span><button class="room-text-button" :disabled="requesting" @click="enableNotifications">开启通知</button></div>
    <FeedbackMessage :message="actionError || error" :action-label="error ? '重试' : undefined" @action="reload" /><button v-if="error" class="room-text-button" @click="reload">重新加载提醒</button>
    <form v-if="editing" class="reminder-editor" @submit.prevent="save">
      <fieldset :disabled="busy">
        <label class="settings-field">提醒内容<input v-model="form.task" required maxlength="500" placeholder="例如：给窗边的花浇水" /></label>
        <label class="settings-field">{{ editing.id ? '下次提醒时间' : '提醒时间' }}<input v-model="form.time" type="datetime-local" step="1" required /></label>
        <label class="settings-field">重复<ChoicePicker v-model="form.recur" label="重复规则" :options="recurrences" :disabled="busy" /></label>
        <label v-if="form.recur !== 'none'" class="settings-field">重复截止时间（可留空）<input v-model="form.until" type="datetime-local" step="1" /></label>
        <div class="reminder-editor-actions"><button type="button" class="room-secondary-button" @click="editing = null">返回列表</button><button class="room-primary-button">{{ busy ? '正在保存…' : '保存提醒' }}</button></div>
      </fieldset>
    </form>
    <template v-else>
      <div class="reminder-toolbar"><input v-model="keyword" type="search" class="control-input" placeholder="搜索提醒" aria-label="搜索提醒" /><button class="room-secondary-button" :disabled="busy" @click="edit()"><RoomIcon name="plus" />新增</button></div>
      <p v-if="loading" class="control-hint" role="status">正在读取提醒…</p>
      <p v-if="!loading && !error && !entries.length" class="room-empty">{{ keyword ? '没有匹配的提醒' : '暂时没有待办的小事，也可以在聊天中请我提醒你。' }}</p>
      <article v-for="item in entries" :key="item.id" class="reminder-entry" :class="{ 'is-overdue': item.time <= now }">
        <div class="reminder-entry-time"><RoomIcon name="bell" /><time>{{ item.time.slice(0, 16) }}</time><span v-if="item.time <= now">待送达</span><span v-else-if="item.recur !== 'none'">{{ item.recur === 'daily' ? '每天' : '每周' }}</span></div>
        <p>{{ item.task }}</p><small v-if="item.recurUntil">重复至 {{ item.recurUntil.slice(0, 16) }}</small><small v-if="item.deliveryError" class="reminder-delivery-error">{{ item.deliveryError }}</small>
        <small v-if="runtime.background">{{ scheduleLabels[item.systemMode] || '等待系统登记' }}</small><small v-if="item.backgroundError" class="reminder-delivery-error">{{ item.backgroundError }}</small>
        <div class="reminder-entry-actions"><button class="room-text-button" :disabled="busy || loading || !!error" @click="edit(item)"><RoomIcon name="edit" />修改</button><button class="room-text-button" :disabled="busy || loading || !!error" @click="remove(item)"><RoomIcon name="trash" />取消提醒</button></div>
      </article>
      <p v-if="total > entries.length" class="control-hint">共 {{ total }} 项，显示前 {{ entries.length }} 项；搜索可缩小范围。</p>
    </template>
  </RoomDrawer>
</template>
<script setup>
import { computed, onMounted, onBeforeUnmount, reactive, ref } from 'vue'
import { appMessageBox as ElMessageBox } from '../../services/appMessageBox'
import { isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification'
import RoomDrawer from './RoomDrawer.vue'
import FeedbackMessage from '../FeedbackMessage.vue'
import RoomIcon from './RoomIcon.vue'
import ChoicePicker from './ChoicePicker.vue'
import { useReminders } from '../../composables/useReminders'
import { manageReminder, reminderRuntime, requestReminderPrecision, openReminderNotifications } from '../../services/reminders'
const props = defineProps({ agent: { type: Object, required: true }, tone: { type: String, default: 'pero' } })
const emit = defineEmits(['close', 'updated'])
const keyword = ref(''), editing = ref(null), busy = ref(false), actionError = ref(''), permission = ref(false), requesting = ref(false)
const { entries, total, now, loading, error, reload } = useReminders(computed(() => props.agent.id), keyword)
const form = reactive({ task: '', time: '', recur: 'none', until: '' })
const runtime = ref({ background: false })
const runtimeReady = ref(false), runtimeError = ref('')
const scheduleLabels = { exact: '已登记准时提醒', inexact: '已登记非精确提醒', pending: '等待系统登记', cancelled: '系统登记已取消', blocked: '等待通知授权', failed: '系统登记失败' }
const recurrences = [{ value: 'none', label: '仅一次' }, { value: 'daily', label: '每天' }, { value: 'weekly', label: '每周' }]
let disposed = false
function edit(item = {}) { editing.value = { ...item }; actionError.value = ''; Object.assign(form, { task: item.task || '', time: item.time?.replace(' ', 'T') || '', recur: item.recur || 'none', until: item.recurUntil?.replace(' ', 'T') || '' }) }
function localTime(value) { if (!value) return ''; const text = value.replace('T', ' '); return text.length === 16 ? `${text}:00` : text }
async function save() {
  if (busy.value) return
  busy.value = true; actionError.value = ''
  try {
    await manageReminder(props.agent.id, { action: editing.value.id ? 'update' : 'create', ...(editing.value.id ? { id: editing.value.id, expectedRevision: editing.value.revision } : {}),
      task: form.task, time: localTime(form.time), recur: form.recur, recurUntil: form.recur === 'none' ? '' : localTime(form.until) })
    if (!disposed) { editing.value = null; emit('updated'); await reload() }
  } catch (failure) { if (!disposed) { actionError.value = failure.message; if (failure.code === 'STALE_REMINDER') await reload() } }
  finally { if (!disposed) busy.value = false }
}
async function remove(item) {
  if (busy.value) return
  busy.value = true; actionError.value = ''
  try {
    await ElMessageBox.confirm(item.task, '取消这条提醒？', { confirmButtonText: '取消提醒', cancelButtonText: '保留' })
    if (disposed) return
    await manageReminder(props.agent.id, { action: 'delete', id: item.id, expectedRevision: item.revision })
    if (!disposed) { emit('updated'); await reload() }
  } catch (failure) { if (!disposed && failure !== 'cancel' && failure !== 'close') { actionError.value = failure.message; await reload() } }
  finally { if (!disposed) busy.value = false }
}
async function enableNotifications() {
  requesting.value = true
  try {
    const alreadyGranted = await isPermissionGranted()
    if (runtime.value.background && alreadyGranted) await openReminderNotifications()
    else {
      const result = await requestPermission()
      if (!disposed) { permission.value = result === 'granted'; if (!permission.value) actionError.value = '通知权限未开启，可在系统设置中允许通知。' }
    }
  }
  catch (failure) { if (!disposed) actionError.value = failure.message }
  finally { if (!disposed) { requesting.value = false; refreshRuntime() } }
}
async function refreshRuntime() {
  try { const [state, allowed] = await Promise.all([reminderRuntime(), isPermissionGranted()]); if (!disposed) { runtime.value = state; runtimeReady.value = true; runtimeError.value = ''; permission.value = state.notifications ?? allowed } }
  catch (failure) { if (!disposed) { runtimeError.value = failure.message || '暂时无法确认系统提醒状态'; console.warn('系统提醒状态读取失败：', failure) } }
}
async function enablePrecision() {
  requesting.value = true
  try { await requestReminderPrecision() }
  catch (failure) { if (!disposed) actionError.value = failure.message }
  finally { if (!disposed) { requesting.value = false; refreshRuntime() } }
}
function onVisible() { if (!document.hidden) { refreshRuntime(); reload() } }
onMounted(() => { refreshRuntime(); window.addEventListener('focus', onVisible); document.addEventListener('visibilitychange', onVisible) })
onBeforeUnmount(() => { disposed = true; window.removeEventListener('focus', onVisible); document.removeEventListener('visibilitychange', onVisible) })
</script>
<style scoped>
.reminder-runtime-hint { margin: 0 0 14px; color: var(--room-muted); font-size: 10px; line-height: 1.7; }
.reminder-permission { display: flex; align-items: center; gap: 8px; padding: 8px 10px; margin-bottom: 16px; background: var(--room-accent-soft); border-radius: 5px; font-size: 11px; line-height: 1.6; }
.reminder-permission > span { flex: 1; }
.reminder-permission button { flex: none; }
.reminder-toolbar { display: flex; gap: 10px; margin: 16px 0; }
.reminder-toolbar input { min-width: 0; }
.reminder-toolbar button { display: flex; align-items: center; gap: 5px; flex: none; }
.reminder-toolbar svg { width: 14px; height: 14px; }
.reminder-entry { padding: 15px; margin: 12px 0; border: 1px solid var(--room-line); border-left: 2px solid var(--room-accent-bright); border-radius: 3px 12px 3px 3px; }
.reminder-entry-time { display: flex; align-items: center; gap: 7px; color: var(--room-accent); font-size: 11px; }
.reminder-entry-time svg { width: 14px; height: 14px; }
.reminder-entry-time > span { margin-left: auto; color: var(--room-muted); font-size: 10px; }
.reminder-entry p { margin: 12px 0; font-size: 13px; line-height: 1.7; white-space: pre-wrap; overflow-wrap: anywhere; }
.reminder-entry small { display: block; margin-top: 8px; font-size: 10px; line-height: 1.6; color: var(--room-muted); }
.reminder-entry .reminder-delivery-error { color: var(--room-danger); }
.reminder-entry-actions { display: flex; gap: 16px; border-top: 1px solid var(--room-line); margin-top: 12px; padding-top: 4px; }
.reminder-entry-actions button { display: flex; align-items: center; gap: 5px; padding-left: 0; padding-right: 0; font-size: 11px; }
.reminder-entry-actions svg { width: 13px; height: 13px; }
.reminder-editor fieldset { padding: 0; margin: 20px 0 0; border: 0; min-width: 0; }
.reminder-editor-actions { display: flex; justify-content: flex-end; gap: 10px; margin-top: 20px; }
</style>
