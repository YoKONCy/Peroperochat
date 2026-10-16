<template>
  <div 
    :class="['role-card', { 'is-active': isActive }]"
    role="button" tabindex="0" :aria-label="`选择角色 ${role.name}`" :aria-pressed="isActive"
    @click="$emit('select', role.id)"
    @keydown.enter.self="$emit('select', role.id)"
    @keydown.space.self.prevent="$emit('select', role.id)"
  >
    <div class="role-avatar">
      <CharacterAvatar class="role-portrait" :src="role.avatar" :label="role.name" />
      <div v-if="isActive" class="active-badge">
        <RoomIcon name="check" />
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
        <button class="action-btn" aria-label="编辑角色" @click="$emit('edit', role.id)">
          <RoomIcon name="edit" />
        </button>
      </el-tooltip>
      <el-tooltip content="复制" placement="top">
        <button class="action-btn" aria-label="复制角色" @click="$emit('duplicate', role.id)">
          <RoomIcon name="copy" />
        </button>
      </el-tooltip>
      <el-tooltip content="导出" placement="top">
        <button class="action-btn" aria-label="导出角色" @click="$emit('export', role.id)">
          <RoomIcon name="download" />
        </button>
      </el-tooltip>
      <el-tooltip content="删除" placement="top">
        <button class="action-btn delete" aria-label="删除角色" @click="$emit('delete', role.id)">
          <RoomIcon name="trash" />
        </button>
      </el-tooltip>
    </div>
  </div>
</template>

<script setup>
import RoomIcon from '../companion/RoomIcon.vue'
import CharacterAvatar from '../companion/CharacterAvatar.vue'
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
.role-card { display: grid; grid-template-columns: 80px minmax(0, 1fr); gap: 12px 18px; padding: 20px; border: 1px solid var(--room-line); border-radius: 3px 14px 3px 3px; background: var(--room-surface); cursor: pointer; position: relative; transition: border-color .18s, background .18s; }
.role-card:hover { border-color: var(--room-accent); }
.role-card.is-active { box-shadow: inset 2px 0 var(--room-accent); border-color: var(--room-accent); background: var(--room-subtle); }
.role-card:focus-visible { outline: 2px solid var(--room-accent); outline-offset: 3px; }
.role-avatar { position: relative; grid-row: span 2; width: 80px; height: 100px; }
.role-avatar .role-portrait { width: 100%; height: 100%; border-radius: 3px 9px 3px 3px; }
.active-badge { position: absolute; right: -6px; bottom: -5px; display: grid; place-items: center; width: 23px; height: 23px; border: 3px solid var(--room-surface); border-radius: 50%; background: var(--room-accent); color: white; }
.active-badge svg { width: 13px; height: 13px; }
.role-info { min-width: 0; }
.role-name { display: block; font-size: 17px; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.role-desc { font-size: 12px; line-height: 1.65; color: var(--room-muted); margin: 7px 0 10px; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
.role-tags { display: flex; gap: 5px; flex-wrap: wrap; }
.tag { padding: 3px 6px; border-radius: 5px; color: var(--room-accent); background: var(--room-accent-soft); font-size: 10px; }
.role-actions { grid-column: 2; display: flex; gap: 6px; }
.action-btn { display: grid; place-items: center; width: 35px; height: 35px; border: 0; border-radius: 9px; background: transparent; color: var(--room-muted); }
.action-btn svg { width: 17px; height: 17px; }
.action-btn:hover { background: var(--room-accent-soft); color: var(--room-accent); }
.action-btn.delete:hover { background: #fff0f3; color: var(--room-danger); }
@media (max-width: 420px) { .role-card { padding: 16px; gap: 12px 14px; } }
</style>
