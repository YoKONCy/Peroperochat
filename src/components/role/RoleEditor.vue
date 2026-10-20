<template>
  <div class="role-editor">
    
    <div class="editor-content" :inert="isSaving ? true : undefined">
      <el-tabs v-model="activeTab">
        <el-tab-pane label="基础信息" name="basic">
          <div class="form-section">
            <div class="avatar-upload">
              <button type="button" class="avatar-preview" aria-label="上传角色头像" @click="triggerAvatarUpload">
                <CharacterAvatar v-if="formData.avatar" class="editor-avatar" :src="formData.avatar" :label="formData.name" />
                <div v-else class="avatar-placeholder">
                  <RoomIcon name="camera" />
                  <span>上传头像</span>
                </div>
              </button>
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
              
              <el-form-item label="静态立绘或 CG">
                <el-input v-model="formData.standing_image" placeholder="应用内素材路径或 HTTP(S) 图片地址，可留空" />
                <button type="button" class="room-text-button" :disabled="artLoading" @click="pickArt('standing_image')">选择 PNG／JPEG／WebP 图片</button>
                <span class="form-hint">显示模式可随时切换为通用 Pio。角色包导入、导出时保留此素材地址。</span>
              </el-form-item>
              <el-form-item label="图片排版"><el-select v-model="formData.illustration_layout"><el-option label="透明立绘" value="standing" /><el-option label="完整 CG 卡片" value="card" /></el-select></el-form-item>
              <el-form-item label="房间背景">
                <div class="room-background-editor">
                  <el-input v-if="!formData.room_background.startsWith('data:')" v-model="formData.room_background" placeholder="图片地址，留空使用默认房间" />
                  <span v-else class="form-hint">已选择本地背景图片，将随角色一起保存和导出。</span>
                  <div class="room-background-actions"><button type="button" class="room-text-button" :disabled="artLoading" @click="pickArt('room_background')">上传背景图片</button><button v-if="formData.room_background" type="button" class="room-text-button" :disabled="artLoading" @click="formData.room_background = ''">使用默认房间</button></div>
                  <span class="form-hint">支持不超过 5MB 的 PNG、JPEG、WebP。上传的背景固定显示；默认房间会在切屏时按设备本地时间更新，其他自定义角色默认使用 Pero 房间。</span>
                  <img v-if="formData.room_background && !backgroundPreviewFailed" class="room-background-preview" :src="formData.room_background" alt="所选房间背景预览" decoding="async" @error="backgroundPreviewFailed = true" />
                  <span v-else-if="backgroundPreviewFailed && formData.room_background" class="form-hint">背景暂时无法预览，请核对图片地址或重新上传。</span>
                </div>
              </el-form-item>
              <input ref="artInput" type="file" accept="image/png,image/jpeg,image/webp" hidden @change="readArt" />
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
                      <RoomIcon name="help" />
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
                      <RoomIcon name="help" />
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
                      <RoomIcon name="help" />
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
<el-form-item label="闲置互动台词"><el-input v-model="formData.idle_messages_text" type="textarea" :rows="5" placeholder="每行一条台词" /><span class="form-hint">角色闲置时使用，一行一条。</span></el-form-item>
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
        <el-button :disabled="isSaving" @click="$emit('close')">取消</el-button>
        <el-button type="primary" @click="handleSave" :loading="isSaving" :disabled="artLoading">
          保存角色
        </el-button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { ElMessage } from 'element-plus'
import RoomIcon from '../companion/RoomIcon.vue'
import CharacterAvatar from '../companion/CharacterAvatar.vue'

const props = defineProps({
  saving: { type: Boolean, default: false },
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
const isSaving = computed(() => props.saving)
const avatarInput = ref(null)
const newTag = ref('')
const artInput = ref(null), artLoading = ref(false)
const backgroundPreviewFailed = ref(false)
let artField = 'standing_image', artReader = null, artVersion = 0
function pickArt(field) { artField = field; artInput.value?.click() }
function readArt(event) {
  const file = event.target.files?.[0], field = artField
  event.target.value = ''
  if (!file) return
  if (!['image/png', 'image/jpeg', 'image/webp'].includes(file.type) || file.size > 5 * 1024 * 1024) { ElMessage.error('请选择不超过 5MB 的 PNG、JPEG 或 WebP 图片'); return }
  artReader?.abort(); const version = ++artVersion
  const reader = new FileReader(); artReader = reader; artLoading.value = true
  reader.onload = () => { if (version === artVersion) { formData[field] = reader.result; artLoading.value = false } }
  reader.onerror = () => { if (version === artVersion) { artLoading.value = false; ElMessage.error('图片读取失败，请重新选择') } }
  reader.readAsDataURL(file)
}
onBeforeUnmount(() => { artVersion++; artReader?.abort() })

const formData = reactive({
  id: '',
  name: '',
  avatar: '',
  standing_image: '',
  room_background: '',
  illustration_layout: 'standing',
  description: '',
  tags: [],
  idle_messages_text: '',
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

watch(() => formData.room_background, () => { backgroundPreviewFailed.value = false })

function resetForm() {
  artVersion++; artReader?.abort(); artLoading.value = false
  if (props.role && !props.isNew) {
    formData.id = props.role.id || ''
    formData.name = props.role.name || ''
    formData.avatar = props.role.avatar || ''
    formData.standing_image = props.role.standing_image || ''
    formData.room_background = props.role.room_background || ''
    formData.illustration_layout = props.role.illustration_layout || 'standing'
    formData.description = props.role.description || ''
    formData.tags = [...(props.role.tags || [])]
    formData.idle_messages_text = Array.isArray(props.role.idle_messages) ? props.role.idle_messages.join('\n') : ''
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
    formData.id = `custom_${Date.now()}`
    formData.name = '新角色'
    formData.avatar = ''
    formData.standing_image = ''
    formData.room_background = ''
    formData.illustration_layout = 'standing'
    formData.description = ''
    formData.tags = []
    formData.idle_messages_text = ''
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
  resetForm()
})

watch(() => [props.role, props.isNew], () => {
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

function handleSave() {
  if (isSaving.value || artLoading.value) return
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
  
  // 保存结果由父页面等待后端事务后展示，事件发出不等于保存成功。
  emit('save', { ...(props.role || {}), ...formData })
}
</script>

<style scoped>
.role-editor { display: flex; flex-direction: column; height: 100%; min-height: 0; width: 100%; background: var(--room-surface); }
.editor-content { flex: 1; min-height: 0; overflow-y: auto; padding: 0 24px; overscroll-behavior: contain; }
.form-section { padding: 18px 0 8px; }
.avatar-upload { display: flex; margin-bottom: 26px; }
.avatar-preview { padding: 0; width: 86px; height: 100px; border: 1px solid var(--room-line); border-radius: 3px 10px 3px 3px; overflow: hidden; background: var(--room-subtle); }
.avatar-preview .editor-avatar { width: 100%; height: 100%; }
.avatar-placeholder { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; height: 100%; color: var(--room-muted); font-size: 11px; }
.form-hint { font-size: 11px; color: var(--room-muted); line-height: 1.65; margin-top: 6px; }
.room-background-editor { display: grid; gap: 8px; width: 100%; }.room-background-actions { display: flex; align-items: center; gap: 15px; flex-wrap: wrap; }.room-background-preview { display: block; width: 100%; height: 150px; object-fit: cover; object-position: center 45%; border: 1px solid var(--room-line); border-radius: 10px; }
.tags-input { display: flex; flex-wrap: wrap; gap: 7px; width: 100%; padding: 8px; border: 1px solid var(--room-line); border-radius: 5px; }
.tags-input input { min-width: 100px; flex: 1; border: 0; padding: 4px; outline: 0; background: transparent; color: var(--room-ink); font-size: 13px; }
.tags-input:focus-within { border-color: var(--room-accent); }
.label-with-help { display: flex; align-items: center; gap: 8px; }
.label-with-help svg { width: 14px; height: 14px; color: var(--room-muted); }
.template-section { border-top: 1px solid var(--room-line); padding: 14px 0 20px; }
.template-section h4 { font-size: 12px; font-weight: 500; color: var(--room-muted); }
.template-grid { display: flex; flex-wrap: wrap; gap: 8px; }
.template-btn { border: 1px solid var(--room-line); border-radius: 8px; background: var(--room-subtle); padding: 9px 12px; color: var(--room-muted); font-size: 12px; }
.template-btn:hover { border-color: var(--room-accent); color: var(--room-accent); }
.editor-footer { display: flex; justify-content: flex-end; padding: 16px 24px calc(16px + env(safe-area-inset-bottom)); border-top: 1px solid var(--room-line); background: var(--room-surface); flex: none; }
.footer-right { display: flex; gap: 8px; }
.footer-right :deep(.el-button + .el-button) { margin-left: 0; }
@media (max-width: 480px) { .editor-content { padding: 0 18px; } :deep(.el-col-8), :deep(.el-col-12) { flex: 0 0 100%; max-width: 100%; } :deep(.el-input__inner), :deep(.el-textarea__inner), .tags-input input { font-size: 13px; } }
@supports (-webkit-touch-callout: none) { :deep(.el-input__inner), :deep(.el-textarea__inner), .tags-input input { font-size: 16px; } }
</style>
