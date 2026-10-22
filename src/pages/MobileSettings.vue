<template>
  <CompanionPage class="settings-page" title="设置" subtitle="模型、偏好与应用数据" @back="goBack">
    <div class="settings-layout" :class="{ 'settings-layout--overview': tab === 'overview' }">
      <aside v-show="tab !== 'overview'" class="settings-navigation">
        <div class="settings-partner"><CharacterAvatar class="settings-portrait" :src="agent.avatar" :label="agent.name" /><div><CompanionEmblem /><strong>{{ agent.name }}</strong><small>当前伙伴</small></div></div>
        <nav aria-label="设置分类" role="tablist">
          <button v-for="(item, index) in categories" :id="`settings-tab-${item.id}`" :key="item.id" type="button" role="tab" :aria-selected="tab === item.id" :aria-controls="`settings-panel-${item.id}`" :tabindex="tab === item.id ? 0 : -1" :class="{ selected: tab === item.id }" @click="tab = item.id" @keydown="onCategoryKey($event, index)"><RoomIcon :name="item.icon" /><span>{{ item.label }}</span><i v-if="dirtySections[item.id]" aria-label="未保存" /></button>
        </nav>
      </aside>
      <section :id="`settings-panel-${tab}`" class="settings-content" role="tabpanel" :aria-labelledby="`settings-tab-${tab}`">
        <div :key="tab" class="settings-panel-body">
          <header v-if="tab !== 'overview'" class="settings-section-heading"><span class="settings-section-symbol"><RoomIcon :name="currentCategory.icon" /></span><div><h2>{{ currentCategory.label }}</h2><p>{{ currentCategory.description }}</p></div><span v-if="dirtySections[tab]" class="settings-unsaved">未保存</span></header>
          <FeedbackMessage :message="loadError" action-label="重试" @action="load" /><button v-if="loadError" class="room-text-button" @click="load"><RoomIcon name="refresh" />重新加载设置</button>
          <p v-if="loading" class="control-hint" role="status">正在读取设置…</p>

          <div v-if="tab === 'overview'" class="settings-overview">
            <button class="settings-overview-profile" @click="router.push('/roles')"><CharacterAvatar :src="agent.avatar" :label="agent.name" /><span><strong>{{ agent.name }}</strong><small>独立模式</small><small>每一位伙伴，都有自己的日常</small></span><RoomIcon name="arrow" /></button>
            <section class="settings-overview-group"><button v-for="item in categories.filter(item => ['api', 'model', 'user', 'role'].includes(item.id))" :key="item.id" :data-ui-tone="uiSectionTone(item.id)" @click="tab = item.id"><span class="overview-symbol"><RoomIcon :name="item.icon" /></span><span><strong>{{ item.label }}</strong><small>{{ item.description }}</small></span><RoomIcon name="arrow" /></button></section>
            <section class="settings-overview-group"><button data-ui-tone="violet" @click="tab = 'appearance'"><span class="overview-symbol"><RoomIcon name="palette" /></span><span><strong>显示与主题</strong><small>{{ displayPreferences.displayMode === 'live2d' ? '通用 Pio Live2D' : '静态立绘与 CG' }} · 配色与玻璃材质</small></span><i class="overview-color" /><RoomIcon name="arrow" /></button></section>
            <section class="settings-overview-group"><button @click="router.push({ path: '/', query: { panel: 'remote' } })"><span class="overview-symbol"><RoomIcon name="remote" /></span><span><strong>运行模式与远程连接</strong><small>独立陪伴 / infOS 客户端</small></span><RoomIcon name="arrow" /></button></section>
            <section class="settings-overview-group"><button @click="router.push({ path: '/', query: { panel: 'reminders' } })"><span class="overview-symbol"><RoomIcon name="bell" /></span><span><strong>日常提醒</strong><small>时间、待办与通知状态</small></span><RoomIcon name="arrow" /></button><button data-ui-tone="violet" @click="tab = 'memory'"><span class="overview-symbol"><RoomIcon name="diary" /></span><span><strong>日记</strong><small>与 {{ agent.name }} 的点滴记录</small></span><RoomIcon name="arrow" /></button></section>
            <section class="settings-overview-group"><button data-ui-tone="amber" @click="tab = 'data'"><span class="overview-symbol"><RoomIcon name="document" /></span><span><strong>数据与备份</strong><small>导出、恢复与本地数据管理</small></span><RoomIcon name="arrow" /></button></section>
          </div>
          <RoomAppearanceSettings v-if="tab === 'appearance'" />
          <form v-if="tab === 'api'" @submit.prevent="saveApi">
            <fieldset :disabled="loading || !loaded || !!busy">
              <div class="settings-field-grid">
                <label class="settings-field">服务商<ChoicePicker v-model="provider" label="选择服务商" :display-label="providerLabel" :options="providerChoices" :disabled="loading || !loaded || !!busy" @change="chooseProvider"><template #leading><BrandIcon :brand="providerBrand" :size="18" fallback="provider" /></template><template #option-leading="{ option }"><BrandIcon :brand="resolveProviderBrand(option.value)" :size="22" fallback="provider" /></template></ChoicePicker></label>
                <label class="settings-field">API 协议<ChoicePicker v-model="api.apiFormat" label="选择 API 协议" :options="protocolChoices" :disabled="loading || !loaded || !!busy" /></label>
              </div>
              <label class="settings-field">API 地址<input v-model="api.base" type="url" required spellcheck="false" autocomplete="off" placeholder="https://api.example.com/v1" /><small>支持自定义代理与本地服务</small></label>
              <label class="settings-field">API 密钥<div class="settings-secret"><input v-model="api.key" :type="showKey ? 'text' : 'password'" autocomplete="off" spellcheck="false" placeholder="填写密钥，本地服务可留空" /><button type="button" class="room-icon-button" :aria-label="showKey ? '隐藏密钥' : '显示密钥'" :aria-pressed="showKey" @click="showKey = !showKey"><RoomIcon name="eye" /></button></div></label>
              <label class="settings-field">模型名称<div class="settings-model-input"><BrandIcon :brand="resolveModelBrand(api.model)" :size="20" /><input v-model="api.model" required spellcheck="false" placeholder="填写模型 ID，或从下方选择" /></div></label>
              <div class="model-discovery"><button type="button" class="room-secondary-button" :aria-busy="busy === 'fetch'" @click="fetchModels"><RoomIcon name="refresh" :class="{ 'room-spinning': busy === 'fetch' }" />{{ busy === 'fetch' ? '正在获取…' : '获取模型列表' }}</button><input v-model="modelSearch" class="control-input" type="search" aria-label="搜索模型" placeholder="搜索模型 ID" /></div>
              <div v-if="models.length" class="model-options" aria-label="可用模型"><button v-for="model in filteredModels.slice(0, 100)" :key="model.id" type="button" :aria-pressed="api.model === model.id" :class="{ selected: api.model === model.id }" @click="api.model = model.id"><BrandIcon :brand="resolveModelBrand(model.id)" :size="20" /><span>{{ model.id }}</span><RoomIcon v-if="api.model === model.id" class="model-selected-check" name="check" /></button><p v-if="!filteredModels.length" class="control-hint">没有匹配的模型</p></div>
              <p v-if="filteredModels.length > 100" class="control-hint">匹配 {{ filteredModels.length }} 个模型，显示前 100 个；输入关键词可缩小范围。</p>
              <footer class="settings-save"><span>连接配置对所有本地角色生效</span><button class="room-primary-button" :disabled="!dirtySections.api">{{ busy === 'save-api' ? '正在保存…' : '保存连接' }}</button></footer>
            </fieldset>
          </form>

          <form v-else-if="tab === 'model'" @submit.prevent="saveParameters">
            <fieldset :disabled="loading || !loaded || !!busy">
              <section class="local-reasoning-settings"><header><ReasoningIcon :level="reasoningLevel(parameters.reasoningEffort)" /><strong>思考强度</strong><small>按模型能力选择</small></header><ReasoningPicker v-model="parameters.reasoningEffort" :options="reasoningChoices" :disabled="loading || !loaded || !!busy" /></section>
              <div class="model-number-grid local-model-numbers">
                <RemoteModelNumber tone="rose" :value="parameters.temperature" label="温度" help="较低更稳，较高更灵活" icon="sampling" :min="0" :max="2" @change="parameters.temperature = $event" />
                <RemoteModelNumber tone="mint" :value="parameters.topP" label="Top P" help="控制用词的选择范围" icon="filter" :min="0" :max="1" @change="parameters.topP = $event" />
                <RemoteModelNumber :value="parameters.maxTokens" label="最大输出" help="一次回复的长度上限" icon="reply-length" unit="Token" :min="128" :max="65536" integer required @change="parameters.maxTokens = $event" />
                <RemoteModelNumber tone="violet" :value="parameters.contextWindow" label="上下文窗口" help="可使用的完整上下文" icon="context-window" unit="Token" :min="1152" :max="2000000" integer required @change="parameters.contextWindow = $event" />
                <RemoteModelNumber tone="amber" class="local-history-number" :value="parameters.memoryRounds" label="历史对话轮次" help="一轮包含一条输入和一条回复" icon="chat" unit="轮" :min="1" :max="200" integer required @change="parameters.memoryRounds = $event" />
              </div>
              <label class="settings-toggle local-stream-toggle"><RoomIcon name="pulse" /><span><strong>流式回复</strong><small>边生成，边显示</small></span><RoomSwitch v-model="parameters.stream" label="流式回复" /></label>
              <footer class="settings-save"><span>未编辑的高级参数会保留</span><button class="room-primary-button" :disabled="!dirtySections.model">{{ busy === 'save-model' ? '正在保存…' : '保存参数' }}</button></footer>
            </fieldset>
          </form>

          <form v-else-if="tab === 'user'" @submit.prevent="saveUser">
            <fieldset :disabled="loading || !loaded || !!busy"><label class="settings-field">你的名字<input v-model="user.name" maxlength="200" autocomplete="name" placeholder="希望角色怎样称呼你" /></label><label class="settings-field">关于你<textarea v-model="user.persona" rows="9" maxlength="100000" placeholder="介绍你的喜好、日常或希望她了解的事情" /><small>这些资料会用于本地角色的对话。</small></label><footer class="settings-save"><span /><button class="room-primary-button" :disabled="!dirtySections.user">{{ busy === 'save-user' ? '正在保存…' : '保存资料' }}</button></footer></fieldset>
          </form>

          <div v-else-if="tab === 'role'" class="settings-destinations"><button @click="router.push('/roles')"><span class="destination-icon"><RoomIcon name="profile" /></span><span><strong>角色管理</strong><small>{{ agentList.length }} 位伙伴 · 编辑人设、头像与互动台词</small></span><RoomIcon name="arrow" /></button><p class="control-hint">角色的创建、编辑、导入和导出统一在角色管理中完成。</p></div>
          <div v-else-if="tab === 'memory'" class="settings-destinations"><button @click="router.push({ path: '/', query: { panel: 'journal' } })"><span class="destination-icon"><RoomIcon name="diary" /></span><span><strong>{{ agent.name }} 的日记</strong><small>搜索、查看来源、修订和删除</small></span><RoomIcon name="arrow" /></button><p class="control-hint">日记按角色独立保存，与房间中的日记入口使用同一界面。</p></div>

          <div v-else-if="tab === 'data'" class="settings-data-list">
            <article><div><RoomIcon name="download" /><h3>导出备份</h3></div><p>导出配置、角色、日记与对话，保存为 JSON 文件。</p><button class="room-secondary-button" :disabled="!!busy" @click="exportBackup">{{ busy === 'export' ? '正在导出…' : '导出全部数据' }}</button></article>
            <article><div><RoomIcon name="upload" /><h3>恢复备份</h3></div><p>从备份文件恢复数据。确认导入后会覆盖当前数据。</p><input ref="importInput" type="file" accept=".json,application/json" hidden @change="importBackup" /><button class="room-secondary-button" :disabled="!!busy" @click="importInput?.click()">{{ busy.startsWith('import') ? '正在处理…' : '选择备份文件' }}</button></article>
            <article class="settings-danger"><div><RoomIcon name="trash" /><h3>重置所有数据</h3></div><p>删除全部对话、日记与偏好设置。此操作无法撤销。</p><button class="room-danger-button" :disabled="!!busy" @click="resetData">重置所有数据</button></article>
          </div>
        </div>
      </section>
    </div>
  </CompanionPage>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import { onBeforeRouteLeave, useRouter } from 'vue-router'
import { appMessageBox as ElMessageBox } from '../services/appMessageBox'
import { appMessage as ElMessage } from '../services/appFeedback'
import FeedbackMessage from '../components/FeedbackMessage.vue'
import CompanionPage from '../components/companion/CompanionPage.vue'
import RoomIcon from '../components/companion/RoomIcon.vue'
import CharacterAvatar from '../components/companion/CharacterAvatar.vue'
import CompanionEmblem from '../components/companion/CompanionEmblem.vue'
import ChoicePicker from '../components/companion/ChoicePicker.vue'
import ReasoningPicker from '../components/companion/ReasoningPicker.vue'
import ReasoningIcon from '../components/companion/ReasoningIcon.vue'
import { reasoningLevel } from '../utils/reasoningAppearance'
import RoomSwitch from '../components/companion/RoomSwitch.vue'
import RemoteModelNumber from '../components/remote/RemoteModelNumber.vue'
import { uiSectionTone } from '../utils/uiPalette'
import BrandIcon from '../components/companion/BrandIcon.vue'
import RoomAppearanceSettings from '../components/companion/RoomAppearanceSettings.vue'
import { displayPreferences } from '../stores/appearance'
import { modelBrandDefinition, resolveModelBrand, resolveProviderBrand } from '../utils/modelBrands'
import { AGENTS, activeAgentId, agentList } from '../stores/agents'
import { getApiKey, getConfig, exportAllData, importAllData, resetAll } from '../api'
import { localInvoke } from '../services/localChat'
import { getModelConfiguration, saveApiConfiguration, saveModelParameters } from '../services/standalone'

const router = useRouter()
const tab = ref(router.currentRoute.value.query.section === 'data' ? 'data' : 'overview')
function goBack() { if (tab.value === 'overview') router.push('/'); else tab.value = 'overview' }
const categories = [
  { id: 'overview', label: '全部设置', icon: 'settings', description: '房间里的显示与使用偏好' },
  { id: 'appearance', label: '显示与主题', icon: 'palette', description: '角色显示、配色与玻璃材质' },
  { id: 'api', label: '模型连接', icon: 'connection', description: '选择聊天使用的服务、协议与模型' },
  { id: 'model', label: '回复偏好', icon: 'sliders', description: '调整回复风格、思考强度与长度' },
  { id: 'user', label: '个人资料', icon: 'profile', description: '让伙伴更了解你' },
  { id: 'role', label: '角色', icon: 'group', description: '管理每位伙伴的设定与外观' },
  { id: 'memory', label: '日记', icon: 'diary', description: '查看和管理角色记住的事情' },
  { id: 'data', label: '应用数据', icon: 'document', description: '备份、恢复与重置' },
]
const currentCategory = computed(() => categories.find(item => item.id === tab.value))
const agent = computed(() => AGENTS[activeAgentId.value] || { name: '伙伴' })
const api = reactive({ base: '', model: '', key: '', apiFormat: 'chat_completions' })
const parameters = reactive({ temperature: null, topP: null, memoryRounds: 40, stream: true, reasoningEffort: '', maxTokens: 4096, contextWindow: 32768 })
const user = reactive({ name: '', persona: '' })
const snapshots = reactive({ api: '', model: '', user: '' })
const loading = ref(true)
const loaded = ref(false)
const loadError = ref('')
const busy = ref('')
const showKey = ref(false)
const provider = ref('custom')
const models = ref([])
const modelSearch = ref('')
const importInput = ref(null)
const reasoningChoices = [{ value: '', label: '跟随模型默认值' }, ...Object.entries({ off: '关闭', none: '不思考', minimal: '极低', low: '低', medium: '中', high: '高', xhigh: '极高', max: '最高' }).map(([value, label]) => ({ value, label }))]
const protocolChoices = [
  { value: 'chat_completions', label: 'OpenAI Chat Completions', shortLabel: 'OpenAI Chat', description: 'OpenAI 兼容聊天接口' },
  { value: 'responses', label: 'OpenAI Responses', shortLabel: 'Responses', description: 'OpenAI Responses 接口' },
  { value: 'anthropic', label: 'Anthropic Messages', shortLabel: 'Anthropic', description: 'Claude 原生消息接口' },
  { value: 'gemini', label: 'Google Gemini', shortLabel: 'Gemini', description: 'Gemini 原生生成接口' },
]
const providers = [
  { id: 'openai', name: 'OpenAI', base: 'https://api.openai.com/v1', format: 'chat_completions' },
  { id: 'anthropic', name: 'Anthropic', base: 'https://api.anthropic.com/v1', format: 'anthropic' },
  { id: 'gemini', name: 'Google Gemini', base: 'https://generativelanguage.googleapis.com/v1beta', format: 'gemini' },
  { id: 'siliconflow', name: '硅基流动', base: 'https://api.siliconflow.cn/v1' },
  { id: 'deepseek', name: 'DeepSeek', base: 'https://api.deepseek.com' },
  { id: 'moonshot', name: 'Moonshot', base: 'https://api.moonshot.cn/v1' },
  { id: 'dashscope', name: '阿里百炼', base: 'https://dashscope.aliyuncs.com/compatible-mode/v1' },
  { id: 'volcengine', name: '火山引擎', base: 'https://ark.cn-beijing.volces.com/api/v3' },
  { id: 'groq', name: 'Groq', base: 'https://api.groq.com/openai/v1' },
  { id: 'zhipu', name: '智谱', base: 'https://open.bigmodel.cn/api/paas/v4' },
  { id: 'minimax', name: 'MiniMax', base: 'https://api.minimax.chat/v1' },
  { id: 'mistral', name: 'Mistral', base: 'https://api.mistral.ai/v1' },
  { id: 'yi', name: '零一万物', base: 'https://api.lingyiwanwu.com/v1' },
  { id: 'xai', name: 'xAI', base: 'https://api.x.ai/v1' },
  { id: 'stepfun', name: '阶跃星辰', base: 'https://api.stepfun.com/v1' },
  { id: 'hunyuan', name: '腾讯混元', base: 'https://api.hunyuan.cloud.tencent.com/v1' },
]
let disposed = false
const providerChoices = [{ value: 'custom', label: '自定义连接' }, ...providers.map(item => ({ value: item.id, label: item.name, description: item.base }))]
const providerBrand = computed(() => resolveProviderBrand(provider.value, api.base))
const providerLabel = computed(() => modelBrandDefinition(providerBrand.value)?.label || '自定义连接')
async function onCategoryKey(event, index) {
  const offset = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[event.key]
  if (offset === undefined && !['Home', 'End'].includes(event.key)) return
  event.preventDefault()
  const target = event.key === 'Home' ? 0 : event.key === 'End' ? categories.length - 1 : (index + offset + categories.length) % categories.length
  tab.value = categories[target].id
  await nextTick(); if (!disposed) document.getElementById(`settings-tab-${tab.value}`)?.focus()
}
let loadVersion = 0
const parameterValues = () => ({ ...parameters, temperature: parameters.temperature === '' ? null : parameters.temperature, topP: parameters.topP === '' ? null : parameters.topP, reasoningEffort: parameters.reasoningEffort || null })
const sectionValues = section => section === 'model' ? parameterValues() : { ...(section === 'api' ? api : user) }
const dirtySections = computed(() => Object.fromEntries(['api', 'model', 'user'].map(section => [section, loaded.value && JSON.stringify(sectionValues(section)) !== snapshots[section]])))
const filteredModels = computed(() => models.value.filter(model => model.id.toLowerCase().includes(modelSearch.value.trim().toLowerCase())))
const dismissed = error => error === 'cancel' || error === 'close'
function readText(value) { try { return JSON.parse(value) || '' } catch { return value || '' } }
function chooseProvider() {
  const selected = providers.find(item => item.id === provider.value)
  if (selected) { api.base = selected.base; api.apiFormat = selected.format || 'chat_completions' }
  models.value = []; modelSearch.value = ''
}
async function load() {
  if ((loading.value && loadVersion > 0) || busy.value) return
  const version = ++loadVersion
  loading.value = true; loadError.value = ''
  try {
    const [config, key, name, persona] = await Promise.all([getModelConfiguration(), getApiKey(), getConfig('ppc.userName'), getConfig('ppc.userPersonaText')])
    if (disposed || version !== loadVersion) return
    Object.assign(api, { base: config.base, model: config.model, apiFormat: config.apiFormat, key: key || '' })
    for (const field of Object.keys(parameters)) parameters[field] = config[field] ?? (field === 'reasoningEffort' ? '' : null)
    Object.assign(user, { name: readText(name), persona: readText(persona) })
    provider.value = providers.find(item => item.base === api.base && (item.format || 'chat_completions') === api.apiFormat)?.id || 'custom'
    for (const section of ['api', 'model', 'user']) snapshots[section] = JSON.stringify(sectionValues(section))
    loaded.value = true
  } catch (error) { if (!disposed && version === loadVersion) loadError.value = error.message || '设置读取失败，请重试' }
  finally { if (!disposed && version === loadVersion) loading.value = false }
}
async function run(kind, task, success) {
  if (busy.value) return
  busy.value = kind
  try { await task(); if (!disposed && success) ElMessage.success(success) }
  catch (error) { if (!disposed && !dismissed(error)) ElMessage.error(error.message || '操作失败，请重试') }
  finally { if (!disposed) busy.value = '' }
}
async function save(section, operation) {
  if (!loaded.value || busy.value) return
  const values = sectionValues(section)
  await run(`save-${section}`, async () => { await operation(values); if (!disposed) snapshots[section] = JSON.stringify(values) }, '已保存')
}
const saveApi = () => save('api', saveApiConfiguration)
const saveParameters = () => save('model', saveModelParameters)
const saveUser = () => save('user', values => localInvoke('local_save_settings', { section: 'user', values }))
async function fetchModels() {
  await run('fetch', async () => {
    const result = await localInvoke('fetch_models', { apiBase: api.base, apiKey: api.key, apiFormat: api.apiFormat })
    if (disposed) return
    models.value = Array.isArray(result) ? result : result.data || []
    ElMessage.success(`已获取 ${models.value.length} 个模型`)
  })
}
function download(data) {
  const url = URL.createObjectURL(new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' }))
  const anchor = document.createElement('a')
  anchor.href = url; anchor.download = `peroperochat-backup-${Date.now()}.json`
  document.body.appendChild(anchor); anchor.click(); anchor.remove()
  setTimeout(() => URL.revokeObjectURL(url), 1000)
}
const exportBackup = () => run('export', async () => { const data = await exportAllData(); if (!disposed) download(data) }, '备份已导出')
async function importBackup(event) {
  const file = event.target.files?.[0]
  event.target.value = ''
  if (!file) return
  await run('import', async () => {
    await ElMessageBox.confirm('导入后将覆盖当前配置、角色、日记与对话。确定恢复这份备份吗？', '恢复备份', { confirmButtonText: '恢复备份', cancelButtonText: '取消', type: 'warning' })
    if (disposed) return
    const data = JSON.parse(await file.text())
    if (disposed) return
    await importAllData(data)
    if (!disposed) location.reload()
  })
}
async function resetData() {
  await run('reset', async () => {
    await ElMessageBox.prompt('将删除全部对话、日记与偏好设置。请输入“重置所有数据”确认。', '重置所有数据', { confirmButtonText: '重置', cancelButtonText: '取消', type: 'warning', inputPlaceholder: '重置所有数据', inputValidator: value => value === '重置所有数据' || '请输入完整的确认文字', confirmButtonClass: 'el-button--danger' })
    if (disposed) return
    await resetAll()
    if (!disposed) location.reload()
  })
}
onBeforeRouteLeave(async () => {
  if (busy.value.startsWith('save-') || ['import', 'reset'].includes(busy.value)) { ElMessage.info('请等待当前操作完成'); return false }
  if (!Object.values(dirtySections.value).some(Boolean)) return true
  try { await ElMessageBox.confirm('还有尚未保存的更改，离开后不会保留。', '离开设置？', { confirmButtonText: '离开', cancelButtonText: '继续编辑' }); return true }
  catch { return false }
})
onMounted(load)
onBeforeUnmount(() => { disposed = true; loadVersion++ })
</script>

<style scoped>
.settings-layout--overview { display: block !important; }
.settings-layout--overview .settings-content { height: 100%; padding: 0; border: 0; background: transparent; backdrop-filter: none; }
.settings-layout--overview .settings-content::before { display: none; }
.settings-overview { max-width: 680px; margin: 0 auto; }
.settings-overview-profile { width: 100%; display: flex; align-items: center; gap: 16px; background: var(--room-glass); border: 1px solid #ffffffc9; border-radius: 22px; padding: 21px 18px; margin-bottom: 15px; text-align: left; backdrop-filter: blur(18px); }
.settings-overview-profile > :first-child { width: 62px; height: 62px; border: 3px solid white; border-radius: 50%; }.settings-overview-profile > span { flex: 1; min-width: 0; }.settings-overview-profile strong { display: block; font-size: 22px; font-weight: 600; }.settings-overview-profile small { display: block; margin-top: 5px; color: var(--room-muted); font-size: 11px; }
.settings-overview-group { padding: 0 15px; margin-bottom: 13px; border: 1px solid #ffffffc9; border-radius: 20px; background: var(--room-glass); backdrop-filter: blur(18px); }
.settings-overview-group > button { display: flex; gap: 12px; align-items: center; width: 100%; text-align: left; border: 0; border-bottom: 1px solid var(--room-line); background: transparent; padding: 15px 0; }.settings-overview-group > button:last-child { border-bottom: 0; }
.settings-overview-group button > span:nth-child(2) { flex: 1; min-width: 0; }.settings-overview-group strong { display: block; font-size: 13px; font-weight: 600; }.settings-overview-group small { display: block; margin-top: 5px; font-size: 10px; line-height: 1.6; color: var(--room-muted); }
.overview-symbol { display: grid; place-items: center; width: 35px; height: 35px; background: #ffffff9c; border: 1px solid #dce6f680; border-radius: 12px; color: var(--room-accent); flex: none; }.overview-symbol svg { width: 20px; height: 20px; }.settings-overview-group button > svg:last-child, .settings-overview-profile > svg:last-child { width: 13px; height: 13px; color: var(--room-muted); }
.overview-color { width: 20px; height: 20px; border-radius: 50%; background: var(--room-accent); border: 2px solid white; margin-right: 2px; }
.settings-page :deep(.companion-page-body) { display: flex; overflow: hidden; }
.settings-layout { flex: 1; min-height: 0; min-width: 0; display: grid; grid-template-columns: 200px minmax(0, 1fr); gap: 24px; }
.settings-navigation { min-height: 0; overflow-y: auto; }
.settings-partner { position: relative; display: flex; gap: 15px; align-items: center; padding: 0 0 22px; margin-bottom: 10px; border-bottom: 1px solid var(--room-line); }
.settings-partner::after { content: ''; width: 28px; height: 2px; position: absolute; bottom: -1px; left: 0; background: var(--room-accent); }
.settings-portrait { width: 74px; height: 92px; clip-path: polygon(0 0, calc(100% - 8px) 0, 100% 8px, 100% 100%, 8px 100%, 0 calc(100% - 8px)); }
.settings-partner > div { display: flex; flex-direction: column; gap: 6px; min-width: 0; }
.settings-partner > div > svg { width: 32px; height: 28px; color: var(--room-accent); margin-bottom: 3px; }
.settings-partner strong { font-size: 17px; font-weight: 700; overflow: hidden; text-overflow: ellipsis; }
.settings-partner small { font-size: 10px; color: var(--room-muted); }
.settings-navigation nav { display: flex; flex-direction: column; gap: 5px; }
.settings-navigation nav button { position: relative; display: flex; align-items: center; gap: 11px; min-height: 42px; padding: 10px 12px; border: 0; border-left: 2px solid transparent; border-radius: 0 6px 6px 0; background: transparent; color: var(--room-muted); text-align: left; font-size: 12px; }
.settings-navigation nav button.selected { background: var(--room-surface); border-left-color: var(--room-accent); color: var(--room-ink); font-weight: 600; }
.settings-navigation nav button.selected > svg { color: var(--room-accent); }
.settings-navigation nav button > svg { width: 16px; height: 16px; }
.settings-navigation nav button i { width: 4px; height: 4px; border-radius: 50%; background: var(--room-accent); margin-left: auto; }
.settings-content { position: relative; min-height: 0; min-width: 0; padding: 22px; border: 1px solid var(--room-line); border-radius: 3px 16px 3px 3px; background: var(--room-surface); overflow-y: auto; overscroll-behavior: contain; scrollbar-width: thin; scrollbar-color: var(--room-line) transparent; }
.settings-content::before { content: ''; position: absolute; top: 0; left: 22px; width: 30px; height: 2px; background: var(--room-accent); }
.settings-panel-body { animation: room-content-in .22s var(--room-ease); }
.settings-navigation nav button.selected > svg { animation: room-check-in .28s var(--room-spring); }
.settings-section-heading { display: flex; align-items: center; gap: 11px; margin-bottom: 21px; padding-bottom: 17px; border-bottom: 1px solid var(--room-line); }
.settings-section-symbol { display: grid; place-items: center; width: 32px; height: 36px; color: var(--room-accent); background: var(--room-accent-soft); clip-path: polygon(0 0, calc(100% - 6px) 0, 100% 6px, 100% 100%, 6px 100%, 0 calc(100% - 6px)); flex: none; }
.settings-section-heading > div { flex: 1; min-width: 0; }
.settings-section-heading h2 { margin: 0; font-size: 14px; font-weight: 650; }
.settings-section-heading p { margin: 5px 0 0; color: var(--room-muted); font-size: 10px; line-height: 1.6; }
.settings-unsaved { align-self: flex-start; padding: 3px 5px; background: var(--room-accent-soft); color: var(--room-accent); font-size: 9px; flex: none; }
fieldset { border: 0; padding: 0; margin: 0; min-width: 0; }
.settings-field-grid { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: 0 14px; }
.settings-secret { display: flex; align-items: center; border: 1px solid var(--room-line); border-radius: 5px; background: var(--room-subtle); padding-right: 3px; }
.settings-secret:focus-within { border-color: var(--room-accent); }
.settings-secret input { border: 0; box-shadow: none !important; }
.settings-secret .room-icon-button { width: 34px; height: 34px; color: var(--room-muted); }
.settings-model-input { display: flex; align-items: center; gap: 10px; padding-left: 12px; border: 1px solid var(--room-line); border-radius: 5px; background: var(--room-subtle); transition: border-color .18s, box-shadow .18s; }
.settings-model-input:focus-within { border-color: var(--room-accent); box-shadow: 0 0 0 2px var(--room-accent-soft); }
.settings-model-input input { min-width: 0; border: 0; box-shadow: none !important; background: transparent; }
.model-discovery { display: flex; gap: 10px; }
.model-discovery button { flex: none; display: flex; align-items: center; gap: 7px; font-size: 11px; }
.model-discovery button svg { width: 14px; height: 14px; }
.model-discovery input { min-width: 0; font-size: 12px; }
.model-options { max-height: 200px; overflow-y: auto; border: 1px solid var(--room-line); border-radius: 4px; padding: 4px; margin-top: 10px; }
.model-options button { display: flex; align-items: center; justify-content: space-between; gap: 10px; width: 100%; padding: 10px; border: 0; border-bottom: 1px solid var(--room-line); background: transparent; font-size: 11px; text-align: left; }
.model-options button:last-child { border-bottom: 0; }
.model-options button > span:not(.brand-icon) { flex: 1; min-width: 0; overflow-wrap: anywhere; }
.model-selected-check { width: 16px; height: 16px; animation: room-check-in .26s var(--room-spring); }
.model-options button.selected { background: var(--room-accent-soft); color: var(--room-accent); }
@media (hover: hover) { .model-options button:hover:not(.selected) { background: var(--room-subtle); } }
.settings-save { display: flex; align-items: center; justify-content: space-between; gap: 13px; border-top: 1px solid var(--room-line); padding-top: 16px; margin-top: 20px; }
.settings-save > span { font-size: 10px; color: var(--room-muted); line-height: 1.6; }
.settings-save button { flex: none; }
.settings-toggle { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 4px 0; }
.settings-toggle > span:not(.room-switch) { display: flex; flex-direction: column; gap: 6px; }
.settings-toggle strong { font-size: 12px; font-weight: 500; }
.settings-toggle small { color: var(--room-muted); font-size: 10px; }
.settings-destinations > button { display: flex; align-items: center; gap: 13px; width: 100%; padding: 16px 0; border: 0; border-bottom: 1px solid var(--room-line); background: transparent; text-align: left; }
.destination-icon { display: grid; place-items: center; width: 34px; height: 40px; color: var(--room-accent); flex: none; }
.settings-destinations > button > span:nth-child(2) { display: flex; flex-direction: column; gap: 7px; flex: 1; }
.settings-destinations strong { font-size: 13px; font-weight: 600; }
.settings-destinations small { color: var(--room-muted); font-size: 11px; line-height: 1.6; }
.settings-destinations > .control-hint { margin: 16px 0 0; }
.settings-data-list { display: flex; flex-direction: column; gap: 22px; }
.settings-data-list article + article { border-top: 1px solid var(--room-line); padding-top: 22px; }
.settings-data-list article > div { display: flex; align-items: center; gap: 9px; color: var(--room-accent); }
.settings-data-list h3 { margin: 0; color: var(--room-ink); font-size: 13px; font-weight: 600; }
.settings-data-list p { font-size: 11px; line-height: 1.7; color: var(--room-muted); margin: 9px 0 14px; }
.settings-data-list .settings-danger > div, .settings-data-list .settings-danger h3 { color: var(--room-danger); }
@media (max-width: 760px) { .settings-layout { grid-template-columns: 1fr; grid-template-rows: auto minmax(0, 1fr); gap: 12px; } .settings-navigation { overflow: visible; } .settings-partner { display: none; } .settings-navigation nav { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 2px 6px; } .settings-navigation nav button { min-height: 37px; gap: 7px; padding: 9px 7px; border: 0; border-bottom: 2px solid transparent; border-radius: 0; font-size: 11px; white-space: nowrap; } .settings-navigation nav button.selected { border-bottom-color: var(--room-accent); background: #ffffff8c; } .settings-navigation nav button > svg { width: 14px; height: 14px; } .settings-navigation nav button i { position: absolute; top: 5px; right: 4px; } .settings-content { padding: 17px 15px; } .settings-content::before { left: 15px; } .settings-section-heading { margin-bottom: 18px; padding-bottom: 14px; } }
@media (max-width: 340px) { .settings-field-grid { gap: 0 10px; } .settings-navigation nav button { gap: 5px; padding-left: 4px; padding-right: 4px; } .model-discovery { flex-wrap: wrap; } .settings-save > span { max-width: 46%; } }
</style>
