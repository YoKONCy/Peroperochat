<template>
  <section class="phase2 phase-insights" :aria-busy="busy">
    <RemoteFeedback :message="error" />
    <template v-if="kind === 'usage'">
      <div class="phase-periods"><button v-for="value in ['today', '7d', '30d']" :key="value" :class="{ selected: period === value }" :aria-pressed="period === value" :disabled="busy" @click="period = value; run(load)">{{ remoteChoiceLabel(value) }}</button><button :aria-label="t('筛选')" @click="filterOpen = true"><RemoteIcon name="filter" /></button></div>
      <p v-if="filters.agentId || filters.modelId || filters.threadId" class="phase-context"><RemoteIcon name="filter" />{{ filterDescription }}<button class="room-text-button" :disabled="busy" @click="clearFilters">{{ t('清除筛选') }}</button></p>
      <template v-if="data?.totals">
        <section class="phase-stats-hero"><small>{{ t('累计用量') }}</small><div class="phase-stats-number"><strong>{{ remoteNumber(data.totals.totalTokens) }}</strong><span>Token</span></div><div class="phase-stats-split"><div><small>{{ t('输入') }}</small><strong>{{ remoteNumber(data.totals.inputTokens) }}</strong></div><div><small>{{ t('输出') }}</small><strong>{{ remoteNumber(data.totals.outputTokens) }}</strong></div><div><small>{{ t('调用') }}</small><strong>{{ remoteNumber(data.totals.requests) }}</strong></div></div></section>
        <section v-if="data.buckets?.length" class="phase-card"><h3><RemoteIcon name="chart" />{{ t('用量趋势') }}</h3><svg class="phase-chart" viewBox="0 0 280 110" role="img" :aria-label="t('用量趋势')"><path class="grid" d="M0 15h280M0 55h280M0 95h280" /><path class="fill" :d="chartPath + ' L280 110 L0 110Z'" /><path class="curve" :d="chartPath" /></svg><div class="phase-chart-labels"><span>{{ data.buckets[0].label || data.buckets[0].key }}</span><span>{{ data.buckets.at(-1).label || data.buckets.at(-1).key }}</span></div></section>
        <section v-if="data.models?.length" class="phase-card"><h3><RemoteIcon name="model" />{{ t('模型分布') }}</h3><div v-for="item in data.models" :key="item.key" class="phase-usage-model"><div><BrandIcon :brand="resolveModelBrand(item.modelId) || resolveProviderBrand(item.provider)" :size="19" /><span>{{ item.modelId || item.key }}</span><small>{{ remoteNumber(item.totalTokens) }}</small></div><div class="phase-bar"><i :style="{ width: `${Math.min(100, Math.max(0, Number(item.totalTokens || 0) / Math.max(1, Number(data.totals.totalTokens || 0)) * 100))}%` }" /></div></div></section>
        <p v-if="!data.totals.requests" class="phase-empty">{{ t('尚无用量记录') }}</p>
        <p class="phase-context"><RemoteIcon name="clock" />{{ data.timeZone }} · {{ remoteDate(data.generatedAt) }}</p>
      </template>
    </template>
    <template v-else-if="data">
      <section class="phase-health"><RemoteIcon :name="remoteState.connection === 'online' ? 'check' : 'wifi'" /><div><h3>{{ t(remoteState.connection === 'online' ? '服务已连接' : '暂时离线') }}</h3><p>infOS {{ data.version || '' }}</p></div></section>
      <div class="phase-metric-grid"><div><small>{{ t('运行任务') }}</small><strong>{{ jobsLoaded ? remoteNumber(jobs.length) : '—' }}</strong></div><div><small>{{ t('在线设备') }}</small><strong>{{ remoteNumber(data.gateway?.connectedNodes) }}</strong></div><div><small>{{ t('已启用角色') }}</small><strong>{{ remoteNumber(data.agents?.enabled) }}</strong></div></div>
      <section class="phase-card"><h3><RemoteIcon name="pulse" />{{ t('服务端资源') }}</h3><dl class="phase-facts"><dt>CPU</dt><dd>{{ remoteNumber(data.runtime?.cpuPercent, { maximumFractionDigits: 1 }) }} %</dd><dt>{{ t('内存') }}</dt><dd>{{ remoteNumber(data.runtime?.memoryUsage?.rss, { maximumFractionDigits: 1 }) }} MB</dd></dl></section>
      <section class="phase-card"><h3><RemoteIcon name="pulse" />{{ t('当前在忙什么') }}</h3><button v-for="item in jobs" :key="item.id || item.taskId" class="phase-running-job" :disabled="!item.threadId || !item.agentId" @click="$emit('thread', { threadId: item.threadId, agentId: item.agentId, title: item.title })"><RemoteAvatar :id="item.agentId" :label="agentName(item.agentId)" /><span>{{ item.title || item.instruction || item.name || t('执行任务') }}<small>{{ agentName(item.agentId) }} · {{ remoteChoiceLabel(item.status || item.state || 'running') }}</small></span><RemoteIcon name="next" /></button><p v-if="jobsLoaded && !jobs.length" class="phase-empty">{{ t('没有正在执行的任务') }}</p><RemoteFeedback :message="jobsError" /></section>
      <section class="phase-card"><h3><RemoteIcon name="shield" />{{ t('连接与版本') }}</h3><dl class="phase-facts"><dt>{{ t('服务版本') }}</dt><dd>{{ data.version || '—' }}</dd><dt>{{ t('运行时间') }}</dt><dd>{{ uptime }}</dd><dt>{{ t('服务端存储') }}</dt><dd>{{ remoteNumber(data.storage?.sqliteSizeMB, { maximumFractionDigits: 1 }) }} MB</dd></dl></section>
    </template>
    <p v-if="busy && !data" class="phase-empty" role="status">{{ t('正在读取…') }}</p><button class="room-secondary-button" :disabled="busy" @click="run(load)"><RemoteIcon name="refresh" />{{ t('刷新') }}</button>
    <RoomDrawer v-if="filterOpen" :title="t('筛选')" :busy="busy" @close="filterOpen = false"><form class="phase2 phase-filters" @submit.prevent="applyFilters"><RemoteSelect v-model="filterDraft.agentId" :label="t('角色')" :options="agentOptions" @update:model-value="reloadFilterOptions" /><RemoteSelect v-model="modelSelection" :label="t('模型')" :options="modelOptions" /><RemoteSelect v-model="filterDraft.threadId" :label="t('会话')" :options="threadOptions" /><button class="room-primary-button" :disabled="busy || optionsBusy" type="submit">{{ t('应用筛选') }}</button></form></RoomDrawer>
  </section>
</template>
<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import RemoteFeedback from './RemoteFeedback.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteAvatar from './RemoteAvatar.vue'
import RemoteSelect from './RemoteSelect.vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import BrandIcon from '../companion/BrandIcon.vue'
import { resolveModelBrand, resolveProviderBrand } from '../../utils/modelBrands'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { remoteState } from '../../stores/remote'
import { t, remoteChoiceLabel, remoteNumber, remoteDate } from '../../i18n/remote'
const props = defineProps({ kind: String })
defineEmits(['thread'])
const { read, run, guard, busy, writing, error } = useRemoteManagement()
const data = ref(null), jobs = ref([]), jobsLoaded = ref(false), jobsError = ref(''), period = ref('7d'), filterOpen = ref(false), filters = ref({}), filterDraft = ref({}), options = ref({}), optionsBusy = ref(false)
let optionsVersion = 0
defineExpose({ busy, writing })
function agentName(id) { return remoteState.agents.find(item => item.id === id)?.name || id || '—' }
const agentOptions = computed(() => [{ value: '', label: t('全部角色') }, ...remoteState.agents.filter(item => !item.isUser).map(item => ({ value: item.id, label: item.name }))])
const modelOptions = computed(() => [{ value: '', label: t('全部模型') }, ...(options.value.models || []).map(item => ({ value: JSON.stringify([item.provider, item.modelId]), label: item.modelId }))])
const threadOptions = computed(() => [{ value: '', label: t('全部会话') }, ...(options.value.threads || []).map(item => ({ value: item.id, label: item.title || t('会话') }))])
const modelSelection = computed({ get: () => filterDraft.value.modelId ? JSON.stringify([filterDraft.value.provider, filterDraft.value.modelId]) : '', set: value => { const pair = value ? JSON.parse(value) : []; filterDraft.value = { ...filterDraft.value, provider: pair[0] || '', modelId: pair[1] || '' } } })
const filterDescription = computed(() => [filters.value.agentId ? agentName(filters.value.agentId) : '', filters.value.modelId, filters.value.threadId ? t('会话') : ''].filter(Boolean).join(' · '))
const chartPath = computed(() => { const buckets = data.value?.buckets || [], max = Math.max(1, ...buckets.map(item => Number(item.totalTokens) || 0)); return buckets.map((item, index) => `${index ? 'L' : 'M'}${buckets.length === 1 ? 140 : index * 280 / (buckets.length - 1)} ${100 - Math.max(0, Number(item.totalTokens) || 0) / max * 85}`).join(' ') })
const uptime = computed(() => { const seconds = Number(data.value?.runtime?.uptime ?? data.value?.uptime); return Number.isFinite(seconds) ? `${Math.floor(seconds / 3600)}h ${Math.floor(seconds % 3600 / 60)}m` : '—' })
async function load() {
  if (props.kind === 'usage') {
    const params = { period: period.value, timeZone: Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC', ...filters.value }
    for (const key of Object.keys(params)) if (!params[key]) delete params[key]
    data.value = await read('system.usage', { params })
  } else {
    data.value = await read('system.info'); jobsError.value = ''
    try { const value = await read('runtime.tasks'); jobs.value = Array.isArray(value) ? value : value?.tasks || value?.items || []; jobsLoaded.value = true }
    catch (failure) { guard(); jobsError.value = failure.message; jobsLoaded.value = false }
  }
}
async function reloadFilterOptions() { const version = ++optionsVersion; optionsBusy.value = true; try { const value = await read('system.usage_filters', { params: filterDraft.value.agentId ? { agentId: filterDraft.value.agentId } : {} }); if (version === optionsVersion) { options.value = value || {}; if (!threadOptions.value.some(item => item.value === filterDraft.value.threadId)) filterDraft.value.threadId = '' } } catch (failure) { try { guard(); if (version === optionsVersion) error.value = failure.message } catch { /* 已离开页面。 */ } } finally { if (version === optionsVersion) optionsBusy.value = false } }
function applyFilters() { filters.value = { ...filterDraft.value }; filterOpen.value = false; run(load) }
function clearFilters() { filters.value = {}; run(load) }
watch(filterOpen, open => { if (open) { filterDraft.value = { ...filters.value }; reloadFilterOptions() } })
onMounted(() => run(load))
</script>
