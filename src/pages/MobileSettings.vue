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
      <div class="menu-btn" @click="drawerVisible = true">
        <span class="menu-text">导航</span>
        <el-icon><Menu /></el-icon>
      </div>
    </div>

    <!-- 侧边抽屉导航 -->
    <el-drawer
      v-model="drawerVisible"
      title="设置导航"
      direction="ltr"
      size="70%"
      :show-close="false"
      class="settings-drawer"
    >
      <div class="drawer-menu">
        <div 
          v-for="item in menuItems" 
          :key="item.id" 
          :class="['drawer-item', { active: tab === item.id }]"
          @click="switchTab(item.id)"
        >
          <el-icon><component :is="item.icon" /></el-icon>
          <span>{{ item.label }}</span>
        </div>
      </div>
    </el-drawer>

    <!-- 设置主体 -->
    <div class="settings-body">
      <!-- 设置内容区 -->
      <div class="content-area">
        <Transition name="fade" mode="out-in">
          <div :key="tab" class="content-card">
            
            <!-- 角色选择与管理 -->
            <div v-if="tab === 'role'" class="section">
              <div class="section-header">
                <el-icon><Avatar /></el-icon>
                <h3>角色管理</h3>
                <el-button type="primary" class="create-btn" :icon="Plus" @click="createNewAgent" circle></el-button>
              </div>
              
              <div class="agent-selection-container">
                <div 
                  v-for="(agent, id) in AGENTS" 
                  :key="id" 
                  class="agent-select-card"
                  :class="{ active: activeAgentId === id }"
                  @click="activeAgentId = id"
                >
                  <div class="agent-avatar-placeholder">
                    {{ agent.name.charAt(0).toUpperCase() }}
                  </div>
                  <div class="agent-info">
                    <div class="agent-name">{{ agent.name }}</div>
                    <div class="agent-desc">{{ id === 'pero' ? '看板娘' : (id === 'nana' ? '雌小鬼AI' : '自定义角色') }}</div>
                  </div>
                  <div class="agent-actions">
                    <div class="status-indicator" v-if="activeAgentId === id">
                      <span class="dot"></span> 使用中
                    </div>
                    <el-dropdown trigger="click" @command="(cmd) => handleAgentCommand(cmd, id)" v-else-if="!['pero', 'nana'].includes(id)">
                      <el-icon class="more-btn" @click.stop><MoreFilled /></el-icon>
                      <template #dropdown>
                        <el-dropdown-menu>
                          <el-dropdown-item command="edit" :icon="Edit">编辑配置</el-dropdown-item>
                          <el-dropdown-item 
                            command="delete" 
                            :icon="Delete" 
                            class="danger-text"
                            :disabled="['pero', 'nana'].includes(id)"
                          >
                            删除角色
                          </el-dropdown-item>
                        </el-dropdown-menu>
                      </template>
                    </el-dropdown>
                    <el-button 
                      v-if="activeAgentId === id && !['pero', 'nana'].includes(id)" 
                      type="primary" 
                      link 
                      :icon="Edit" 
                      @click.stop="editAgent(id)"
                    >配置</el-button>
                  </div>
                </div>
              </div>

              <!-- 高级角色管理入口 -->
              <div class="advanced-role-entry">
                <el-button type="primary" plain class="advanced-btn" @click="goToRoleManager">
                  <el-icon><Setting /></el-icon>
                  <span>高级角色管理</span>
                  <el-icon class="arrow"><ArrowRight /></el-icon>
                </el-button>
                <p class="entry-hint">支持导入导出、详细人设编辑、角色模板等高级功能</p>
              </div>

              <!-- 编辑弹窗 -->
              <el-dialog
                v-model="showAgentEditor"
                :title="isNewAgent ? '新建角色' : '编辑角色'"
                width="95%"
                destroy-on-close
                append-to-body
                class="agent-editor-dialog"
              >
                <el-tabs v-model="editorActiveTab" class="editor-tabs">
                  <!-- Tab 1: 基础信息 -->
                  <el-tab-pane name="basic">
                    <template #label>
                      <span class="custom-tab-label">
                        <el-icon><Avatar /></el-icon>
                        <span>基础信息</span>
                      </span>
                    </template>
                    <el-form :model="editingAgentForm" label-position="top" class="editor-form">
                      <el-alert 
                        v-if="['pero', 'nana'].includes(editingAgentIdRaw)"
                        title="默认角色核心设定不可更改"
                        type="warning" 
                        show-icon 
                        :closable="false" 
                        class="mb-2" 
                      />
                      <div class="form-row">
                        <el-form-item label="角色名称" class="half-width">
                          <el-input 
                            v-model="editingAgentForm.name" 
                            placeholder="例如：Mio" 
                            :prefix-icon="User" 
                            :disabled="['pero', 'nana'].includes(editingAgentIdRaw)"
                          />
                        </el-form-item>
                        <el-form-item label="角色ID" v-if="isNewAgent" class="half-width">
                          <el-input v-model="editingAgentIdRaw" placeholder="例如：mio_001" :prefix-icon="Key" />
                        </el-form-item>
                      </div>
                      
                      <el-form-item label="欢迎语">
                        <el-input 
                          v-model="editingAgentForm.welcome_message" 
                          type="textarea" 
                          :rows="3" 
                          placeholder="你好呀！我是..." 
                          resize="none"
                        />
                      </el-form-item>
                      
                      <el-divider content-position="left"><el-icon><Operation /></el-icon> 默认状态</el-divider>
                      
                      <div class="state-grid">
                        <el-form-item label="Mood (心情)">
                          <el-input v-model="editingAgentForm.default_state.mood" placeholder="开心" />
                        </el-form-item>
                        <el-form-item label="Vibe (氛围)">
                          <el-input v-model="editingAgentForm.default_state.vibe" placeholder="充满活力" />
                        </el-form-item>
                        <el-form-item label="Mind (想法)">
                          <el-input v-model="editingAgentForm.default_state.mind" placeholder="希望能帮到主人" />
                        </el-form-item>
                      </div>
                    </el-form>
                  </el-tab-pane>

                  <!-- Tab 2: 核心设定 -->
                  <el-tab-pane name="core">
                    <template #label>
                      <span class="custom-tab-label">
                        <el-icon><Cpu /></el-icon>
                        <span>核心设定</span>
                      </span>
                    </template>
                    <el-form :model="editingAgentForm" label-position="top" class="editor-form">
                      <el-alert 
                        title="Prompt 设定决定了角色的性格和行为方式" 
                        type="info" 
                        show-icon 
                        :closable="false" 
                        class="mb-2" 
                      />
                      <el-form-item label="System Prompt (系统指令)">
                        <el-input 
                          v-model="editingAgentForm.system_prompt" 
                          type="textarea" 
                          :rows="8" 
                          placeholder="<System_Core_Setting>..." 
                          :disabled="['pero', 'nana'].includes(editingAgentIdRaw)"
                        />
                      </el-form-item>
                      <el-form-item label="Persona Prompt (人设模块)">
                        <el-input 
                          v-model="editingAgentForm.persona_prompt" 
                          type="textarea" 
                          :rows="8" 
                          placeholder="<Identity_Module>..." 
                          :disabled="['pero', 'nana'].includes(editingAgentIdRaw)"
                        />
                      </el-form-item>
                    </el-form>
                  </el-tab-pane>

                  <!-- Tab 3: 互动台词 -->
                  <el-tab-pane name="interaction">
                    <template #label>
                      <span class="custom-tab-label">
                        <el-icon><ChatDotRound /></el-icon>
                        <span>互动台词</span>
                      </span>
                    </template>
                    <el-form :model="editingAgentForm" label-position="top" class="editor-form">
                      <el-alert 
                        title="每行一句，看板娘闲置时会随机播放" 
                        type="success" 
                        show-icon 
                        :closable="false" 
                        class="mb-2" 
                      />
                      <el-form-item>
                        <el-input 
                          v-model="editingAgentForm.idle_messages_text" 
                          type="textarea" 
                          :rows="15" 
                          placeholder="今天天气真好呀！&#10;主人在忙什么呢？&#10;好想吃甜点..." 
                        />
                      </el-form-item>
                    </el-form>
                  </el-tab-pane>
                </el-tabs>

                <template #footer>
                  <span class="dialog-footer">
                    <el-button @click="showAgentEditor = false">取消</el-button>
                    <el-button type="primary" @click="saveAgentData">保存配置</el-button>
                  </span>
                </template>
              </el-dialog>
            </div>



            <!-- API 设置 -->
            <div v-if="tab === 'api'" class="section">
              <div class="section-header">
                <el-icon><Cpu /></el-icon>
                <h3>API 配置</h3>
              </div>
              <el-form label-position="top">
                <el-form-item label="服务商 (Provider)">
                  <el-select
                    v-model="selectedProvider"
                    placeholder="选择服务商"
                    style="width: 100%"
                    @change="handleProviderChange"
                  >
                    <el-option
                      v-for="p in providers"
                      :key="p.value"
                      :label="p.label"
                      :value="p.value"
                    />
                  </el-select>
                </el-form-item>
                <el-form-item label="模型名称">
                  <el-input v-model="modelName" placeholder="请先获取模型" />
                </el-form-item>
                <el-form-item label="API 地址">
                  <el-input v-model="apiBase" placeholder="https://api.openai.com" :disabled="selectedProvider !== 'custom'" />
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
                <el-form-item :label="`我的人设`">
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

            <!-- 数据管理 -->
            <div v-if="tab === 'data'" class="section">
              <div class="section-header">
                <el-icon><Warning /></el-icon>
                <h3>数据管理</h3>
              </div>
              
              <div class="data-card">
                <h4><i class="fas fa-file-export"></i> 导出全部数据</h4>
                <p>将所有配置、角色、记忆及对话数据导出为一个 JSON 文件，方便备份或在其他设备上使用。</p>
                <el-button type="primary" @click="handleExportAllData">
                  <i class="fas fa-download"></i> 导出数据
                </el-button>
              </div>
              
              <div class="data-card">
                <h4><i class="fas fa-file-import"></i> 导入全部数据</h4>
                <p>从之前导出的 JSON 文件中恢复所有数据。警告：此操作将覆盖当前的所有数据。</p>
                <input 
                  ref="importDataInput"
                  type="file" 
                  accept=".json"
                  style="display: none"
                  @change="handleImportAllData"
                />
                <el-button @click="triggerImportData">
                  <i class="fas fa-upload"></i> 选择文件并导入
                </el-button>
              </div>
              
              <div class="data-card danger">
                <h4><i class="fas fa-trash-alt"></i> 数据重置</h4>
                <p>将删除所有对话、记忆及偏好设置，恢复到初始状态。</p>
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
import { invoke } from '@tauri-apps/api/core'
import { useRouter } from 'vue-router'
import { ElMessageBox, ElMessage } from 'element-plus'
import { resetAll, AGENTS, getActiveAgentId, setActiveAgentId, saveApiKey, getApiKey, getAllMemories, deleteMemory as apiDeleteMemory, getConfig, saveConfig, saveAgent, deleteAgent, exportAllData, importAllData } from '../api'
import { marked } from 'marked'
import DOMPurify from 'dompurify'
import { 
  Search, Connection, ArrowLeft, ArrowRight,
  Cpu, Operation, User, Collection, Warning, Avatar, Setting,
  Edit, Menu, Plus, Delete, ChatDotRound, Key, MoreFilled
} from '@element-plus/icons-vue'

const tab = ref('role')
const editorActiveTab = ref('basic')
const drawerVisible = ref(false)

function handleAgentCommand(cmd, id) {
  if (cmd === 'edit') editAgent(id)
  else if (cmd === 'delete') handleDeleteAgent(id)
}

function switchTab(id) {
  tab.value = id
  drawerVisible.value = false
}

// Markdown 渲染
function renderMarkdown(text) {
  try {
    if (!text) return ''
    const clean = cleanMessageContent(text)
    
    // 更加防御性的 marked 调用
    let html = ''
    if (typeof marked === 'function') {
      html = marked(clean)
    } else if (marked && typeof marked.parse === 'function') {
      html = marked.parse(clean)
    } else {
      html = String(clean)
    }
    
    return DOMPurify && typeof DOMPurify.sanitize === 'function' 
      ? DOMPurify.sanitize(html) 
      : String(html)
  } catch (e) {
    console.error('Markdown render error:', e)
    return String(text || '')
  }
}

// 清理消息中的隐藏标签
function cleanMessageContent(text) {
  try {
    if (typeof text !== 'string') return String(text || '')
    if (text === '__loading__') {
      const agentId = getActiveAgentId()
      const agentName = AGENTS[agentId]?.name || 'Pero'
      return `${agentName}正在思考...`
    }
    
    // 移除所有 XML 标签及其内容
    return text
      .replace(/<([A-Z_]+)>[\s\S]*?<\/\1>/g, '')
      .replace(/\[\[\[NIT_CALL\]\]\][\s\S]*?\[\[\[NIT_END\]\]\]/g, '')
      .trim()
  } catch {
    return String(text || '')
  }
}

const menuItems = [
  { id: 'role', label: '角色', icon: Avatar },
  { id: 'api', label: 'API', icon: Cpu },
  { id: 'remote', label: '远程', icon: Connection },
  { id: 'model', label: '模型', icon: Operation },
  { id: 'user', label: '用户', icon: User },
  { id: 'memory', label: '记忆', icon: Collection },
  { id: 'data', label: '数据', icon: Warning },
]

// Role & Rules Settings
const activeAgentId = ref(getActiveAgentId())

// Agent Management Logic
const showAgentEditor = ref(false)
const isNewAgent = ref(false)
const editingAgentIdRaw = ref('') // ID for new agent or tracking edited agent
const editingAgentForm = ref({
  name: '',
  system_prompt: '',
  persona_prompt: '',
  welcome_message: '',
  default_state: { mood: '', vibe: '', mind: '' }
})

function createNewAgent() {
  isNewAgent.value = true
  editingAgentIdRaw.value = ''
  editingAgentForm.value = {
    name: '',
    system_prompt: '<System_Core_Setting>\n</System_Core_Setting>',
    persona_prompt: '<Identity_Module>\n</Identity_Module>',
    welcome_message: '你好呀！',
    default_state: { mood: '开心', vibe: '充满活力', mind: '希望能帮到主人' },
    idle_messages_text: ''
  }
  editorActiveTab.value = 'basic'
  showAgentEditor.value = true
}

function editAgent(id) {
  const agent = AGENTS[id]
  if (!agent) return
  isNewAgent.value = false
  editingAgentIdRaw.value = id
  // Deep copy to avoid direct mutation before save
  const formData = JSON.parse(JSON.stringify(agent))
  formData.idle_messages_text = Array.isArray(formData.idle_messages) ? formData.idle_messages.join('\n') : ''
  editingAgentForm.value = formData
  editorActiveTab.value = 'basic'
  showAgentEditor.value = true
}

async function handleDeleteAgent(id) {
  try {
    await ElMessageBox.confirm(`确定要删除角色 ${AGENTS[id].name} 吗？`, '警告', {
      type: 'warning',
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      confirmButtonClass: 'el-button--danger'
    })
    
    if (id === activeAgentId.value) {
      ElMessage.warning('不能删除当前正在使用的角色')
      return
    }
    
    const success = await deleteAgent(id)
    if (success) {
      ElMessage.success('角色已删除')
    } else {
      ElMessage.error('删除失败')
    }
  } catch { /* cancel */ }
}

async function saveAgentData() {
  if (!editingAgentForm.value.name) return ElMessage.warning('请输入角色名称')
  
  let id = editingAgentIdRaw.value.trim()
  if (!id) return ElMessage.warning('请输入角色ID')
  
  if (isNewAgent.value) {
    if (AGENTS[id]) return ElMessage.warning('该角色ID已存在')
  }
  
  // Process idle messages
  const idleMsgs = editingAgentForm.value.idle_messages_text 
    ? editingAgentForm.value.idle_messages_text.split('\n').map(s => s.trim()).filter(Boolean)
    : []
  
  const finalData = JSON.parse(JSON.stringify(editingAgentForm.value))
  finalData.idle_messages = idleMsgs
  delete finalData.idle_messages_text // Clean up temporary field
  
  const success = await saveAgent(id, finalData)
  if (success) {
    // Sync to waifu-tips.js via localStorage
    try {
      const waifuData = {
        welcome: editingAgentForm.value.welcome_message,
      }
      // Map idle messages to waifu-tips format
      idleMsgs.forEach((msg, idx) => {
        if (idx < 50) { // Limit to 50 messages
           waifuData[`idleMessages_${String(idx + 1).padStart(2, '0')}`] = msg
        }
      })
      
      localStorage.setItem(`ppc.${id}.waifu.texts`, JSON.stringify(waifuData))
      
      // If we are editing the active agent, trigger an update immediately
      if (id === activeAgentId.value) {
        window.dispatchEvent(new CustomEvent('ppc:agent-switched', { 
          detail: { agentId: id } 
        }))
      }
    } catch (e) {
      console.warn('Failed to sync waifu texts:', e)
    }

    ElMessage.success('保存成功')
    showAgentEditor.value = false
  } else {
    ElMessage.error('保存失败')
  }
}



const providerDefaults = {
  siliconflow: 'https://api.siliconflow.cn/v1',
  deepseek: 'https://api.deepseek.com',
  moonshot: 'https://api.moonshot.cn/v1',
  dashscope: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
  volcengine: 'https://ark.cn-beijing.volces.com/api/v3',
  groq: 'https://api.groq.com/openai/v1',
  zhipu: 'https://open.bigmodel.cn/api/paas/v4',
  minimax: 'https://api.minimax.chat/v1',
  mistral: 'https://api.mistral.ai/v1',
  yi: 'https://api.lingyiwanwu.com/v1',
  xai: 'https://api.x.ai/v1',
  stepfun: 'https://api.stepfun.com/v1',
  hunyuan: 'https://api.hunyuan.cloud.tencent.com/v1'
}

const providers = [
  { label: '自定义 / OpenAI', value: 'custom' },
  { label: 'SiliconFlow (硅基流动)', value: 'siliconflow' },
  { label: 'DeepSeek (深度求索)', value: 'deepseek' },
  { label: 'Moonshot (Kimi)', value: 'moonshot' },
  { label: 'DashScope (阿里百炼)', value: 'dashscope' },
  { label: 'Volcengine (火山引擎)', value: 'volcengine' },
  { label: 'Groq', value: 'groq' },
  { label: 'Zhipu (智谱GLM)', value: 'zhipu' },
  { label: 'MiniMax', value: 'minimax' },
  { label: 'Mistral', value: 'mistral' },
  { label: '01.AI (零一万物)', value: 'yi' },
  { label: 'xAI (Grok)', value: 'xai' },
  { label: 'StepFun (阶跃星辰)', value: 'stepfun' },
  { label: 'Hunyuan (腾讯混元)', value: 'hunyuan' }
]

const selectedProvider = ref('custom')

function handleProviderChange(val) {
  if (val && providerDefaults[val]) {
    apiBase.value = providerDefaults[val]
  } else if (val === 'custom') {
    // Keep current or clear? Let's keep current to be safe, or maybe empty if it was a default.
    // If switching to custom, the user probably wants to edit what's there or type a new one.
  }
}



// Watch active agent change (Global Switch)
watch(activeAgentId, async (newVal) => {
  await setActiveAgentId(newVal)
  
  // 触发全局事件通知 Live2D 角色已切换，以便同步更新台词
  window.dispatchEvent(new CustomEvent('ppc:agent-switched', { 
    detail: { agentId: newVal } 
  }))
  
  ElMessage.success(`已切换为 ${AGENTS[newVal].name}`)
})



const modelName = ref('请先获取模型')
const apiBase = ref('https://api.openai.com')
const apiKey = ref('')
const availableModels = ref([])
const modelSearch = ref('')
const userPersonaText = ref('')
const userName = ref('')
const temperature = ref(0.7)
const topP = ref(1)
const stream = ref(false)
const memoryRounds = ref(40)
const memorySearch = ref('')
const allMemories = ref([])

const remoteEnabled = ref(false)
const remoteUrl = ref('')
const remoteToken = ref('')
const importDataInput = ref(null)

const router = useRouter()
function goHome() { router.push('/') }
function goToRoleManager() { router.push('/roles') }

const filteredModels = computed(() => {
  const kw = modelSearch.value.trim().toLowerCase()
  return (availableModels.value || []).filter(m => 
    String(m.id).toLowerCase().includes(kw)
  )
})

const filteredMemories = computed(() => {
  try {
    const memories = Array.isArray(allMemories.value) ? allMemories.value : []
    const kw = (memorySearch.value || '').trim().toLowerCase()
    if (!kw) return memories
    return memories.filter(m => 
      String(m.content || '').toLowerCase().includes(kw) || 
      (Array.isArray(m.tags) ? m.tags : []).some(t => String(t || '').toLowerCase().includes(kw))
    )
  } catch (e) {
    console.error('Filter memories error:', e)
    return []
  }
})

async function loadMemories() {
  try {
    allMemories.value = await getAllMemories()
  } catch {
    ElMessage.error('加载记忆失败')
  }
}

async function deleteMemory(id) {
  try {
    await ElMessageBox.confirm('确定要删除这条记忆吗？', '提示', { 
      type: 'warning',
      confirmButtonClass: 'el-button--danger'
    })
    await apiDeleteMemory(id)
    await loadMemories()
    ElMessage.success('已删除')
  } catch { /* ignore */ }
}

watch(tab, (newTab) => {
  if (newTab === 'memory') loadMemories()
})

// Rust Store 辅助
async function getStoreConfig(k, def) {
  const v = await getConfig(k)
  if (!v) return def
  try { return JSON.parse(v) } catch { return v }
}
async function saveStoreConfig(k, v) {
  const val = typeof v === 'object' ? JSON.stringify(v) : String(v)
  await saveConfig(k, val)
}

async function applySettings() {
  await saveStoreConfig('ppc.modelName', modelName.value)
  await saveStoreConfig('ppc.apiBase', apiBase.value)
  // API Key 单独处理
  await saveApiKey(apiKey.value)
  ElMessage.success('API配置已保存')
}

async function applyRemoteSettings() {
  await saveStoreConfig('ppc.remoteEnabled', remoteEnabled.value)
  await saveStoreConfig('ppc.remoteUrl', remoteUrl.value.trim())
  await saveStoreConfig('ppc.remoteToken', remoteToken.value.trim())
  ElMessage.success('远程连接设置已保存')
}

async function applyModelSettings() {
  await saveStoreConfig('ppc.modelSettings', { 
    temperature: Number(temperature.value), 
    topP: Number(topP.value), 
    stream: !!stream.value,
    memoryRounds: Number(memoryRounds.value)
  })
  ElMessage.success('已保存')
}

async function applyUserSettings() { 
  await saveStoreConfig('ppc.userPersonaText', String(userPersonaText.value || ''))
  await saveStoreConfig('ppc.userName', String(userName.value || ''))
  ElMessage.success('已保存') 
}

async function fetchModels() {
  if (!apiKey.value) return ElMessage.warning('请先输入API秘钥')
  const loading = ElMessage({ message: '正在获取模型列表...', duration: 0 })
  try {
    // 使用 Rust 后端获取模型列表，避免 CORS 问题
    const data = await invoke('fetch_models', { 
      apiBase: apiBase.value, 
      apiKey: apiKey.value 
    })
    
    // 兼容 OpenAI 格式返回 { object: "list", data: [...] }
    if (data && Array.isArray(data.data)) {
      availableModels.value = data.data
      ElMessage.success(`成功获取 ${availableModels.value.length} 个模型`)
    } else if (Array.isArray(data)) {
       availableModels.value = data
       ElMessage.success(`成功获取 ${availableModels.value.length} 个模型`)
    } else {
      console.warn('Unknown model response format:', data)
      ElMessage.warning('获取到的模型格式无法解析')
    }
  } catch (e) {
    console.error('Fetch models failed:', e)
    ElMessage.error(`获取失败: ${e}`)
  } finally {
    loading.close()
  }
}

async function handleResetAll() {
  try {
    const agentName = AGENTS[getActiveAgentId()]?.name || 'Pero'
    const { value, action } = await ElMessageBox.prompt(
      `<div class="danger-main-text">主人，真的要让${agentName}忘掉你吗？o(╥﹏╥)o</div>` +
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
  } catch { /* ignore */ }
}

// 导出全部数据
async function handleExportAllData() {
  try {
    const loading = ElMessage({ message: '正在导出数据...', duration: 0 })
    const data = await exportAllData()
    
    // 创建下载链接
    const jsonString = JSON.stringify(data, null, 2)
    const blob = new Blob([jsonString], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19)
    a.download = `peropero-chat-backup-${timestamp}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
    
    loading.close()
    ElMessage.success('数据导出成功！')
  } catch (e) {
    console.error('导出数据失败:', e)
    ElMessage.error('导出数据失败，请查看控制台了解详情')
  }
}

// 触发文件选择
function triggerImportData() {
  importDataInput.value?.click()
}

// 处理导入数据
async function handleImportAllData(event) {
  const file = event.target.files?.[0]
  if (!file) return
  
  try {
    const { value } = await ElMessageBox.confirm(
      '导入数据将覆盖当前所有数据，确定要继续吗？',
      '确认导入',
      {
        confirmButtonText: '确定导入',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )
    
    if (!value) {
      event.target.value = ''
      return
    }
    
    const loading = ElMessage({ message: '正在导入数据...', duration: 0 })
    
    // 读取文件内容
    const text = await file.text()
    const data = JSON.parse(text)
    
    // 导入数据
    await importAllData(data)
    
    loading.close()
    ElMessage.success('数据导入成功！正在重新加载...')
    
    // 清空文件输入
    event.target.value = ''
    
    // 刷新页面
    setTimeout(() => location.reload(), 1500)
  } catch (e) {
    console.error('导入数据失败:', e)
    if (e.message?.includes('JSON')) {
      ElMessage.error('导入失败：无效的JSON文件格式')
    } else {
      ElMessage.error('导入数据失败，请查看控制台了解详情')
    }
    event.target.value = ''
  }
}

// Init settings
onMounted(async () => {
  modelName.value = await getStoreConfig('ppc.modelName', '请先获取模型')
  apiBase.value = await getStoreConfig('ppc.apiBase', 'https://api.openai.com')
  apiKey.value = await getApiKey() || ''
  
  const mSettings = await getStoreConfig('ppc.modelSettings', {})
  if (mSettings.temperature !== undefined) temperature.value = mSettings.temperature
  if (mSettings.topP !== undefined) topP.value = mSettings.topP
  if (mSettings.stream !== undefined) stream.value = mSettings.stream
  if (mSettings.memoryRounds !== undefined) memoryRounds.value = mSettings.memoryRounds

  userPersonaText.value = await getStoreConfig('ppc.userPersonaText', '')
  userName.value = await getStoreConfig('ppc.userName', '')

  const rEnabled = await getStoreConfig('ppc.remoteEnabled', 'false')
  remoteEnabled.value = (rEnabled === 'true' || rEnabled === true)
  remoteUrl.value = await getStoreConfig('ppc.remoteUrl', '')
  remoteToken.value = await getStoreConfig('ppc.remoteToken', '')

  // Set provider selection based on loaded apiBase
  const foundProvider = Object.entries(providerDefaults).find(([, v]) => v === apiBase.value)
  if (foundProvider) {
    selectedProvider.value = foundProvider[0]
  } else {
    selectedProvider.value = 'custom'
  }
  

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

.menu-btn {
  height: 36px;
  padding: 0 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 18px;
  background: #3b82f6;
  color: white;
  box-shadow: 0 4px 10px rgba(59, 130, 246, 0.3);
  cursor: pointer;
  transition: all 0.2s;
  font-size: 14px;
  gap: 6px;
  font-weight: 600;
}

.menu-btn:active {
  transform: scale(0.95);
  box-shadow: 0 2px 5px rgba(59, 130, 246, 0.2);
}

.menu-text {
  font-size: 13px;
}

/* 设置主体 */
.settings-body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* 抽屉菜单样式 */
.drawer-menu {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 10px;
}

.drawer-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  border-radius: 16px;
  color: #64748b;
  background: #f8fafc;
  transition: all 0.2s;
  cursor: pointer;
  border: 1px solid transparent;
}

.drawer-item:active {
  transform: scale(0.98);
}

.drawer-item.active {
  background: #eff6ff;
  color: #3b82f6;
  border-color: #bfdbfe;
  font-weight: 600;
}

.drawer-item .el-icon {
  font-size: 22px;
}

.drawer-item span {
  font-size: 16px;
}

:deep(.settings-drawer) {
  border-radius: 0 24px 24px 0 !important;
  background: rgba(255, 255, 255, 0.98) !important;
}

:deep(.el-drawer__header) {
  margin-bottom: 0;
  padding: 24px;
  border-bottom: 1px solid rgba(0,0,0,0.05);
  color: #1e293b;
  font-weight: 700;
}

:deep(.el-drawer__body) {
  padding: 16px !important;
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
  font-weight: 700;
}

.section-header.small h3 {
  font-size: 15px;
  margin: 0;
  color: #64748b;
}

.section h4 {
  margin: 0 0 8px 0;
  color: #334155;
  font-size: 16px;
  font-weight: 600;
}

.section p {
  margin: 0 0 16px 0;
  color: #64748b;
  font-size: 14px;
  line-height: 1.6;
}

.section-header.danger .el-icon { color: #ef4444; }

.danger-card {
  background: rgba(254, 242, 242, 0.8);
  border: 1px dashed rgba(239, 68, 68, 0.3);
  border-radius: 16px;
  padding: 16px;
  margin-top: 8px;
}

/* 角色列表样式 */
.agent-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 12px;
}

.agent-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.6);
  border-radius: 12px;
  border: 1px solid rgba(0, 0, 0, 0.05);
  transition: all 0.2s;
}

.agent-card:active {
  transform: scale(0.99);
  background: rgba(255, 255, 255, 0.8);
}

.agent-card-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.agent-card-name {
  font-weight: 600;
  color: #334155;
  font-size: 15px;
}

.agent-card-sub {
  font-size: 12px;
  color: #94a3b8;
  font-family: monospace;
}

.agent-card-actions {
  display: flex;
  gap: 8px;
}

.state-row {
  display: flex;
  gap: 12px;
}
.state-row .el-form-item {
  flex: 1;
}

/* 记忆列表 */
.memory-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.empty {
  text-align: center;
  color: #94a3b8;
  padding: 40px 0;
  font-size: 14px;
}
.memory-item {
  background: rgba(255, 255, 255, 0.6);
  border-radius: 12px;
  padding: 12px;
  border: 1px solid rgba(0, 0, 0, 0.03);
}
.memory-content {
  font-size: 14px;
  color: #334155;
  margin-bottom: 8px;
  max-height: 100px;
  overflow-y: auto;
}
.memory-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 8px;
}
.memory-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: #cbd5e1;
}
.search-input {
  margin-bottom: 16px;
}

/* 数据管理卡片样式 */
.data-card {
  background: #fff;
  border-radius: 16px;
  padding: 20px;
  margin-bottom: 16px;
  border: 1px solid rgba(0, 0, 0, 0.05);
  transition: all 0.3s ease;
}

.data-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  transform: translateY(-1px);
}

.data-card.danger {
  background: linear-gradient(135deg, #fef2f2 0%, #fee2e2 100%);
  border: 1px dashed rgba(239, 68, 68, 0.4);
}

.data-card.danger:hover {
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.15);
}

.data-card h4 {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  font-size: 16px;
  font-weight: 600;
  color: #1e293b;
}

.data-card h4 i {
  font-size: 18px;
}

.data-card.danger h4 {
  color: #dc2626;
}

.data-card p {
  margin-bottom: 16px;
  color: #64748b;
  font-size: 14px;
  line-height: 1.6;
}

.data-card .el-button {
  border-radius: 12px;
  padding: 10px 20px;
  font-weight: 500;
  transition: all 0.2s;
}

.data-card .el-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 10px rgba(59, 130, 246, 0.2);
}

.data-card.danger .el-button {
  background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
  border: none;
}

.data-card.danger .el-button:hover {
  box-shadow: 0 4px 10px rgba(239, 68, 68, 0.3);
}

/* 过渡动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* 滑块样式优化 */
.slider-group {
  background: rgba(255, 255, 255, 0.5);
  padding: 16px;
  border-radius: 12px;
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.slider-item span {
  font-size: 12px;
  color: #64748b;
  margin-bottom: 4px;
  display: block;
}

/* API Tools */
.api-tools {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}
.mini-table {
  border-radius: 8px;
  overflow: hidden;
  margin-bottom: 16px;
  font-size: 12px;
}

.full-btn {
  width: 100%;
  margin-top: 8px;
  height: 44px;
  border-radius: 12px;
  font-weight: 600;
  font-size: 16px;
}

.role-group {
  width: 100%;
  display: flex;
  flex-wrap: wrap;
}
.role-group :deep(.el-radio-button) {
  flex: 1;
}
.role-group :deep(.el-radio-button__inner) {
  width: 100%;
  border-radius: 0;
  padding: 12px 0;
}
.role-group :deep(.el-radio-button:first-child .el-radio-button__inner) {
  border-radius: 8px 0 0 8px;
}
.role-group :deep(.el-radio-button:last-child .el-radio-button__inner) {
  border-radius: 0 8px 8px 0;
}
</style>

<!-- Global Styles for Agent Editor Dialog (Required due to append-to-body) -->
<style>
.agent-selection-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.agent-select-card {
  display: flex;
  align-items: center;
  padding: 16px;
  background: #fff;
  border-radius: 16px;
  border: 1px solid rgba(0,0,0,0.03);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: pointer;
  position: relative;
  overflow: hidden;
}

.agent-select-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.03);
}

.agent-select-card.active {
  background: linear-gradient(135deg, #eff6ff 0%, #f0f9ff 100%);
  border-color: #bfdbfe;
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.1);
}

.agent-avatar-placeholder {
  width: 48px;
  height: 48px;
  border-radius: 14px;
  background: linear-gradient(135deg, #e2e8f0 0%, #cbd5e1 100%);
  color: #fff;
  font-size: 20px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 16px;
  box-shadow: 0 2px 6px rgba(0,0,0,0.05);
}

.agent-select-card.active .agent-avatar-placeholder {
  background: linear-gradient(135deg, #3b82f6 0%, #60a5fa 100%);
  box-shadow: 0 4px 10px rgba(59, 130, 246, 0.2);
}

.agent-info {
  flex: 1;
}

.agent-name {
  font-size: 16px;
  font-weight: 700;
  color: #1e293b;
  margin-bottom: 2px;
}

.agent-select-card.active .agent-name {
  color: #2563eb;
}

.agent-desc {
  font-size: 12px;
  color: #94a3b8;
}

.agent-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 600;
  color: #22c55e;
  background: #dcfce7;
  padding: 4px 10px;
  border-radius: 20px;
}

.status-indicator .dot {
  width: 6px;
  height: 6px;
  background: #22c55e;
  border-radius: 50%;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.4); }
  70% { transform: scale(1); box-shadow: 0 0 0 6px rgba(34, 197, 94, 0); }
  100% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(34, 197, 94, 0); }
}

.more-btn {
  padding: 8px;
  border-radius: 50%;
  color: #94a3b8;
  cursor: pointer;
  transition: all 0.2s;
}

.more-btn:hover {
  background: #f1f5f9;
  color: #64748b;
}

.create-btn {
  width: 36px;
  height: 36px;
  min-height: unset;
  padding: 0;
  margin: 0 !important;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 10px rgba(59, 130, 246, 0.3);
}

.section-header {
  justify-content: space-between;
}

.danger-text {
  color: #ef4444;
}

/* Agent Editor Dialog Styles */
.agent-editor-dialog {
  border-radius: 20px !important;
  overflow: hidden;
}

.agent-editor-dialog .el-dialog__header {
  margin-right: 0;
  padding: 20px 24px;
  border-bottom: 1px solid #f1f5f9;
}

.agent-editor-dialog .el-dialog__body {
  padding: 0 !important;
}

.agent-editor-dialog .el-dialog__footer {
  padding: 16px 24px;
  border-top: 1px solid #f1f5f9;
  background: #fafafa;
}

.editor-tabs {
  min-height: 400px;
}

.editor-tabs .el-tabs__header {
  margin: 0;
  padding: 0 24px;
  background: #fff;
  border-bottom: 1px solid #f1f5f9;
}

.editor-tabs .el-tabs__content {
  padding: 24px;
  max-height: 60vh;
  overflow-y: auto;
}

.custom-tab-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
}

.editor-form .el-form-item {
  margin-bottom: 20px;
}

.form-row {
  display: flex;
  gap: 16px;
}

.half-width {
  flex: 1;
}

.state-group {
  background: #f8fafc;
  padding: 16px;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
}

.group-title {
  font-size: 13px;
  font-weight: 600;
  color: #64748b;
  margin-bottom: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.state-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.mb-2 {
  margin-bottom: 16px;
}

/* --- Beautification Overrides --- */

/* Tabs Beautification */
.editor-tabs .el-tabs__item {
  height: 50px;
  font-size: 15px;
  color: #64748b;
  transition: all 0.3s;
}

.editor-tabs .el-tabs__item.is-active {
  color: #3b82f6;
  font-weight: 600;
}

.editor-tabs .el-tabs__active-bar {
  background-color: #3b82f6;
  height: 3px;
  border-radius: 3px;
}

/* Input Fields Beautification */
.agent-editor-dialog .el-input__wrapper,
.agent-editor-dialog .el-textarea__inner {
  box-shadow: none !important;
  background-color: #f1f5f9;
  border: 1px solid transparent;
  border-radius: 12px;
  padding: 8px 12px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.agent-editor-dialog .el-textarea__inner {
  padding: 12px;
  resize: none;
}

.agent-editor-dialog .el-input__wrapper:hover,
.agent-editor-dialog .el-textarea__inner:hover {
  background-color: #e2e8f0;
}

.agent-editor-dialog .el-input__wrapper.is-focus,
.agent-editor-dialog .el-textarea__inner:focus {
  background-color: #fff;
  border-color: #3b82f6;
  box-shadow: 0 0 0 4px rgba(59, 130, 246, 0.1) !important;
}

/* Form Labels */
.agent-editor-dialog .el-form-item__label {
  font-weight: 600;
  color: #334155;
  margin-bottom: 8px !important;
}

/* Dialog Header */
.agent-editor-dialog .el-dialog__title {
  font-weight: 700;
  color: #1e293b;
  font-size: 18px;
}

.agent-editor-dialog .el-dialog__headerbtn .el-dialog__close {
  font-size: 20px;
  color: #94a3b8;
  transition: color 0.2s;
}

.agent-editor-dialog .el-dialog__headerbtn .el-dialog__close:hover {
  color: #ef4444;
}

/* Footer Buttons */
.agent-editor-dialog .el-button {
  border-radius: 10px;
  padding: 10px 24px;
  font-weight: 600;
  height: auto;
  border: none;
}

.agent-editor-dialog .el-button--default {
  background: #f1f5f9;
  color: #64748b;
}

.agent-editor-dialog .el-button--default:hover {
  background: #e2e8f0;
  color: #475569;
}

.agent-editor-dialog .el-button--primary {
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.3);
  transition: all 0.2s;
}

.agent-editor-dialog .el-button--primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 16px rgba(37, 99, 235, 0.4);
}

.agent-editor-dialog .el-button--primary:active {
  transform: translateY(1px);
  box-shadow: 0 2px 6px rgba(37, 99, 235, 0.2);
}

/* Alert Styles */
.agent-editor-dialog .el-alert {
  border-radius: 12px;
  border: none;
}

.agent-editor-dialog .el-alert--info {
  background-color: #eff6ff;
  color: #1e40af;
}

.agent-editor-dialog .el-alert--success {
  background-color: #f0fdf4;
  color: #166534;
}

/* Divider */
.agent-editor-dialog .el-divider__text {
  background-color: #fff;
  color: #94a3b8;
  font-weight: 500;
  font-size: 12px;
}

/* 高级角色管理入口 */
.advanced-role-entry {
  margin-top: 20px;
  padding: 16px;
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
  border-radius: 16px;
  border: 1px dashed #cbd5e1;
}

.advanced-btn {
  width: 100%;
  height: 48px;
  border-radius: 12px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  background: #fff;
  border-color: #3b82f6;
  color: #3b82f6;
  transition: all 0.2s;
}

.advanced-btn:hover {
  background: #eff6ff;
  border-color: #2563eb;
  color: #2563eb;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.15);
}

.advanced-btn .arrow {
  margin-left: auto;
  font-size: 14px;
}

.entry-hint {
  margin: 12px 0 0;
  font-size: 12px;
  color: #94a3b8;
  text-align: center;
}
</style>