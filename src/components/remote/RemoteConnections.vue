<template>
  <RoomDrawer title="连接与运行模式" subtitle="把 infOS 的陪伴与进展带在身边" :busy="busy" @close="$emit('close')">
    <div class="remote-mode-intro"><RoomIcon name="connection" /><div><strong>{{ activeId ? '当前运行于 infOS 客户端' : '从独立空间，前往 infOS' }}</strong><p>远程聊天与任务由服务端执行。独立空间暂停，两套角色、记忆与草稿分别保存。</p></div></div>
    <p v-if="pendingCount" class="remote-notice">当前连接有 {{ pendingCount }} 项操作待核对，切换后仍保留在原连接中。</p>
    <p v-if="error" class="room-error" role="alert">{{ error }}</p>
    <p v-if="savedName" class="remote-notice">「{{ savedName }}」已保存到本机，可以从连接卡再次进入。</p>
    <p v-if="busy" class="remote-connect-progress" role="status"><RoomIcon class="room-spinning" name="refresh" />{{ progressLabel }}</p>
    <div class="remote-connections-heading"><h3>已保存的连接</h3><button type="button" class="room-text-button" :disabled="busy || profilesLoading" @click="loadProfiles">{{ profilesLoading ? '读取中…' : '刷新' }}</button></div>
    <p v-if="listError" class="room-error" role="alert">{{ listError }}</p>
    <p v-else-if="!profilesLoading && !profiles.length" class="remote-help">先在电脑端 infOS 的「局域网与设备」开启访问，再生成配对码。</p>
    <div class="remote-profile-list">
      <article v-for="profile in profiles" :key="profile.id">
        <button class="remote-profile-select" :disabled="busy || activeId === profile.id" @click="expired(profile) ? reauthorize(profile) : enter(profile)"><RoomIcon name="connection" /><span><strong>{{ profile.displayName || 'infOS' }}</strong><small>{{ profile.endpoint }}</small></span><span>{{ activeId === profile.id ? '当前连接' : expired(profile) ? '重新授权' : '进入' }}</span></button>
        <button class="room-text-button" :disabled="busy" @click="reauthorize(profile)">重新授权</button>
        <small v-if="profile.device" class="remote-help">{{ expired(profile) ? '设备会话已过期' : `授权至 ${expiry(profile.device.expiresAt)}` }}</small>
        <button v-if="activeId !== profile.id" class="room-text-button" :disabled="busy" @click="removing = profile.id" aria-label="移除此连接"><RoomIcon name="trash" /></button>
        <div v-if="removing === profile.id" class="remote-inline-confirm"><span>移除连接配置？本机记录仍保留，服务端设备授权需在 infOS 中单独撤销。</span><button :disabled="busy" @click="remove(profile.id)">移除</button><button :disabled="busy" @click="removing = null">保留</button></div>
      </article>
    </div>
    <form class="remote-connection-form" @submit.prevent="save">
      <h3>{{ editingId ? '重新授权原连接' : '新的连接' }}</h3>
      <p v-if="editingId" class="remote-help">确认同一 infOS 实例后更新设备会话，保留草稿与阅读记录。</p>
      <label>服务地址<input v-model="endpoint" type="url" placeholder="https://infos.example.com 或 http://电脑IP:9122" autocomplete="url" required :disabled="busy || !!editingId" /></label>
      <small class="remote-help">使用手机可访问的服务地址。局域网连接填写电脑 IP，不能使用电脑上的 localhost。</small>
      <label>连接方式<select v-model="method" :disabled="busy" @change="secret = ''"><option value="pair">设备配对码</option><option value="login">管理凭据登录</option><option value="bearer">直接 Bearer 凭据（兼容）</option></select></label>
      <label>{{ method === 'pair' ? '一次性配对码' : '访问凭据' }}<input ref="secretInput" v-model="secret" :type="method === 'pair' ? 'text' : 'password'" :autocomplete="method === 'pair' ? 'one-time-code' : 'off'" autocapitalize="off" spellcheck="false" required :placeholder="method === 'pair' ? '在 infOS「局域网与设备」生成配对码' : '服务端提供的凭据'" :disabled="busy" maxlength="512" /></label>
      <template v-if="method !== 'bearer'"><label>设备名称<input v-model="deviceName" required maxlength="80" :disabled="busy" /></label><label class="remote-checkbox"><input v-model="remember" type="checkbox" :disabled="busy" />保持登录 30 天</label></template>
      <small v-if="method !== 'bearer' && !remember" class="remote-help">授权仅在当前应用进程内保留，完全退出后需重新配对。</small>
      <p v-if="method === 'pair'" class="remote-help">配对码五分钟内有效，仅可兑换一次。</p>
      <label v-if="/^http:/i.test(endpoint.trim())" class="remote-checkbox"><input v-model="allowInsecure" type="checkbox" :disabled="busy" />允许此地址使用明文 HTTP</label>
      <button class="room-primary-button" type="submit" :disabled="busy || !endpoint.trim() || !secret.trim()">{{ busy ? progressLabel : editingId ? '更新授权并进入' : '连接并进入' }}</button>
      <button v-if="editingId" class="room-text-button" type="button" :disabled="busy" @click="newConnection">改为新建连接</button>
    </form>
    <template #footer><button v-if="activeId" class="room-secondary-button" :disabled="busy" @click="standalone"><RoomIcon name="back" />回到独立模式</button><span v-else class="remote-help">凭据交由系统安全存储。</span><button v-if="currentProfile?.device && !logoutConfirm" class="room-text-button" :disabled="busy" @click="logoutConfirm = true">撤销此设备授权…</button><div v-if="logoutConfirm" class="remote-inline-confirm"><span>撤销授权后需要重新配对，原连接记录保留。</span><button :disabled="busy" @click="logout">确认撤销</button><button :disabled="busy" @click="logoutConfirm = false">保留授权</button></div></template>
  </RoomDrawer>
</template>
<script setup>
import { computed, ref, onMounted, onBeforeUnmount, nextTick } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RoomIcon from './RemoteIcon.vue'
import { remoteConnections, remoteClient } from '../../services/remote'
import { modeState, switchMode, refreshMode, currentToken } from '../../stores/mode'
import { remotePendingRequests } from '../../stores/remote'
const props = defineProps({ prepareSwitch: Function })
const emit = defineEmits(['close'])
const profiles = ref([]), endpoint = ref(''), secret = ref(''), allowInsecure = ref(false), busy = ref(false), error = ref(''), removing = ref(null)
const method = ref('pair'), deviceName = ref('PeroperoChat'), remember = ref(true), editingId = ref(null)
const logoutConfirm = ref(false)
const secretInput = ref(null), profilesLoading = ref(false), listError = ref(''), savedName = ref(''), step = ref('preparing')
const activeId = computed(() => modeState.snapshot?.selectedTarget?.mode === 'remote' ? modeState.snapshot.selectedTarget.profileId : null)
const pendingCount = computed(() => remotePendingRequests.size)
const currentProfile = computed(() => profiles.value.find(profile => profile.id === activeId.value))
const progressLabel = computed(() => ({ preparing: '正在保存当前空间…', authorizing: '正在核对设备授权…', switching: '正在切换运行模式…', removing: '正在移除连接…', revoking: '正在撤销设备授权…' }[step.value]))
let disposed = false, listVersion = 0
async function loadProfiles() {
  const version = ++listVersion; profilesLoading.value = true; listError.value = ''
  try { const data = await remoteConnections.list(); if (!disposed && version === listVersion) profiles.value = data }
  catch (failure) { if (!disposed && version === listVersion) listError.value = failure.message }
  finally { if (!disposed && version === listVersion) profilesLoading.value = false }
}
async function run(action) {
  if (busy.value) return
  busy.value = true; error.value = ''; savedName.value = ''; step.value = 'preparing'
  try { await action() } catch (failure) { if (!disposed) error.value = failure.message } finally { if (!disposed) busy.value = false }
}
async function prepare() { await props.prepareSwitch?.(); if (disposed) throw new Error('连接面板已关闭') }
async function enter(profile) { await run(async () => { await prepare(); step.value = 'switching'; await switchMode({ mode: 'remote', profileId: profile.id }); emit('close') }) }
async function save() {
  await run(async () => {
    await prepare(); step.value = 'authorizing'
    let profile
    try {
      profile = await remoteConnections.save({ endpoint: endpoint.value.trim(), method: method.value,
        token: method.value === 'bearer' ? secret.value : '', code: method.value === 'bearer' ? '' : secret.value,
        name: deviceName.value, remember: remember.value, profileId: editingId.value, allowInsecure: allowInsecure.value })
    } finally { secret.value = '' }
    if (disposed) return
    // 保存与进入分别处理，切换失败也保留卡片，避免再次消耗一次性配对码。
    listVersion++; profilesLoading.value = false
    profiles.value = [...profiles.value.filter(item => item.id !== profile.id), profile]
    savedName.value = profile.displayName || 'infOS'; editingId.value = profile.id
    step.value = 'switching'; await refreshMode()
    if (disposed) return
    await switchMode({ mode: 'remote', profileId: profile.id })
    emit('close')
  })
}
async function reauthorize(profile) { editingId.value = profile.id; endpoint.value = profile.endpoint; secret.value = ''; method.value = 'pair'; allowInsecure.value = /^http:/i.test(profile.endpoint); error.value = ''; await nextTick(); secretInput.value?.focus() }
function newConnection() { editingId.value = null; endpoint.value = ''; secret.value = ''; allowInsecure.value = false; savedName.value = '' }
function expired(profile) { return !!profile.device && profile.device.expiresAt <= Date.now() }
function expiry(value) { const date = new Date(value); return Number.isNaN(date.getTime()) ? '未知时间' : date.toLocaleString() }
async function logout() { await run(async () => { await prepare(); step.value = 'revoking'; await remoteClient.logoutDevice(currentToken()); logoutConfirm.value = false; if (currentProfile.value) await reauthorize(currentProfile.value) }) }
async function standalone() { await run(async () => { await prepare(); step.value = 'switching'; await switchMode({ mode: 'standalone' }); emit('close') }) }
async function remove(id) { await run(async () => { step.value = 'removing'; await remoteConnections.remove(id); await loadProfiles(); removing.value = null; if (editingId.value === id) newConnection() }) }
onMounted(loadProfiles)
onBeforeUnmount(() => { disposed = true; listVersion++; secret.value = '' })
</script>
<style scoped>
.remote-mode-intro { display: flex; gap: 13px; align-items: flex-start; padding: 15px; margin-bottom: 18px; border: 1px solid var(--room-line); border-radius: 15px; background: var(--room-accent-soft); }.remote-mode-intro > svg { width: 25px; height: 25px; flex: none; color: var(--room-accent); }.remote-mode-intro strong { font-size: 13px; font-weight: 600; }.remote-mode-intro p { font-size: 11px; line-height: 1.8; color: var(--room-muted); margin: 7px 0 0; }
.remote-connections-heading { display: flex; align-items: center; justify-content: space-between; }.remote-connections-heading h3 { font-size: 13px; font-weight: 600; margin: 10px 0; }.remote-connect-progress { display: flex; gap: 9px; align-items: center; font-size: 11px; color: var(--room-accent); }.remote-connect-progress svg { width: 15px; height: 15px; }.remote-profile-list { display: grid; gap: 10px; }.remote-profile-list article { border: 1px solid var(--room-line); border-radius: 15px; padding: 8px 12px; background: var(--room-glass); }.remote-profile-select { padding: 8px 0; width: 100%; border: 0; background: transparent; text-align: left; display: flex; align-items: center; gap: 10px; }.remote-profile-select > svg { width: 22px; height: 22px; color: var(--room-accent); flex: none; }.remote-profile-select > span:first-of-type { flex: 1; min-width: 0; display: grid; gap: 5px; }.remote-profile-select strong { font-size: 13px; font-weight: 600; }.remote-profile-select small { font-size: 10px; color: var(--room-muted); overflow-wrap: anywhere; }.remote-profile-select > span:last-child { font-size: 11px; color: var(--room-accent); flex: none; }
.remote-connection-form { margin-top: 24px; }.remote-connection-form h3 { font-size: 15px; font-weight: 600; }.remote-connection-form label { display: grid; gap: 8px; margin: 16px 0 0; font-size: 12px; }.remote-connection-form :is(input:not([type='checkbox']), select) { width: 100%; min-height: 42px; border: 1px solid var(--room-line); border-radius: 11px; background: var(--room-subtle); color: var(--room-ink); padding: 10px 12px; font: inherit; }.remote-connection-form .remote-checkbox { display: flex; align-items: center; gap: 8px; }.remote-connection-form > .room-primary-button { width: 100%; margin-top: 18px; }.remote-help { font-size: 11px; line-height: 1.8; color: var(--room-muted); }.remote-connection-form > small { display: block; margin-top: 7px; }.remote-notice { font-size: 11px; line-height: 1.8; padding: 11px 13px; border: 1px solid var(--room-line); border-radius: 11px; background: var(--room-accent-soft); }.remote-inline-confirm { display: flex; gap: 9px; align-items: center; flex-wrap: wrap; width: 100%; font-size: 11px; padding: 9px 0; }.remote-inline-confirm span { flex: 1; min-width: 120px; line-height: 1.7; }.remote-inline-confirm button { border: 1px solid var(--room-line); padding: 6px 10px; border-radius: 8px; background: var(--room-surface); }
</style>
