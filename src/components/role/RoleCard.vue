<template>
  <div 
    :class="['role-card', { 'is-active': isActive }]"
    @click="$emit('select', role.id)"
  >
    <div class="role-avatar">
      <img v-if="role.avatar" :src="role.avatar" :alt="role.name" />
      <div v-else class="avatar-placeholder">
        <span>{{ role.name.charAt(0) }}</span>
      </div>
      <div v-if="isActive" class="active-badge">
        <i class="fas fa-check"></i>
      </div>
    </div>
    
    <div class="role-info">
      <div class="role-header">
        <span class="role-name">{{ role.name }}</span>
      </div>
      <p class="role-desc">{{ role.description || '暂无描述' }}</p>
      <div class="role-tags" v-if="role.tags && role.tags.length > 0">
        <span v-for="tag in role.tags.slice(0, 3)" :key="tag" class="tag">
          {{ tag }}
        </span>
      </div>
    </div>
    
    <div class="role-actions" @click.stop>
      <el-tooltip content="编辑" placement="top">
        <button class="action-btn" @click="$emit('edit', role.id)">
          <i class="fas fa-pen"></i>
        </button>
      </el-tooltip>
      <el-tooltip content="复制" placement="top">
        <button class="action-btn" @click="$emit('duplicate', role.id)">
          <i class="fas fa-copy"></i>
        </button>
      </el-tooltip>
      <el-tooltip content="导出" placement="top">
        <button class="action-btn" @click="$emit('export', role.id)">
          <i class="fas fa-download"></i>
        </button>
      </el-tooltip>
      <el-tooltip content="删除" placement="top">
        <button class="action-btn delete" @click="$emit('delete', role.id)">
          <i class="fas fa-trash"></i>
        </button>
      </el-tooltip>
    </div>
  </div>
</template>

<script setup>
defineProps({
  role: {
    type: Object,
    required: true
  },
  isActive: {
    type: Boolean,
    default: false
  }
})

defineEmits(['select', 'edit', 'delete', 'export', 'duplicate'])
</script>

<style scoped>
.role-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  background: rgba(255, 255, 255, 0.95);
  border-radius: 20px;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.03);
}

.role-card:hover {
  background: rgba(255, 255, 255, 1);
  transform: translateY(-3px);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.08);
}

.role-card.is-active {
  border-color: rgba(59, 130, 246, 0.4);
  background: linear-gradient(135deg, rgba(244, 114, 182, 0.08), rgba(59, 130, 246, 0.08));
}

.role-avatar {
  position: relative;
  width: 64px;
  height: 64px;
  flex-shrink: 0;
}

.role-avatar img {
  width: 100%;
  height: 100%;
  border-radius: 18px;
  object-fit: cover;
  border: 3px solid rgba(255, 255, 255, 0.9);
  box-shadow: 0 6px 16px rgba(244, 114, 182, 0.25);
}

.avatar-placeholder {
  width: 100%;
  height: 100%;
  border-radius: 18px;
  background: linear-gradient(135deg, #f472b6, #3b82f6);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  font-weight: 700;
  color: white;
  text-transform: uppercase;
  box-shadow: 0 6px 16px rgba(244, 114, 182, 0.25);
}

.active-badge {
  position: absolute;
  bottom: -2px;
  right: -2px;
  width: 26px;
  height: 26px;
  background: linear-gradient(135deg, #10b981, #059669);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 12px;
  border: 3px solid white;
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3);
}

.role-info {
  flex: 1;
  min-width: 0;
}

.role-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.role-name {
  font-size: 18px;
  font-weight: 700;
  color: #1e293b;
}

.role-desc {
  font-size: 13px;
  color: #64748b;
  margin: 0 0 8px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.4;
}

.role-tags {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.tag {
  font-size: 11px;
  padding: 4px 12px;
  background: rgba(59, 130, 246, 0.08);
  color: #3b82f6;
  border-radius: 10px;
  font-weight: 600;
}

.role-actions {
  display: flex;
  flex-direction: column;
  gap: 6px;
  opacity: 0;
  transition: opacity 0.2s;
}

.role-card:hover .role-actions {
  opacity: 1;
}

.action-btn {
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 12px;
  background: rgba(0, 0, 0, 0.03);
  color: #64748b;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.action-btn:hover {
  background: linear-gradient(135deg, #f472b6, #3b82f6);
  color: white;
  transform: scale(0.95);
}

.action-btn.delete:hover {
  background: linear-gradient(135deg, #ef4444, #dc2626);
}

@media (max-width: 640px) {
  .role-card {
    padding: 16px;
    gap: 12px;
  }
  
  .role-avatar {
    width: 52px;
    height: 52px;
  }
  
  .avatar-placeholder {
    font-size: 18px;
  }
  
  .role-name {
    font-size: 16px;
  }
  
  .role-actions {
    opacity: 1;
    flex-direction: row;
  }
  
  .action-btn {
    width: 32px;
    height: 32px;
  }
}
</style>
