<template>
  <div class="role-list">
    <div class="toolbar">
      <div class="search-box">
        <i class="fas fa-search"></i>
        <input 
          v-model="searchQuery" 
          type="text" 
          placeholder="搜索角色..."
        />
      </div>
      <div class="toolbar-actions">
        <el-tooltip content="导入角色" placement="top">
          <button class="tool-btn" @click="triggerImport">
            <i class="fas fa-file-import"></i>
          </button>
        </el-tooltip>
        <input 
          ref="importInput"
          type="file" 
          accept=".json"
          style="display: none"
          @change="handleImport"
        />
      </div>
    </div>
    
    <div class="roles-container">
      <TransitionGroup name="list" tag="div" class="roles-grid">
        <RoleCard
          v-for="role in filteredRoles"
          :key="role.id"
          :role="role"
          :is-active="role.id === activeRoleId"
          @select="handleSelect"
          @edit="$emit('edit', $event)"
          @delete="handleDelete"
          @export="handleExport"
          @duplicate="handleDuplicate"
        />
      </TransitionGroup>
      
      <div class="add-role-card" @click="$emit('create')">
        <div class="add-icon">
          <i class="fas fa-plus"></i>
        </div>
        <span class="add-text">新建角色</span>
      </div>
      
      <div v-if="filteredRoles.length === 0" class="empty-state">
        <i class="fas fa-user-slash"></i>
        <p>没有找到角色</p>
        <button class="create-btn" @click="$emit('create')">
          <i class="fas fa-plus"></i> 创建新角色
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { ElMessageBox, ElMessage } from 'element-plus'
import RoleCard from './RoleCard.vue'

const props = defineProps({
  roles: {
    type: Object,
    required: true
  },
  activeRoleId: {
    type: String,
    default: ''
  },
  roleList: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['select', 'edit', 'create', 'delete', 'export', 'duplicate', 'import'])

const searchQuery = ref('')
const importInput = ref(null)

const filteredRoles = computed(() => {
  let list = props.roleList
  
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    list = list.filter(role => 
      role.name.toLowerCase().includes(query) ||
      role.description?.toLowerCase().includes(query) ||
      role.tags?.some(tag => tag.toLowerCase().includes(query))
    )
  }
  
  return list
})

function handleSelect(roleId) {
  emit('select', roleId)
}

async function handleDelete(roleId) {
  const role = props.roles[roleId]
  if (!role) return
  
  try {
    await ElMessageBox.confirm(
      `确定要删除角色「${role.name}」吗？此操作不可恢复。`,
      '删除确认',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    emit('delete', roleId)
  } catch {
    // 用户取消删除
  }
}

function handleExport(roleId) {
  emit('export', roleId)
  ElMessage.success('角色配置已导出')
}

function handleDuplicate(roleId) {
  emit('duplicate', roleId)
  ElMessage.success('角色已复制')
}

function triggerImport() {
  importInput.value?.click()
}

async function handleImport(e) {
  const file = e.target.files?.[0]
  if (!file) return
  
  try {
    emit('import', file)
    ElMessage.success('角色导入成功')
  } catch (err) {
    ElMessage.error(`导入失败: ${err.message}`)
  }
  
  e.target.value = ''
}
</script>

<style scoped>
.role-list {
  display: flex;
  flex-direction: column;
  gap: 24px;
  height: 100%;
}

.toolbar {
  display: flex;
  gap: 16px;
  align-items: center;
}

.search-box {
  flex: 1;
  position: relative;
}

.search-box i {
  position: absolute;
  left: 18px;
  top: 50%;
  transform: translateY(-50%);
  color: #94a3b8;
  font-size: 16px;
  pointer-events: none;
}

.search-box input {
  width: 100%;
  padding: 14px 16px 14px 46px;
  border: 1px solid rgba(255, 255, 255, 0.8);
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.6);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.02);
  font-size: 15px;
  color: #1e293b;
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.search-box input::placeholder {
  color: #94a3b8;
  font-weight: 500;
}

.search-box input:focus {
  outline: none;
  background: rgba(255, 255, 255, 0.9);
  border-color: #818cf8;
  box-shadow: 0 8px 20px rgba(129, 140, 248, 0.15), 0 0 0 3px rgba(129, 140, 248, 0.1);
  transform: translateY(-1px);
}

.toolbar-actions {
  display: flex;
  gap: 10px;
}

.tool-btn {
  width: 50px;
  height: 50px;
  border: 1px solid rgba(255, 255, 255, 0.8);
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.6);
  backdrop-filter: blur(10px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.02);
  color: #64748b;
  font-size: 16px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  display: flex;
  align-items: center;
  justify-content: center;
}

.tool-btn:hover {
  background: white;
  color: #818cf8;
  border-color: #818cf8;
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(129, 140, 248, 0.15);
}

.roles-container {
  flex: 1;
  overflow-y: auto;
  padding-right: 8px;
  padding-bottom: 20px;
}

/* 自定义滚动条风格 */
.roles-container::-webkit-scrollbar {
  width: 6px;
}
.roles-container::-webkit-scrollbar-track {
  background: transparent;
}
.roles-container::-webkit-scrollbar-thumb {
  background: rgba(148, 163, 184, 0.2);
  border-radius: 10px;
}
.roles-container::-webkit-scrollbar-thumb:hover {
  background: rgba(148, 163, 184, 0.4);
}

.roles-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 20px;
  margin-bottom: 20px;
}

.add-role-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  padding: 30px;
  background: rgba(255, 255, 255, 0.4);
  backdrop-filter: blur(10px);
  border: 2px dashed rgba(148, 163, 184, 0.4);
  border-radius: 24px;
  cursor: pointer;
  transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
  min-height: 160px;
}

.add-role-card:hover {
  background: rgba(255, 255, 255, 0.7);
  border-color: #818cf8;
  border-style: solid;
  transform: translateY(-4px);
  box-shadow: 0 12px 30px rgba(129, 140, 248, 0.1);
}

.add-icon {
  width: 54px;
  height: 54px;
  border-radius: 50%;
  background: linear-gradient(135deg, #f472b6, #818cf8);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 22px;
  box-shadow: 0 8px 20px rgba(244, 114, 182, 0.3);
  transition: transform 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.add-role-card:hover .add-icon {
  transform: scale(1.1) rotate(90deg);
}

.add-text {
  color: #475569;
  font-size: 15px;
  font-weight: 700;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 24px;
  color: #94a3b8;
}

.empty-state i {
  font-size: 56px;
  margin-bottom: 20px;
  opacity: 0.5;
}

.empty-state p {
  font-size: 16px;
  font-weight: 500;
  margin-bottom: 24px;
}

.create-btn {
  padding: 14px 28px;
  border: none;
  border-radius: 18px;
  background: linear-gradient(135deg, #f472b6, #818cf8);
  color: white;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 10px;
  transition: all 0.3s;
  box-shadow: 0 8px 20px rgba(244, 114, 182, 0.3);
}

.create-btn:hover {
  transform: translateY(-2px) scale(1.02);
  box-shadow: 0 12px 28px rgba(244, 114, 182, 0.4);
}

.list-enter-active,
.list-leave-active {
  transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

.list-enter-from {
  opacity: 0;
  transform: translateY(30px) scale(0.98);
}

.list-leave-to {
  opacity: 0;
  transform: scale(0.95);
}

@media (max-width: 768px) {
  .roles-grid {
    grid-template-columns: 1fr;
  }
}
</style>
