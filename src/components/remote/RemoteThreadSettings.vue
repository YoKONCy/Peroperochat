<template>
  <RoomDrawer title="会话设置" subtitle="仅作用于当前会话" :busy="busy || modelBusy" :before-close="beforeClose" @close="emit('close')">
    <p v-if="error" class="room-error">{{ error }}</p><p v-if="notice" class="remote-notice">{{ notice }}</p><RemoteAction v-if="unresolved" icon="alert" tone="danger" @click="$emit('pending')">操作结果待核对</RemoteAction>
    <RemoteModelControls :thread-id="threadId" :disabled="busy" @changed="$emit('updated')" @busy="modelBusy = $event" @pending="$emit('pending')" />
    <nav class="remote-subnav" aria-label="会话设置分类"><button v-for="item in panels" :key="item.id" :class="{ selected: panel === item.id }" @click="panel = item.id"><RemoteIcon :name="item.icon" />{{ item.label }}</button></nav>
    <form v-if="panel === 'tools' && settings" @submit.prevent="saveTools"><label class="remote-search"><RemoteIcon name="search" /><input v-model="toolSearch" placeholder="查找工具" aria-label="查找工具" /></label><p class="remote-help">{{ settings.tools?.length || 0 }} 个工具 · 系统锁定或当前模式不可用的项目不能调整。</p><div class="remote-thread-tools"><label v-for="tool in visibleTools" :key="tool.name"><span>{{ tool.label || tool.name }}<small>{{ tool.description }}{{ tool.modeAllowed === false ? ' · 当前模式不可用' : '' }}{{ tool.locked ? ' · 系统锁定' : '' }}</small></span><RoomSwitch :model-value="!disabledTools.includes(tool.name)" :disabled="busy || unresolved || tool.locked || tool.modeAllowed === false" :label="tool.label || tool.name" @update:model-value="toggleTool(tool.name, $event)" /></label></div><p v-if="!visibleTools.length" class="room-empty">没有匹配的工具</p><footer class="remote-edit-footer"><span>{{ toolsDirty ? '工具选择尚未保存' : '已读取当前设置' }}</span><RemoteAction icon="check" tone="primary" type="submit" :disabled="busy || unresolved || !toolsDirty">保存工具选择</RemoteAction></footer></form>
    <section v-if="panel === 'execution' && settings"><form @submit.prevent="saveExecution"><h2>工具执行方式</h2><div class="remote-execution-options"><label v-for="choice in executionChoices" :key="choice.value" :class="{ selected: execution.mode === choice.value }"><input v-model="execution.mode" type="radio" :value="choice.value" name="execution-mode" :disabled="busy || unresolved" /><strong>{{ choice.label }}</strong><small>{{ executionHelp[choice.value] }}</small></label></div><RemoteAction icon="shield" tone="primary" type="submit" :disabled="busy || unresolved || execution.mode === savedExecution">保存执行方式</RemoteAction></form><h2>当前执行</h2><div class="remote-purpose-actions"><RemoteAction icon="stop" :disabled="busy || unresolved" @click="confirmation = 'pause'">暂停</RemoteAction><RemoteAction icon="send" :disabled="busy || unresolved" @click="confirmation = 'resume'">继续</RemoteAction></div><form @submit.prevent="inject"><label class="remote-field">追加执行指令<textarea v-model="instruction" rows="3" maxlength="16000" /></label><RemoteAction icon="send" type="submit" :disabled="busy || unresolved || !instruction.trim()">追加指令</RemoteAction></form></section>
    <section v-if="panel === 'context'"><h2>上下文与记录</h2><div class="remote-purpose-actions"><RemoteAction icon="brain" :disabled="busy" @click="inspect('thread.flow')">查看心流</RemoteAction><RemoteAction icon="tasks" :disabled="busy" @click="inspect('thread.tokens')">Token 预算</RemoteAction><RemoteAction icon="download" :disabled="busy" @click="exportHistory">导出完整会话</RemoteAction><RemoteAction v-if="exporting" icon="close" @click="cancelExport = true">取消导出</RemoteAction></div><details class="remote-secondary-operations"><summary><RemoteIcon name="shield" />会话维护</summary><p class="remote-help">清空心流会影响此会话角色后续使用的上下文状态。</p><RemoteAction icon="trash" tone="danger" :disabled="busy || unresolved" @click="confirmation = 'flow'">清空会话心流</RemoteAction></details></section>
    <RoomDrawer v-if="confirmation" title="确认会话操作" :busy="busy" @close="confirmation = null"><p v-if="error" class="room-error">{{ error }}</p><p>{{ confirmation === 'execution' ? executionHelp[execution.mode] : confirmation === 'flow' ? '清空此会话角色的心流状态？' : confirmation === 'pause' ? '暂停此会话的当前执行？' : '继续此会话的执行？' }}</p><RemoteAction icon="check" :tone="confirmation === 'flow' ? 'danger' : 'primary'" :disabled="busy || unresolved" @click="confirm">确认{{ confirmation === 'flow' ? '清空' : '修改' }}</RemoteAction></RoomDrawer>
    <RoomDrawer v-if="result !== undefined" title="会话状态" @close="result = undefined"><RemoteOperationResult :value="result" :operation="resultOperation" /></RoomDrawer>
    <RoomDrawer v-if="leaving" title="放弃未保存的设置？" @close="leaving = false"><p>工具选择、执行方式或追加指令还有未提交的内容。</p><RemoteAction icon="close" tone="danger" @click="emit('close')">放弃修改并关闭</RemoteAction></RoomDrawer>
  </RoomDrawer>
</template>
<script setup>
import { ref, computed, onMounted } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RoomSwitch from '../companion/RoomSwitch.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteModelControls from './RemoteModelControls.vue'
import RemoteOperationResult from './RemoteOperationResult.vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { exportRemoteThread } from '../../services/remoteManagement'
import { readRemote, performRemote } from '../../stores/remote'
const props = defineProps({ threadId: { type: String, required: true }, agentId: String })
const emit = defineEmits(['close', 'updated', 'pending'])
const { token, read, write, run, guard, busy, error, unresolved, fail } = useRemoteManagement()
const settings = ref(null), execution = ref({ mode: 'manual' }), disabledTools = ref([]), instruction = ref(''), result = ref(undefined), confirmation = ref(null), notice = ref('')
const panel = ref('tools'), toolSearch = ref(''), savedTools = ref('[]'), savedExecution = ref('manual'), leaving = ref(false), resultOperation = ref('')
const panels = [{ id: 'tools', label: '工具', icon: 'tools' }, { id: 'execution', label: '执行', icon: 'shield' }, { id: 'context', label: '上下文', icon: 'memory' }]
const visibleTools = computed(() => (settings.value?.tools || []).filter(tool => `${tool.label || ''} ${tool.name} ${tool.description || ''}`.toLowerCase().includes(toolSearch.value.trim().toLowerCase())))
const toolsDirty = computed(() => JSON.stringify([...disabledTools.value].sort()) !== savedTools.value)
const executionChoices = [{ value: 'manual', label: '手动审批' }, { value: 'auto', label: '自动执行' }, { value: 'auto_approve', label: '自动审批' }, { value: 'full', label: '完整权限' }]
const exporting = ref(false), cancelExport = ref(false)
const modelBusy = ref(false)
const executionHelp = { manual: '需要确认的操作由你审批。', auto: '普通工具自动执行，终端与工作区外删除仍需你确认。', auto_approve: '终端与工作区外删除交给审批模型判断。', full: '已开放的工具直接执行，不再进行审批或风险拦截。' }
async function load() { const value = await readRemote({ kind: 'thread_tools', id: props.threadId }, { allowCached: false }); guard(); settings.value = value; execution.value.mode = value.executionMode || (value.autoExecuteTools ? 'auto' : 'manual'); savedExecution.value = execution.value.mode; disabledTools.value = value.disabledTools || (value.tools || []).filter(item => item.userEnabled === false).map(item => item.name); savedTools.value = JSON.stringify([...disabledTools.value].sort()) }
async function action(input) { try { guard(); const value = await performRemote(input, { token }); guard(); return value } catch (failure) { fail(failure); throw failure } }
function toggleTool(name, enabled) { disabledTools.value = enabled ? disabledTools.value.filter(item => item !== name) : [...new Set([...disabledTools.value, name])] }
async function saveExecution() { confirmation.value = 'execution' }
async function saveTools() { await run(async () => { await action({ kind: 'thread_tools', id: props.threadId, disabledTools: disabledTools.value }); savedTools.value = JSON.stringify([...disabledTools.value].sort()); notice.value = '工具设置已保存'; emit('updated') }) }
function beforeClose() { if (toolsDirty.value || execution.value.mode !== savedExecution.value || instruction.value.trim()) { leaving.value = true; return false } return true }
async function inspect(operation) { await run(async () => { resultOperation.value = operation; result.value = await read(operation, { id: props.threadId, ...(operation === 'thread.flow' ? { params: { agentId: props.agentId } } : { body: { agentId: props.agentId } }) }) }) }
async function confirm() { await run(async () => { if (confirmation.value === 'execution') { await action({ kind: 'execution_mode', id: props.threadId, mode: execution.value.mode }); savedExecution.value = execution.value.mode } else if (confirmation.value === 'flow') await write('thread.clear_flow', { id: props.threadId, params: { agentId: props.agentId } }); else await write(`thread.${confirmation.value}`, { body: { threadId: props.threadId } }); confirmation.value = null; notice.value = '会话设置已更新'; emit('updated') }) }
async function inject() { await run(async () => { await write('thread.inject', { body: { threadId: props.threadId, instruction: instruction.value } }); instruction.value = ''; notice.value = '指令已追加'; emit('updated') }) }
async function exportHistory() { await run(async () => { exporting.value = true; cancelExport.value = false; try { const data = await exportRemoteThread(read, props.threadId, { onProgress: (count, total) => { notice.value = `正在读取记录 ${count} / ${total}` }, cancelled: () => cancelExport.value }); notice.value = data?.cancelled ? '已取消保存' : '导出文件已交给系统保存' } finally { exporting.value = false } }) }
onMounted(() => run(load))
</script>
