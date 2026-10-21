<template>
  <section class="phase2 phase-rooms" :class="{ 'is-in-room': current }">
    <RemoteFeedback v-if="active && !operation" :message="error" /><RemoteFeedback v-if="active" :message="notice" tone="success" /><RemoteFeedback v-if="active" :message="storageError" />
    <template v-if="!current">
      <section class="phase-room-overview"><small>{{ t('我的据点') }}</small><h2>{{ selectedFacility?.name || t('我的据点') }}</h2><p>{{ selectedFacility?.description || t('房间、成员与环境') }}</p><RemoteIcon name="home" /></section>
      <RemoteSelect v-if="facilities.length > 1" v-model="facilityId" :label="t('选择据点')" :options="facilities.map(item => ({ value: String(item.id), label: item.name }))" :disabled="busy" />
      <label class="phase-search"><RemoteIcon name="search" /><input v-model="search" :placeholder="t('查找房间')" :aria-label="t('查找房间')" /></label>
      <h3 class="phase-section-title"><RemoteIcon name="home" />{{ t('房间') }}<button :disabled="busy || uncertain" @click="configure(facilities.length ? 'create' : 'facility')"><RemoteIcon name="plus" />{{ t(facilities.length ? '新房间' : '请先创建一个据点') }}</button></h3>
      <div class="phase-room-grid"><button v-for="(room, index) in visibleRooms" :key="room.id" class="phase-room-tile" :disabled="busy" @click="enter(room)"><RoomVignette :variant="index % 3" /><strong>{{ room.name }}</strong><p>{{ room.description || t('进入群聊') }}</p><span class="phase-room-avatars"><RemoteAvatar v-for="id in (room.agents || []).slice(0, 3)" :key="id" :id="id" :label="agentName(id)" fit="contain" /><small>{{ room.agents?.length ? t('{0} 位角色在场', [room.agents.length]) : t('暂时无人') }}</small></span></button></div>
      <p v-if="!visibleRooms.length && !busy" class="phase-empty">{{ t('没有找到对应内容') }}</p><button v-if="!rooms.length && !busy" class="room-secondary-button" @click="configure(facilities.length ? 'create' : 'facility')"><RemoteIcon name="plus" />{{ t('创建房间') }}</button>
      <details class="phase-fold"><summary><RemoteIcon name="shield" />{{ t('设施与管家') }}<RemoteIcon name="down" /></summary><div class="remote-purpose-actions"><RemoteAction icon="plus" :disabled="busy || uncertain" @click="configure('facility')">{{ t('创建设施') }}</RemoteAction><RemoteAction icon="settings" :disabled="busy || uncertain" @click="configure('butler')">{{ t('管家设置') }}</RemoteAction><RemoteAction icon="refresh" :disabled="busy" @click="run(load)">{{ t('刷新') }}</RemoteAction></div></details>
    </template>
    <template v-else>
      <header class="phase-room-heading"><button class="room-icon-button" :aria-label="t('返回')" :disabled="busy" @click="exitRoom"><RemoteIcon name="back" /></button><div><small>{{ selectedFacility?.name }} · {{ t('{0} 位角色在场', [members.length]) }}</small><h2>{{ current.name }}</h2></div><button class="room-icon-button" :aria-label="t('房间里的大家')" @click="operation = 'members'"><RemoteIcon name="people" /></button></header>
      <button v-if="environment.length" class="phase-room-environment" @click="configure('environment')"><RemoteIcon name="sun" /><span v-for="item in environment.slice(0, 2)" :key="item[0]">{{ item[1] }}</span><RemoteIcon name="down" /></button>
      <RemoteFeedback v-if="active" :message="uncertain ? t('发送结果待确认') : ''" :action-label="t('去核对')" @action="$emit('pending')" />
      <div ref="history" class="phase-room-history" @scroll.passive="savePosition">
        <p v-if="loadingRoom && !projection" class="phase-empty" role="status">{{ t('正在读取…') }}</p>
        <p v-else-if="!rows.length" class="phase-empty">{{ t(members.length ? '这里还没有消息' : '先邀请角色进来，就能一起聊天。') }}<button v-if="!members.length" class="room-text-button" @click="configure('member')">{{ t('邀请角色') }}</button></p>
        <article v-for="row in rows" :key="row.message.messageId" class="phase-group-message" :class="{ mine: row.message.role === 'user', system: row.message.role === 'system' }"><RemoteAvatar v-if="row.message.role === 'assistant'" :id="row.message.senderId" :label="agentName(row.message.senderId)" fit="contain" /><div><header><strong>{{ row.message.role === 'user' ? t('你') : agentName(row.message.senderId) }}</strong><time>{{ time(row.message.timestamp) }}</time><button v-if="row.message.role === 'user'" class="room-icon-button" :aria-label="t('删除这轮群聊')" :disabled="busy || uncertain" @click="deleting = row.message.messageId; configure('delete_message')"><RemoteIcon name="more" /></button></header><div class="phase-group-bubble"><RemoteSurface v-if="row.surface" :surface="row.surface" :disabled="!online || !active" @resource="$emit('resource', $event)" @updated="refreshProjection" @resize="followLatest" /><RemoteRichText v-else :source="row.message.content || ''" :node-id="'room-message-' + row.message.messageId" @resource="$emit('resource', $event)" /></div></div></article>
        <div v-if="round" class="phase-group-round"><details><summary><span class="phase-dot" /><span>{{ roundLabel }}</span><small>{{ round.agents.filter(agent => ['completed', 'failed'].includes(agent.status)).length }} / {{ round.agents.length }}</small><RemoteIcon name="down" /></summary><div v-for="agent in round.agents" :key="agent.id" class="phase-round-member"><RemoteAvatar :id="agent.id" :label="agentName(agent.id)" /><span>{{ agentName(agent.id) }}</span><small>{{ t(agent.status === 'queued' ? '准备接话' : agent.status === 'tool' ? '正在使用工具' : agent.status === 'completed' ? '接话结束' : agent.status === 'failed' ? '回复失败' : '正在回复') }}</small></div></details></div>
        <article v-for="agent in draftReplies" :key="agent.id" class="phase-group-message"><RemoteAvatar :id="agent.id" :label="agentName(agent.id)" /><div><header><strong>{{ agentName(agent.id) }}</strong><small>{{ t(agent.status === 'failed' ? '回复失败' : agent.status === 'completed' ? '接话结束' : '正在回复') }}</small></header><div class="phase-group-bubble"><RemoteStreamText :source="agent.draft" :active="agent.status === 'streaming'" @resize="followLatest" /></div></div></article>
      </div>
      <button v-if="!atEnd" class="phase-room-latest" @click="jumpLatest"><RemoteIcon name="down" />{{ t('回到最新') }}</button>
      <form class="phase-group-composer" @submit.prevent="send"><div class="phase-mention-row"><span>{{ t('这次想问') }}</span><button v-for="item in mentionOptions" :key="item.value" type="button" :class="{ selected: mention === item.value }" :aria-pressed="mention === item.value" :disabled="busy || loadingRoom" @click="mention = item.value; queueSave()"><RemoteAvatar v-if="!['', '@all'].includes(item.value)" :id="item.value" :label="item.label" />{{ item.label }}</button></div><div class="phase-compose-line"><textarea ref="composer" v-model="message" rows="1" maxlength="16000" :placeholder="t('和大家说点什么…')" :aria-label="t('群聊')" :disabled="busy || loadingRoom || !online" @input="editDraft" /><button class="room-primary-button" type="submit" :aria-label="t('发送')" :disabled="busy || loadingRoom || !online || uncertain || !enabledMembers.length || !message.trim()"><RemoteIcon name="send" /></button></div></form>
    </template>
    <RoomDrawer v-if="operation" :title="t(titles[operation] || '房间设置')" :busy="busy" :before-close="beforeCloseOperation" @close="operation = null"><RemoteFeedback :message="error" />
      <template v-if="operation === 'members'"><div class="phase2 phase-destinations"><button v-for="member in members" :key="member.agentId" @click="mention = member.agentId; operation = null; queueSave(); composer?.focus()"><RemoteAvatar :id="member.agentId" :label="agentName(member.agentId)" fit="contain" /><span><strong>{{ agentName(member.agentId) }}</strong><small>{{ t('这次想问') }}</small></span><RemoteIcon name="chat" /></button></div><div class="remote-purpose-actions"><RemoteAction icon="plus" :disabled="busy || uncertain" @click="configure('member')">{{ t('邀请角色') }}</RemoteAction><RemoteAction icon="edit" :disabled="busy || uncertain" @click="configure('edit')">{{ t('房间资料') }}</RemoteAction><RemoteAction icon="sun" :disabled="busy || uncertain" @click="configure('environment')">{{ t('房间环境') }}</RemoteAction><RemoteAction icon="shield" :disabled="busy || uncertain" @click="configure('command')">{{ t('管家指令') }}</RemoteAction></div><RemoteAction icon="trash" tone="danger" :disabled="busy || uncertain" @click="configure('delete')">{{ t('删除房间') }}</RemoteAction></template>
      <form v-else @submit.prevent="submit"><p v-if="operation === 'member'" class="remote-help">{{ t('角色会移动到当前房间。') }}</p><p v-if="['delete', 'delete_message'].includes(operation)" class="remote-operation-scope">{{ t(operation === 'delete' ? '确认删除此房间及其聊天记录？' : '删除这轮用户发言及关联回复。') }}</p><RemoteFormSections v-model="form" kind="rooms" :fields="fields" :disabled="busy" /><RemoteAction :icon="operation.startsWith('delete') ? 'trash' : 'check'" :tone="operation.startsWith('delete') ? 'danger' : 'primary'" type="submit" :disabled="busy || uncertain || operation === 'member' && !form.agentId">{{ t(operation.startsWith('delete') ? '确认删除' : '保存') }}</RemoteAction></form>
    </RoomDrawer>
    <RoomDrawer v-if="discardOperation" :title="t('修改还没有保存')" @close="cancelDiscard"><p>{{ t('离开后，这次未保存的修改会丢失。') }}</p><div class="remote-purpose-actions"><RemoteAction @click="cancelDiscard">{{ t('继续编辑') }}</RemoteAction><RemoteAction tone="danger" @click="operation = null; discardOperation = false; decide?.(true)">{{ t('放弃修改') }}</RemoteAction></div></RoomDrawer>
  </section>
</template>
<script setup>
import { computed, ref, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import RemoteFeedback from './RemoteFeedback.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteAvatar from './RemoteAvatar.vue'
import RemoteSurface from './RemoteSurface.vue'
import RemoteRichText from './RemoteRichText.vue'
import RemoteStreamText from './RemoteStreamText.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteSelect from './RemoteSelect.vue'
import RemoteFormSections from './RemoteFormSections.vue'
import RoomVignette from './RoomVignette.vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import { field, inferFields } from '../../services/remoteManagement'
import { readPresentation, writePresentation } from '../../services/remotePresentation'
import { remoteClient } from '../../services/remote'
import { remoteState, remotePendingRequests } from '../../stores/remote'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { t, remoteDate } from '../../i18n/remote'
const props = defineProps({ agentId: String, active: { type: Boolean, default: true } })
const emit = defineEmits(['pending', 'resource', 'room-change'])
const { token, read, write, run, guard, busy, writing, error, unresolved } = useRemoteManagement()
const rooms = ref([]), facilities = ref([]), current = ref(null), projection = ref(null), facilityId = ref(''), search = ref(''), message = ref(''), mention = ref(''), operation = ref(null), form = ref({}), baseline = ref('{}'), deleting = ref(''), discardOperation = ref(false), loadingRoom = ref(false), notice = ref(''), storageError = ref(''), history = ref(null), composer = ref(null), atEnd = ref(true), round = ref(null)
let disposed = false, timer = null, saveTimer = null, reading = false, roomVersion = 0, projectionRequest = 0, draftRequestId = null, stopEvents = null, frame = null, events = [], decide = null, refreshing = false
const online = computed(() => remoteState.connection === 'online')
const selectedFacility = computed(() => facilities.value.find(item => String(item.id) === facilityId.value))
const visibleRooms = computed(() => rooms.value.filter(room => (!facilityId.value || String(room.facilityId) === facilityId.value) && `${room.name} ${room.description || ''}`.toLowerCase().includes(search.value.toLowerCase())))
const members = computed(() => projection.value?.members || [])
const enabledMembers = computed(() => members.value.filter(member => remoteState.agents.some(agent => agent.id === member.agentId && agent.isEnabled !== false)))
const uncertain = computed(() => unresolved.value || [...remotePendingRequests.values()].some(item => item.state === 'unknown' && item.context?.operation === 'rooms.message' && String(item.context.id) === String(current.value?.id)))
const environment = computed(() => { try { const value = current.value?.environmentJson ? JSON.parse(current.value.environmentJson) : {}; return Object.entries(value).filter(([, value]) => ['string', 'number', 'boolean'].includes(typeof value)) } catch { return [] } })
const rows = computed(() => { const surfaces = new Map((projection.value?.surfaces || []).map(surface => [surface.surfaceId, surface])); return (projection.value?.messages || []).map(message => ({ message, surface: surfaces.get(message.surfaceId) })) })
const mentionOptions = computed(() => [{ value: '', label: t('自动选择') }, { value: '@all', label: t('全体成员') }, ...enabledMembers.value.map(member => ({ value: member.agentId, label: agentName(member.agentId) }))])
const roundLabel = computed(() => { if (round.value?.completed) return t('接话结束'); const active = round.value?.agents.find(agent => ['streaming', 'tool'].includes(agent.status)); return active ? `${agentName(active.id)} · ${t(active.status === 'tool' ? '正在使用工具' : '正在回复')}` : t('准备接话') })
const draftReplies = computed(() => (round.value?.agents || []).filter(agent => agent.draft && !projection.value?.messages?.some(message => message.role === 'assistant' && message.senderId === agent.id && message.pairId === round.value.pairId)))
const titles = { create: '创建房间', facility: '创建设施', edit: '房间资料', member: '邀请角色', environment: '房间环境', command: '管家指令', butler: '管家设置', members: '房间里的大家', delete: '删除房间', delete_message: '删除这轮群聊' }
const fields = computed(() => operation.value === 'create' ? [field('facilityId', 'number', { translateChoices: false, label: t('选择据点'), choices: facilities.value.map(item => ({ value: item.id, label: item.name })) }), field('name', 'text', { required: true, label: t('房间名称') }), field('description', 'long', { label: t('一句话介绍') })] : operation.value === 'member' ? [field('agentId', 'text', { translateChoices: false, label: t('选择角色'), choices: remoteState.agents.filter(item => !item.isUser && item.isEnabled !== false && !members.value.some(member => member.agentId === item.id)).map(item => ({ value: item.id, label: item.name })) })] : inferFields(form.value))
const hasChanges = computed(() => !!operation.value && operation.value !== 'members' && JSON.stringify(form.value) !== baseline.value)
watch(() => current.value?.id, value => emit('room-change', value || null))
function agentName(id) { return remoteState.agents.find(item => item.id === id)?.name || id || t('角色') }
function time(value) { return remoteDate(value, { hour: '2-digit', minute: '2-digit' }) }
function snapshot() { return { draft: message.value, mention: mention.value, offset: Math.max(0, history.value?.scrollTop || 0), atEnd: atEnd.value, sendRequestId: draftRequestId } }
async function persistDraft() { clearTimeout(saveTimer); if (!current.value || loadingRoom.value) return; const id = String(current.value.id), value = snapshot(); try { await writePresentation({ kind: 'room', roomId: id }, value, token); if (!disposed) storageError.value = '' } catch (failure) { if (!disposed) { storageError.value = failure.message; throw failure } } }
function queueSave() { clearTimeout(saveTimer); saveTimer = setTimeout(() => persistDraft().catch(() => {}), 300) }
function editDraft() { draftRequestId = null; queueSave(); if (composer.value) { composer.value.style.height = 'auto'; composer.value.style.height = `${Math.min(100, composer.value.scrollHeight)}px` } }
function savePosition() { if (history.value) { atEnd.value = history.value.scrollHeight - history.value.scrollTop - history.value.clientHeight < 64; queueSave() } }
async function followLatest() { if (!atEnd.value) return; await nextTick(); if (history.value && atEnd.value) history.value.scrollTop = history.value.scrollHeight }
function jumpLatest() { atEnd.value = true; followLatest() }
function cancelDiscard() { discardOperation.value = false; decide?.(false); decide = null }
async function prepareLeave() { if (writing.value) throw new Error(t('正在处理，请稍后再试。')); if (hasChanges.value) { discardOperation.value = true; const approved = await new Promise(resolve => { decide = resolve }); decide = null; if (!approved) throw Object.assign(new Error(''), { code: 'UI_CANCELLED' }) }; await persistDraft() }
function beforeCloseOperation() { if (hasChanges.value) { discardOperation.value = true; return false }; return true }
async function exitRoom() { try { await prepareLeave(); current.value = null; projection.value = null; round.value = null; roomVersion++ } catch (failure) { if (failure.code !== 'UI_CANCELLED') error.value = failure.message } }
defineExpose({ busy, hasChanges, prepareLeave, back: exitRoom, enter, current, configure })
async function load() { const [allRooms, allFacilities] = await Promise.all([read('rooms.list'), read('facilities.list')]); rooms.value = Array.isArray(allRooms) ? allRooms : []; facilities.value = Array.isArray(allFacilities) ? allFacilities : []; if (!facilities.value.some(item => String(item.id) === facilityId.value)) facilityId.value = String(facilities.value[0]?.id || ''); if (current.value) { const room = rooms.value.find(item => String(item.id) === String(current.value.id)); if (!room) { current.value = null; projection.value = null; round.value = null; roomVersion++ } else current.value = room } }
async function loadRoom() {
  if (!current.value || !props.active || document.hidden) return
  const id = String(current.value.id), version = roomVersion, request = ++projectionRequest
  const value = await read('rooms.projection', { id, params: { limit: 100 } })
  if (!value || value.protocolVersion !== 1 || String(value.roomId) !== id || !Array.isArray(value.messages) || !Array.isArray(value.members) || !Array.isArray(value.surfaces)) throw new Error(t('暂时没能读取，稍后再试。'))
  if (!props.active || request !== projectionRequest || version !== roomVersion || String(current.value?.id) !== id) return
  if (typeof value.roomName === 'string') current.value = { ...current.value, name: value.roomName }
  // 据点 revision 是末条消息编号，删除末轮后可能减小；只用请求顺序拦截迟到响应。
  if (projection.value?.revision === value.revision && projection.value.messages.length === value.messages.length && value.messages.every((message, index) => message.messageId === projection.value.messages[index].messageId && message.content === projection.value.messages[index].content) && JSON.stringify(projection.value.members) === JSON.stringify(value.members)) return
  projection.value = value
  if (!mentionOptions.value.some(option => option.value === mention.value)) mention.value = ''
  await followLatest()
}
async function refreshProjection() { if (refreshing || disposed || !props.active || !current.value) return; refreshing = true; try { await loadRoom() } catch (failure) { if (!disposed && props.active) error.value = failure.message } finally { refreshing = false } }
async function enter(room) { await run(async () => { await persistDraft(); const version = ++roomVersion; clearTimeout(saveTimer); loadingRoom.value = true; current.value = room; projection.value = null; round.value = null; message.value = ''; mention.value = ''; draftRequestId = null; atEnd.value = true; try { const saved = await readPresentation({ kind: 'room', roomId: String(room.id) }, token); guard(); if (version !== roomVersion) return; message.value = saved?.draft || ''; mention.value = saved?.mention || ''; draftRequestId = saved?.sendRequestId || null; atEnd.value = saved?.atEnd !== false; await loadRoom(); await nextTick(); if (history.value && version === roomVersion) history.value.scrollTop = atEnd.value ? history.value.scrollHeight : saved?.offset || 0 } finally { if (version === roomVersion) loadingRoom.value = false } }) }
async function configure(op) { if (op === 'create' && !facilities.value.length) op = 'facility'; await run(async () => { if (op === 'butler') { const value = await read('butler.get'); form.value = { enabled: !!value.enabled } } else form.value = op === 'facility' ? { name: '', description: '' } : op === 'create' ? { facilityId: Number(facilityId.value || facilities.value[0]?.id), name: '', description: '' } : op === 'edit' ? { name: current.value.name, description: current.value.description || '' } : op === 'member' ? { agentId: '' } : op === 'environment' ? { key: '', value: '' } : op === 'command' ? { command: '' } : {}; baseline.value = JSON.stringify(form.value); operation.value = op }) }
async function submit() { let created; await run(async () => { notice.value = ''; const op = operation.value, id = String(current.value?.id || ''), map = { facility: 'facilities.create', create: 'rooms.create', edit: 'rooms.save', member: 'rooms.move', environment: 'rooms.environment', command: 'rooms.butler', butler: 'butler.save', delete: 'rooms.delete', delete_message: 'rooms.delete_message' }, body = { ...form.value }; if (op === 'member') body.roomId = current.value.id; if (['facility', 'create', 'edit'].includes(op) && !body.name?.trim()) throw new Error(t('请填写名称')); if (op === 'member' && !body.agentId) throw new Error(t('请选择角色')); const response = await write(map[op], { id: ['edit', 'environment', 'command', 'delete', 'delete_message'].includes(op) ? id : null, params: op === 'delete_message' ? { messageId: String(deleting.value) } : {}, body: Object.keys(body).length ? body : null }); if (op === 'delete') { await writePresentation({ kind: 'room', roomId: id }, null, token); current.value = null; projection.value = null; roomVersion++ }; operation.value = null; round.value = null; notice.value = op === 'command' && typeof response?.message === 'string' ? response.message : t('已保存'); if (op === 'facility' && response?.id) facilityId.value = String(response.id); await load(); await loadRoom(); if (op === 'create' && response?.id) created = rooms.value.find(room => String(room.id) === String(response.id)) }); if (created) await enter(created) }
async function send() { if (!message.value.trim() || uncertain.value || !enabledMembers.value.length) return; await run(async () => { const id = String(current.value.id), version = roomVersion, content = message.value; draftRequestId = crypto.randomUUID(); await persistDraft(); const result = await write('rooms.message', { id, requestId: draftRequestId, body: { content, senderId: 'user', role: 'user', mentions: mention.value ? [mention.value] : [] } }); if (version !== roomVersion) return; message.value = ''; draftRequestId = null; atEnd.value = true; await persistDraft(); if (result?.roundId && round.value?.id !== result.roundId) round.value = { id: result.roundId, pairId: result.roundId, completed: false, agents: (result.agentIds || (result.agentId ? [result.agentId] : [])).map(id => ({ id, status: 'queued', draft: '' })) }; await loadRoom() }) }
function applyEvent(payload) {
  if (!props.active || String(payload.roomId) !== String(current.value?.id) || !payload.roundId || String(payload.roundId).length > 300) return
  if (!round.value || round.value.id !== payload.roundId) round.value = { id: payload.roundId, pairId: payload.pairId || payload.roundId, completed: false, agents: [] }
  const value = round.value, action = payload.action
  if (action === 'stronghold_round_started') value.agents = (Array.isArray(payload.agentIds) ? payload.agentIds : []).slice(0, 100).map(id => ({ id, status: 'queued', draft: '' }))
  else if (action === 'stronghold_round_completed') { value.completed = true; refreshProjection() }
  else if (typeof payload.agentId === 'string') {
    let agent = value.agents.find(item => item.id === payload.agentId)
    if (!agent && value.agents.length < 100) { agent = { id: payload.agentId, status: 'queued', draft: '' }; value.agents.push(agent) }
    if (!agent) return
    const states = { stronghold_agent_queued: 'queued', stronghold_agent_started: 'streaming', stronghold_agent_delta: 'streaming', stronghold_agent_tool_started: 'tool', stronghold_agent_tool_completed: 'streaming', stronghold_agent_completed: 'completed', stronghold_agent_failed: 'failed' }
    if (states[action]) agent.status = states[action]
    if (action === 'stronghold_agent_delta') agent.draft = (typeof payload.replace === 'string' ? payload.replace : agent.draft + String(payload.delta || '')).slice(-131072)
    if (['stronghold_agent_completed', 'stronghold_agent_failed'].includes(action)) refreshProjection()
  }
  followLatest()
}
async function poll() { clearTimeout(timer); if (disposed) return; if (props.active && !document.hidden && online.value && !busy.value && !reading) { reading = true; try { if (current.value) await loadRoom(); else await load() } catch (failure) { if (!disposed && props.active) error.value = failure.message } finally { reading = false } }; if (!disposed && props.active && !document.hidden) timer = setTimeout(poll, 10000) }
function visibility() { if (document.hidden) persistDraft().catch(() => {}); else if (props.active) { round.value = null; poll() } }
watch(() => props.active, active => { if (active) { round.value = null; poll() } else { clearTimeout(timer); operation.value = null; error.value = ''; notice.value = ''; persistDraft().catch(() => {}) } })
watch(online, value => { round.value = null; if (value && props.active) poll() })
watch(uncertain, async (value, previous) => {
  if (value || !previous || !current.value || !draftRequestId) return
  const id = String(current.value.id), request = draftRequestId, version = roomVersion
  try { const saved = await readPresentation({ kind: 'room', roomId: id }, token); guard(); if (version === roomVersion && draftRequestId === request && saved && !saved.sendRequestId) { message.value = saved.draft || ''; draftRequestId = null } }
  catch (failure) { if (!disposed) storageError.value = failure.message }
})
onMounted(async () => {
  document.addEventListener('visibilitychange', visibility)
  await run(load)
  if (disposed) return
  try {
    const dispose = await remoteClient.listen(event => { if (!props.active || document.hidden || event.kind !== 'gateway') return; try { guard() } catch { return }; const payload = event.data?.payload; if (!payload?.action?.startsWith('stronghold_')) return; if (events.length >= 256) { events = []; refreshProjection(); return }; events.push(payload); if (frame == null) frame = requestAnimationFrame(() => { frame = null; const values = events; events = []; values.forEach(applyEvent) }) })
    if (disposed) dispose(); else stopEvents = dispose
  } catch { /* 实时监听不可用时，仍通过房间快照刷新消息。 */ }
  if (!disposed) poll()
})
onBeforeUnmount(() => { persistDraft().catch(() => {}); disposed = true; roomVersion++; clearTimeout(timer); clearTimeout(saveTimer); if (frame != null) cancelAnimationFrame(frame); stopEvents?.(); decide?.(false); document.removeEventListener('visibilitychange', visibility) })
</script>
