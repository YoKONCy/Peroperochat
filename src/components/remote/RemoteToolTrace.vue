<template>
  <details class="remote-tool-trace" :class="{ 'remote-nit-trace': isScript, 'remote-tool-child': !!node.props.parentCallId }" :data-color="display.color" :data-state="state" @toggle="activate">
    <summary><span class="remote-tool-glyph"><RemoteIcon :name="display.icon" /></span><span class="remote-tool-heading"><strong>{{ display.label }}<span v-if="isScript" class="remote-nit-label">NIT</span></strong><small>{{ target || display.category }}</small><small v-if="counts.total" class="remote-tool-result-hint remote-tool-children-summary" :class="{ 'has-failures': counts.failed }">{{ childSummary }}</small><small v-if="result?.props.isError" class="remote-tool-result-hint">{{ resultView.output.trim().slice(0, 160) }}</small><small v-else-if="result && resultView.facts.length" class="remote-tool-result-hint">{{ resultView.facts.join(' · ') }}</small></span><span class="remote-tool-meta"><span class="remote-tool-state">{{ remoteStatus(state) }}</span><small v-if="duration">{{ duration }}</small></span><RemoteIcon class="remote-trace-chevron" name="down" /></summary>
    <div v-if="openedOnce" class="remote-tool-body">
      <p v-if="node.parentIssue" class="remote-help">{{ node.parentIssue }}。</p>
      <section v-if="counts.total" class="remote-nit-children" aria-label="NIT 内部工具调用"><header><RemoteIcon name="route" /><strong>内部工具调用</strong><span>{{ counts.total }} 项</span></header><p class="remote-help">按服务端调用开始顺序排列；各项可分别展开。</p><RemoteToolTrace v-for="child in node.children" :key="child.nodeId" :node="child" :thread-id="threadId" /></section>
      <p v-else-if="isScript" class="remote-help">{{ ['assembling', 'calling'].includes(state) ? '尚未收到内部工具调用记录。' : '此记录未包含独立的内部工具调用，脚本与返回详情仍可查看。' }}</p>
      <component :is="isScript ? 'details' : 'div'" class="remote-script-source"><summary v-if="isScript"><RemoteIcon name="code" />脚本与返回详情</summary>
      <div v-if="!orphan" class="remote-trace-section"><h4>调用参数 <small v-if="state === 'assembling'">正在接收<span v-if="Number.isFinite(node.props.receivedChars)"> · {{ node.props.receivedChars }} 字符</span></small></h4><RemoteTraceText :value="callArgs" /><p v-if="node.props.previewTruncated && (state === 'assembling' || callArgs === node.props.argsPreview)" class="remote-help">参数预览已截断。</p></div>
      <div v-if="result" class="remote-trace-section">
        <h4>{{ result.props.isError ? '执行错误' : '返回结果' }}</h4>
        <div v-if="!result.props.isError && resultView.facts.length" class="remote-tool-facts"><span v-for="fact in resultView.facts" :key="fact">{{ fact }}</span></div>
        <ol v-if="!result.props.isError && resultView.steps.length" class="remote-trace-plan"><li v-for="(step, index) in resultView.steps" :key="step.id || index" :data-state="step.status"><RemoteIcon :name="step.status === 'completed' ? 'check' : step.status === 'blocked' ? 'alert' : 'diamond'" /><span>{{ step.title }}</span><small>{{ stepStatus(step.status) }}</small></li></ol>
        <div v-if="!result.props.isError && resultView.matches.length" class="remote-trace-matches"><div v-for="(match, index) in resultView.matches" :key="index"><strong>{{ match.file }}<small v-if="match.line !== null && match.line !== undefined">:{{ match.line }}</small></strong><p v-if="match.content">{{ match.content }}</p></div><small>以上为前 {{ resultView.matches.length }} 项预览，原始结果见下方。</small></div>
        <RemoteTraceText :value="resultView.output" />
        <p v-if="resultView.truncated" class="remote-help">服务端返回的是截断结果。</p>
        <details v-if="resultView.output !== rawResult" class="remote-trace-original"><summary>查看原始返回</summary><RemoteTraceText :value="rawResult" /></details>
        <p v-if="review" class="remote-trace-review"><RemoteIcon name="shield" />{{ review.decision === 'allow' ? '自动审批允许' : '自动审批拒绝' }}：{{ review.reason }}</p>
        <button v-if="node.props.name === 'update_plan' && threadId" class="room-text-button" :disabled="planLoading" @click="loadPlan">{{ planLoading ? '正在读取清单…' : '查看当前行动清单' }}</button>
        <p v-if="planError" class="room-error" role="alert">{{ planError }}</p>
        <template v-if="planOpened"><p class="remote-help">当前行动清单{{ planView?.cached ? ' · 缓存内容' : '' }}</p><RemoteTaskPlan v-if="planView?.data" :plan="planView.data" /><p v-else class="remote-help">当前会话没有行动清单。</p></template>
      </div>
      <p v-else class="remote-help">{{ state === 'assembling' ? '参数尚未接收完整。' : state === 'failed' ? '此调用未返回结果，请结合后续错误记录核对。' : state === 'completed' ? '服务端标记已完成，未提供结果正文。' : '等待服务端返回执行结果…' }}</p>
      </component>
      <footer class="remote-trace-identity"><span v-if="node.props.name">{{ node.props.name }}</span><span v-if="node.props.callId">调用 {{ node.props.callId }}</span><span v-if="node.props.parentCallId">所属调用 {{ node.props.parentCallId }}</span></footer>
    </div>
  </details>
</template>
<script setup>
import { computed, ref, onBeforeUnmount } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteTraceText from './RemoteTraceText.vue'
import RemoteTaskPlan from './RemoteTaskPlan.vue'
import { remoteViews, readRemote } from '../../stores/remote'
import { currentToken, assertToken } from '../../stores/mode'
import { remoteItems, remoteStatus } from '../../utils/remoteMarkdown'
import { toolPresentation, toolTarget, toolResultView, traceDuration, traceText, nestedTraceCounts } from '../../utils/remoteTrace'
const props = defineProps({ node: { type: Object, required: true }, threadId: String })
const isScript = computed(() => props.node.kind === 'tool-call' && props.node.props.name === 'run_script')
const openedOnce = ref(false)
function activate(event) { if (event.target === event.currentTarget && event.currentTarget.open) openedOnce.value = true }
const counts = computed(() => nestedTraceCounts(props.node.children))
const childSummary = computed(() => {
  const value = counts.value, parts = [`内部调用 ${value.total} 项`]
  if (value.failed) parts.push(`失败 ${value.failed}`)
  if (value.active) parts.push(`进行中 ${value.active}`)
  if (value.completed) parts.push(`已完成 ${value.completed}`)
  if (value.unknown) parts.push(`状态待核对 ${value.unknown}`)
  return parts.join(' · ')
})
const planLoading = ref(false), planOpened = ref(false), planError = ref('')
const planView = computed(() => remoteViews.get(JSON.stringify({ kind: 'plan', threadId: props.threadId })))
let disposed = false
async function loadPlan() {
  if (planLoading.value || !props.threadId) return
  const token = currentToken(), threadId = props.threadId
  planLoading.value = true; planError.value = ''
  try { await readRemote({ kind: 'plan', threadId }); assertToken(token, 'remote'); if (!disposed && props.threadId === threadId) planOpened.value = true }
  catch (failure) { if (!disposed && props.threadId === threadId) planError.value = failure.message }
  finally { if (!disposed && props.threadId === threadId) planLoading.value = false }
}
onBeforeUnmount(() => { disposed = true })
const orphan = computed(() => props.node.kind === 'tool-result')
const result = computed(() => orphan.value ? props.node : props.node.resultNode)
const display = computed(() => orphan.value ? { label: '操作结果', icon: 'tool', category: '未匹配到调用参数', color: 'purple' } : toolPresentation(props.node.props.name, remoteItems(remoteViews.get(JSON.stringify({ kind: 'tools' }))?.data)))
const state = computed(() => result.value?.props.isError ? 'failed' : orphan.value ? 'completed' : props.node.props.state || 'unknown')
const callArgs = computed(() => props.node.props.state === 'assembling' ? props.node.props.argsPreview ?? props.node.props.args : props.node.props.args ?? props.node.props.argsPreview)
const target = computed(() => toolTarget(callArgs.value, display.value.style))
const duration = computed(() => traceDuration(result.value?.props.durationMs))
const rawResult = computed(() => traceText(result.value?.props.result))
const resultView = computed(() => toolResultView(props.node.props.name, result.value?.props.result))
const review = computed(() => { const value = result.value?.props.automaticApproval; return value?.source === 'automatic_reviewer' && ['allow', 'deny'].includes(value.decision) && typeof value.reason === 'string' ? value : null })
const stepStatus = status => ({ pending: '待处理', in_progress: '进行中', completed: '已完成', blocked: '受阻' }[status] || status)
</script>
