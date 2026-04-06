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
  gap: 18px;
  padding: 24px;
  background: rgba(255, 255, 255, 0.6);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-radius: 24px;
  border: 1px solid rgba(255, 255, 255, 0.8);
  cursor: pointer;
  transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
  position: relative;
  overflow: hidden;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.02), inset 0 0 0 1px rgba(255, 255, 255, 0.2);
}

.role-card:hover {
  background: rgba(255, 255, 255, 0.85);
  transform: translateY(-4px) scale(1.01);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.06), 0 1px 3px rgba(0, 0, 0, 0.04);
}

.role-card.is-active {
  border: 1px solid rgba(59, 130, 246, 0.5);
  background: linear-gradient(135deg, rgba(244, 114, 182, 0.1), rgba(59, 130, 246, 0.1));
  box-shadow: 0 10px 30px rgba(59, 130, 246, 0.15), inset 0 0 0 1px rgba(255, 255, 255, 0.5);
}

.role-card.is-active::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 6px;
  height: 100%;
  background: linear-gradient(to bottom, #f472b6, #3b82f6);
  border-radius: 20px 0 0 20px;
}

.role-avatar {
  position: relative;
  width: 72px;
  height: 72px;
  flex-shrink: 0;
}

.role-avatar img {
  width: 100%;
  height: 100%;
  border-radius: 20px;
  object-fit: cover;
  border: 2px solid rgba(255, 255, 255, 0.9);
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.08);
}

.avatar-placeholder {
  width: 100%;
  height: 100%;
  border-radius: 20px;
  background: linear-gradient(135deg, #f472b6, #818cf8);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 28px;
  font-weight: 700;
  color: white;
  text-transform: uppercase;
  box-shadow: 0 8px 16px rgba(244, 114, 182, 0.3);
}

.active-badge {
  position: absolute;
  bottom: -4px;
  right: -4px;
  width: 28px;
  height: 28px;
  background: linear-gradient(135deg, #10b981, #059669);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 13px;
  border: 3px solid white;
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.4);
  z-index: 2;
}

.role-info {
  flex: 1;
  min-width: 0;
}

.role-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.role-name {
  font-size: 19px;
  font-weight: 800;
  color: #1e293b;
  letter-spacing: -0.02em;
}

.role-desc {
  font-size: 13px;
  color: #64748b;
  margin: 0 0 10px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.5;
}

.role-tags {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.tag {
  font-size: 11px;
  padding: 4px 10px;
  background: rgba(241, 245, 249, 0.8);
  color: #475569;
  border-radius: 8px;
  font-weight: 600;
  border: 1px solid rgba(226, 232, 240, 0.8);
}

.role-card.is-active .tag {
  background: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
  border-color: rgba(59, 130, 246, 0.2);
}

.role-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
  opacity: 0;
  transform: translateX(10px);
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.role-card:hover .role-actions {
  opacity: 1;
  transform: translateX(0);
}

.action-btn {
  width: 38px;
  height: 38px;
  border: none;
  border-radius: 12px;
  background: white;
  color: #64748b;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.action-btn:hover {
  background: linear-gradient(135deg, #f472b6, #818cf8);
  color: white;
  transform: scale(1.05);
  box-shadow: 0 4px 12px rgba(244, 114, 182, 0.3);
}

.action-btn.delete:hover {
  background: linear-gradient(135deg, #ef4444, #f43f5e);
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
}

@media (max-width: 640px) {
  .role-card {
    padding: 16px;
    gap: 14px;
    border-radius: 20px;
  }
  
  .role-avatar {
    width: 56px;
    height: 56px;
  }
  
  .avatar-placeholder {
    font-size: 22px;
  }
  
  .role-name {
    font-size: 17px;
  }
  
  .role-actions {
    opacity: 1;
    transform: translateX(0);
    flex-direction: row;
    position: absolute;
    top: 16px;
    right: 16px;
  }
  
  .action-btn {
    width: 32px;
    height: 32px;
    background: rgba(255, 255, 255, 0.8);
  }
}
</style>
