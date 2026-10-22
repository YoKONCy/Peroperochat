<template>
  <section class="remote-surface" :aria-label="label">
    <template v-for="(node, index) in nodes" :key="`${surface.generation}:${node.nodeId}`">
      <RemoteRichText v-if="node.kind === 'markdown'" :source="node.props.source || ''" :node-id="node.nodeId" :streaming="isStreaming(node)" :suspended="disabled || !!surface.suspended" :interrupted="surface.state === 'failed'" :context="context" @resource="$emit('resource', $event)" @resize="$emit('resize')" />
      <div v-else-if="isTool(node)" class="chat-trace-run" :class="{ 'is-first': !isTool(nodes[index - 1]), 'is-last': !isTool(nodes[index + 1]) }"><header v-if="!isTool(nodes[index - 1])" class="chat-trace-heading"><RoomIcon name="route" /><span>执行轨迹</span><small>点击查看详情</small></header><RemoteToolTrace :node="node" :thread-id="surface.threadId" @resize="$emit('resize')" /><RemoteResourceCard v-for="file in toolFiles(node)" :key="file.id" :value="file" :context="context" @open="$emit('resource', $event)" @resize="$emit('resize')" /></div>
      <RemoteReasoning v-else-if="['native-reasoning', 'thinking'].includes(node.kind)" :node="node" :active="surface.state === 'open' && !disabled && !surface.suspended" :context="context" @resource="$emit('resource', $event)" @resize="$emit('resize')" />
      <details v-else-if="node.kind === 'model-retry'" class="remote-model-retry" :data-state="node.props.state"><summary><RoomIcon name="refresh" /><strong>{{ retryStatus(node.props.state) }}</strong><span v-if="Number.isSafeInteger(node.props.attempt)">已重试 {{ node.props.attempt }} 次<span v-if="Number.isSafeInteger(node.props.maxRetries)"> / 最多 {{ node.props.maxRetries }} 次</span></span></summary><p>{{ node.props.reason || '上游模型请求发生重试。' }}</p><p v-if="node.props.partialResponse" class="remote-help">上次请求已返回部分内容，请结合后续记录查看。</p></details>
      <RemoteRichBlock v-else-if="['code', 'math', 'mermaid'].includes(node.kind)" :block="{ kind: node.kind === 'math' ? 'formula' : node.kind === 'mermaid' ? 'diagram' : 'code', source: node.props.source || node.props.code || '', language: node.props.language || 'text' }" :context="context" @resource="$emit('resource', $event)" @resize="$emit('resize')" />
      <div v-else-if="node.kind === 'status'" class="remote-execution-status" role="status"><span class="remote-status-dot" />{{ node.props.message || remoteStatus(node.props.state) }}</div>
      <p v-else-if="node.kind === 'error'" class="room-error">{{ node.props.message || '执行暂时未完成，请刷新核对' }}</p>
      <RemoteResourceCard v-else-if="node.kind === 'attachment'" :value="attachment(node)" :context="context" @open="$emit('resource', $event)" @resize="$emit('resize')" />
      <section v-else-if="node.kind === 'approval'" class="remote-input-card remote-approval-card"><span class="remote-eyebrow"><RoomIcon name="shield" />需要你确认 · {{ risk(node.props.riskLevel) }}</span><h3>{{ node.props.title || node.props.toolName }}</h3><p class="remote-help">请核对操作，再交给角色继续。</p><details><summary><RoomIcon name="code" />查看操作详情<RoomIcon name="down" /></summary><pre>{{ format(node.props.summary) }}</pre></details><div class="remote-actions"><button :disabled="disabled || busy || preview" @click="approval(node, 'deny_once')"><RoomIcon name="close" />拒绝</button><button class="room-primary-button" :disabled="disabled || busy || preview" @click="approval(node, 'allow_once')"><RoomIcon name="check" />允许这一次</button></div></section>
      <form v-else-if="node.kind === 'input'" class="remote-input-card" @submit.prevent="answer(node)">
        <span class="remote-eyebrow">等待你的回复</span><h3>{{ node.props.title }}</h3>
        <template v-if="node.props.inputKind !== 'background_task'">
          <fieldset v-for="question in questions(node)" :key="question.id" :disabled="disabled || busy || preview"><legend>{{ question.question }}</legend><p v-if="question.context" class="remote-help">{{ question.context }}</p><label v-for="option in question.options" :key="option.id" class="remote-option"><input :type="question.multiple ? 'checkbox' : 'radio'" :name="`${node.nodeId}:${question.id}`" :checked="selected(node, question, option.id)" @change="choose(node, question, option.id, $event.target.checked)" /><span>{{ option.label }}<small v-if="option.description">{{ option.description }}</small></span></label><textarea v-if="question.allowFreeText !== false" :value="messages[`${node.nodeId}:${question.id}`] || ''" @input="messages[`${node.nodeId}:${question.id}`] = $event.target.value" placeholder="也可以写下你的想法…" maxlength="16000" rows="2" /></fieldset>
          <button class="room-primary-button" type="submit" :disabled="disabled || busy || preview">提交回复</button>
        </template>
        <template v-else><p>{{ node.props.question }}</p><textarea v-model="taskMessage" :disabled="disabled || busy || preview" placeholder="补充说明（可选）" maxlength="2000" /><div class="remote-actions"><button type="button" :disabled="disabled || busy || preview" @click="taskAnswer(node, 'deny_once')">拒绝</button><button type="button" class="room-primary-button" :disabled="disabled || busy || preview" @click="taskAnswer(node, 'allow_once')">确认并继续</button></div></template>
      </form>
      <section v-else-if="node.kind === 'panel'" class="remote-field-panel"><h3>{{ node.props.title }}</h3><p v-if="node.props.subtitle" class="remote-help">{{ node.props.subtitle }}</p><dl><template v-for="field in node.props.fields || []" :key="field.key"><dt>{{ field.label }}</dt><dd>{{ field.value }}</dd></template></dl></section>
      <div v-else-if="node.kind === 'table'" class="remote-table"><table><thead><tr><th v-for="column in node.props.columns" :key="column.key">{{ column.label }}</th></tr></thead><tbody><tr v-for="(row, index) in node.props.rows" :key="index"><td v-for="column in node.props.columns" :key="column.key">{{ row[column.key] }}</td></tr></tbody></table></div>
      <div v-else-if="node.kind === 'progress'" class="remote-progress"><span>{{ node.props.stage || '服务端尚未提供阶段说明' }}</span><template v-if="progressValue(node.props.value) !== null"><progress :value="node.props.value" max="100" :aria-label="node.props.stage || '任务进度'" /><small>{{ node.props.value }}%</small></template></div>
      <section v-else-if="node.kind === 'notification'" class="remote-notification"><h3>{{ node.props.title }}</h3><p>{{ node.props.message }}</p></section>
      <section v-else-if="node.kind === 'document'"><RemoteRichText v-if="node.props.format === 'markdown'" :source="node.props.content || ''" :node-id="node.nodeId" :streaming="isStreaming(node)" :context="context" @resource="$emit('resource', $event)" @resize="$emit('resize')" /><RemoteRichBlock v-else-if="['html', 'svg'].includes(node.props.format)" :block="{ kind: 'web', source: node.props.content || '', language: node.props.format }" :context="context" @resize="$emit('resize')" /><pre v-else>{{ typeof node.props.content === 'string' ? node.props.content : format(node.props.content) }}</pre></section>
      <p v-else class="remote-help">此内容适合在 infOS 桌面端查看。</p>
    </template>
    <p v-if="preview && surface.nodes.some(node => ['approval', 'input'].includes(node.kind))" class="remote-help">正在同步可操作的请求…</p>
    <RemoteFeedback :message="error" />
  </section>
</template>
<script setup>
import RemoteFeedback from './RemoteFeedback.vue'
import { computed, reactive, ref } from 'vue'
import RoomIcon from './RemoteIcon.vue'
import RemoteResourceCard from './RemoteResourceCard.vue'
import RemoteRichText from './RemoteRichText.vue'
import RemoteRichBlock from './RemoteRichBlock.vue'
import RemoteToolTrace from './RemoteToolTrace.vue'
import RemoteReasoning from './RemoteReasoning.vue'
import { traceRows } from '../../utils/remoteTrace'
import { filesFromTool } from '../../services/remoteResources'
import '../../styles/remote-trace.css'
import '../../styles/remote-rich.css'
import { remoteStatus } from '../../utils/remoteMarkdown'
import { submitRemoteInput } from '../../stores/remote'
import { currentToken, assertToken } from '../../stores/mode'
function isTool(node) { return !!node && ['tool-call', 'tool-result'].includes(node.kind) }
const props = defineProps({ surface: { type: Object, required: true }, label: String, disabled: Boolean })
const emit = defineEmits(['resource', 'updated', 'resize'])
const busy = ref(false), error = ref(''), messages = reactive({}), selections = reactive({}), taskMessage = ref('')
const preview = computed(() => props.surface.surfaceId.startsWith('conversation:'))
const nodes = computed(() => traceRows(props.surface.nodes))
const context = computed(() => ({ agentId: props.surface.principalId, threadId: props.surface.threadId, taskId: props.surface.taskId }))
const isStreaming = node => props.surface.state === 'open' && node.props.phase !== 'committed' && node.lifecycle !== 'disposed'
const attachment = node => node.props.origin || node.props.resource ? node.props.resource || node.props : { kind: 'attachment', id: node.props.id, name: node.props.name, mimeType: node.props.mimeType, size: node.props.sizeBytes, caption: node.props.caption }
function toolFiles(node) {
  const known = new Set(props.surface.nodes.filter(item => item.kind === 'attachment').map(item => item.props.id))
  const pending = [node], visited = new Set(), files = []
  // NIT 内部交付也出现在主时间线，调用详情仍按原来的层级保留。
  while (pending.length && visited.size < 128) {
    const current = pending.shift()
    if (visited.has(current.nodeId)) continue
    visited.add(current.nodeId)
    for (const file of filesFromTool(current)) if (!known.has(file.id)) { known.add(file.id); files.push(file) }
    pending.push(...(current.children || []))
  }
  return files
}
const format = value => JSON.stringify(value, null, 2)
const risk = value => ({ low: '低风险', medium: '中风险', high: '高风险', critical: '高风险' }[value] || '请核对操作内容')
const retryStatus = state => ({ waiting: '模型请求等待重试', retrying: '正在重试模型请求', recovered: '模型请求已恢复', failed: '模型重试失败', cancelled: '模型重试已取消' }[state] || '模型请求重试记录')
const progressValue = value => typeof value === 'number' && Number.isFinite(value) && value >= 0 && value <= 100 ? value : null
const questions = node => node.props.questions?.length ? node.props.questions : [{ id: 'question', question: node.props.question, options: node.props.options || [], multiple: true, allowFreeText: node.props.allowFreeText }]
function selected(node, question, id) { return (selections[`${node.nodeId}:${question.id}`] || []).includes(id) }
function choose(node, question, id, checked) { const key = `${node.nodeId}:${question.id}`; const values = new Set(selections[key] || []); if (checked) { if (!question.multiple) values.clear(); values.add(id) } else values.delete(id); selections[key] = [...values] }
async function send(node, action, payload) {
  if (props.disabled || busy.value || preview.value) return
  const token = currentToken(), surface = props.surface
  busy.value = true; error.value = ''
  try { const projection = await submitRemoteInput({ surfaceId: surface.surfaceId, nodeId: node.nodeId, generation: surface.generation, action, payload }); assertToken(token, 'remote'); emit('updated', projection) }
  catch (failure) { try { assertToken(token, 'remote') } catch { return }; error.value = failure.message }
  finally { busy.value = false }
}
function approval(node, decision) { return send(node, 'approval.resolve', { approvalId: node.props.approvalId, decision }) }
function answer(node) {
  const answers = questions(node).map(question => ({ questionId: question.id, selectedOptionIds: selections[`${node.nodeId}:${question.id}`] || [], message: messages[`${node.nodeId}:${question.id}`] || '' }))
  if (answers.some(item => !item.selectedOptionIds.length && !item.message.trim())) { error.value = '请回答每一道问题后再提交'; return }
  return send(node, 'agent-input.resolve', { inputId: node.props.inputId, answers })
}
function taskAnswer(node, decision) { return send(node, 'background-task.submit-input', { taskId: node.props.inputId, decision, message: taskMessage.value }) }
</script>
