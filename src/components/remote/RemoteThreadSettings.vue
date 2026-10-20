<template>
  <RoomDrawer title="会话设置" subtitle="模型、工具与执行方式" :busy="busy" @close="$emit('close')"><p v-if="error" class="room-error">{{ error }}</p><p v-if="notice" class="remote-notice">{{ notice }}</p><p v-if="unresolved" class="room-error">操作结果待核对。<button @click="$emit('pending')">前往通知</button></p>
    <form v-if="model" @submit.prevent="saveModel"><h2>本会话模型</h2><RemoteFields v-model="model" :fields="modelFields" :disabled="busy" /><button class="room-primary-button" :disabled="busy || unresolved">保存模型设置</button></form>
    <form v-if="settings" @submit.prevent="saveExecution"><h2>工具执行方式</h2><RemoteFields v-model="execution" :fields="executionFields" :disabled="busy" /><p class="remote-help">{{ executionHelp[execution.mode] }}</p><button class="room-primary-button" :disabled="busy || unresolved">保存执行方式</button></form>
    <form v-if="settings" @submit.prevent="saveTools"><h2>本会话工具</h2><div class="remote-thread-tools"><label v-for="tool in settings.tools || []" :key="tool.name"><span>{{ tool.label || tool.name }}<small>{{ tool.description }}{{ tool.modeAllowed === false ? ' · 当前交互模式不可用' : '' }}{{ tool.locked ? ' · 系统协议' : '' }}</small></span><RoomSwitch :model-value="!disabledTools.includes(tool.name)" :disabled="busy || tool.locked" :label="tool.label || tool.name" @update:model-value="toggleTool(tool.name, $event)" /></label></div><button class="room-primary-button" :disabled="busy || unresolved">保存工具设置</button></form>
    <div class="remote-manager-actions"><button :disabled="busy" @click="run(async () => { result = await read('thread.flow', { id: threadId, params: { agentId } }) })">当前心流</button><button :disabled="busy" @click="run(async () => { result = await read('thread.tokens', { id: threadId, body: { agentId } }) })">Token 预算</button><button :disabled="busy || unresolved" @click="confirmation = 'flow'">清空会话心流</button><button :disabled="busy || unresolved" @click="confirmation = 'pause'">暂停执行</button><button :disabled="busy || unresolved" @click="confirmation = 'resume'">继续执行</button></div><form @submit.prevent="inject"><label class="remote-field">追加执行指令<textarea v-model="instruction" rows="3" maxlength="16000" /></label><button class="room-secondary-button" :disabled="busy || unresolved || !instruction.trim()">追加指令</button></form>
    <RoomDrawer v-if="confirmation" title="确认会话操作" :busy="busy" @close="confirmation = null"><p v-if="error" class="room-error">{{ error }}</p><p>{{ confirmation === 'execution' ? executionHelp[execution.mode] : confirmation === 'flow' ? '清空此会话角色的心流状态？' : confirmation === 'pause' ? '暂停此会话的当前执行？' : '继续此会话的执行？' }}</p><button class="room-primary-button" :disabled="busy || unresolved" @click="confirm">确认</button></RoomDrawer><RoomDrawer v-if="result !== undefined" title="会话状态" @close="result = undefined"><RemoteValue :value="result" /></RoomDrawer>
    <div class="remote-manager-actions"><button :disabled="busy" @click="exportHistory">导出完整会话</button><button v-if="exporting" @click="cancelExport = true">取消导出</button></div>
  </RoomDrawer>
</template>
<script setup>
import { ref, computed, onMounted } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RoomSwitch from '../companion/RoomSwitch.vue'
import RemoteFields from './RemoteFields.vue'
import RemoteValue from './RemoteValue.vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { field, exportRemoteThread } from '../../services/remoteManagement'
import { readRemote, performRemote } from '../../stores/remote'
const props = defineProps({ threadId: { type: String, required: true }, agentId: String })
const emit = defineEmits(['close', 'updated', 'pending'])
const { read, write, run, guard, busy, error, unresolved, fail } = useRemoteManagement()
const settings = ref(null), model = ref(null), models = ref([]), execution = ref({ mode: 'manual' }), disabledTools = ref([]), instruction = ref(''), result = ref(undefined), confirmation = ref(null), notice = ref('')
const exporting = ref(false), cancelExport = ref(false)
const modelFields = computed(() => [field('modelConfigId', 'number', { nullable: true, choices: models.value.map(item => ({ value: Number(item.id), label: `${item.name} · ${item.modelId}` })) }), field('reasoningEffort', 'text', { nullable: true, choices: ['off', 'low', 'medium', 'high', 'xhigh', 'max'] })])
const executionHelp = { manual: '需要确认的操作由你审批。', auto: '普通工具自动执行，终端与工作区外删除仍需你确认。', auto_approve: '终端与工作区外删除交给审批模型判断。', full: '已开放的工具直接执行，不再进行审批或风险拦截。' }
const executionFields = [field('mode', 'text', { label: '执行方式', choices: [{ value: 'manual', label: '手动审批' }, { value: 'auto', label: '自动执行' }, { value: 'auto_approve', label: '自动审批' }, { value: 'full', label: '完整权限' }] })]
async function load() { const values = await Promise.all([readRemote({ kind: 'thread_tools', id: props.threadId }, { allowCached: false }), read('thread.models', { id: props.threadId })]); guard(); settings.value = values[0]; execution.value.mode = values[0].executionMode || (values[0].autoExecuteTools ? 'auto' : 'manual'); disabledTools.value = values[0].disabledTools || values[0].tools.filter(item => item.userEnabled === false).map(item => item.name); model.value = { modelConfigId: values[1].modelConfigId ?? null, reasoningEffort: values[1].reasoningEffort ?? null }; models.value = values[1].models || [] }
async function action(input) { try { const value = await performRemote(input); guard(); return value } catch (failure) { fail(failure); throw failure } }
function toggleTool(name, enabled) { disabledTools.value = enabled ? disabledTools.value.filter(item => item !== name) : [...new Set([...disabledTools.value, name])] }
async function saveModel() { await run(async () => { await write('thread.save_models', { id: props.threadId, body: model.value }); notice.value = '会话模型已保存'; emit('updated'); await load() }) }
async function saveExecution() { confirmation.value = 'execution' }
async function saveTools() { await run(async () => { await action({ kind: 'thread_tools', id: props.threadId, disabledTools: disabledTools.value }); notice.value = '工具设置已保存'; emit('updated'); await load() }) }
async function confirm() { await run(async () => { if (confirmation.value === 'execution') await action({ kind: 'execution_mode', id: props.threadId, mode: execution.value.mode }); else if (confirmation.value === 'flow') await write('thread.clear_flow', { id: props.threadId, params: { agentId: props.agentId } }); else await write(`thread.${confirmation.value}`, { body: { threadId: props.threadId } }); confirmation.value = null; notice.value = '会话设置已更新'; emit('updated'); await load() }) }
async function inject() { await run(async () => { await write('thread.inject', { body: { threadId: props.threadId, instruction: instruction.value } }); instruction.value = ''; notice.value = '指令已追加'; emit('updated') }) }
async function exportHistory() { await run(async () => { exporting.value = true; cancelExport.value = false; try { const data = await exportRemoteThread(read, props.threadId, { onProgress: (count, total) => { notice.value = `正在读取记录 ${count} / ${total}` }, cancelled: () => cancelExport.value }); notice.value = data?.cancelled ? '已取消保存' : '导出文件已交给系统保存' } finally { exporting.value = false } }) }
onMounted(() => run(load))
</script>
