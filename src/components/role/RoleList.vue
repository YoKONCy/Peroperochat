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
  gap: 20px;
  height: 100%;
}

.toolbar {
  display: flex;
  gap: 12px;
  align-items: center;
}

.search-box {
  flex: 1;
  position: relative;
}

.search-box i {
  position: absolute;
  left: 14px;
  top: 50%;
  transform: translateY(-50%);
  color: #94a3b8;
}

.search-box input {
  width: 100%;
  padding: 12px 16px 12px 42px;
  border: 1px solid rgba(0, 0, 0, 0.05);
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.8);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.03);
  font-size: 14px;
  transition: all 0.2s;
}

.search-box input:focus {
  outline: none;
  border-color: #3b82f6;
  background: white;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.toolbar-actions {
  display: flex;
  gap: 8px;
}

.tool-btn {
  width: 44px;
  height: 44px;
  border: 1px solid rgba(0, 0, 0, 0.05);
  border-radius: 16px;
  background: white;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.03);
  color: #64748b;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.tool-btn:hover {
  background: #3b82f6;
  color: white;
  border-color: #3b82f6;
  transform: scale(0.95);
}

.roles-container {
  flex: 1;
  overflow-y: auto;
  padding-right: 8px;
}

.roles-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

.add-role-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 24px;
  background: rgba(255, 255, 255, 0.8);
  border: 2px dashed rgba(59, 130, 246, 0.3);
  border-radius: 20px;
  cursor: pointer;
  transition: all 0.3s;
  min-height: 140px;
}

.add-role-card:hover {
  background: rgba(59, 130, 246, 0.05);
  border-color: #3b82f6;
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08);
}

.add-icon {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: linear-gradient(135deg, #f472b6, #3b82f6);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 20px;
}

.add-text {
  color: #64748b;
  font-size: 14px;
  font-weight: 500;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  color: #94a3b8;
}

.empty-state i {
  font-size: 48px;
  margin-bottom: 16px;
}

.empty-state p {
  font-size: 16px;
  margin-bottom: 20px;
}

.create-btn {
  padding: 12px 24px;
  border: none;
  border-radius: 16px;
  background: linear-gradient(135deg, #f472b6, #3b82f6);
  color: white;
  font-size: 14px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: transform 0.2s, box-shadow 0.2s;
  box-shadow: 0 4px 12px rgba(244, 114, 182, 0.3);
}

.create-btn:hover {
  transform: scale(1.05);
  box-shadow: 0 6px 20px rgba(244, 114, 182, 0.4);
}

.list-enter-active,
.list-leave-active {
  transition: all 0.3s ease;
}

.list-enter-from {
  opacity: 0;
  transform: translateY(20px);
}

.list-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}

@media (max-width: 768px) {
  .roles-grid {
    grid-template-columns: 1fr;
  }
}
</style>
