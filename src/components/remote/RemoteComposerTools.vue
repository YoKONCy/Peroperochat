<template>
  <div class="remote-composer-tools">
    <RemoteModelControls :thread-id="threadId" :disabled="blocked || running" @changed="$emit('changed')" @capabilities="$emit('capabilities', $event)" @pending="$emit('pending')" @busy="$emit('busy', $event)" />
    <div class="remote-composer-shortcuts"><button type="button" :disabled="blocked" @click="$emit('settings')"><RemoteIcon name="tools" />工具与执行</button><button type="button" :disabled="blocked" @click="$emit('voice')"><RemoteIcon name="audio" />语音输入</button><button type="button" :disabled="blocked" @click="commands = !commands"><RemoteIcon name="terminal" />快捷操作</button><button type="button" class="remote-budget-chip" :disabled="!budget" :aria-expanded="budgetOpen" @click="budgetOpen = !budgetOpen"><RemoteIcon name="diamond" />{{ budget ? budget.maxInputTokens > 0 ? `${compact(budget.usedTokens)} / ${compact(budget.maxInputTokens)}` : '未配置上下文' : budgetError ? '预算暂不可用' : running ? '生成中' : '估算中…' }}</button></div>
    <nav v-if="commands" class="remote-quick-commands" aria-label="聊天快捷操作"><RemoteAction icon="plus" :disabled="blocked || running" @click="$emit('new-thread'); commands = false">新会话</RemoteAction><RemoteAction icon="close" :disabled="!content" @click="$emit('clear'); commands = false">清空输入</RemoteAction><RemoteAction icon="folder" @click="$emit('workbench'); commands = false">引用工作区</RemoteAction></nav>
    <div v-if="budgetOpen && budget" class="remote-budget-details"><strong>本次输入预算</strong><progress v-if="budget.maxInputTokens > 0" :value="budget.usedTokens || 0" :max="budget.maxInputTokens || 1" aria-label="预计上下文占用" /><p>{{ budget.usedTokens }} / {{ budget.maxInputTokens }} Token{{ budget.memoryRetrievalDeferred ? ' · 不含待检索记忆' : '' }}</p><p class="remote-help">{{ budget.maxInputTokens > 0 ? '这是服务端估算，实际调用可能变化。' : '当前模型尚未声明上下文窗口，可在模型设置中补充。' }}</p></div>
    <div v-if="recovery" class="remote-recovery-note"><RemoteIcon name="refresh" /><span>{{ recovery.canRestore ? '上次回滚已完成，重新生成尚未提交。' : ['saved', 'ready'].includes(recovery.state) ? '上次消息修订已保存，未继续回滚。' : '上次重新生成结果需要核对。' }}</span><button v-if="recovery.canRestore" type="button" @click="$emit('restore', recovery)">恢复输入</button><button v-else-if="['saved', 'ready'].includes(recovery.state)" type="button" @click="clearRecovery">知道了</button><button v-else-if="recovery.canDismiss" type="button" @click="clearRecovery">核对已完成，收起提示</button><button v-else type="button" @click="$emit('pending')">去核对</button></div><p v-if="recoveryError" class="remote-controls-error">{{ recoveryError }}</p>
  </div>
</template>
<script setup>
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import RemoteModelControls from './RemoteModelControls.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteAction from './RemoteAction.vue'
import { remoteClient } from '../../services/remote'
import { currentToken, assertToken, sameToken } from '../../stores/mode'
const props = defineProps({ threadId: String, agentId: String, content: String, blocked: Boolean, running: Boolean })
defineEmits(['settings', 'voice', 'new-thread', 'clear', 'workbench', 'changed', 'capabilities', 'pending', 'restore', 'busy'])
const token = currentToken(), budget = ref(null), budgetError = ref(''), budgetOpen = ref(false), commands = ref(false), recovery = ref(null)
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
