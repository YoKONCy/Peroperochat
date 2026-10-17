<template>
  <section class="remote-surface" :aria-label="label">
    <template v-for="node in surface.nodes" :key="`${surface.generation}:${node.nodeId}`">
      <div v-if="node.kind === 'markdown'" class="markdown-body" v-html="remoteMarkdown(node.props.source)" />
      <details v-else-if="['native-reasoning', 'thinking', 'tool-call', 'tool-result'].includes(node.kind)" class="remote-trace"><summary><RoomIcon :name="node.kind.startsWith('tool') ? 'sliders' : 'model'" />{{ traceTitle(node) }}<span>{{ remoteStatus(node.props.state) }}</span></summary><pre>{{ node.props.source || node.props.result || node.props.argsPreview || node.props.args }}</pre></details>
      <div v-else-if="['code', 'math', 'mermaid'].includes(node.kind)" class="remote-code"><small>{{ node.kind === 'mermaid' ? '图表源码' : node.kind === 'math' ? '公式' : node.props.language || '代码' }}</small><pre>{{ node.props.source || node.props.code || '' }}</pre></div>
      <div v-else-if="node.kind === 'status'" class="remote-execution-status" role="status"><span class="remote-status-dot" />{{ node.props.message || remoteStatus(node.props.state) }}</div>
      <p v-else-if="node.kind === 'error'" class="room-error">{{ node.props.message || '执行暂时未完成，请刷新核对' }}</p>
      <button v-else-if="node.kind === 'attachment'" class="remote-attachment" @click="$emit('resource', { kind: 'attachment', id: node.props.id, name: node.props.name })"><RoomIcon name="document" /><span>{{ node.props.name }}<small>{{ node.props.caption || '点击查看附件' }}</small></span><RoomIcon name="arrow" /></button>
      <section v-else-if="node.kind === 'approval'" class="remote-input-card"><span class="remote-eyebrow">需要你确认 · {{ risk(node.props.riskLevel) }}</span><h3>{{ node.props.title || node.props.toolName }}</h3><details><summary>查看操作详情</summary><pre>{{ format(node.props.summary) }}</pre></details><div class="remote-actions"><button :disabled="disabled || busy || preview" @click="approval(node, 'deny_once')">拒绝</button><button class="room-primary-button" :disabled="disabled || busy || preview" @click="approval(node, 'allow_once')">允许这一次</button></div></section>
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
      <div v-else-if="node.kind === 'progress'" class="remote-progress"><span>{{ node.props.stage || '正在处理' }}</span><progress v-if="Number.isFinite(node.props.value)" :value="node.props.value" max="1" /></div>
      <section v-else-if="node.kind === 'notification'" class="remote-notification"><h3>{{ node.props.title }}</h3><p>{{ node.props.message }}</p></section>
      <section v-else-if="node.kind === 'document'"><div v-if="node.props.format === 'markdown'" class="markdown-body" v-html="remoteMarkdown(node.props.content)" /><pre v-else>{{ typeof node.props.content === 'string' ? node.props.content : format(node.props.content) }}</pre></section>
      <p v-else class="remote-help">此内容适合在 infOS 桌面端查看。</p>
    </template>
    <p v-if="preview && surface.nodes.some(node => ['approval', 'input'].includes(node.kind))" class="remote-help">正在同步可操作的请求…</p>
    <p v-if="error" class="room-error" role="alert">{{ error }}</p>
  </section>
</template>
<script setup>
import { computed, reactive, ref } from 'vue'
import RoomIcon from '../companion/RoomIcon.vue'
import { remoteMarkdown, remoteStatus } from '../../utils/remoteMarkdown'
import { submitRemoteInput } from '../../stores/remote'
import { currentToken, assertToken } from '../../stores/mode'
const props = defineProps({ surface: { type: Object, required: true }, label: String, disabled: Boolean })
const emit = defineEmits(['resource', 'updated'])
const busy = ref(false), error = ref(''), messages = reactive({}), selections = reactive({}), taskMessage = ref('')
const preview = computed(() => props.surface.surfaceId.startsWith('conversation:'))
const format = value => JSON.stringify(value, null, 2)
const risk = value => ({ low: '低风险', medium: '中风险', high: '高风险', critical: '高风险' }[value] || '请核对操作内容')
const traceTitle = node => node.kind === 'tool-call' ? `执行 ${node.props.name || '操作'}` : node.kind === 'tool-result' ? '操作结果' : '思考过程'
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
