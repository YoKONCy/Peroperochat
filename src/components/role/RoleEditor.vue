<template>
  <div class="role-editor">
    <div class="editor-header">
      <h2>{{ isNew ? '创建新角色' : `编辑角色: ${formData.name}` }}</h2>
      <button class="close-btn" @click="$emit('close')">
        <i class="fas fa-times"></i>
      </button>
    </div>
    
    <div class="editor-content">
      <el-tabs v-model="activeTab">
        <el-tab-pane label="基础信息" name="basic">
          <div class="form-section">
            <div class="avatar-upload">
              <div class="avatar-preview" @click="triggerAvatarUpload">
                <img v-if="formData.avatar" :src="formData.avatar" />
                <div v-else class="avatar-placeholder">
                  <i class="fas fa-camera"></i>
                  <span>上传头像</span>
                </div>
              </div>
              <input 
                ref="avatarInput"
                type="file"
                accept="image/*"
                style="display: none"
                @change="handleAvatarUpload"
              />
            </div>
            
            <el-form :model="formData" label-position="top">
              <el-row :gutter="16">
                <el-col :span="12">
                  <el-form-item label="角色 ID" required>
                    <el-input 
                      v-model="formData.id" 
                      placeholder="唯一标识，如：pero"
                      :disabled="!isNew"
                    />
                    <span class="form-hint">创建后不可修改</span>
                  </el-form-item>
                </el-col>
                <el-col :span="12">
                  <el-form-item label="角色名称" required>
                    <el-input v-model="formData.name" placeholder="显示名称" />
                  </el-form-item>
                </el-col>
              </el-row>
              
              <el-form-item label="角色描述">
                <el-input 
                  v-model="formData.description"
                  type="textarea"
                  :rows="2"
                  placeholder="简要描述这个角色的特点..."
                />
              </el-form-item>
              
              <el-form-item label="标签">
                <div class="tags-input">
                  <el-tag
                    v-for="tag in formData.tags"
                    :key="tag"
                    closable
                    @close="removeTag(tag)"
                  >
                    {{ tag }}
                  </el-tag>
                  <input 
                    v-model="newTag"
                    type="text"
                    placeholder="输入标签后回车"
                    @keyup.enter="addTag"
                  />
                </div>
              </el-form-item>
            </el-form>
          </div>
        </el-tab-pane>
        
        <el-tab-pane label="人设配置" name="prompts">
          <div class="form-section">
            <el-form :model="formData" label-position="top">
              <el-form-item>
                <template #label>
                  <div class="label-with-help">
                    <span>系统提示词</span>
                    <el-tooltip content="定义 AI 的基本行为规则和安全约束">
                      <i class="fas fa-question-circle"></i>
                    </el-tooltip>
                  </div>
                </template>
                <el-input 
                  v-model="formData.system_prompt"
                  type="textarea"
                  :rows="8"
                  placeholder="系统级提示词，用于定义 AI 的基本行为..."
                />
              </el-form-item>
              
              <el-form-item>
                <template #label>
                  <div class="label-with-help">
                    <span>人设提示词</span>
                    <el-tooltip content="定义角色的性格、外貌、背景等特征">
                      <i class="fas fa-question-circle"></i>
                    </el-tooltip>
                  </div>
                </template>
                <el-input 
                  v-model="formData.persona_prompt"
                  type="textarea"
                  :rows="12"
                  placeholder="详细描述角色的性格、外貌、说话风格..."
                />
              </el-form-item>
              
              <el-form-item>
                <template #label>
                  <div class="label-with-help">
                    <span>后置提示词</span>
                    <el-tooltip content="附加在用户消息后的提示，用于引导回复格式">
                      <i class="fas fa-question-circle"></i>
                    </el-tooltip>
                  </div>
                </template>
                <el-input 
                  v-model="formData.post_system_prompt"
                  type="textarea"
                  :rows="4"
                  placeholder="可选：附加在每条消息后的提示..."
                />
              </el-form-item>
            </el-form>
            
            <div class="template-section">
              <h4>快速模板</h4>
              <div class="template-grid">
                <button 
                  v-for="template in templates" 
                  :key="template.name"
                  class="template-btn"
                  @click="applyTemplate(template)"
                >
                  {{ template.name }}
                </button>
              </div>
            </div>
          </div>
        </el-tab-pane>
        
        <el-tab-pane label="对话配置" name="chat">
          <div class="form-section">
            <el-form :model="formData" label-position="top">
              <el-form-item label="欢迎语">
                <el-input 
                  v-model="formData.welcome_message"
                  type="textarea"
                  :rows="3"
                  placeholder="角色首次对话时的问候语..."
                />
              </el-form-item>
              
              <el-divider>默认状态</el-divider>
              
              <el-row :gutter="16">
                <el-col :span="8">
                  <el-form-item label="心情">
                    <el-input v-model="formData.default_state.mood" placeholder="如：开心" />
                  </el-form-item>
                </el-col>
                <el-col :span="8">
                  <el-form-item label="氛围">
                    <el-input v-model="formData.default_state.vibe" placeholder="如：温柔" />
                  </el-form-item>
                </el-col>
                <el-col :span="8">
                  <el-form-item label="内心独白">
                    <el-input v-model="formData.default_state.mind" placeholder="角色的内心想法" />
                  </el-form-item>
                </el-col>
              </el-row>
            </el-form>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
    
    <div class="editor-footer">
      <div class="footer-left"></div>
      <div class="footer-right">
        <el-button @click="$emit('close')">取消</el-button>
        <el-button type="primary" @click="handleSave" :loading="isSaving">
          <i class="fas fa-save"></i> 保存
        </el-button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, watch } from 'vue'
import { ElMessage } from 'element-plus'

const props = defineProps({
  role: {
    type: Object,
    default: null
  },
  isNew: {
    type: Boolean,
    default: false
  },
  templates: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['close', 'save'])

const activeTab = ref('basic')
const isSaving = ref(false)
const avatarInput = ref(null)
const newTag = ref('')

const formData = reactive({
  id: '',
  name: '',
  avatar: '',
  description: '',
  tags: [],
  system_prompt: '',
  persona_prompt: '',
  post_system_prompt: '',
  welcome_message: '',
  default_state: {
    mood: '',
    vibe: '',
    mind: ''
  }
})

function resetForm() {
  console.log('resetForm called, props.role:', props.role, 'isNew:', props.isNew)
  if (props.role && !props.isNew) {
    console.log('Loading existing role')
    formData.id = props.role.id || ''
    formData.name = props.role.name || ''
    formData.avatar = props.role.avatar || ''
    formData.description = props.role.description || ''
    formData.tags = props.role.tags || []
    formData.system_prompt = props.role.system_prompt || ''
    formData.persona_prompt = props.role.persona_prompt || ''
    formData.post_system_prompt = props.role.post_system_prompt || ''
    formData.welcome_message = props.role.welcome_message || ''
    formData.default_state = {
      mood: props.role.default_state?.mood || '',
      vibe: props.role.default_state?.vibe || '',
      mind: props.role.default_state?.mind || ''
    }
  } else {
    console.log('Creating new role')
    formData.id = `custom_${Date.now()}`
    formData.name = '新角色'
    formData.avatar = ''
    formData.description = ''
    formData.tags = []
    formData.system_prompt = ''
    formData.persona_prompt = ''
    formData.post_system_prompt = ''
    formData.welcome_message = '你好！'
    formData.default_state = {
      mood: '平静',
      vibe: '正常',
      mind: ''
    }
  }
}

onMounted(() => {
  console.log('RoleEditor mounted')
  resetForm()
})

watch(() => [props.role, props.isNew], () => {
  console.log('props changed, resetting form')
  resetForm()
}, { deep: true })

function triggerAvatarUpload() {
  avatarInput.value?.click()
}

function handleAvatarUpload(e) {
  const file = e.target.files?.[0]
  if (!file) return
  
  if (file.size > 2 * 1024 * 1024) {
    ElMessage.warning('图片大小不能超过 2MB')
    return
  }
  
  const reader = new FileReader()
  reader.onload = (ev) => {
    formData.avatar = ev.target.result
  }
  reader.readAsDataURL(file)
  
  e.target.value = ''
}

function addTag() {
  const tag = newTag.value.trim()
  if (tag && !formData.tags.includes(tag)) {
    formData.tags.push(tag)
  }
  newTag.value = ''
}

function removeTag(tag) {
  const index = formData.tags.indexOf(tag)
  if (index > -1) {
    formData.tags.splice(index, 1)
  }
}

function applyTemplate(template) {
  if (template.persona_prompt) {
    formData.persona_prompt = template.persona_prompt
  }
  if (template.default_state) {
    Object.assign(formData.default_state, template.default_state)
  }
  if (template.tags) {
    template.tags.forEach(tag => {
      if (!formData.tags.includes(tag)) {
        formData.tags.push(tag)
      }
    })
  }
  ElMessage.success(`已应用「${template.name}」模板`)
}

async function handleSave() {
  if (!formData.id.trim()) {
    ElMessage.error('请输入角色 ID')
    activeTab.value = 'basic'
    return
  }
  if (!formData.name.trim()) {
    ElMessage.error('请输入角色名称')
    activeTab.value = 'basic'
    return
  }
  
  isSaving.value = true
  try {
    emit('save', { ...formData })
    ElMessage.success('角色保存成功')
  } catch (err) {
    ElMessage.error(`保存失败: ${err.message}`)
  } finally {
    isSaving.value = false
  }
}
</script>

<style scoped>
.role-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: white;
  border-radius: 24px;
  overflow: hidden;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.1);
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 24px 28px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.04);
  background: linear-gradient(135deg, rgba(253, 242, 248, 0.5), rgba(240, 249, 255, 0.5));
}

.editor-header h2 {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  color: #1e293b;
}

.close-btn {
  width: 40px;
  height: 40px;
  border: none;
  border-radius: 14px;
  background: white;
  color: #64748b;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.close-btn:hover {
  background: linear-gradient(135deg, #f472b6, #ef4444);
  color: white;
  transform: scale(0.95);
}

.editor-content {
  flex: 1;
  overflow-y: auto;
  padding: 0 28px;
}

.form-section {
  padding: 24px 0;
}

.avatar-upload {
  display: flex;
  justify-content: center;
  margin-bottom: 28px;
}

.avatar-preview {
  width: 120px;
  height: 120px;
  border-radius: 32px;
  overflow: hidden;
  cursor: pointer;
  border: 4px solid rgba(59, 130, 246, 0.2);
  transition: all 0.3s;
  box-shadow: 0 6px 20px rgba(244, 114, 182, 0.15);
}

.avatar-preview:hover {
  border-color: rgba(59, 130, 246, 0.6);
  transform: scale(1.05);
  box-shadow: 0 10px 32px rgba(244, 114, 182, 0.3);
}

.avatar-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-placeholder {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, #f472b6, #3b82f6);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: white;
  gap: 6px;
}

.avatar-placeholder i {
  font-size: 32px;
}

.avatar-placeholder span {
  font-size: 12px;
  font-weight: 600;
}

.form-hint {
  font-size: 12px;
  color: #94a3b8;
  margin-top: 6px;
}

.label-with-help {
  display: flex;
  align-items: center;
  gap: 8px;
}

.label-with-help i {
  color: #94a3b8;
  cursor: help;
}

.tags-input {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 10px 14px;
  border: 1px solid rgba(0, 0, 0, 0.06);
  border-radius: 16px;
  min-height: 48px;
  background: rgba(248, 250, 252, 0.8);
}

.tags-input input {
  flex: 1;
  min-width: 100px;
  border: none;
  outline: none;
  font-size: 14px;
  background: transparent;
}

.template-section {
  margin-top: 28px;
  padding-top: 28px;
  border-top: 1px solid rgba(0, 0, 0, 0.04);
}

.template-section h4 {
  margin: 0 0 16px 0;
  font-size: 15px;
  font-weight: 600;
  color: #64748b;
}

.template-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.template-btn {
  padding: 10px 18px;
  border: 1px solid rgba(0, 0, 0, 0.06);
  border-radius: 14px;
  background: white;
  color: #64748b;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.03);
  font-weight: 500;
}

.template-btn:hover {
  border-color: rgba(59, 130, 246, 0.4);
  color: #3b82f6;
  background: rgba(59, 130, 246, 0.06);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.1);
}

.editor-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 28px;
  border-top: 1px solid rgba(0, 0, 0, 0.04);
  background: rgba(248, 250, 252, 0.8);
}

.footer-right {
  display: flex;
  gap: 12px;
}

:deep(.el-tabs__header) {
  margin-bottom: 0;
}

:deep(.el-tabs__item) {
  font-size: 14px;
  font-weight: 600;
}

:deep(.el-form-item__label) {
  font-weight: 600;
  color: #334155;
}

:deep(.el-textarea__inner) {
  font-family: inherit;
  line-height: 1.7;
  border-radius: 14px;
}

:deep(.el-input__wrapper) {
  border-radius: 14px;
}

@media (max-width: 640px) {
  .editor-header {
    padding: 20px;
  }
  
  .editor-content {
    padding: 0 20px;
  }
  
  .editor-footer {
    flex-direction: column;
    gap: 12px;
    padding: 16px;
  }
  
  .footer-left,
  .footer-right {
    width: 100%;
    justify-content: center;
  }
  
  .avatar-preview {
    width: 100px;
    height: 100px;
    border-radius: 24px;
  }
}
</style>
