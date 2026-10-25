<template>
  <section class="phase2 phase-reminders"><RemoteFeedback v-if="active && !form" :message="error" /><RemoteFeedback v-if="active" :message="notice" tone="success" /><RemoteFeedback v-if="active" :message="unresolved ? t('发送结果待确认') : ''" :action-label="t('去核对')" @action="$emit('pending')" />
    <div class="phase-reminder-toolbar"><label><RemoteIcon name="calendar" /><input v-model="day" type="date" :aria-label="t('日期')" /></label><button class="room-text-button" :disabled="busy || unresolved || !agents.length" @click="create"><RemoteIcon name="plus" />{{ t('新安排') }}</button></div>
    <div class="phase-week-strip"><button v-for="item in week" :key="item.key" :class="{ selected: day === item.key }" :aria-pressed="day === item.key" @click="day = item.key"><small>{{ item.weekday }}</small>{{ item.date }}<i v-if="items.some(row => stamp(row.time) === item.key)" /></button></div>
    <div class="phase-reminder-view"><button class="room-text-button" :aria-pressed="allDays" @click="allDays = !allDays"><RemoteIcon name="filter" />{{ t(allDays ? '全部' : '日期') }}</button><button class="room-icon-button" :disabled="busy" :aria-label="t('刷新')" @click="run(load)"><RemoteIcon name="refresh" /></button></div>
    <p v-if="!visibleItems.length && !busy" class="phase-empty">{{ t('没有这天的安排') }}</p>
    <article v-for="item in visibleItems" :key="item.id" class="phase-schedule-row"><time>{{ remoteDate(item.time, allDays ? { month: 'numeric', day: 'numeric', hour: '2-digit', minute: '2-digit' } : { hour: '2-digit', minute: '2-digit' }) }}</time><section><h3>{{ item.content }}</h3><p>{{ item.repeatIntervalMinutes ? repeatLabel(item.repeatIntervalMinutes) : t('仅这一次') }} · {{ remoteChoiceLabel(item.status) }}</p><div><RemoteAvatar :id="item.agentId" :label="agentName(item.agentId)" /><span>{{ agentName(item.agentId) }}</span><button class="room-icon-button" :aria-label="t('查看详情')" :disabled="busy" @click="selected = item"><RemoteIcon name="more" /></button></div></section></article>
    <RoomDrawer v-if="form" :title="t('新安排')" :busy="busy" :before-close="beforeClose" @close="form = null"><form class="phase2 phase-reminder-form" @submit.prevent="save"><RemoteFeedback :message="error" /><label class="remote-field"><span>{{ t('提醒内容') }}</span><textarea v-model="form.instruction" required maxlength="4000" rows="4" /></label><label class="remote-field"><span>{{ t('执行时间') }}</span><input v-model="form.time" type="datetime-local" required /></label><label class="remote-field"><span>{{ t('由谁提醒') }}</span><RemoteSelect v-model="form.agentId" :label="t('由谁提醒')" :options="agents.map(item => ({ value: item.id, label: item.name }))" /></label><label class="remote-field"><span>{{ t('重复') }}</span><RemoteSelect v-model="form.repeat" :label="t('重复')" :options="repeatOptions" /></label><label v-if="form.repeat === 'custom'" class="remote-field"><span>{{ t('其他间隔') }} / {{ t('分钟') }}</span><input v-model.number="form.interval" type="number" min="1" max="525600" required /></label><button class="room-primary-button" type="submit" :disabled="busy || unresolved">{{ t('保存') }}</button></form></RoomDrawer>
    <RoomDrawer v-if="selected" :title="t('定时安排')" :busy="busy" @close="selected = null"><div class="phase2 phase-card"><h3><RemoteIcon name="bell" />{{ selected.content }}</h3><p>{{ remoteDate(selected.time) }}</p><p>{{ agentName(selected.agentId) }} · {{ remoteChoiceLabel(selected.status) }}</p><p v-if="selected.lastError">{{ selected.lastError }}</p></div><div class="remote-purpose-actions"><RemoteAction v-if="selected.repeatIntervalMinutes && ['pending', 'paused'].includes(selected.status)" :icon="selected.status === 'paused' ? 'play' : 'pause'" :disabled="busy || unresolved" @click="act(selected.status === 'paused' ? 'resume' : 'pause')">{{ t(selected.status === 'paused' ? '恢复' : '暂停') }}</RemoteAction><RemoteAction v-if="selected.status === 'failed'" icon="refresh" :disabled="busy || unresolved" @click="act('retry')">{{ t('重试') }}</RemoteAction><RemoteAction v-if="['pending', 'paused', 'failed'].includes(selected.status)" icon="trash" tone="danger" :disabled="busy || unresolved" @click="confirmCancel = true">{{ t('取消安排') }}</RemoteAction></div></RoomDrawer>
    <RoomDrawer v-if="confirmCancel" :title="t('取消安排')" :busy="busy" @close="confirmCancel = false"><p>{{ selected?.content }}</p><RemoteAction tone="danger" icon="trash" :disabled="busy || unresolved" @click="act('cancel')">{{ t('确认取消') }}</RemoteAction></RoomDrawer>
    <RoomDrawer v-if="discard" :title="t('修改还没有保存')" @close="cancelDiscard"><p>{{ t('离开后，这次未保存的修改会丢失。') }}</p><div class="remote-purpose-actions"><RemoteAction @click="cancelDiscard">{{ t('继续编辑') }}</RemoteAction><RemoteAction tone="danger" @click="form = null; discard = false; decision?.(true)">{{ t('放弃修改') }}</RemoteAction></div></RoomDrawer>
  </section>
</template>
<script setup>
import { computed, ref, onMounted, onBeforeUnmount, watch } from 'vue'
import RemoteAvatar from './RemoteAvatar.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteSelect from './RemoteSelect.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { remoteState } from '../../stores/remote'
import { t, remoteChoiceLabel, remoteDate, remoteLocale } from '../../i18n/remote'
const props = defineProps({ agentId: String, active: Boolean })
defineEmits(['pending'])
const { read, write, run, busy, writing, error, unresolved } = useRemoteManagement()
const items = ref([]), day = ref(stamp(new Date())), allDays = ref(false), form = ref(null), selected = ref(null), confirmCancel = ref(false), baseline = ref(''), discard = ref(false), notice = ref('')
let decision = null, timer = null, disposed = false
const agents = computed(() => remoteState.agents.filter(item => !item.isUser && item.isEnabled !== false))
const repeatOptions = computed(() => [{ value: '', label: t('仅这一次') }, { value: '60', label: t('每小时') }, { value: '1440', label: t('每天') }, { value: '10080', label: t('每周') }, { value: 'custom', label: t('其他间隔') }])
const week = computed(() => { const date = new Date(day.value + 'T12:00:00'); date.setDate(date.getDate() - (date.getDay() + 6) % 7); return Array.from({ length: 7 }, (_, index) => { const item = new Date(date); item.setDate(item.getDate() + index); return { key: stamp(item), date: item.getDate(), weekday: item.toLocaleDateString(remoteLocale.value, { weekday: 'narrow' }) } }) })
const visibleItems = computed(() => items.value.filter(item => allDays.value || stamp(item.time) === day.value).sort((a, b) => new Date(a.time) - new Date(b.time)))
const hasChanges = computed(() => !!form.value && JSON.stringify(form.value) !== baseline.value)
watch(day, value => { if (!stamp(value)) day.value = stamp(new Date()) })
function stamp(value) { const date = new Date(value); return Number.isFinite(date.getTime()) ? `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}` : '' }
function agentName(id) { return remoteState.agents.find(item => item.id === id)?.name || id }
function repeatLabel(value) { return repeatOptions.value.find(item => item.value === String(value))?.label || `${value} ${t('分钟')}` }
async function load() { const value = await read('scheduler.reminders'); items.value = Array.isArray(value) ? value : value?.items || [] }
function create() { form.value = { instruction: '', agentId: props.agentId || agents.value[0]?.id || '', time: `${day.value}T18:00`, repeat: '', interval: 60 }; baseline.value = JSON.stringify(form.value) }
function beforeClose() { if (hasChanges.value) { discard.value = true; return false }; return true }
function cancelDiscard() { discard.value = false; decision?.(false); decision = null }
async function prepareLeave() { if (writing.value) throw new Error(t('正在处理，请稍后再试。')); if (hasChanges.value) { discard.value = true; const approved = await new Promise(resolve => { decision = resolve }); decision = null; if (!approved) throw Object.assign(new Error(''), { code: 'UI_CANCELLED' }) } }
async function save() { await run(async () => { notice.value = ''; const value = form.value, date = new Date(value.time), interval = value.repeat === 'custom' ? Number(value.interval) : value.repeat ? Number(value.repeat) : null; if (!value.instruction.trim() || value.instruction.length > 4000 || !value.agentId || !Number.isFinite(date.getTime()) || interval !== null && (!Number.isInteger(interval) || interval < 1 || interval > 525600)) throw new Error(t('请检查填写的内容')); await write('scheduler.create', { body: { agentId: value.agentId, time: date.toISOString(), instruction: value.instruction, ...(interval ? { repeatIntervalMinutes: interval } : {}) } }); form.value = null; notice.value = t('已保存'); await load() }) }
async function act(action) { if (!selected.value) return; await run(async () => { notice.value = ''; await write(`scheduler.${action}`, { id: selected.value.id, body: { agentId: selected.value.agentId } }); selected.value = null; confirmCancel.value = false; notice.value = t({ pause: '已暂停', resume: '已恢复', retry: '已重新安排', cancel: '已取消' }[action] || '已保存'); await load() }) }
function refreshActive() { clearTimeout(timer); if (disposed || !props.active) return; if (!document.hidden && remoteState.connection === 'online' && !busy.value) run(load); timer = setTimeout(refreshActive, 30000) }
watch(() => props.active, active => { if (!active) { selected.value = null; confirmCancel.value = false; if (!hasChanges.value) form.value = null; error.value = ''; notice.value = '' }; refreshActive() })
onMounted(() => { document.addEventListener('visibilitychange', refreshActive); refreshActive() })
onBeforeUnmount(() => { disposed = true; clearTimeout(timer); decision?.(false); document.removeEventListener('visibilitychange', refreshActive) })
defineExpose({ busy, hasChanges, prepareLeave, create })
</script>
