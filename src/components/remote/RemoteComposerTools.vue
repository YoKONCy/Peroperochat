<template>
  <div class="chat-composer-content">
    <div class="chat-composer-config">
      <RemoteModelControls compact :thread-id="threadId" :disabled="blocked || running" @changed="$emit('changed')" @capabilities="$emit('capabilities', $event)" @pending="$emit('pending')" @busy="$emit('busy', $event)" />
      <button type="button" class="chat-context-meter" aria-label="查看上下文用量" @click="budgetOpen = true">{{ budgetLabel }}</button>
    </div>
    <slot />
    <div class="chat-composer-footer">
      <RemoteSelect compact :model-value="interactionMode || 'chat'" label="对话模式" :options="modeOptions" :disabled="modeDisabled" @update:model-value="$emit('interaction', $event)"><template #prefix><RemoteIcon :name="modeIcon" /></template></RemoteSelect>
      <button type="button" :disabled="blocked" @click="$emit('settings')"><RemoteIcon name="tools" />工具</button>
      <button type="button" :disabled="blocked" @click="$emit('voice')"><RemoteIcon name="audio" />语音</button>
      <button type="button" aria-label="更多输入选项" @click="commands = true"><RemoteIcon name="more" /></button>
      <span class="chat-composer-origin"><RemoteIcon name="shield" />infOS</span>
    </div>
    <button v-if="recovery" type="button" class="chat-recovery-chip" @click="recoveryOpen = true"><RemoteIcon name="refresh" /><span>{{ recovery.canRestore ? '有一段输入可以恢复' : ['saved', 'ready'].includes(recovery.state) ? '消息修改已保存' : '上次回复需要确认' }}</span><RemoteIcon name="arrow" /></button>
    <RoomDrawer v-if="commands" title="更多输入选项" @close="commands = false"><div class="chat-options-grid">
      <RemoteAction icon="plus" :disabled="blocked || running" @click="$emit('new-thread'); commands = false">新建会话</RemoteAction>
      <RemoteAction icon="close" :disabled="!content" @click="$emit('clear'); commands = false">清空输入</RemoteAction>
      <RemoteAction icon="folder" @click="$emit('workbench'); commands = false">引用工作区</RemoteAction>
      <RemoteAction icon="image" @click="$emit('image-mode'); commands = false">{{ imageLabel || '图片识别方式' }}</RemoteAction>
    </div></RoomDrawer>
    <RoomDrawer v-if="budgetOpen" title="上下文用量" @close="budgetOpen = false"><section class="chat-context-details">
      <template v-if="budget"><strong>{{ compact(budget.usedTokens) }}<small v-if="budget.maxInputTokens > 0"> / {{ compact(budget.maxInputTokens) }} Token</small></strong><progress v-if="budget.maxInputTokens > 0" :value="budget.usedTokens || 0" :max="budget.maxInputTokens" aria-label="预计上下文占用" /><p>{{ budget.maxInputTokens > 0 ? '预计用量，实际使用可能略有变化。' : '当前模型尚未设置上下文大小。' }}</p></template>
      <template v-else><p>{{ running ? '角色正在回复，完成后会更新用量。' : blocked ? '连接恢复后可以查看用量。' : '暂时没有可用的估算。' }}</p><RemoteAction icon="refresh" :disabled="blocked || running" @click="loadBudget">重新估算</RemoteAction></template>
    </section></RoomDrawer>
    <RoomDrawer v-if="recoveryOpen && recovery" title="继续上次的对话" @close="recoveryOpen = false"><p class="remote-help">{{ recovery.canRestore ? '上次重新生成没有完成，可以把原输入放回输入框。' : ['saved', 'ready'].includes(recovery.state) ? '消息修改已经保存，还没有重新生成。' : '先看看对应会话的回复，避免重复发送。' }}</p><RemoteFeedback :message="recoveryError" /><div class="remote-purpose-actions"><RemoteAction v-if="recovery.canRestore" icon="edit" tone="primary" @click="$emit('restore', recovery); recoveryOpen = false">恢复输入</RemoteAction><RemoteAction v-else-if="['saved', 'ready'].includes(recovery.state) || recovery.canDismiss" icon="check" @click="clearRecovery(); recoveryOpen = false">知道了</RemoteAction><RemoteAction v-else icon="bell" @click="$emit('pending'); recoveryOpen = false">查看结果</RemoteAction></div></RoomDrawer>
  </div>
</template>
<script setup>
import RemoteFeedback from './RemoteFeedback.vue'
import { computed, ref, watch, onMounted, onBeforeUnmount } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RemoteSelect from './RemoteSelect.vue'
import RemoteModelControls from './RemoteModelControls.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteAction from './RemoteAction.vue'
import { remoteClient } from '../../services/remote'
import { currentToken, assertToken, sameToken } from '../../stores/mode'
const props = defineProps({ threadId: String, agentId: String, content: String, blocked: Boolean, running: Boolean, modeDisabled: Boolean, interactionMode: String, imageLabel: String })
defineEmits(['interaction', 'image-mode', 'settings', 'voice', 'new-thread', 'clear', 'workbench', 'changed', 'capabilities', 'pending', 'restore', 'busy'])
const token = currentToken(), budget = ref(null), budgetError = ref(''), budgetOpen = ref(false), commands = ref(false), recovery = ref(null)
const recoveryOpen = ref(false)
const modeOptions = [{ value: 'minimal', label: '极简', description: '轻量对话' }, { value: 'chat', label: '聊天', description: '日常交流' }, { value: 'work', label: '工作', description: '处理任务与工具操作' }]
const modeIcon = computed(() => props.interactionMode === 'work' ? 'code' : 'chat')
const budgetLabel = computed(() => budget.value?.maxInputTokens > 0 ? `${compact(budget.value.usedTokens)} / ${compact(budget.value.maxInputTokens)}` : props.running ? '生成中' : '上下文')
let timer, request = 0, disposed = false
function compact(value) { if (!Number.isFinite(value)) return '—'; return value >= 1000 ? `${(value / 1000).toFixed(1)}k` : String(value) }
async function loadBudget() {
  const serial = ++request, id = props.threadId
  if (props.blocked || props.running || document.hidden || !id) { budget.value = null; return }
  try { const { data } = await remoteClient.manage({ operation: 'thread.tokens', id, params: {}, body: { agentId: props.agentId, content: props.content || '' } }, { token }); assertToken(token, 'remote'); if (!disposed && serial === request && id === props.threadId) { budget.value = data; budgetError.value = '' } }
  catch (failure) { if (!disposed && serial === request) { budget.value = null; budgetError.value = failure.message } }
}
function schedule() { request++; clearTimeout(timer); if (props.blocked || props.running) { budget.value = null; return }; timer = setTimeout(loadBudget, 450) }
async function loadRecovery() { if (!props.threadId || props.blocked) return; const id = props.threadId; try { const { data } = await remoteClient.regenerationRecovery(id, token); if (!disposed && id === props.threadId) recovery.value = data } catch { /* 本机恢复记录不可用时不影响普通发送。 */ } }
const recoveryError = ref('')
async function clearRecovery() { if (!recovery.value) return; try { assertToken(token, 'remote'); await remoteClient.discardRegeneration(recovery.value.planId, token); if (!disposed) { recovery.value = null; recoveryError.value = '' } } catch (failure) { if (!disposed) recoveryError.value = failure.message } }
function refresh(event) { if (event?.detail && (!sameToken(event.detail.token, token) || event.detail.threadId !== props.threadId)) return; schedule(); loadRecovery() }
watch(() => [props.content, props.threadId, props.blocked, props.running], schedule, { immediate: true })
watch(() => props.threadId, () => { recovery.value = null; loadRecovery() }, { immediate: true })
watch(() => props.blocked, value => { if (!value) loadRecovery() })
onMounted(() => { window.addEventListener('ppc:conversation-changed', refresh); window.addEventListener('ppc:thread-model-changed', refresh); document.addEventListener('visibilitychange', refresh) })
onBeforeUnmount(() => { disposed = true; request++; clearTimeout(timer); window.removeEventListener('ppc:conversation-changed', refresh); window.removeEventListener('ppc:thread-model-changed', refresh); document.removeEventListener('visibilitychange', refresh) })
defineExpose({ clearRecovery, refresh: loadRecovery })
</script>
