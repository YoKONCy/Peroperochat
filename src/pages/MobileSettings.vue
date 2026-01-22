<template>
  <div class="settings-container">
    <!-- 顶部导航 -->
    <div class="header">
      <div class="back-btn" @click="goHome">
        <el-icon><ArrowLeft /></el-icon>
      </div>
      <div class="header-title">
        <h2>设置中心</h2>
        <span>Settings</span>
      </div>
      <div class="placeholder"></div>
    </div>

    <!-- 设置主体 -->
    <div class="settings-body">
      <!-- 快捷菜单卡片 -->
      <div class="menu-grid">
        <div 
          v-for="item in menuItems" 
          :key="item.id" 
          :class="['menu-item', { active: tab === item.id }]"
          @click="tab = item.id"
        >
          <div class="menu-icon">
            <el-icon><component :is="item.icon" /></el-icon>
          </div>
          <span class="menu-label">{{ item.label }}</span>
        </div>
      </div>

      <!-- 设置内容区 -->
      <div class="content-area">
        <Transition name="fade-slide" mode="out-in">
          <div :key="tab" class="content-card">
            
            <!-- 角色选择 -->
            <div v-if="tab === 'role'" class="section">
              <div class="section-header">
                <el-icon><Avatar /></el-icon>
                <h3>角色选择</h3>
              </div>
              <el-form label-position="top">
                <div class="info-banner">
                  <p>在此选择当前与你交互的 AI 角色。</p>
                </div>
                <el-form-item label="当前交互角色">
                  <el-radio-group v-model="activeAgentId" size="large" class="role-group">
                    <el-radio-button label="pero">Pero (软萌AI)</el-radio-button>
                    <el-radio-button label="nana">Nana (雌小鬼)</el-radio-button>
                  </el-radio-group>
                </el-form-item>
              </el-form>
            </div>

            <!-- 规则配置 (New) -->
            <div v-if="tab === 'rules'" class="section">
              <div class="section-header">
                <el-icon><Edit /></el-icon>
                <h3>规则配置</h3>
              </div>
              <el-form label-position="top">
                <div class="info-banner">
                  <p>分别为不同角色配置专属的规则补充。</p>
                </div>
                <el-form-item label="选择要配置的角色">
                  <el-radio-group v-model="editingRuleAgentId" size="large">
                    <el-radio-button label="pero">Pero</el-radio-button>
                    <el-radio-button label="nana">Nana</el-radio-button>
                  </el-radio-group>
                </el-form-item>
                <el-form-item :label="`${AGENTS[editingRuleAgentId]?.name || '角色'} 规则补充 (注入到 <Output_Constraint> 末尾)`">
                  <el-input 
                    v-model="ruleSupplement" 
                    type="textarea" 
                    :rows="8" 
                    placeholder="在此输入 <ADD>...</ADD> 规则，例如：
<ADD>
- 禁止使用颜文字
- 必须每句话都带“喵”
</ADD>" 
                  />
                </el-form-item>
                <el-button type="primary" class="full-btn" @click="saveRuleSettings">保存 {{ AGENTS[editingRuleAgentId]?.name }} 的规则</el-button>
              </el-form>
            </div>

            <!-- API 设置 -->
            <div v-if="tab === 'api'" class="section">
              <div class="section-header">
                <el-icon><Cpu /></el-icon>
                <h3>API 配置</h3>
              </div>
              <el-form label-position="top">
                <el-form-item label="模型名称">
                  <el-input v-model="modelName" placeholder="请先获取模型" />
                </el-form-item>
                <el-form-item label="API 地址">
                  <el-input v-model="apiBase" placeholder="https://api.openai.com" />
                </el-form-item>
                <el-form-item label="API 秘钥">
                  <el-input v-model="apiKey" type="password" placeholder="sk-..." show-password />
                </el-form-item>
                <div class="api-tools">
                  <el-button type="primary" plain @click="fetchModels">获取模型</el-button>
                  <el-input v-model="modelSearch" placeholder="搜索模型..." clearable />
                </div>
                <el-table :data="filteredModels" height="200" @row-click="row => modelName = row.id" class="mini-table">
                  <el-table-column width="40">
                    <template #default="{ row }">
                      <el-radio v-model="modelName" :label="row.id"><span></span></el-radio>
                    </template>
                  </el-table-column>
                  <el-table-column prop="id" label="可用模型列表" />
                </el-table>
                <el-button type="primary" class="full-btn" @click="applySettings">保存 API 设置</el-button>
              </el-form>
            </div>

            <!-- 远程连接 -->
            <div v-if="tab === 'remote'" class="section">
              <div class="section-header">
                <el-icon><Connection /></el-icon>
                <h3>远程连接</h3>
              </div>
              <div class="info-banner">
                <p>连接到 PC 集成版或私有服务器，共享记忆与更强 AI 能力。</p>
              </div>
              <el-form label-position="top">
                <el-form-item label="启用远程模式">
                  <el-switch v-model="remoteEnabled" active-text="开启" inactive-text="关闭" />
                </el-form-item>
                <el-form-item label="服务器地址">
                  <el-input v-model="remoteUrl" placeholder="http://192.168.x.x:3000" :disabled="!remoteEnabled" />
                </el-form-item>
                <el-form-item label="访问令牌 (Handshake Token)">
                  <el-input v-model="remoteToken" type="password" placeholder="pero_default_token" show-password :disabled="!remoteEnabled" />
                </el-form-item>
                <el-button type="primary" class="full-btn" @click="applyRemoteSettings" :disabled="!remoteEnabled">保存并连接</el-button>
              </el-form>
            </div>

            <!-- 模型偏好 -->
            <div v-if="tab === 'model'" class="section">
              <div class="section-header">
                <el-icon><Operation /></el-icon>
                <h3>模型参数</h3>
              </div>
              <el-form label-position="top">
                <div class="slider-group">
                  <div class="slider-item">
                    <span>Temperature: {{ temperature }}</span>
                    <el-slider v-model="temperature" :min="0" :max="2" :step="0.01" />
                  </div>
                  <div class="slider-item">
                    <span>Top P: {{ topP }}</span>
                    <el-slider v-model="topP" :min="0" :max="1" :step="0.01" />
                  </div>
                </div>
                <el-form-item label="记忆轮次 (1轮=1对对话)">
                  <el-input-number v-model="memoryRounds" :min="1" :max="500" style="width: 100%" />
                </el-form-item>
                <el-form-item label="流式传输">
                  <el-switch v-model="stream" />
                </el-form-item>
                <el-button type="primary" class="full-btn" @click="applyModelSettings">确定</el-button>
              </el-form>
            </div>

            <!-- 用户设定 -->
            <div v-if="tab === 'user'" class="section">
              <div class="section-header">
                <el-icon><User /></el-icon>
                <h3>用户设定</h3>
              </div>
              <el-form label-position="top">
                <el-form-item label="我的名字">
                  <el-input v-model="userName" placeholder="填写你的名字" />
                </el-form-item>
                <el-form-item label="我的人设 (让 Pero 更好的理解你)">
                  <el-input v-model="userPersonaText" type="textarea" :rows="8" placeholder="例如：性格温柔、喜欢吃甜食..." />
                </el-form-item>
                <el-button type="primary" class="full-btn" @click="applyUserSettings">确定</el-button>
              </el-form>
            </div>

            <!-- 记忆管理 -->
            <div v-if="tab === 'memory'" class="section">
              <div class="section-header">
                <el-icon><Collection /></el-icon>
                <h3>记忆管理</h3>
              </div>
              <el-input v-model="memorySearch" placeholder="搜索记忆..." clearable class="search-input">
                <template #prefix><el-icon><Search /></el-icon></template>
              </el-input>
              <div class="memory-list">
                <div v-if="filteredMemories.length === 0" class="empty">暂无相关记忆</div>
                <div v-for="m in filteredMemories" :key="m.id" class="memory-item">
                  <div class="memory-content markdown-body" v-html="renderMarkdown(m.content)"></div>
                  <div v-if="m.tags && m.tags.length" class="memory-tags">
                    <el-tag 
                      v-for="tag in m.tags" 
                      :key="tag" 
                      size="small" 
                      type="info" 
                      effect="plain" 
                      round
                      class="m-tag"
                    >
                      {{ tag }}
                    </el-tag>
                  </div>
                  <div class="memory-footer">
                    <span class="m-time">{{ m.realTime || '未知时间' }}</span>
                    <el-button type="danger" size="small" link @click="deleteMemory(m.id)">删除</el-button>
                  </div>
                </div>
              </div>
            </div>

            <!-- 危险区域 -->
            <div v-if="tab === 'danger'" class="section">
              <div class="section-header danger">
                <el-icon><Warning /></el-icon>
                <h3>危险区域</h3>
              </div>
              <div class="danger-card">
                <h4>数据重置</h4>
                <p>将删除所有对话、记忆及偏好设置。</p>
                <el-button type="danger" @click="handleResetAll">重置所有数据</el-button>
              </div>
            </div>

          </div>
        </Transition>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import axios from 'axios'
import { ElMessageBox, ElMessage } from 'element-plus'
import { getDefaultPrompts, resetAll, AGENTS, getActiveAgentId, setActiveAgentId } from '../api'
import { db } from '../db'
import { marked } from 'marked'
import DOMPurify from 'dompurify'
import { 
  Search, Delete, InfoFilled, Connection, ArrowLeft, 
  Cpu, Operation, User, Collection, Warning, Cpu as ApiIcon, Avatar,
  Edit
} from '@element-plus/icons-vue'

const tab = ref('role')

// Markdown 渲染
function renderMarkdown(text) {
  if (!text) return ''
  const clean = cleanMessageContent(text)
  const html = marked.parse(clean)
  return DOMPurify.sanitize(html)
}

// 清理消息中的隐藏标签
function cleanMessageContent(text) {
  if (!text) return ''
  if (text === '__loading__') return '正在思考...'
  
  // 移除所有 XML 标签及其内容
  return text
    .replace(/<([A-Z_]+)>[\s\S]*?<\/\1>/g, '')
    .replace(/\[\[\[NIT_CALL\]\]\][\s\S]*?\[\[\[NIT_END\]\]\]/g, '')
    .trim()
}

const menuItems = [
  { id: 'role', label: '角色', icon: Avatar },
  { id: 'rules', label: '规则', icon: Edit },
  { id: 'api', label: 'API', icon: Cpu },
  { id: 'remote', label: '远程', icon: Connection },
  { id: 'model', label: '模型', icon: Operation },
  { id: 'user', label: '用户', icon: User },
  { id: 'memory', label: '记忆', icon: Collection },
  { id: 'danger', label: '危险', icon: Warning },
]

// Role & Rules Settings
const activeAgentId = ref(getActiveAgentId())
const editingRuleAgentId = ref('pero') // 当前正在编辑规则的角色ID
const ruleSupplement = ref('')

// Load rule supplement for the agent currently being edited
const loadRuleSupplement = () => {
  const agentId = editingRuleAgentId.value
  ruleSupplement.value = localStorage.getItem(`ppc.${agentId}.ruleSupplement`) || ''
}

// Watch active agent change (Global Switch)
watch(activeAgentId, (newVal) => {
  setActiveAgentId(newVal)
  ElMessage.success(`已切换为 ${AGENTS[newVal].name}`)
})

// Watch editing agent change (Reload Rules)
watch(editingRuleAgentId, () => {
  loadRuleSupplement()
})

const saveRuleSettings = () => {
  const agentId = editingRuleAgentId.value
  localStorage.setItem(`ppc.${agentId}.ruleSupplement`, ruleSupplement.value)
  ElMessage.success(`${AGENTS[agentId].name} 的规则已保存`)
}

const modelName = ref('请先获取模型')
const apiBase = ref('https://api.openai.com')
const apiKey = ref('')
const availableModels = ref([])
const modelSearch = ref('')
const ownerFilter = ref('')
const systemPrompt = ref('')
const personaText = ref('')
const userPersonaText = ref('')
const userName = ref('')
const postSystemPrompt = ref('')
const temperature = ref(0.7)
const topP = ref(1)
const frequencyPenalty = ref(0)
const presencePenalty = ref(0)
const stream = ref(false)
const memoryRounds = ref(40)
const memorySearch = ref('')
const allMemories = ref([])

const remoteEnabled = ref(false)
const remoteUrl = ref('')
const remoteToken = ref('')

const router = useRouter()
function goHome() { router.push('/') }

const filteredModels = computed(() => {
  const kw = modelSearch.value.trim().toLowerCase()
  return (availableModels.value || []).filter(m => 
    String(m.id).toLowerCase().includes(kw)
  )
})

const filteredMemories = computed(() => {
  const kw = memorySearch.value.trim().toLowerCase()
  if (!kw) return allMemories.value
  return allMemories.value.filter(m => 
    String(m.content).toLowerCase().includes(kw) || 
    (m.tags || []).some(t => String(t).toLowerCase().includes(kw))
  )
})

async function loadMemories() {
  try {
    allMemories.value = await db.memories.orderBy('timestamp').reverse().toArray()
  } catch (e) {
    ElMessage.error('加载记忆失败')
  }
}

async function deleteMemory(id) {
  try {
    await ElMessageBox.confirm('确定要删除这条记忆吗？', '提示', { 
      type: 'warning',
      confirmButtonClass: 'el-button--danger'
    })
    await db.memories.delete(id)
    await loadMemories()
    ElMessage.success('已删除')
  } catch (_) {}
}

watch(tab, (newTab) => {
  if (newTab === 'memory') loadMemories()
})

// localStorage 辅助
function lsGet(k, def) {
  const v = localStorage.getItem(k)
  if (v === null) return def
  try { return JSON.parse(v) } catch(_) { return v }
}
function lsSet(k, v) {
  localStorage.setItem(k, typeof v === 'object' ? JSON.stringify(v) : v)
}

function applySettings() {
  lsSet('ppc.modelName', modelName.value)
  lsSet('ppc.apiBase', apiBase.value)
  lsSet('ppc.apiKey', apiKey.value)
  ElMessage.success('API配置已保存')
}

function applyRemoteSettings() {
  lsSet('ppc.remoteEnabled', remoteEnabled.value)
  lsSet('ppc.remoteUrl', remoteUrl.value.trim())
  lsSet('ppc.remoteToken', remoteToken.value.trim())
  ElMessage.success('远程连接设置已保存')
}

function applyModelSettings() {
  lsSet('ppc.modelSettings', { 
    temperature: Number(temperature.value), 
    topP: Number(topP.value), 
    frequencyPenalty: Number(frequencyPenalty.value), 
    presencePenalty: Number(presencePenalty.value), 
    stream: !!stream.value,
    memoryRounds: Number(memoryRounds.value)
  })
  ElMessage.success('已保存')
}

function applyUserSettings() { 
  lsSet('ppc.userPersonaText', String(userPersonaText.value || ''))
  lsSet('ppc.userName', String(userName.value || ''))
  ElMessage.success('已保存') 
}

async function fetchModels() {
  if (!apiKey.value) return ElMessage.warning('请先输入API秘钥')
  const loading = ElMessage({ message: '正在获取模型列表...', duration: 0 })
  try {
    const res = await axios.get(`${apiBase.value}/v1/models`, {
      headers: { 'Authorization': `Bearer ${apiKey.value}` }
    })
    availableModels.value = res.data.data
    ElMessage.success(`成功获取 ${availableModels.value.length} 个模型`)
  } catch (e) {
    ElMessage.error('获取失败，请检查API地址或密钥')
  } finally {
    loading.close()
  }
}

async function handleResetAll() {
  try {
    const { value, action } = await ElMessageBox.prompt(
      '<div class="danger-main-text">主人，真的要让Pero酱忘掉你吗？o(╥﹏╥)o</div>' +
      '<div class="danger-sub-text">（此操作将清空所有数据，如需继续，请在文本框中输入“我们还会再见的...”）</div>',
      '危险操作确认',
      {
        inputValue: '',
        inputPlaceholder: '请输入：我们还会再见的...',
        confirmButtonText: '继续',
        cancelButtonText: '取消',
        type: 'error',
        customClass: 'danger-reset-box',
        center: true,
        dangerouslyUseHTMLString: true,
      }
    )
    if (action === 'confirm') {
      if (String(value || '').trim() !== '我们还会再见的...') {
        ElMessage.error('输入不匹配，已取消')
        return
      }
      const r = await resetAll()
      if (r) {
        ElMessage.success('已重置全部数据')
        setTimeout(() => location.reload(), 1000)
      } else {
        ElMessage.error('重置失败')
      }
    }
  } catch (_) {}
}

onMounted(async () => {
  modelName.value = lsGet('ppc.modelName', '')
  apiBase.value = lsGet('ppc.apiBase', 'https://api.openai.com')
  apiKey.value = lsGet('ppc.apiKey', '')
  
  const mSet = lsGet('ppc.modelSettings', {})
  temperature.value = mSet.temperature ?? 0.7
  topP.value = mSet.topP ?? 1
  frequencyPenalty.value = mSet.frequencyPenalty ?? 0
  presencePenalty.value = mSet.presencePenalty ?? 0
  stream.value = !!mSet.stream
  memoryRounds.value = mSet.memoryRounds ?? 40

  remoteEnabled.value = lsGet('ppc.remoteEnabled', false)
  remoteUrl.value = lsGet('ppc.remoteUrl', '')
  remoteToken.value = lsGet('ppc.remoteToken', '')

  userPersonaText.value = lsGet('ppc.userPersonaText', '')
  userName.value = lsGet('ppc.userName', '')

  const r = await getDefaultPrompts()
  systemPrompt.value = lsGet('ppc.systemPrompt', r.system_prompt_default)
  personaText.value = lsGet('ppc.personaText', r.persona_prompt_default)
  postSystemPrompt.value = lsGet('ppc.postSystemPrompt', r.post_prompt_default)

  // Initialize editing rule agent ID
  editingRuleAgentId.value = activeAgentId.value
  loadRuleSupplement()
})
</script>

<style scoped>
.settings-container {
  min-height: 100vh;
  background: linear-gradient(135deg, #fdf2f8 0%, #f0f9ff 100%);
  display: flex;
  flex-direction: column;
  padding-bottom: env(safe-area-inset-bottom);
}

/* 头部样式 */
.header {
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: rgba(255, 255, 255, 0.7);
  backdrop-filter: blur(10px);
  position: sticky;
  top: 0;
  z-index: 100;
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

.back-btn {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  background: white;
  box-shadow: 0 2px 8px rgba(0,0,0,0.05);
  cursor: pointer;
}

.header-title {
  text-align: center;
}

.header-title h2 {
  font-size: 16px;
  margin: 0;
  color: #1e293b;
}

.header-title span {
  font-size: 10px;
  color: #94a3b8;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.placeholder { width: 40px; }

/* 设置主体 */
.settings-body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* 菜单网格 */
.menu-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}

.menu-item {
  background: rgba(255, 255, 255, 0.6);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.8);
  border-radius: 16px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  transition: all 0.3s;
  cursor: pointer;
}

.menu-item.active {
  background: #3b82f6;
  border-color: #3b82f6;
  box-shadow: 0 8px 16px rgba(59, 130, 246, 0.2);
}

.menu-icon {
  font-size: 20px;
  color: #64748b;
}

.active .menu-icon, .active .menu-label {
  color: white;
}

.menu-label {
  font-size: 12px;
  color: #475569;
  font-weight: 500;
}

/* 内容区域 */
.content-area {
  flex: 1;
}

.content-card {
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(20px);
  border-radius: 24px;
  border: 1px solid rgba(255, 255, 255, 0.5);
  padding: 20px;
  min-height: 400px;
  box-shadow: 0 10px 25px rgba(0,0,0,0.03);
}

.section-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 20px;
}

.section-header .el-icon {
  font-size: 20px;
  color: #3b82f6;
}

.section-header h3 {
  font-size: 18px;
  margin: 0;
  color: #1e293b;
}

.section-header.danger .el-icon { color: #ef4444; }

/* 小组件样式 */
.api-tools {
  display: flex;
  gap: 10px;
  margin-bottom: 12px;
}

.mini-table {
  border-radius: 12px;
  margin-bottom: 20px;
  font-size: 13px;
}

.full-btn {
  width: 100%;
  height: 44px;
  border-radius: 12px;
  font-weight: 600;
}

.info-banner {
  background: #eff6ff;
  border-radius: 12px;
  padding: 12px;
  margin-bottom: 20px;
  font-size: 13px;
  color: #1e40af;
  line-height: 1.5;
}

.slider-group {
  margin-bottom: 24px;
}

.slider-item {
  margin-bottom: 16px;
}

.slider-item span {
  font-size: 13px;
  color: #64748b;
}

/* New Role Group Styles */
.role-group {
  width: 100%;
}
</style>