<template>
  <div><p v-if="error" class="room-error">{{ error }}</p><p v-if="unresolved" class="room-error">发送结果待核对。<button @click="$emit('pending')">去核对</button></p>
    <div class="remote-manager-actions"><button :disabled="busy" @click="run(load)">刷新房间</button><button :disabled="busy || unresolved" @click="configure('facility')">创建设施</button><button :disabled="busy || unresolved" @click="configure('create')">新房间</button><button :disabled="busy || unresolved" @click="configure('butler')">管家设置</button></div>
    <div class="remote-manager-records"><button v-for="room in rooms" :key="room.id" class="remote-manager-record" @click="enter(room)"><span><strong>{{ room.name }}</strong><small>{{ room.description || room.id }} · {{ room.agents?.length || 0 }} 位角色</small></span><RemoteIcon name="arrow" /></button></div>
    <template v-if="current"><div class="remote-manager-card"><h2>{{ current.name }}</h2><p class="remote-help">{{ memberNames }}</p><details><summary>房间环境与资料</summary><RemoteValue :value="current" /></details><div class="remote-manager-actions"><button :disabled="busy" @click="run(loadRoom)">刷新消息</button><button :disabled="busy || unresolved" @click="configure('edit')">房间资料</button><button :disabled="busy || unresolved" @click="configure('member')">邀请 / 移动角色</button><button :disabled="busy || unresolved" @click="configure('environment')">环境</button><button :disabled="busy || unresolved" @click="configure('command')">管家指令</button><button :disabled="busy || unresolved" @click="configure('delete')">删除房间</button></div></div>
      <div class="remote-room-history"><article v-for="surface in projection?.surfaces || []" :key="surface.surfaceId" class="remote-message"><div class="remote-message-label"><span>{{ sender(surface.messageId) }}</span><button class="remote-message-tools" v-if="surface.messageId" aria-label="删除此轮群聊" :disabled="busy || unresolved" @click="deleting = String(surface.messageId); operation = 'delete_message'; form = {}"><RemoteIcon name="trash" /></button></div><RemoteSurface :surface="surface" :disabled="remoteState.connection !== 'online'" @resource="$emit('resource', $event)" @updated="run(loadRoom)" /></article></div>
      <form class="remote-manager-card" @submit.prevent="send"><label class="remote-field"><span>提及角色</span><RemoteSelect v-model="mention" label="提及角色" :options="mentionOptions" :disabled="busy" /></label><label class="remote-field"><span>群聊消息</span><textarea v-model="message" rows="3" maxlength="16000" required /></label><button class="room-primary-button" :disabled="busy || unresolved || !message.trim()">发送</button></form>
    </template>
    <RoomDrawer v-if="operation" :title="titles[operation]" :busy="busy" @close="operation = null"><p v-if="error" class="room-error">{{ error }}</p><p v-if="operation === 'delete' || operation === 'delete_message'" class="remote-notice">{{ operation === 'delete' ? '确认删除此房间及其聊天记录？' : '按 infOS 规则删除此轮用户发言及关联回复。' }}</p><form @submit.prevent="submit"><RemoteFields v-model="form" :fields="fields" :disabled="busy" /><button class="room-primary-button" :disabled="busy || unresolved">确认</button></form></RoomDrawer>
  </div>
</template>
<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { field, inferFields } from '../../services/remoteManagement'
import { remoteState } from '../../stores/remote'
import RemoteIcon from './RemoteIcon.vue'
import RemoteFields from './RemoteFields.vue'
import RemoteSurface from './RemoteSurface.vue'
import RemoteSelect from './RemoteSelect.vue'
import RemoteValue from './RemoteValue.vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
const props = defineProps({ agentId: String })
defineEmits(['pending', 'resource'])
const { read, write, run, busy, error, unresolved } = useRemoteManagement()
const rooms = ref([]), facilities = ref([]), current = ref(null), projection = ref(null), message = ref(''), mention = ref(''), operation = ref(null), form = ref({}), deleting = ref('')
const titles = { facility: '创建设施', create: '创建房间', edit: '房间资料', member: '移动角色到此房间', environment: '更新环境变量', command: '管家指令', butler: '管家设置', delete: '删除房间', delete_message: '删除这轮群聊？' }
const fields = computed(() => operation.value === 'create' ? [field('facilityId', 'number', { choices: facilities.value.map(item => ({ value: item.id, label: item.name })) }), field('name'), field('description', 'long')] : operation.value === 'member' ? [field('agentId', 'text', { choices: remoteState.agents.filter(item => item.isEnabled !== false && !item.isUser).map(item => ({ value: item.id, label: item.name })) })] : inferFields(form.value))
const memberNames = computed(() => (projection.value?.members || []).map(member => agentName(member.agentId)).join('、'))
const mentionOptions = computed(() => [{ value: '', label: '自动选择' }, { value: '@all', label: '全体成员' }, ...(projection.value?.members || []).map(member => ({ value: member.agentId, label: agentName(member.agentId) }))])
const hasChanges = computed(() => !!message.value.trim())
defineExpose({ busy, hasChanges })
let timer = null, disposed = false, polling = false
function agentName(id) { return remoteState.agents.find(item => item.id === id)?.name || id }
function sender(messageId) { const message = projection.value?.messages?.find(item => item.messageId === String(messageId)); return message?.role === 'user' ? '你' : agentName(message?.senderId || '') }
async function load() { const values = await Promise.all([read('rooms.list'), read('facilities.list')]); rooms.value = values[0]; facilities.value = values[1]; if (current.value) { current.value = rooms.value.find(item => item.id === current.value.id) || null; if (current.value) await loadRoom(); else projection.value = null } }
async function loadRoom() { if (!current.value) return; const id = current.value.id; const value = await read('rooms.projection', { id, params: { limit: 100 } }); if (current.value?.id === id) projection.value = value }
async function enter(room) { await run(async () => { if (message.value.trim()) throw new Error('请先发送或清空当前群聊输入'); projection.value = null; current.value = room; await loadRoom() }) }
async function configure(op) { await run(async () => { if (op === 'butler') { const value = await read('butler.get'); form.value = { enabled: !!value.enabled } } else form.value = op === 'facility' ? { name: '', description: '' } : op === 'create' ? { facilityId: facilities.value[0]?.id ?? 0, name: '', description: '' } : op === 'edit' ? { name: current.value.name, description: current.value.description || '' } : op === 'member' ? { agentId: props.agentId || '' } : op === 'environment' ? { key: '', value: '' } : op === 'command' ? { command: '' } : {}; operation.value = op }) }
async function submit() { await run(async () => { const op = operation.value, map = { facility: 'facilities.create', create: 'rooms.create', edit: 'rooms.save', member: 'rooms.move', environment: 'rooms.environment', command: 'rooms.butler', butler: 'butler.save', delete: 'rooms.delete', delete_message: 'rooms.delete_message' }; const body = { ...form.value }; if (op === 'member') body.roomId = current.value.id; await write(map[op], { id: ['edit', 'environment', 'command', 'delete', 'delete_message'].includes(op) ? current.value.id : null, params: op === 'delete_message' ? { messageId: deleting.value } : {}, body: Object.keys(body).length ? body : null }); if (op === 'delete') { current.value = null; projection.value = null }; operation.value = null; await load() }) }
async function send() { await run(async () => { await write('rooms.message', { id: current.value.id, body: { content: message.value, senderId: 'user', role: 'user', mentions: mention.value ? [mention.value] : [] } }); message.value = ''; await loadRoom() }) }
async function poll() { if (disposed) return; if (!document.hidden && remoteState.connection === 'online' && current.value && !busy.value && !polling) { polling = true; try { await loadRoom() } catch (failure) { if (!disposed) error.value = failure.message } finally { polling = false } }; if (!disposed) timer = setTimeout(poll, 5000) }
onMounted(() => { run(load); poll() })
onBeforeUnmount(() => { disposed = true; clearTimeout(timer) })
</script>
