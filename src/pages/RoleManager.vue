<template>
  <CompanionPage title="角色管理" subtitle="每位伙伴都有自己的设定" @back="goBack">
    <template #actions><button class="room-secondary-button" @click="exportAllRoles"><RoomIcon name="download" />导出全部</button></template>
    
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
    
    <RoomDrawer v-if="showEditor" :title="isNewRole ? '创建角色' : `编辑 ${editingRole?.name || '角色'}`" subtitle="角色资料与对话设定" :tone="activeRoleId === 'nana' ? 'nana' : 'pero'" variant="editor" :busy="savingRole" @close="closeEditor">
          <RoleEditor
            :key="isNewRole ? 'new' : editingRole?.id"
            :role="editingRole"
            :is-new="isNewRole"
            :templates="roleTemplates"
            :saving="savingRole"
            @close="closeEditor"
            @save="handleSaveRole"
          />
    </RoomDrawer>
  </CompanionPage>
</template>

<script setup>
import { ref, computed, onMounted, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { appMessage as ElMessage } from '../services/appFeedback'
import { Loading } from '@element-plus/icons-vue'
import { useRoleManager } from '../composables/useRoleManager'
import RoleList from '../components/role/RoleList.vue'
import RoleEditor from '../components/role/RoleEditor.vue'
import CompanionPage from '../components/companion/CompanionPage.vue'
import RoomDrawer from '../components/companion/RoomDrawer.vue'
import RoomIcon from '../components/companion/RoomIcon.vue'

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
const savingRole = ref(false)
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
  editingRole.value = roles[roleId]
  isNewRole.value = false
  nextTick(() => {
    showEditor.value = true
  })
}

function closeEditor() {
  if (savingRole.value) return
  showEditor.value = false
  editingRole.value = null
}

async function handleSaveRole(roleData) {
  if (savingRole.value) return
  savingRole.value = true
  try {
    if (isNewRole.value) {
      await createRole(roleData)
      ElMessage.success('角色创建成功')
    } else {
      await saveRole(roleData)
      ElMessage.success('角色保存成功')
    }
    savingRole.value = false
    closeEditor()
  } catch (err) {
    ElMessage.error(`保存失败: ${err.message}`)
  } finally { savingRole.value = false }
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
.main-content { min-height: 200px; }
.loading-state { display: flex; align-items: center; justify-content: center; gap: 12px; min-height: 240px; color: var(--room-muted); font-size: 13px; }
</style>
