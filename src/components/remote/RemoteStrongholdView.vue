<template>
  <div class="remote-stronghold-view pixel-font">
    <!-- 标签切换 -->
    <div class="tabs pixel-border-sm">
      <div 
        v-for="t in ['members', 'rooms']" 
        :key="t"
        :class="['tab', { active: activeTab === t }]"
        @click="activeTab = t"
      >
        {{ t === 'members' ? '据点成员' : '房间列表' }}
      </div>
    </div>

    <!-- 成员列表 -->
    <div v-if="activeTab === 'members'" class="tab-content">
      <div class="member-grid">
        <div v-for="agent in agents" :key="agent.id" class="member-card pixel-border-sm">
          <div class="card-left">
            <img :src="getFullUrl(agent.avatar)" class="member-avatar pixel-border-sm" />
            <div class="member-info">
              <div class="member-name">{{ agent.name }}</div>
              <div :class="['member-status', agent.is_active ? 'active' : 'inactive']">
                {{ agent.is_active ? '活跃中' : '休息中' }}
              </div>
            </div>
          </div>
          <button 
            class="pixel-btn action-btn" 
            :class="{ 'btn-active': agent.is_active }"
            @click="toggleAgent(agent)"
          >
            {{ agent.is_active ? '驱逐' : '召唤' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 房间列表 -->
    <div v-if="activeTab === 'rooms'" class="tab-content">
      <div class="room-list">
        <div v-for="room in rooms" :key="room.id" class="room-card pixel-border-sm">
          <div class="room-info">
            <div class="room-name"># {{ room.name }}</div>
            <div class="room-desc">{{ room.description || '暂无描述' }}</div>
            <div class="room-members">
              <span v-for="m in room.members" :key="m" class="member-tag">{{ m }}</span>
            </div>
          </div>
          <button class="pixel-btn enter-btn">进入</button>
        </div>
      </div>
    </div>

    <div v-if="isUpdating" class="loading-overlay">
      <div class="pixel-loader">更新中...</div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'

const props = defineProps({
  pcUrl: String,
  token: String
})

const activeTab = ref('members')
const agents = ref([])
const rooms = ref([])
const isUpdating = ref(false)

const getFullUrl = (path) => {
  if (!path) return ''
  if (path.startsWith('http')) return path
  return `${props.pcUrl.replace('/api', '')}${path}`
}

const fetchAgents = async () => {
  try {
    const res = await fetch(`${props.pcUrl}/api/agents`, {
      headers: { 'Authorization': `Bearer ${props.token}` }
    })
    if (res.ok) {
      agents.value = await res.json()
    }
  } catch (e) {
    console.error('获取 Agent 列表失败:', e)
  }
}

const fetchRooms = async () => {
  try {
    const res = await fetch(`${props.pcUrl}/api/stronghold/rooms`, {
      headers: { 'Authorization': `Bearer ${props.token}` }
    })
    if (res.ok) {
      rooms.value = await res.json()
    }
  } catch (e) {
    console.error('获取房间列表失败:', e)
  }
}

const toggleAgent = async (agent) => {
  isUpdating.value = true
  const action = agent.is_active ? 'deactivate' : 'activate'
  try {
    const res = await fetch(`${props.pcUrl}/api/agents/${agent.id}/${action}`, {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${props.token}` }
    })
    if (res.ok) {
      await fetchAgents()
    }
  } catch (e) {
    console.error('更新 Agent 状态失败:', e)
  } finally {
    isUpdating.value = false
  }
}

onMounted(() => {
  fetchAgents()
  fetchRooms()
})
</script>

<style scoped>
.remote-stronghold-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #111;
  position: relative;
}

.tabs {
  display: flex;
  background: #222;
  margin: 10px;
}

.tab {
  flex: 1;
  padding: 12px;
  text-align: center;
  cursor: pointer;
  font-size: 14px;
}

.tab.active {
  background: #0088ff;
  color: #fff;
  font-weight: bold;
}

.tab-content {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.member-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.member-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background: #222;
}

.card-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.member-avatar {
  width: 48px;
  height: 48px;
  object-fit: cover;
}

.member-name {
  font-size: 16px;
  font-weight: bold;
  color: #fff;
}

.member-status {
  font-size: 10px;
  margin-top: 4px;
}

.member-status.active { color: #0f0; }
.member-status.inactive { color: #666; }

.action-btn {
  padding: 8px 16px;
  font-size: 12px;
  min-width: 80px;
}

.btn-active {
  background: #ff4444;
}

.room-list {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.room-card {
  padding: 15px;
  background: #222;
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.room-name {
  font-size: 16px;
  font-weight: bold;
  color: #0ff;
  margin-bottom: 5px;
}

.room-desc {
  font-size: 12px;
  color: #aaa;
  margin-bottom: 10px;
}

.room-members {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.member-tag {
  font-size: 10px;
  padding: 2px 6px;
  background: #333;
  color: #eee;
  border: 1px solid #555;
}

.enter-btn {
  background: #0088ff;
  padding: 6px 12px;
  font-size: 12px;
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0,0,0,0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
}

.pixel-loader {
  padding: 20px;
  border: 4px solid #fff;
  background: #000;
  color: #0f0;
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
}

.pixel-border-sm {
  border: 2px solid #fff;
}

.pixel-btn {
  border: 2px solid #fff;
  background: #444;
  color: #fff;
  cursor: pointer;
  font-family: inherit;
}

.pixel-btn:active {
  transform: scale(0.95);
}
</style>
