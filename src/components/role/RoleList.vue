<template>
  <div class="role-list">
    <div class="toolbar">
      <div class="search-box">
        <RoomIcon name="search" />
        <input 
          v-model="searchQuery" 
          type="text" 
          placeholder="搜索角色..."
        />
      </div>
      <div class="toolbar-actions">
        <el-tooltip content="导入角色" placement="top">
          <button class="tool-btn" aria-label="导入角色" @click="triggerImport">
            <RoomIcon name="upload" />
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
      
      <button class="add-role-card" @click="$emit('create')">
        <div class="add-icon">
          <RoomIcon name="plus" />
        </div>
        <span class="add-text">新建角色</span>
      </button>
      
      <div v-if="filteredRoles.length === 0" class="empty-state">
        <RoomIcon name="profile" />
        <p>没有找到角色</p>
        <button class="create-btn" @click="$emit('create')">
          <RoomIcon name="plus" /> 创建新角色
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { ElMessageBox } from 'element-plus'
import { appMessage as ElMessage } from '../../services/appFeedback'
import RoleCard from './RoleCard.vue'
import RoomIcon from '../companion/RoomIcon.vue'

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
}

function handleDuplicate(roleId) {
  emit('duplicate', roleId)
}

function triggerImport() {
  importInput.value?.click()
}

async function handleImport(e) {
  const file = e.target.files?.[0]
  if (!file) return
  
  try {
    emit('import', file)
  } catch (err) {
    ElMessage.error(`导入失败: ${err.message}`)
  }
  
  e.target.value = ''
}
</script>

<style scoped>
.role-list { display: flex; flex-direction: column; gap: 22px; }
.toolbar { display: flex; align-items: center; gap: 12px; }
.search-box { position: relative; flex: 1; }
.search-box > svg { position: absolute; left: 14px; top: 50%; transform: translateY(-50%); color: var(--room-muted); width: 18px; height: 18px; }
.search-box input { width: 100%; height: 44px; padding: 11px 14px 11px 42px; border: 1px solid var(--room-line); border-radius: 11px; background: var(--room-surface); color: var(--room-ink); font-size: 14px; outline: none; }
.search-box input:focus { border-color: var(--room-accent); box-shadow: 0 0 0 3px var(--room-accent-soft); }
.toolbar-actions { display: flex; gap: 8px; }
.tool-btn { display: grid; place-items: center; height: 44px; width: 44px; border: 1px solid var(--room-line); border-radius: 11px; background: var(--room-surface); color: var(--room-muted); }
.roles-container { display: flex; flex-direction: column; gap: 14px; }
.roles-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(100%, 340px), 1fr)); gap: 14px; }
.add-role-card { display: flex; align-items: center; justify-content: center; gap: 10px; min-height: 64px; padding: 15px; border: 1px dashed #c7d0dc; border-radius: 16px; background: transparent; color: var(--room-muted); font-size: 13px; }
.add-role-card:hover { background: var(--room-accent-soft); color: var(--room-accent); border-color: var(--room-accent); }
.add-icon { display: flex; }
.empty-state { display: flex; flex-direction: column; align-items: center; padding: 40px 20px; color: var(--room-muted); font-size: 13px; }
.create-btn { display: flex; align-items: center; gap: 7px; padding: 10px 14px; border: 1px solid var(--room-line); border-radius: 10px; background: var(--room-surface); }
.list-enter-active, .list-leave-active { transition: opacity .2s; }
.list-enter-from, .list-leave-to { opacity: 0; }
@media (max-width: 600px) { .search-box input { font-size: 16px; } }
</style>
