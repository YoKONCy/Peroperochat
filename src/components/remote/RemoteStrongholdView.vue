<template>
  <div class="remote-stronghold-view">
    <!-- 标签切换 -->
    <div class="tabs pixel-border-sm">
      <button
        v-for="t in ['members', 'rooms']" 
        :key="t"
        :class="['tab', { active: activeTab === t }]"
        @click="activeTab = t"
      >
        {{ t === 'members' ? '据点成员' : '房间列表' }}
      </button>
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
.remote-stronghold-view { position: relative; height: 100%; overflow-y: auto; padding: 24px max(18px, calc((100% - 1000px) / 2)); background: var(--room-page); }
.tabs { display: flex; gap: 6px; border-bottom: 1px solid var(--room-line); margin-bottom: 24px; }
.tab { border: 0; border-bottom: 2px solid transparent; padding: 12px 16px; background: transparent; color: var(--room-muted); font-size: 13px; }
.tab.active { color: var(--room-accent); border-bottom-color: var(--room-accent); }
.member-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(100%, 300px), 1fr)); gap: 12px; }
.member-card { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 18px; border: 1px solid var(--room-line); border-radius: 14px; background: var(--room-surface); }
.card-left { display: flex; align-items: center; gap: 12px; min-width: 0; }
.member-avatar { width: 44px; height: 50px; object-fit: cover; border-radius: 11px; }
.member-name { font-size: 14px; font-weight: 500; }
.member-status { margin-top: 6px; font-size: 11px; color: var(--room-muted); }
.member-status.active { color: var(--room-accent); }
.pixel-btn { border: 1px solid var(--room-line); border-radius: 9px; padding: 9px 12px; background: var(--room-surface); color: var(--room-ink); font-size: 12px; }
.btn-active { background: var(--room-accent-soft); color: var(--room-accent); }
.room-list { display: flex; flex-direction: column; gap: 12px; }
.room-card { display: flex; align-items: center; justify-content: space-between; gap: 20px; border: 1px solid var(--room-line); border-radius: 14px; background: var(--room-surface); padding: 20px; }
.room-name { font-size: 15px; font-weight: 500; }
.room-desc { color: var(--room-muted); font-size: 12px; line-height: 1.7; margin-top: 8px; }
.room-members { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 12px; }
.member-tag { font-size: 10px; border-radius: 5px; padding: 3px 6px; background: var(--room-accent-soft); color: var(--room-accent); }
.loading-overlay { position: absolute; inset: 0; display: grid; place-items: center; background: #ffffffb3; backdrop-filter: blur(2px); }
.pixel-loader { font-size: 13px; color: var(--room-muted); }
</style>
