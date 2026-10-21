<template>
  <RoomDrawer title="切换空间" :busy="busy || scanning" @close="$emit('close')">
    <div class="connections-panel">
      <nav class="connections-spaces" aria-label="运行空间">
        <button type="button" :class="{ selected: !activeId }" :aria-current="!activeId ? 'true' : undefined" :disabled="busy || scanning || !activeId" @click="standalone"><span class="connections-space-icon"><RoomIcon name="home" /></span><span><strong>独立空间</strong><small>{{ !activeId ? '当前空间' : '回到房间' }}</small></span><RoomIcon v-if="!activeId" name="check" /></button>
        <button type="button" :class="{ selected: activeId }" :aria-current="activeId ? 'true' : undefined" :disabled="busy || scanning" @click="showConnections"><span class="connections-space-icon"><RoomIcon name="connection" /></span><span><strong>infOS</strong><small>{{ activeId ? '当前空间' : '连接电脑' }}</small></span><RoomIcon v-if="activeId" name="check" /></button>
      </nav>
      <p v-if="pendingCount" class="connections-pending"><RoomIcon name="clock" />{{ pendingCount }} 项操作待核对，切换后仍会保留。</p>
      <RemoteFeedback :message="error" />
      <RemoteFeedback :message="savedName ? '连接已保存' : ''" tone="success" />
      <p v-if="busy" class="connections-progress" role="status"><RoomIcon class="room-spinning" name="refresh" />{{ progressLabel }}</p>

      <section ref="connectionSection" class="connections-saved" aria-label="已保存的 infOS">
        <div class="connections-heading"><h3>我的 infOS</h3><div><button type="button" class="connections-icon-button" :disabled="busy || scanning || profilesLoading" aria-label="刷新连接" @click="loadProfiles"><RoomIcon :class="{ 'room-spinning': profilesLoading }" name="refresh" /></button><button type="button" class="connections-add" :disabled="busy || scanning" @click="newConnection"><RoomIcon name="plus" />添加</button></div></div>
        <RemoteFeedback :message="listError" />
        <p v-if="profilesLoading && !profiles.length" class="connections-empty">正在找回连接…</p>
        <p v-else-if="!listError && !profiles.length" class="connections-empty">扫一下电脑上的配对二维码，就能接着聊。</p>
        <div class="connections-list">
          <article v-for="profile in profiles" :key="profile.id" class="connection-card" :class="{ 'is-current': activeId === profile.id }">
            <div class="connection-identity"><span class="connection-glyph"><RoomIcon name="connection" /></span><div><strong :title="profile.displayName">{{ profileName(profile) }}</strong><small :title="profile.endpoint">{{ profile.endpoint }}</small></div><span class="connection-state" :class="{ 'needs-pairing': expired(profile) }">{{ expired(profile) ? '需配对' : activeId === profile.id ? '使用中' : '已保存' }}</span></div>
            <div class="connection-actions"><button type="button" class="connection-enter" :disabled="busy || scanning || activeId === profile.id && !expired(profile)" @click="expired(profile) ? reauthorize(profile) : enter(profile)">{{ expired(profile) ? '重新配对' : activeId === profile.id ? '当前连接' : '进入空间' }}<RoomIcon v-if="activeId !== profile.id" name="arrow" /></button><button type="button" class="connection-more" :disabled="busy || scanning" :aria-expanded="expandedProfile === profile.id" :aria-label="`${profileName(profile)} 的连接选项`" @click="expandedProfile = expandedProfile === profile.id ? null : profile.id; removing = null"><RoomIcon name="more" /></button></div>
            <div v-if="expandedProfile === profile.id" class="connection-details">
              <dl><div><dt>名称</dt><dd>{{ profile.displayName || 'infOS' }}</dd></div><div><dt>地址</dt><dd>{{ profile.endpoint }}</dd></div><div v-if="profile.device"><dt>登录有效期</dt><dd>{{ expired(profile) ? '已过期' : expiry(profile.device.expiresAt) }}</dd></div></dl>
              <div class="connection-detail-actions"><button type="button" :disabled="busy || scanning" @click="reauthorize(profile)"><RoomIcon name="shield" />重新配对</button><button v-if="activeId !== profile.id" type="button" :disabled="busy || scanning" @click="removing = profile.id"><RoomIcon name="trash" />移除连接</button></div>
              <div v-if="removing === profile.id" class="connections-confirm"><p>从手机移除？电脑上的设备授权会保留。</p><div><button type="button" :disabled="busy" @click="removing = null">保留</button><button type="button" class="is-danger" :disabled="busy" @click="remove(profile.id)">移除</button></div></div>
            </div>
          </article>
        </div>
      </section>

      <form v-if="formOpen" ref="pairingForm" class="connections-form" @submit.prevent="save" @invalid.capture="revealInvalidField">
        <div class="connections-heading"><h3>{{ editingId ? '重新配对' : '添加 infOS' }}</h3><button type="button" class="connections-icon-button" :disabled="busy || scanning" aria-label="收起配对表单" @click="closeForm"><RoomIcon name="close" /></button></div>
        <div class="connections-pair-actions"><button type="button" class="connections-scan" :disabled="busy || scanning" @click="scanPairing"><RoomIcon name="scan" />扫码连接</button><button type="button" :disabled="busy || scanning" :aria-expanded="showLinkInput" @click="showLinkInput = !showLinkInput"><RoomIcon name="link" />粘贴链接</button></div>
        <div v-if="showLinkInput" class="connections-link"><label class="connections-field">配对链接<textarea v-model="pairingLink" rows="2" maxlength="4096" placeholder="粘贴 infOS 配对链接" :disabled="busy || scanning" /></label><button type="button" :disabled="busy || scanning || !pairingLink.trim()" @click="readPairingLink">填入配对信息<RoomIcon name="arrow" /></button></div>
        <RemoteFeedback :message="pairingNotice" tone="info" />
        <details class="connections-manual" :open="manualOpen" @toggle="manualOpen = $event.target.open">
          <summary>手动填写<RoomIcon name="down" /></summary>
          <fieldset :disabled="busy || scanning" class="connections-fields">
            <label class="connections-field">服务地址<input v-model="endpoint" type="url" placeholder="http://电脑 IP:9122" autocomplete="url" required :disabled="!!editingId" /></label>
            <label class="connections-field">{{ method === 'pair' ? '配对码' : method === 'login' ? '管理口令' : '访问令牌' }}<input ref="secretInput" v-model="secret" :type="method === 'pair' ? 'text' : 'password'" :autocomplete="method === 'pair' ? 'one-time-code' : 'off'" autocapitalize="off" spellcheck="false" required :placeholder="method === 'pair' ? '输入一次性配对码' : '输入 infOS 提供的凭据'" maxlength="512" /></label>
            <label v-if="method !== 'bearer'" class="connections-check"><input v-model="remember" type="checkbox" /><span>保持登录 <small>30 天</small></span></label>
            <p v-if="method !== 'bearer' && !remember" class="connections-hint">退出应用后需要重新配对。</p>
            <details class="connections-advanced"><summary>更多选项<RoomIcon name="down" /></summary><div class="connections-fields"><div class="connections-field"><span>登录方式</span><ChoicePicker v-model="method" label="登录方式" :options="connectionMethods" :disabled="busy || scanning" @change="secret = ''" /></div><label v-if="method !== 'bearer'" class="connections-field">设备名称<input v-model="deviceName" required maxlength="80" /></label><p class="connections-hint">地址请填写电脑 IP 或域名；配对码在电脑端「局域网与设备」中生成。</p></div></details>
            <label v-if="/^http:/i.test(endpoint.trim())" class="connections-check"><input v-model="allowInsecure" type="checkbox" /><span>允许未加密连接</span></label>
            <button class="connections-submit" type="submit" :disabled="busy || scanning || !endpoint.trim() || !secret.trim()">{{ busy ? progressLabel : editingId ? '重新配对并进入' : '连接并进入' }}<RoomIcon name="arrow" /></button>
            <button v-if="editingId" type="button" class="connections-text" :disabled="busy || scanning" @click="newConnection">添加其他 infOS</button>
          </fieldset>
        </details>
      </form>

      <details class="connections-settings" @toggle="settingsOpen = $event.target.open"><summary><RoomIcon name="bell" /><span>通知与连接选项</span><RoomIcon name="down" /></summary><div class="connections-settings-body"><p class="connections-online"><span>在线消息提醒</span><small>{{ activeId && remoteState.connection === 'online' ? '已连接' : '连接后接收' }}</small></p><RemoteBackground v-if="activeId && settingsOpen" :key="activeId" /><p v-if="!activeId" class="connections-hint">进入 infOS 后可设置后台接收。</p><template v-if="currentProfile?.device"><button v-if="!logoutConfirm" type="button" class="connections-text" :disabled="busy || scanning" @click="logoutConfirm = true">退出此连接的登录</button><div v-else class="connections-confirm"><p>退出后需要重新配对，对话记录会保留。</p><div><button type="button" :disabled="busy" @click="logoutConfirm = false">取消</button><button type="button" class="is-danger" :disabled="busy" @click="logout">退出登录</button></div></div></template></div></details>
    </div>
  </RoomDrawer>
  <Teleport to="body"><div v-if="scanning" class="remote-scanner-controls" role="status"><span>对准 infOS 配对二维码</span><button type="button" @click="cancelScan">取消扫描</button></div></Teleport>
</template>
<script setup>
import RemoteFeedback from './RemoteFeedback.vue'
import { computed, ref, onMounted, onBeforeUnmount, nextTick } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import ChoicePicker from '../companion/ChoicePicker.vue'
import RemoteBackground from './RemoteBackground.vue'
import RoomIcon from './RemoteIcon.vue'
import { remoteConnections, remoteClient } from '../../services/remote'
import { modeState, switchMode, refreshMode, currentToken } from '../../stores/mode'
import { remotePendingRequests, remoteState } from '../../stores/remote'
const props = defineProps({ prepareSwitch: Function })
const emit = defineEmits(['close'])
const profiles = ref([]), endpoint = ref(''), secret = ref(''), allowInsecure = ref(false), busy = ref(false), error = ref(''), removing = ref(null)
const method = ref('pair'), deviceName = ref('PeroperoChat'), remember = ref(true), editingId = ref(null)
const connectionMethods = [{ value: 'pair', label: '配对码' }, { value: 'login', label: '管理口令' }, { value: 'bearer', label: '访问令牌' }]
const formOpen = ref(false), manualOpen = ref(false), expandedProfile = ref(null), profilesLoaded = ref(false), settingsOpen = ref(false)
const connectionSection = ref(null), pairingForm = ref(null)
const logoutConfirm = ref(false)
const scanning = ref(false), showLinkInput = ref(false), pairingLink = ref(''), pairingNotice = ref('')
let scanner = null, scanVersion = 0
const secretInput = ref(null), profilesLoading = ref(false), listError = ref(''), savedName = ref(''), step = ref('preparing')
const activeId = computed(() => modeState.snapshot?.selectedTarget?.mode === 'remote' ? modeState.snapshot.selectedTarget.profileId : null)
const pendingCount = computed(() => remotePendingRequests.size)
const currentProfile = computed(() => profiles.value.find(profile => profile.id === activeId.value))
const progressLabel = computed(() => ({ preparing: '正在准备…', authorizing: '正在配对…', switching: '正在切换空间…', removing: '正在移除…', revoking: '正在退出登录…' }[step.value]))
let disposed = false, listVersion = 0
async function applyPairingLink(raw) {
  const version = scanVersion
  const link = await remoteConnections.decodePairing(raw)
  if (disposed || version !== scanVersion) return
  if (editingId.value && endpoint.value !== link.endpoint) throw new Error('二维码来自另一个服务地址，请先改为新建连接')
  endpoint.value = link.endpoint; secret.value = link.code; method.value = 'pair'; allowInsecure.value = false
  formOpen.value = true; manualOpen.value = true
  pairingLink.value = ''; showLinkInput.value = false
  pairingNotice.value = link.requiresInsecure ? '此地址未加密，请确认后勾选继续连接。' : '已填入，确认地址后就可以连接了。'
}
async function readPairingLink() {
  const version = ++scanVersion
  error.value = ''
  try { await applyPairingLink(pairingLink.value) } catch (failure) { if (!disposed && version === scanVersion) error.value = failure.message }
}
async function scanPairing() {
  if (busy.value || scanning.value) return
  const version = ++scanVersion
  scanning.value = true; error.value = ''; pairingNotice.value = ''
  try {
    scanner = await import('@tauri-apps/plugin-barcode-scanner')
    if (disposed || version !== scanVersion) return
    const permission = await scanner.requestPermissions()
    if (disposed || version !== scanVersion) return
    if (permission !== 'granted') throw new Error('相机权限未开放，可以手动输入配对码或粘贴配对链接')
    document.body.classList.add('ppc-pairing-scanning')
    const result = await scanner.scan({ windowed: true, formats: [scanner.Format.QRCode], cameraDirection: 'back' })
    if (!disposed && version === scanVersion) await applyPairingLink(result.content)
  } catch (failure) { if (!disposed && version === scanVersion) error.value = typeof failure?.message === 'string' ? failure.message : '扫描未完成，桌面端可使用粘贴配对链接或手动输入' }
  finally { if (version === scanVersion) { document.body.classList.remove('ppc-pairing-scanning'); if (!disposed) scanning.value = false } }
}
async function cancelScan() {
  const version = ++scanVersion
  document.body.classList.remove('ppc-pairing-scanning')
  try { if (scanner) await scanner.cancel() }
  catch { /* 扫描已经结束时无需再取消。 */ }
  finally { if (version === scanVersion) scanning.value = false }
}
async function loadProfiles() {
  const version = ++listVersion; profilesLoading.value = true; listError.value = ''
  try {
    const data = await remoteConnections.list()
    if (!disposed && version === listVersion) {
      profiles.value = data
      if (!profilesLoaded.value && !data.length) formOpen.value = true
      profilesLoaded.value = true
    }
  }
  catch (failure) { if (!disposed && version === listVersion) listError.value = failure.message }
  finally { if (!disposed && version === listVersion) profilesLoading.value = false }
}
async function run(action) {
  if (busy.value || scanning.value) return
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
async function reauthorize(profile) {
  scanVersion++
  editingId.value = profile.id; endpoint.value = profile.endpoint; secret.value = ''; method.value = 'pair'
  allowInsecure.value = /^http:/i.test(profile.endpoint); error.value = ''; pairingNotice.value = ''; pairingLink.value = ''; showLinkInput.value = false
  formOpen.value = true; manualOpen.value = true; expandedProfile.value = null; removing.value = null
  await nextTick()
  if (!disposed) secretInput.value?.focus()
}
async function newConnection() {
  scanVersion++
  editingId.value = null; endpoint.value = ''; secret.value = ''; allowInsecure.value = false; savedName.value = ''
  method.value = 'pair'; pairingLink.value = ''; pairingNotice.value = ''; showLinkInput.value = false
  formOpen.value = true; manualOpen.value = false; removing.value = null
  await nextTick()
  if (!disposed) pairingForm.value?.scrollIntoView({ block: 'nearest', behavior: 'auto' })
}
function closeForm() { scanVersion++; formOpen.value = false; manualOpen.value = false; showLinkInput.value = false; secret.value = ''; pairingLink.value = ''; pairingNotice.value = '' }
function showConnections() { if (!profiles.value.length) newConnection(); else connectionSection.value?.scrollIntoView({ block: 'nearest', behavior: 'auto' }) }
function revealInvalidField(event) {
  // 展开必填字段所在的折叠区，让系统校验能定位到实际输入框。
  for (let parent = event.target.parentElement; parent && parent !== pairingForm.value; parent = parent.parentElement) if (parent.tagName === 'DETAILS') parent.open = true
}
function profileName(profile) {
  const name = profile.displayName || 'infOS'
  return /^infos-node-[a-z\d-]{8,}$/i.test(name) ? `infOS · ${name.slice(-6)}` : name
}
function expired(profile) { return !!profile.device && profile.device.expiresAt <= Date.now() }
function expiry(value) { const date = new Date(value); return Number.isNaN(date.getTime()) ? '未提供' : date.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' }) }
async function logout() { await run(async () => { await prepare(); step.value = 'revoking'; await remoteClient.logoutDevice(currentToken()); logoutConfirm.value = false; if (currentProfile.value) await reauthorize(currentProfile.value) }) }
async function standalone() { await run(async () => { await prepare(); step.value = 'switching'; await switchMode({ mode: 'standalone' }); emit('close') }) }
async function remove(id) { await run(async () => { step.value = 'removing'; await remoteConnections.remove(id); await loadProfiles(); removing.value = null; if (editingId.value === id) newConnection() }) }
onMounted(loadProfiles)
onBeforeUnmount(() => { disposed = true; listVersion++; secret.value = ''; pairingLink.value = ''; cancelScan() })
</script>
<style>
body.ppc-pairing-scanning { background: transparent !important; }
body.ppc-pairing-scanning > * { visibility: hidden; }
body.ppc-pairing-scanning > .remote-scanner-controls { visibility: visible; }
</style>
<style scoped>
/* 连接面板使用独立布局类，避免旧页面横排样式挤压名称和操作。 */
.connections-panel{font-size:13px;line-height:1.65;color:var(--room-ink);min-width:0}
.connections-panel button,.connections-panel input,.connections-panel textarea{font:inherit}
.connections-panel button{cursor:pointer;touch-action:manipulation;-webkit-tap-highlight-color:transparent}
.connections-panel button:disabled{cursor:default;opacity:.5}
.connections-panel button:focus-visible,.connections-panel summary:focus-visible{outline:2px solid var(--room-accent);outline-offset:3px}
.connections-spaces{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:9px;margin:2px 0 21px}
.connections-spaces>button{display:grid;grid-template-columns:29px minmax(0,1fr) 12px;align-items:center;gap:7px;min-width:0;min-height:76px;padding:12px 10px;border:1px solid var(--room-line);border-radius:7px 16px 7px 7px;background:var(--room-surface);color:var(--room-ink);text-align:left}
.connections-spaces>button.selected{border-color:color-mix(in srgb,var(--room-accent) 42%,var(--room-surface));background:color-mix(in srgb,var(--room-accent) 8%,var(--room-surface));box-shadow:inset 0 1px 0 color-mix(in srgb,var(--room-surface) 70%,transparent);opacity:1}
.connections-space-icon{display:grid;place-items:center;width:29px;height:31px;color:var(--room-accent)}.connections-space-icon svg{width:24px;height:24px}.connections-spaces>button>span:nth-child(2){min-width:0;display:grid;gap:3px}.connections-spaces strong{font-size:13px;font-weight:650;white-space:nowrap}.connections-spaces small{font-size:10px;color:var(--room-muted);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.connections-spaces>button>svg{width:12px;height:12px;color:var(--room-accent)}
.connections-heading{display:flex;align-items:center;justify-content:space-between;gap:10px;margin:0 0 9px}.connections-heading h3{margin:0;font-size:13px;font-weight:650;line-height:1.5}.connections-heading>div{display:flex;align-items:center;gap:6px}
.connections-icon-button,.connections-add{display:inline-flex;justify-content:center;align-items:center;flex:none;gap:4px;min-width:36px;min-height:36px;padding:6px;border:0;border-radius:9px;background:transparent;color:var(--room-accent)}.connections-icon-button svg{width:16px;height:16px}.connections-add{padding-inline:9px;background:var(--room-accent-soft);font-size:11px!important}.connections-add svg{width:13px;height:13px}
.connections-empty{margin:7px 0 13px;color:var(--room-muted);font-size:11px;line-height:1.7}
.connections-list{display:grid;gap:10px;min-width:0}.connection-card{min-width:0;border:1px solid var(--room-line);border-radius:7px 15px 12px 12px;background:var(--room-surface);overflow:hidden}.connection-card.is-current{border-color:color-mix(in srgb,var(--room-accent) 36%,var(--room-surface))}
.connection-identity{display:grid;grid-template-columns:30px minmax(0,1fr) auto;align-items:center;gap:9px;min-width:0;padding:13px 12px}.connection-glyph{display:grid;place-items:center;width:30px;height:34px;color:var(--room-accent);background:var(--room-accent-soft);border-radius:5px 10px 5px 5px}.connection-glyph>svg{width:19px;height:19px}.connection-identity>div{min-width:0;display:grid;gap:4px}.connection-identity strong{display:block;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:13px;font-weight:650;line-height:1.55}.connection-identity small{display:block;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:10px;line-height:1.5;color:var(--room-muted)}
.connection-state{font-size:10px;white-space:nowrap;color:var(--room-muted);padding:2px 6px;border-radius:5px;background:var(--room-subtle)}.is-current .connection-state{color:var(--room-accent);background:var(--room-accent-soft)}.connection-state.needs-pairing{color:var(--room-danger,#a35f78);background:color-mix(in srgb,var(--room-danger,#a35f78) 7%,var(--room-surface))}
.connection-actions{display:flex;align-items:center;justify-content:space-between;gap:10px;padding:7px 10px;border-top:1px solid var(--room-line);background:var(--room-subtle)}.connection-enter,.connection-more{display:inline-flex;align-items:center;justify-content:center;gap:8px;min-height:34px;padding:5px 10px;border:0;border-radius:6px 10px 6px 6px;color:var(--room-accent);background:var(--room-accent-soft);font-size:11px!important}.connection-enter>svg{width:13px;height:13px}.connection-enter:disabled{background:none;opacity:1;color:var(--room-muted)}.connection-more{width:36px;min-width:36px;padding:8px;background:var(--room-surface);color:var(--room-muted)}.connection-more>svg{width:17px;height:17px}.connection-more[aria-expanded=true]{color:var(--room-accent)}
.connection-details{padding:12px;border-top:1px solid var(--room-line)}.connection-details dl{display:grid;gap:8px;margin:0;font-size:11px}.connection-details dl>div{display:grid;grid-template-columns:65px minmax(0,1fr);gap:10px}.connection-details dt{color:var(--room-muted)}.connection-details dd{margin:0;overflow-wrap:anywhere}.connection-detail-actions{display:flex;flex-wrap:wrap;gap:9px;margin-top:10px}.connection-detail-actions button{display:inline-flex;align-items:center;gap:5px;min-height:36px;padding:5px 8px;border:1px solid var(--room-line);border-radius:7px;background:var(--room-subtle);color:var(--room-ink);font-size:11px}.connection-detail-actions svg{width:14px;height:14px}
.connections-form{margin-top:19px;padding-top:17px;border-top:1px solid var(--room-line)}.connections-pair-actions{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:9px}.connections-pair-actions>button{display:flex;align-items:center;justify-content:center;gap:7px;min-width:0;min-height:47px;border:1px solid var(--room-line);border-radius:7px 12px 7px 7px;background:var(--room-surface);color:var(--room-ink);font-size:12px}.connections-pair-actions>button.connections-scan{color:var(--room-accent);background:var(--room-accent-soft);border-color:color-mix(in srgb,var(--room-accent) 30%,var(--room-surface))}.connections-pair-actions svg{width:19px;height:19px;flex:none}
.connections-link{padding:12px 0 0}.connections-link>button{display:flex;align-items:center;gap:6px;margin:5px 0 0 auto;min-height:36px;padding:4px 7px;border:0;background:none;color:var(--room-accent);font-size:11px}.connections-link>button svg{width:13px;height:13px}
.connections-panel summary{display:flex;align-items:center;gap:8px;min-height:43px;list-style:none;cursor:pointer;font-size:11px;color:var(--room-muted)}.connections-panel summary::-webkit-details-marker{display:none}.connections-panel summary>svg:last-child{width:13px;height:13px;flex:none;margin-left:auto;transition:transform 160ms}.connections-panel details[open]>summary>svg:last-child{transform:rotate(180deg)}
.connections-manual{margin-top:6px}.connections-fields{display:grid;gap:13px;min-width:0;margin:0;padding:5px 0 0;border:0}.connections-field{display:grid;gap:6px;min-width:0;font-size:11px;font-weight:500;color:var(--room-ink)}.connections-panel .connections-field :is(input,textarea){display:block;min-width:0;width:100%;min-height:42px;margin:0;padding:10px 11px;border:1px solid var(--room-line);border-radius:7px 11px 7px 7px;background:var(--room-subtle);color:var(--room-ink);font-size:13px;line-height:1.6}.connections-field textarea{resize:vertical}.connections-field input:disabled{opacity:.65}.connections-field input::placeholder,.connections-field textarea::placeholder{color:var(--room-muted);font-size:12px}.connections-field input:focus-visible,.connections-field textarea:focus-visible{outline:2px solid color-mix(in srgb,var(--room-accent) 45%,transparent);outline-offset:2px}
.connections-check{display:flex;align-items:flex-start;gap:8px;margin:0;min-width:0;font-size:11px;line-height:1.8;color:var(--room-ink)}.connections-check>span{min-width:0;overflow-wrap:anywhere}.connections-check small{color:var(--room-muted);font-size:10px}.connections-check input{appearance:none;-webkit-appearance:none;display:grid;place-items:center;flex:none;width:16px;height:16px;min-height:0;margin:2px 0 0;padding:0;border:1px solid var(--room-accent);border-radius:4px;background:var(--room-surface)}.connections-check input:checked{background:var(--room-accent)}.connections-check input:checked::after{content:'';width:4px;height:7px;border:solid var(--room-surface);border-width:0 1.5px 1.5px 0;transform:translateY(-1px) rotate(45deg)}
.connections-advanced{border-top:1px dashed var(--room-line);border-bottom:1px dashed var(--room-line);padding-bottom:3px}.connections-advanced[open]{padding-bottom:12px}.connections-advanced .connections-fields{padding-top:0}.connections-panel .connections-hint{margin:0;font-size:10px;font-weight:400;color:var(--room-muted);line-height:1.75}
.connections-submit{display:flex;align-items:center;justify-content:center;gap:10px;min-height:43px;width:100%;padding:10px 13px;border:1px solid var(--room-accent);border-radius:6px 12px 6px 6px;background:var(--room-accent);color:var(--room-surface);font-size:12px!important;font-weight:600!important}.connections-submit>svg{width:15px;height:15px}.connections-text{justify-self:start;min-height:36px;padding:5px 0;border:0;background:none;color:var(--room-accent);font-size:11px!important}
.connections-settings{margin-top:20px;border-top:1px solid var(--room-line)}.connections-settings>summary>svg:first-child{width:15px;height:15px;color:var(--room-accent)}.connections-settings-body{padding:3px 0 0}.connections-online{display:flex;justify-content:space-between;gap:12px;margin:0 0 10px;font-size:11px}.connections-online>small{font-size:10px;color:var(--room-muted)}
.connections-confirm{margin-top:10px;border:1px solid var(--room-line);border-radius:9px;padding:10px;background:var(--room-subtle)}.connections-confirm>p{font-size:11px;line-height:1.7;margin:0 0 8px}.connections-confirm>div{display:flex;justify-content:flex-end;gap:8px}.connections-confirm button{min-height:34px;padding:5px 10px;border:1px solid var(--room-line);border-radius:7px;background:var(--room-surface);color:var(--room-ink);font-size:11px}.connections-confirm .is-danger{color:var(--room-danger,#a35f78)}
.connections-pending,.connections-progress{display:flex;align-items:center;gap:7px;margin:0 0 12px;font-size:11px;color:var(--room-muted);line-height:1.7}.connections-pending>svg,.connections-progress>svg{width:15px;height:15px;flex:none;color:var(--room-accent)}
.remote-scanner-controls{position:fixed;bottom:max(24px,env(safe-area-inset-bottom));left:16px;right:16px;z-index:99999;display:flex;align-items:center;justify-content:space-between;gap:12px;padding:14px;border-radius:16px;background:#1b243beb;color:white;font-size:12px}.remote-scanner-controls button{flex:none;background:white;color:#33466b;border:0;padding:10px 15px;border-radius:12px}
@media(prefers-reduced-motion:reduce){.connections-panel summary>svg{transition:none}}
:global([data-reduce-motion=true]) .connections-panel summary>svg{transition:none}
</style>
