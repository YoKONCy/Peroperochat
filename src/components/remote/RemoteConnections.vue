<template>
  <RoomDrawer title="连接 infOS" subtitle="选择你的角色所在的实例" :busy="busy" @close="$emit('close')">
    <p class="remote-help">进入远程模式后，聊天与任务由 infOS 执行，独立模式暂停。两套数据分别保存。</p>
    <p v-if="pendingCount" class="remote-notice">当前连接有 {{ pendingCount }} 项操作待核对，切换后仍保留在原连接中。</p>
    <p v-if="error" class="room-error" role="alert">{{ error }}</p>
    <div class="remote-profile-list">
      <article v-for="profile in profiles" :key="profile.id">
        <button class="remote-profile-select" :disabled="busy || activeId === profile.id" @click="enter(profile)"><RoomIcon name="connection" /><span><strong>{{ profile.displayName || 'infOS' }}</strong><small>{{ profile.endpoint }}</small></span><span>{{ activeId === profile.id ? '当前连接' : '进入' }}</span></button>
        <button v-if="activeId !== profile.id" class="room-text-button" :disabled="busy" @click="removing = profile.id" aria-label="移除此连接"><RoomIcon name="trash" /></button>
        <div v-if="removing === profile.id" class="remote-inline-confirm"><span>移除这条连接配置？</span><button :disabled="busy" @click="remove(profile.id)">移除</button><button :disabled="busy" @click="removing = null">保留</button></div>
      </article>
    </div>
    <form class="remote-connection-form" @submit.prevent="save">
      <h3>新的连接</h3>
      <label>服务地址<input v-model="endpoint" type="url" placeholder="https://infos.example.com" autocomplete="url" required :disabled="busy" /></label>
      <label>访问凭据<input v-model="secret" type="password" autocomplete="off" placeholder="服务端配置的访问凭据" :disabled="busy" maxlength="4096" /></label>
      <label v-if="endpoint.trim().startsWith('http:')" class="remote-checkbox"><input v-model="allowInsecure" type="checkbox" :disabled="busy" />允许此地址使用明文 HTTP</label>
      <button class="room-primary-button" type="submit" :disabled="busy || !endpoint.trim()">{{ busy ? '正在连接…' : '连接并进入' }}</button>
    </form>
    <template #footer><button v-if="activeId" class="room-secondary-button" :disabled="busy" @click="standalone"><RoomIcon name="back" />回到独立模式</button><span v-else class="remote-help">凭据交由系统安全存储。</span></template>
  </RoomDrawer>
</template>
<script setup>
import { computed, ref, onMounted, onBeforeUnmount } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RoomIcon from '../companion/RoomIcon.vue'
import { remoteConnections } from '../../services/remote'
import { modeState, switchMode } from '../../stores/mode'
import { remotePendingRequests } from '../../stores/remote'
const props = defineProps({ prepareSwitch: Function })
const emit = defineEmits(['close'])
const profiles = ref([]), endpoint = ref(''), secret = ref(''), allowInsecure = ref(false), busy = ref(false), error = ref(''), removing = ref(null)
const activeId = computed(() => modeState.snapshot?.selectedTarget?.mode === 'remote' ? modeState.snapshot.selectedTarget.profileId : null)
const pendingCount = computed(() => remotePendingRequests.size)
let disposed = false
async function run(action) {
  if (busy.value) return
  busy.value = true; error.value = ''
  try { await action() } catch (failure) { if (!disposed) error.value = failure.message } finally { if (!disposed) busy.value = false }
}
async function enter(profile) { await run(async () => { await props.prepareSwitch?.(); await switchMode({ mode: 'remote', profileId: profile.id }); emit('close') }) }
async function save() {
  await run(async () => {
    const profile = await remoteConnections.save({ endpoint: endpoint.value.trim(), token: secret.value, allowInsecure: allowInsecure.value })
    secret.value = ''
    profiles.value = await remoteConnections.list()
    await props.prepareSwitch?.()
    await switchMode({ mode: 'remote', profileId: profile.id })
    emit('close')
  })
}
async function standalone() { await run(async () => { await props.prepareSwitch?.(); await switchMode({ mode: 'standalone' }); emit('close') }) }
async function remove(id) { await run(async () => { await remoteConnections.remove(id); profiles.value = await remoteConnections.list(); removing.value = null }) }
onMounted(() => run(async () => { const data = await remoteConnections.list(); if (!disposed) profiles.value = data }))
onBeforeUnmount(() => { disposed = true; secret.value = '' })
</script>
