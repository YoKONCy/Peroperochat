<template>
  <div class="role-manager">
    <div class="page-header">
      <button class="back-btn" @click="goBack">
        <i class="fas fa-arrow-left"></i>
      </button>
      <h1>角色管理</h1>
      <div class="header-actions">
        <el-tooltip content="导出全部" placement="bottom">
          <button class="header-btn" @click="exportAllRoles">
            <i class="fas fa-file-export"></i>
          </button>
        </el-tooltip>
      </div>
    </div>
    
    <div class="main-content">
      <div v-if="isLoading" class="loading-state">
        <el-icon class="is-loading" :size="32"><Loading /></el-icon>
        <span>加载中...</span>
      </div>
      
      <RoleList
        v-else
        :roles="roles"
        :active-role-id="activeRoleId"
        :role-list="roleList"
        @select="handleSelectRole"
        @create="openCreateEditor"
        @edit="openEditEditor"
        @delete="handleDeleteRole"
        @export="handleExportRole"
        @duplicate="handleDuplicateRole"
        @import="handleImportRole"
      />
    </div>
    
    <Transition name="slide">
      <div v-if="showEditor" class="editor-overlay" @click.self="closeEditor">
        <div class="editor-panel">
          <RoleEditor
            :key="isNewRole ? 'new' : editingRole?.id"
            :role="editingRole"
            :is-new="isNewRole"
            :templates="roleTemplates"
            @close="closeEditor"
            @save="handleSaveRole"
          />
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { Loading } from '@element-plus/icons-vue'
import { useRoleManager } from '../composables/useRoleManager'
import RoleList from '../components/role/RoleList.vue'
import RoleEditor from '../components/role/RoleEditor.vue'

const router = useRouter()

const {
  roles,
  activeRoleId,
  roleList,
  isLoading,
  loadRoles,
  saveRole,
  createRole,
  deleteRole,
  switchRole,
  exportRole,
  importRole,
  duplicateRole,
  getRoleTemplates
} = useRoleManager()

const showEditor = ref(false)
const editingRole = ref(null)
const isNewRole = ref(false)

const roleTemplates = computed(() => getRoleTemplates())

function goBack() {
  router.back()
}

async function handleSelectRole(roleId) {
  try {
    await switchRole(roleId)
    ElMessage.success(`已切换到「${roles[roleId].name}」`)
  } catch (err) {
    ElMessage.error(err.message)
  }
}

function openCreateEditor() {
  editingRole.value = null
  isNewRole.value = true
  nextTick(() => {
    showEditor.value = true
  })
}

function openEditEditor(roleId) {
  console.log('Opening editor for roleId:', roleId)
  console.log('Role data:', roles[roleId])
  editingRole.value = roles[roleId]
  isNewRole.value = false
  nextTick(() => {
    showEditor.value = true
  })
}

function closeEditor() {
  showEditor.value = false
  editingRole.value = null
}

async function handleSaveRole(roleData) {
  try {
    if (isNewRole.value) {
      await createRole(roleData)
      ElMessage.success('角色创建成功')
    } else {
      await saveRole(roleData)
      ElMessage.success('角色保存成功')
    }
    closeEditor()
  } catch (err) {
    ElMessage.error(`保存失败: ${err.message}`)
  }
}

async function handleDeleteRole(roleId) {
  try {
    await deleteRole(roleId)
    ElMessage.success('角色已删除')
  } catch (err) {
    ElMessage.error(err.message)
  }
}

function handleExportRole(roleId) {
  exportRole(roleId)
}

function exportAllRoles() {
  const allRoles = Object.values(roles)
  const exportData = {
    version: '1.0',
    exported_at: new Date().toISOString(),
    roles: allRoles
  }
  
  const blob = new Blob([JSON.stringify(exportData, null, 2)], {
    type: 'application/json'
  })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `peroperochat_roles_${Date.now()}.json`
  a.click()
  URL.revokeObjectURL(url)
  
  ElMessage.success(`已导出 ${allRoles.length} 个角色`)
}

async function handleImportRole(file) {
  try {
    const newRole = await importRole(file)
    ElMessage.success(`角色「${newRole.name}」导入成功`)
  } catch (err) {
    ElMessage.error(`导入失败: ${err.message}`)
  }
}

async function handleDuplicateRole(roleId) {
  try {
    const newRole = await duplicateRole(roleId)
    ElMessage.success(`已创建「${newRole.name}」`)
  } catch (err) {
    ElMessage.error(err.message)
  }
}

onMounted(async () => {
  await loadRoles()
})
</script>

<style scoped>
.role-manager {
  min-height: 100vh;
  background: linear-gradient(135deg, #fdf2f8 0%, #f0f9ff 100%);
  display: flex;
  flex-direction: column;
  padding-bottom: env(safe-area-inset-bottom);
}

.page-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 18px 24px;
  background: rgba(255, 255, 255, 0.6);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.8);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.02);
  z-index: 10;
}

.back-btn {
  width: 44px;
  height: 44px;
  border: 1px solid rgba(255, 255, 255, 0.8);
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.6);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.02);
  color: #1e293b;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
}

.back-btn:hover {
  background: white;
  transform: translateX(-2px);
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.05);
}

.page-header h1 {
  flex: 1;
  margin: 0;
  font-size: 20px;
  font-weight: 800;
  color: #1e293b;
  letter-spacing: -0.02em;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.header-btn {
  width: 44px;
  height: 44px;
  border: 1px solid rgba(255, 255, 255, 0.8);
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.6);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.02);
  color: #1e293b;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.header-btn:hover {
  background: white;
  color: #818cf8;
  border-color: #818cf8;
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(129, 140, 248, 0.15);
}

.main-content {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 300px;
  gap: 16px;
  color: #94a3b8;
}

.editor-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.editor-panel {
  width: 100%;
  max-width: 800px;
  max-height: 90vh;
  border-radius: 24px;
  overflow: hidden;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.15);
}

.slide-enter-active,
.slide-leave-active {
  transition: all 0.3s ease;
}

.slide-enter-active .editor-panel,
.slide-leave-active .editor-panel {
  transition: all 0.3s ease;
}

.slide-enter-from,
.slide-leave-to {
  opacity: 0;
}

.slide-enter-from .editor-panel,
.slide-leave-to .editor-panel {
  transform: translateY(20px) scale(0.95);
}

@media (max-width: 768px) {
  .page-header {
    padding: 12px 16px;
  }
  
  .page-header h1 {
    font-size: 18px;
  }
  
  .main-content {
    padding: 16px;
  }
  
  .editor-overlay {
    padding: 0;
  }
  
  .editor-panel {
    max-width: 100%;
    max-height: 100vh;
    border-radius: 0;
  }
}
</style>
