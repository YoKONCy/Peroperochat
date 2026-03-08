<template>
  <div class="prompt-editor">
    <!-- 编辑器头部 -->
    <div class="editor-header">
      <div class="header-left">
        <h3>{{ title }}</h3>
        <span class="char-count">{{ charCount }} 字符</span>
      </div>
      <div class="header-right">
        <el-tooltip content="插入变量" placement="top">
          <button class="header-btn" @click="showVariableMenu = !showVariableMenu">
            <i class="fas fa-code"></i>
          </button>
        </el-tooltip>
        <el-tooltip content="使用模板" placement="top">
          <button class="header-btn" @click="showTemplateMenu = !showTemplateMenu">
            <i class="fas fa-file-alt"></i>
          </button>
        </el-tooltip>
        <el-tooltip content="清空内容" placement="top">
          <button class="header-btn danger" @click="handleClear">
            <i class="fas fa-trash-alt"></i>
          </button>
        </el-tooltip>
      </div>
    </div>

    <!-- 变量菜单 -->
    <Transition name="fade">
      <div v-if="showVariableMenu" class="variable-menu">
        <div class="menu-header">
          <span>可用变量</span>
          <button class="close-btn" @click="showVariableMenu = false">
            <i class="fas fa-times"></i>
          </button>
        </div>
        <div class="variable-list">
          <button
            v-for="variable in variables"
            :key="variable.key"
            class="variable-item"
            @click="insertVariable(variable.key)"
          >
            <span class="var-key">{{ variable.key }}</span>
            <span class="var-desc">{{ variable.description }}</span>
          </button>
        </div>
      </div>
    </Transition>

    <!-- 模板菜单 -->
    <Transition name="fade">
      <div v-if="showTemplateMenu" class="template-menu">
        <div class="menu-header">
          <span>提示词模板</span>
          <button class="close-btn" @click="showTemplateMenu = false">
            <i class="fas fa-times"></i>
          </button>
        </div>
        <div class="template-list">
          <button
            v-for="template in templates"
            :key="template.name"
            class="template-item"
            @click="applyTemplate(template)"
          >
            <span class="template-name">{{ template.name }}</span>
            <span class="template-desc">{{ template.description }}</span>
          </button>
        </div>
      </div>
    </Transition>

    <!-- 编辑区域 -->
    <div class="editor-body">
      <textarea
        ref="textareaRef"
        v-model="localContent"
        :placeholder="placeholder"
        class="prompt-textarea"
        @input="handleInput"
        @keydown="handleKeydown"
      ></textarea>

      <!-- 行号 -->
      <div class="line-numbers" v-if="showLineNumbers">
        <span v-for="line in lineCount" :key="line">{{ line }}</span>
      </div>
    </div>

    <!-- 底部工具栏 -->
    <div class="editor-footer">
      <div class="footer-left">
        <span class="line-info">第 {{ currentLine }} 行，第 {{ currentColumn }} 列</span>
      </div>
      <div class="footer-right">
        <el-button size="small" @click="handleCancel">取消</el-button>
        <el-button size="small" type="primary" @click="handleConfirm">确定</el-button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'

const props = defineProps({
  modelValue: {
    type: String,
    default: ''
  },
  title: {
    type: String,
    default: '提示词编辑器'
  },
  placeholder: {
    type: String,
    default: '在此输入提示词...'
  },
  showLineNumbers: {
    type: Boolean,
    default: true
  },
  variables: {
    type: Array,
    default: () => [
      { key: '{{user_name}}', description: '用户名称' },
      { key: '{{role_name}}', description: '角色名称' },
      { key: '{{current_time}}', description: '当前时间' },
      { key: '{{current_date}}', description: '当前日期' },
      { key: '{{mood}}', description: '当前心情' },
      { key: '{{vibe}}', description: '当前氛围' }
    ]
  },
  templates: {
    type: Array,
    default: () => [
      {
        name: '基础人设',
        description: '包含基本身份描述的模板',
        content: `<Identity_Module>
# 身份: {{role_name}}

请在此描述角色的基本信息...

**核心性格**:
1. 
2. 
3. 
</Identity_Module>`
      },
      {
        name: '完整人设',
        description: '包含详细设定的完整模板',
        content: `<Identity_Module>
# 身份: {{role_name}}

## 基本信息
- **名称**: 
- **年龄**: 
- **性别**: 

## 外貌描述
请描述角色的外貌特征...

## 核心性格
1. **性格特点1**: 
2. **性格特点2**: 
3. **性格特点3**: 

## 说话风格
请描述角色的说话方式和语气...

## 背景故事
请描述角色的背景故事...
</Identity_Module>`
      },
      {
        name: '系统约束',
        description: '用于定义行为约束的模板',
        content: `<System_Constraint>
# 行为约束

## 必须遵守
- 
- 
- 

## 禁止行为
- 
- 
- 

## 输出格式
请定义期望的输出格式...
</System_Constraint>`
      }
    ]
  }
})

const emit = defineEmits(['update:modelValue', 'confirm', 'cancel'])

const textareaRef = ref(null)
const localContent = ref(props.modelValue)
const showVariableMenu = ref(false)
const showTemplateMenu = ref(false)
const currentLine = ref(1)
const currentColumn = ref(1)

// 字符计数
const charCount = computed(() => localContent.value.length)

// 行数
const lineCount = computed(() => {
  return (localContent.value.match(/\n/g) || []).length + 1
})

// 监听外部值变化
watch(() => props.modelValue, (newVal) => {
  localContent.value = newVal
})

// 处理输入
function handleInput() {
  updateCursorPosition()
  emit('update:modelValue', localContent.value)
}

// 处理键盘事件
function handleKeydown(e) {
  // Tab 键插入两个空格
  if (e.key === 'Tab') {
    e.preventDefault()
    const start = e.target.selectionStart
    const end = e.target.selectionEnd
    localContent.value = 
      localContent.value.substring(0, start) + 
      '  ' + 
      localContent.value.substring(end)
    nextTick(() => {
      e.target.selectionStart = e.target.selectionEnd = start + 2
    })
  }
}

// 更新光标位置
function updateCursorPosition() {
  const textarea = textareaRef.value
  if (!textarea) return

  const text = textarea.value.substring(0, textarea.selectionStart)
  const lines = text.split('\n')
  currentLine.value = lines.length
  currentColumn.value = lines[lines.length - 1].length + 1
}

// 插入变量
function insertVariable(variable) {
  const textarea = textareaRef.value
  if (!textarea) return

  const start = textarea.selectionStart
  const end = textarea.selectionEnd
  
  localContent.value = 
    localContent.value.substring(0, start) + 
    variable + 
    localContent.value.substring(end)
  
  showVariableMenu.value = false
  
  nextTick(() => {
    textarea.focus()
    textarea.selectionStart = textarea.selectionEnd = start + variable.length
    updateCursorPosition()
  })
  
  emit('update:modelValue', localContent.value)
}

// 应用模板
function applyTemplate(template) {
  if (localContent.value.trim()) {
    ElMessageBox.confirm(
      '应用模板将覆盖当前内容，是否继续？',
      '确认覆盖',
      { type: 'warning' }
    ).then(() => {
      localContent.value = template.content
      showTemplateMenu.value = false
      emit('update:modelValue', localContent.value)
      ElMessage.success(`已应用「${template.name}」模板`)
    }).catch(() => {})
  } else {
    localContent.value = template.content
    showTemplateMenu.value = false
    emit('update:modelValue', localContent.value)
    ElMessage.success(`已应用「${template.name}」模板`)
  }
}

// 清空内容
async function handleClear() {
  if (!localContent.value.trim()) return
  
  try {
    await ElMessageBox.confirm(
      '确定要清空所有内容吗？',
      '清空确认',
      { type: 'warning' }
    )
    localContent.value = ''
    emit('update:modelValue', localContent.value)
    ElMessage.success('已清空内容')
  } catch {
    // 用户取消
  }
}

// 取消
function handleCancel() {
  emit('cancel')
}

// 确定
function handleConfirm() {
  emit('confirm', localContent.value)
}

// 挂载后更新光标位置
onMounted(() => {
  updateCursorPosition()
})
</script>

<style scoped>
.prompt-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #fff;
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid #e2e8f0;
}

/* 头部 */
.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-left h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #1e293b;
}

.char-count {
  font-size: 12px;
  color: #94a3b8;
  background: #f1f5f9;
  padding: 2px 8px;
  border-radius: 10px;
}

.header-right {
  display: flex;
  gap: 8px;
}

.header-btn {
  width: 32px;
  height: 32px;
  border: none;
  background: #fff;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #64748b;
  transition: all 0.2s;
}

.header-btn:hover {
  background: #e2e8f0;
  color: #3b82f6;
}

.header-btn.danger:hover {
  background: #fee2e2;
  color: #ef4444;
}

/* 变量菜单 */
.variable-menu,
.template-menu {
  position: absolute;
  top: 56px;
  right: 16px;
  width: 280px;
  background: #fff;
  border-radius: 12px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.15);
  z-index: 100;
  overflow: hidden;
}

.menu-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
  font-size: 13px;
  font-weight: 600;
  color: #1e293b;
}

.close-btn {
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #94a3b8;
  border-radius: 6px;
}

.close-btn:hover {
  background: #e2e8f0;
  color: #64748b;
}

.variable-list,
.template-list {
  max-height: 240px;
  overflow-y: auto;
  padding: 8px;
}

.variable-item,
.template-item {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  border: none;
  background: transparent;
  cursor: pointer;
  text-align: left;
  border-radius: 8px;
  transition: all 0.2s;
}

.variable-item:hover,
.template-item:hover {
  background: #f1f5f9;
}

.var-key,
.template-name {
  font-size: 13px;
  font-weight: 600;
  color: #3b82f6;
  font-family: 'Consolas', monospace;
}

.var-desc,
.template-desc {
  font-size: 12px;
  color: #94a3b8;
}

/* 编辑区域 */
.editor-body {
  flex: 1;
  position: relative;
  display: flex;
  overflow: hidden;
}

.line-numbers {
  width: 40px;
  background: #f8fafc;
  border-right: 1px solid #e2e8f0;
  padding: 12px 8px;
  text-align: right;
  font-family: 'Consolas', monospace;
  font-size: 12px;
  color: #94a3b8;
  line-height: 1.6;
  user-select: none;
  overflow: hidden;
}

.line-numbers span {
  display: block;
}

.prompt-textarea {
  flex: 1;
  padding: 12px 16px;
  border: none;
  outline: none;
  resize: none;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.6;
  color: #1e293b;
  background: #fff;
}

.prompt-textarea::placeholder {
  color: #cbd5e1;
}

/* 底部工具栏 */
.editor-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: #f8fafc;
  border-top: 1px solid #e2e8f0;
}

.footer-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.line-info {
  font-size: 12px;
  color: #94a3b8;
}

.footer-right {
  display: flex;
  gap: 8px;
}

/* 过渡动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s, transform 0.2s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

/* 滚动条样式 */
.variable-list::-webkit-scrollbar,
.template-list::-webkit-scrollbar {
  width: 6px;
}

.variable-list::-webkit-scrollbar-track,
.template-list::-webkit-scrollbar-track {
  background: transparent;
}

.variable-list::-webkit-scrollbar-thumb,
.template-list::-webkit-scrollbar-thumb {
  background: #e2e8f0;
  border-radius: 3px;
}

.variable-list::-webkit-scrollbar-thumb:hover,
.template-list::-webkit-scrollbar-thumb:hover {
  background: #cbd5e1;
}
</style>
