<template>
  <section class="remote-record-detail">
    <header class="remote-record-heading"><RemoteAvatar v-if="kind === 'agents' && record.id" :id="record.id" :label="record.name" fit="contain" /><BrandIcon v-else-if="['models', 'providers'].includes(kind)" :brand="resolveModelBrand(record.modelId) || resolveProviderBrand(record.provider || record.brand, record.apiBase)" :size="32" fallback="model" /><RemoteIcon v-else :name="icon" /><div><small>{{ label }}</small><h3>{{ title }}</h3></div></header>
    <div v-if="tags.length" class="remote-record-badges"><span v-for="tag in tags" :key="tag">{{ tag }}</span></div>
    <article v-if="body" class="remote-record-body">{{ body }}</article>
    <dl v-if="facts.length" class="remote-record-facts"><div v-for="fact in facts" :key="fact.key"><dt>{{ fact.label }}</dt><dd>{{ fact.value }}</dd></div></dl>
    <section v-for="group in summaries" :key="group.key"><h4>{{ group.label }}</h4><dl class="remote-record-facts"><div v-for="item in group.items" :key="item.key"><dt>{{ item.label }}</dt><dd>{{ item.value }}</dd></div></dl></section>
    <section v-for="group in collections" :key="group.key"><h4>{{ group.label }} · {{ group.items.length }}</h4><article v-for="(item, index) in group.items.slice(0, 30)" :key="item?.id || index" class="remote-record-part"><strong>{{ item?.name || item?.title || item?.displayName || item?.id || '第 ' + (index + 1) + ' 项' }}</strong><p>{{ item?.description || item?.message || item?.instruction || item?.endpoint || item?.address || (typeof item !== 'object' ? item : '') }}</p><small v-if="item?.status || item?.state">{{ remoteStatus(item.status || item.state) }}</small><RemoteDataDetails v-if="item && typeof item === 'object'" label="详细信息" :value="item" /></article><p v-if="group.items.length > 30" class="remote-help">其余项目可在详细数据中查看。</p></section>
    <section v-if="kind === 'agents' && record.publicProfile" class="remote-record-profile"><h4>公开资料</h4><dl class="remote-record-facts"><div v-for="(value, key) in record.publicProfile" :key="key"><dt>{{ fieldLabels[key] || key }}</dt><dd>{{ display(value) }}</dd></div></dl></section>
    <section v-if="kind === 'documents' && record.parts?.length" class="remote-record-parts"><h4>文档内容 · {{ record.parts.length }} 个片段</h4><details v-for="(part, index) in record.parts" :key="part.id || index"><summary>{{ part.title || `片段 ${index + 1}` }}</summary><p>{{ part.summary || part.content || '此片段暂无摘要' }}</p></details></section>
    <section v-if="kind === 'task-management' && steps.length" class="remote-record-parts"><h4>执行步骤</h4><ol><li v-for="(step, index) in steps" :key="index">{{ step.title || step.description || step.content || display(step) }}<small>{{ remoteStatus(step.status || step.state) }}</small></li></ol></section>
    <p v-if="record.error" class="room-error">{{ record.error }}</p>
    <RemoteDataDetails :value="record" />
  </section>
</template>
<script setup>
import { computed } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteAvatar from './RemoteAvatar.vue'
import RemoteDataDetails from './RemoteDataDetails.vue'
import BrandIcon from '../companion/BrandIcon.vue'
import { fieldLabels, isSecretField, recordTitle } from '../../services/remoteManagement'
import { resolveModelBrand, resolveProviderBrand } from '../../utils/modelBrands'
import { remoteStatus } from '../../utils/remoteMarkdown'
const props = defineProps({ kind: String, record: { type: Object, default: () => ({}) } })
const label = computed(() => ({ agents: '角色资料', models: '模型配置', providers: '服务商连接', memories: '记忆档案', facts: '事实记录', documents: '知识文档', 'task-management': '执行任务', scheduler: '定时安排', usage: '用量概览', runtime: '运行状态', access: '设备接入', proactive: '陪伴策略', integrations: '集成状态' }[props.kind] || '详情'))
const icon = computed(() => ({ memories: 'memory', facts: 'document', documents: 'files', 'task-management': 'tasks', scheduler: 'bell', access: 'connection', proactive: 'heart' }[props.kind] || 'settings'))
const title = computed(() => { const value = recordTitle(props.record); return value === '记录' ? label.value : value })
const body = computed(() => [props.record.narrative, props.record.content, props.record.instruction, props.record.description, props.record.summary, props.record.currentGoal].find(value => typeof value === 'string' && value) || '')
const tags = computed(() => [props.record.status || props.record.state ? remoteStatus(props.record.status || props.record.state) : null, props.record.enableVision ? '视觉输入' : null, props.record.enableAudioInput ? '音频输入' : null, props.record.archivedAt ? '已归档' : null, ...(Array.isArray(props.record.topics) ? props.record.topics.slice(0, 6) : [])].filter(Boolean).map(String))
function display(value) { if (value == null) return '未设置'; if (typeof value === 'boolean') return value ? '开启' : '关闭'; if (Array.isArray(value)) return `${value.length} 项`; if (typeof value === 'object') return value.name || value.title || '已配置'; return String(value) }
const factKeys = ['message', 'modelId', 'provider', 'protocol', 'apiBase', 'contextWindowTokens', 'maxTokens', 'reasoningEffort', 'temperature', 'importance', 'eventAt', 'createdAt', 'updatedAt', 'time', 'nextRunAt', 'repeatIntervalMinutes', 'enabled', 'isEnabled', 'connected', 'listening', 'port', 'expiresAt', 'total', 'usedTokens', 'maxInputTokens', 'durationMs', 'memoryRetrievalDeferred', 'partCount', 'scope', 'mode', 'timeZone']
const facts = computed(() => Object.entries(props.record).filter(([key, value]) => factKeys.includes(key) && !isSecretField(key) && value != null && typeof value !== 'object').map(([key, value]) => ({ key, label: fieldLabels[key] || ({ usedTokens: '已用 Token', maxInputTokens: '输入上限', message: '说明', partCount: '内容片段' })[key] || key, value: /At$/.test(key) && !Number.isNaN(new Date(value).getTime()) ? new Date(value).toLocaleString() : display(value) })))
const steps = computed(() => { const value = props.record.plan?.steps || props.record.steps; return Array.isArray(value) ? value : [] })
const summaryKeys = ['summary', 'stats', 'counts', 'runtime', 'storage', 'health', 'gateway', 'usage', 'extensions', 'features']
const summaries = computed(() => summaryKeys.flatMap(key => { const value = props.record[key]; if (!value || typeof value !== 'object' || Array.isArray(value)) return []; const items = Object.entries(value).filter(([name, item]) => !isSecretField(name) && item != null && typeof item !== 'object').map(([name, item]) => ({ key: name, label: fieldLabels[name] || name, value: display(item) })); return items.length ? [{ key, label: fieldLabels[key] || key, items }] : [] }))
const collectionKeys = ['devices', 'connections', 'servers', 'tasks', 'sessions', 'warnings', 'issues', 'privateFacts', 'addresses', 'suggestions']
const collections = computed(() => collectionKeys.flatMap(key => Array.isArray(props.record[key]) && props.record[key].length ? [{ key, label: fieldLabels[key] || key, items: props.record[key] }] : []))
</script>
