<template>
  <RoomDrawer :title="tr(formOpen ? editingId ? '重新配对电脑' : '连接电脑上的 infOS' : '连接与模式')" variant="page" :busy="locked" :before-close="beforeClose" @close="$emit('close')">
    <div class="connections-panel">
      <template v-if="!formOpen">
        <header class="connections-intro">
          <span class="connections-current"><i />{{ tr(activeId ? '当前：infOS 客户端' : '当前：本机独立运行') }}</span>
          <h2>{{ tr(activeId ? '你的电脑，随时接着聊' : '把电脑上的 infOS 接到这里') }}</h2>
          <p>{{ tr('infOS 是运行在电脑上的应用。连接后，这里就能使用那台电脑上的角色、对话、任务和文件。') }}</p>
        </header>
        <div class="connections-spaces" :aria-label="tr('两种模式的区别')">
          <article :class="{ selected: !activeId }"><span class="connections-space-icon"><RoomIcon name="home" /></span><strong>{{ tr('独立空间') }}</strong><small>{{ tr('本机的角色与聊天记录') }}</small><p>{{ tr('不用连接电脑，也可以使用本机已配置的陪伴功能。') }}</p><span v-if="!activeId" class="connections-current-label"><RoomIcon name="check" />{{ tr('正在使用') }}</span><button v-else type="button" :disabled="locked" @click="standalone">{{ tr('返回独立空间') }}<RoomIcon name="arrow" /></button></article>
          <article :class="{ selected: activeId }"><span class="connections-space-icon"><RoomIcon name="desktop" /></span><strong>{{ tr('infOS 客户端') }}</strong><small>{{ tr('电脑上的角色与聊天记录') }}</small><p>{{ tr('连接电脑上的 infOS；聊天与任务由电脑处理。') }}</p><span v-if="activeId" class="connections-current-label"><RoomIcon name="check" />{{ tr('正在使用') }}</span><span v-else class="connections-mode-note">{{ tr('需要电脑已安装并打开 infOS') }}</span></article>
        </div>
        <p class="connections-data-note"><RoomIcon name="folder" /><span>{{ tr('两种模式的记录分开保存。切换不会搬移或合并对话，也不会删除本机记录。') }}</span></p>
        <p v-if="pendingCount" class="connections-pending"><RoomIcon name="clock" />{{ tr('还有 {0} 项操作待核对，切换后会保留。', [pendingCount]) }}</p>
      </template>

      <section v-if="error" ref="errorPanel" class="connections-recovery" role="alert" tabindex="-1">
        <strong><RoomIcon name="alert" />{{ tr('这一步还没完成') }}</strong><p>{{ error }}</p><p v-if="recoveryHint" class="connections-recovery-hint">{{ recoveryHint }}</p>
        <button v-if="failureKind === 'scan' && formOpen" type="button" class="connections-text" :disabled="locked" @click="usePairingLink">{{ tr('改用配对链接') }}<RoomIcon name="arrow" /></button>
        <button v-if="formOpen && (failureKind === 'credentials' || pairingStage === 2 && !secret && method === 'pair')" type="button" class="connections-text" :disabled="locked" @click="goToStage(1)">{{ tr('重新获取配对信息') }}<RoomIcon name="arrow" /></button>
        <button v-if="failureKind === 'local-draft'" type="button" class="connections-text" :disabled="locked" @click="returnToDraft">{{ tr('返回独立空间') }}<RoomIcon name="arrow" /></button>
      </section>
      <RemoteFeedback :message="certificateNotice" tone="success" />
      <div v-if="locked && !scanning" class="connections-progress" role="status" aria-live="polite"><RoomIcon class="room-spinning" name="refresh" /><span><strong>{{ progressLabel }}</strong><small>{{ progressDetail }}</small></span></div>
      <p v-if="savedName && !busy" class="connections-saved-notice" role="status"><RoomIcon name="check" />{{ tr('配对已经保存。点下面的「进入客户端」继续，不用重新扫码。') }}</p>

      <section v-if="!formOpen" ref="connectionSection" class="connections-saved" :aria-label="tr('已配对的电脑')">
        <div v-if="profiles.length || profilesLoading || listError" class="connections-heading"><h3>{{ tr('已配对的电脑') }}</h3><button type="button" class="connections-icon-button" :disabled="locked || profilesLoading" :aria-label="tr('刷新连接')" @click="loadProfiles"><RoomIcon :class="{ 'room-spinning': profilesLoading }" name="refresh" /></button></div>
        <RemoteFeedback :message="listError" />
        <button v-if="listError" type="button" class="connections-text" :disabled="locked || profilesLoading" @click="loadProfiles">{{ tr('重新读取已保存的连接') }}</button>
        <p v-if="profilesLoading && !profiles.length" class="connections-empty">{{ tr('正在读取已配对的电脑…') }}</p>
        <div class="connections-list">
          <article v-for="profile in profiles" :key="profile.id" class="connection-card" :class="{ 'is-current': activeId === profile.id }">
            <div class="connection-identity"><span class="connection-glyph"><RoomIcon name="desktop" /></span><div><strong :title="profile.displayName">{{ profileName(profile) }}</strong><small :title="profile.endpoint">{{ profile.endpoint }}</small></div><span class="connection-state" :class="{ 'needs-pairing': needsPairing(profile) }">{{ profileState(profile) }}</span></div>
            <p v-if="needsPairing(profile)" class="connection-pairing-help">{{ tr('需要新的配对码。原来的电脑端记录仍保留。') }}</p>
            <div class="connection-actions"><button type="button" class="connection-enter" :disabled="locked" @click="needsPairing(profile) ? reauthorize(profile) : activeId === profile.id ? $emit('close') : enter(profile)">{{ tr(needsPairing(profile) ? '重新配对' : activeId === profile.id ? '返回客户端' : '进入客户端') }}<RoomIcon name="arrow" /></button><button type="button" class="connection-more" :disabled="locked" :aria-expanded="expandedProfile === profile.id" :aria-label="tr('{0}的连接选项', [profileName(profile)])" @click="expandedProfile = expandedProfile === profile.id ? null : profile.id; removing = null"><RoomIcon name="more" /></button></div>
            <div v-if="expandedProfile === profile.id" class="connection-details">
              <dl><div><dt>{{ tr('名称') }}</dt><dd>{{ profile.displayName || 'infOS' }}</dd></div><div><dt>{{ tr('电脑地址') }}</dt><dd>{{ profile.endpoint }}</dd></div><div v-if="profile.device"><dt>{{ tr('登录有效期') }}</dt><dd>{{ expired(profile) ? tr('已过期') : expiry(profile.device.expiresAt) }}</dd></div></dl>
              <div class="connection-detail-actions"><button v-if="profile.endpoint.startsWith('https:')" type="button" :disabled="locked" @click="checkCertificate(profile)"><RoomIcon name="shield" />{{ tr('核对证书') }}</button><button type="button" :disabled="locked" @click="reauthorize(profile)"><RoomIcon name="scan" />{{ tr('重新配对') }}</button><button v-if="activeId !== profile.id" type="button" :disabled="locked" @click="removing = profile.id"><RoomIcon name="trash" />{{ tr('移除连接') }}</button></div>
              <div v-if="removing === profile.id" class="connections-confirm"><p>{{ tr('只从本机移除这个连接，电脑上的记录和设备授权会保留。') }}</p><div><button type="button" :disabled="locked" @click="removing = null">{{ tr('保留') }}</button><button type="button" class="is-danger" :disabled="locked" @click="remove(profile.id)">{{ tr('移除连接') }}</button></div></div>
            </div>
          </article>
        </div>
        <button type="button" class="connections-submit connections-start" :disabled="locked || profilesLoading" @click="newConnection"><RoomIcon :name="profiles.length ? 'plus' : 'connection'" />{{ tr(profiles.length ? '连接另一台电脑' : '开始连接电脑') }}<RoomIcon name="arrow" /></button>
        <p v-if="!profiles.length && !profilesLoading" class="connections-start-note">{{ tr('先看看步骤也可以，配对成功后才会切换模式。') }}</p>
        <details v-if="!activeId" class="connections-help"><summary><RoomIcon name="help" />{{ tr('还没有安装 infOS？') }}<RoomIcon name="down" /></summary><p>{{ tr('需要先在电脑上安装并打开 infOS，再来配对。现在也可以继续使用独立空间，稍后再连接。') }}</p><div class="connections-help-actions"><button type="button" class="connections-text" :disabled="locked" @click="openInstallGuide">{{ tr('查看 infOS 安装说明') }}<RoomIcon name="open" /></button><button type="button" class="connections-text" :disabled="locked" @click="$emit('close')">{{ tr('先使用独立空间') }}<RoomIcon name="arrow" /></button></div><p>{{ tr('安装说明将在浏览器中打开。') }}</p></details>
      </section>

      <section v-else class="connections-wizard" :aria-label="tr('连接引导')">
        <ol class="connections-steps"><li v-for="(label, index) in stageLabels" :key="label" :data-state="pairingStage > index ? 'done' : pairingStage === index ? 'current' : 'next'" :aria-current="pairingStage === index ? 'step' : undefined"><span><RoomIcon v-if="pairingStage > index" name="check" /><template v-else>{{ index + 1 }}</template></span>{{ tr(label) }}</li></ol>
        <div class="connections-step-heading"><button v-if="pairingStage > 0" type="button" class="connections-icon-button" :aria-label="tr('上一步')" :disabled="locked" @click="goToStage(pairingStage - 1)"><RoomIcon name="back" /></button><h3 ref="stageHeading" tabindex="-1">{{ tr(stageTitles[pairingStage]) }}</h3></div>
        <Transition name="connection-step" mode="out-in">
          <div v-if="pairingStage === 0" key="prepare" class="connections-prepare">
            <div class="connections-device-scene" aria-hidden="true"><RoomIcon name="desktop" /><span><i /><i /><i /></span><RoomIcon name="connection" /></div>
            <ol class="connections-preparation"><li><b>1</b><span><strong>{{ tr('在电脑上打开 infOS') }}</strong><small>{{ tr('连接期间，请让电脑和 infOS 保持运行。') }}</small></span></li><li><b>2</b><span><strong>{{ tr('打开「局域网与设备」') }}</strong><small>{{ tr('在电脑端生成配对二维码，或复制配对链接。') }}</small></span></li><li><b>3</b><span><strong>{{ tr('确认手机能连到这台电脑') }}</strong><small>{{ tr('第一次连接，建议让手机和电脑连同一个 Wi-Fi。') }}</small></span></li></ol>
            <button type="button" class="connections-submit" :disabled="locked" @click="goToStage(1)">{{ tr('电脑准备好了，下一步') }}<RoomIcon name="arrow" /></button>
          </div>
          <div v-else-if="pairingStage === 1" key="pair" class="connections-pairing">
            <p class="connections-step-note">{{ tr('从电脑上的「局域网与设备」获取配对信息，任选一种方式。') }}</p>
            <div class="connections-methods"><button type="button" class="connections-method-primary" :disabled="locked" @click="scanPairing"><RoomIcon name="scan" /><span><strong>{{ tr('扫描电脑上的二维码') }}</strong><small>{{ tr('在手机上使用最方便') }}</small></span><RoomIcon name="next" /></button><button type="button" :disabled="locked" :aria-expanded="showLinkInput" @click="usePairingLink"><RoomIcon name="link" /><span><strong>{{ tr('粘贴配对链接') }}</strong><small>{{ tr('电脑和手机都可以用') }}</small></span><RoomIcon name="next" /></button></div>
            <div v-if="showLinkInput" class="connections-link"><label class="connections-field">{{ tr('从电脑端复制的配对链接') }}<textarea ref="linkInput" v-model="pairingLink" rows="3" maxlength="4096" :placeholder="tr('在这里粘贴完整的配对链接')" :disabled="locked" autocomplete="off" autocapitalize="off" spellcheck="false" /></label><button type="button" :disabled="locked || !pairingLink.trim()" @click="readPairingLink">{{ tr('识别链接，下一步') }}<RoomIcon name="arrow" /></button></div>
            <button type="button" class="connections-manual-entry" :disabled="locked" @click="startManual"><RoomIcon name="edit" />{{ tr('手动填写地址与配对码') }}<RoomIcon name="next" /></button>
          </div>
          <form v-else key="review" ref="pairingForm" class="connections-review" @submit.prevent="save" @invalid.capture="revealInvalidField">
            <p class="connections-step-note">{{ tr('确认这是你要连接的电脑。点击下方按钮后，才会切换到 infOS 客户端。') }}</p>
            <div v-if="inputOrigin !== 'manual' && secret" class="connections-pair-target"><RoomIcon name="desktop" /><span><small>{{ tr('即将连接的电脑地址') }}</small><strong>{{ endpoint }}</strong><em><RoomIcon name="check" />{{ tr('已读取配对信息') }}</em></span></div>
            <details class="connections-manual" :open="manualOpen || !secret" @toggle="manualOpen = detailsOpen($event)">
              <summary>{{ tr(inputOrigin === 'manual' ? '填写电脑端提供的信息' : '查看或修改配对信息') }}<RoomIcon name="down" /></summary>
              <fieldset :disabled="locked" class="connections-fields">
                <label class="connections-field">{{ tr('电脑地址') }}<input v-model="endpoint" type="url" placeholder="http://192.168.1.10:9122" autocomplete="url" required :disabled="!!editingId" @input="allowInsecure = false" /><small class="connections-hint">{{ tr('填写电脑端显示的访问地址，不是手机自己的地址。') }}</small></label>
                <label class="connections-field">{{ tr(method === 'pair' ? '一次性配对码' : method === 'login' ? '管理口令' : '访问令牌') }}<input ref="secretInput" v-model="secret" :type="method === 'pair' ? 'text' : 'password'" :autocomplete="method === 'pair' ? 'one-time-code' : 'off'" autocapitalize="off" spellcheck="false" required :placeholder="tr(method === 'pair' ? '输入电脑端生成的配对码' : '输入 infOS 提供的凭据')" maxlength="512" /></label>
                <p v-if="method === 'pair'" class="connections-hint">{{ tr('配对码过期或用过了？在电脑端重新生成即可。') }}</p>
                <details class="connections-advanced"><summary>{{ tr('高级连接选项') }}<RoomIcon name="down" /></summary><div class="connections-fields"><div class="connections-field"><span>{{ tr('登录方式') }}</span><ChoicePicker v-model="method" :label="tr('登录方式')" :options="connectionMethods" :disabled="locked" @change="secret = ''; inputOrigin = 'manual'" /></div><label v-if="method !== 'bearer'" class="connections-field">{{ tr('本机设备名称') }}<input v-model="deviceName" required maxlength="80" /><small class="connections-hint">{{ tr('电脑端会用这个名称识别当前设备。') }}</small></label></div></details>
              </fieldset>
            </details>
            <fieldset :disabled="locked" class="connections-fields connections-confirm-fields">
              <label v-if="method !== 'bearer'" class="connections-check"><input v-model="remember" type="checkbox" /><span>{{ tr('记住这台设备') }}<small>{{ tr(remember ? '登录有效期 30 天，可在电脑端撤销授权。' : '退出应用后，需要重新配对。') }}</small></span></label>
              <div v-if="insecureEndpoint" class="connections-network-consent"><strong><RoomIcon name="shield" />{{ tr('这条连接没有加密') }}</strong><p>{{ tr('这个地址使用 HTTP。请只在你信任的网络中，连接自己的电脑。') }}</p><label class="connections-check"><input v-model="allowInsecure" type="checkbox" /><span>{{ tr('我信任这台电脑和当前网络，允许连接') }}</span></label></div>
              <p class="connections-hint">{{ tr('进入后会显示电脑上的内容。本机独立空间的对话不会自动同步过去。') }}</p>
              <button class="connections-submit" type="submit" :disabled="locked || !endpoint.trim() || !secret.trim() || insecureEndpoint && !allowInsecure">{{ locked ? progressLabel : tr('连接并进入 infOS') }}<RoomIcon name="arrow" /></button>
            </fieldset>
          </form>
        </Transition>
        <button type="button" class="connections-later" :disabled="locked" @click="closeForm">{{ tr('暂时不连接，返回模式页') }}</button>
      </section>

      <details v-if="!formOpen && activeId" class="connections-settings" @toggle="settingsOpen = detailsOpen($event)"><summary><RoomIcon name="bell" /><span>{{ tr('通知与连接选项') }}</span><RoomIcon name="down" /></summary><div class="connections-settings-body"><p class="connections-online"><span>{{ tr('在线消息提醒') }}</span><small>{{ tr(remoteState.connection === 'online' ? '已连接' : '连接后接收') }}</small></p><RemoteBackground v-if="settingsOpen" :key="activeId" /><template v-if="currentProfile?.device"><button v-if="!logoutConfirm" type="button" class="connections-text" :disabled="locked" @click="logoutConfirm = true">{{ tr('退出此连接的登录') }}</button><div v-else class="connections-confirm"><p>{{ tr('退出后需要重新配对，电脑上的对话记录会保留。') }}</p><div><button type="button" :disabled="locked" @click="logoutConfirm = false">{{ tr('取消') }}</button><button type="button" class="is-danger" :disabled="locked" @click="logout">{{ tr('退出登录') }}</button></div></div></template></div></details>
      <details v-if="!formOpen || pairingStage === 0" class="connections-help"><summary><RoomIcon name="help" />{{ tr('连不上时，先检查这些') }}<RoomIcon name="down" /></summary><ul><li>{{ tr('电脑上的 infOS 是否仍在运行，电脑是否进入了休眠？') }}</li><li>{{ tr('手机与电脑是否在同一个网络？访客 Wi-Fi 可能不允许设备互相连接。') }}</li><li>{{ tr('地址是否和电脑端一致？不要填写 localhost 或 127.0.0.1。') }}</li><li>{{ tr('配对码是否过期？可以在电脑端生成新的配对码。') }}</li></ul></details>
    </div>
  </RoomDrawer>
  <RemoteCertificateConfirmation v-if="certificateInspection" :inspection="certificateInspection" @confirm="finishCertificate(true)" @cancel="finishCertificate(false)" />
  <Teleport to="body"><div v-if="scanning" class="remote-scanner-controls" role="status"><span>{{ tr('对准电脑上 infOS 的配对二维码') }}</span><button type="button" @click="cancelScan">{{ tr('取消扫描') }}</button></div></Teleport>
</template>
<script setup>
import { detailsOpen } from '../../utils/domEvents'
import RemoteFeedback from './RemoteFeedback.vue'
import RemoteCertificateConfirmation from './RemoteCertificateConfirmation.vue'
import { t } from '../../i18n/remote'
import { computed, ref, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { open as openExternal } from '@tauri-apps/plugin-shell'
import RoomDrawer from '../companion/RoomDrawer.vue'
import ChoicePicker from '../companion/ChoicePicker.vue'
import RemoteBackground from './RemoteBackground.vue'
import RoomIcon from './RemoteIcon.vue'
import { remoteConnections, remoteClient } from '../../services/remote'
import { modeState, switchMode, refreshMode, currentToken } from '../../stores/mode'
import { remotePendingRequests, remoteState } from '../../stores/remote'
const tr = (source, values = []) => t(source, values, 'connections')
const certificateInspection = ref(null), certificateNotice = ref('')
let certificateResolve = null
function finishCertificate(accepted) { certificateInspection.value = null; certificateResolve?.(accepted); certificateResolve = null }
function confirmCertificate(inspection) { certificateInspection.value = inspection; return new Promise(resolve => { certificateResolve = resolve }) }
const props = defineProps({ prepareSwitch: Function })
const emit = defineEmits(['close'])
const profiles = ref([]), endpoint = ref(''), secret = ref(''), allowInsecure = ref(false), busy = ref(false), error = ref(''), removing = ref(null)
const method = ref('pair'), deviceName = ref('PeroperoChat'), remember = ref(true), editingId = ref(null)
const connectionMethods = computed(() => [{ value: 'pair', label: tr('一次性配对码') }, { value: 'login', label: tr('管理口令') }, { value: 'bearer', label: tr('访问令牌') }])
const formOpen = ref(false), manualOpen = ref(false), expandedProfile = ref(null), settingsOpen = ref(false)
const connectionSection = ref(null), pairingForm = ref(null), stageHeading = ref(null), linkInput = ref(null), errorPanel = ref(null)
const pairingStage = ref(0), inputOrigin = ref('manual'), decoding = ref(false), failureKind = ref('')
const stageLabels = ['电脑准备', '获取配对信息', '核对并连接']
const stageTitles = ['先在电脑上准备好', '把配对信息带到这里', '确认后，进入电脑上的 infOS']
const logoutConfirm = ref(false)
const scanning = ref(false), showLinkInput = ref(false), pairingLink = ref('')
let scanner = null, scanVersion = 0
const secretInput = ref(null), profilesLoading = ref(false), listError = ref(''), savedName = ref(''), step = ref('preparing')
const activeId = computed(() => modeState.snapshot?.selectedTarget?.mode === 'remote' ? modeState.snapshot.selectedTarget.profileId : null)
const pendingCount = computed(() => remotePendingRequests.size)
const currentProfile = computed(() => profiles.value.find(profile => profile.id === activeId.value))
const locked = computed(() => busy.value || scanning.value || decoding.value)
const insecureEndpoint = computed(() => /^http:/i.test(endpoint.value.trim()))
const progressLabel = computed(() => tr(decoding.value ? '正在识别配对链接…' : ({ preparing: '正在检查当前状态…', authorizing: '正在与电脑配对…', switching: activeId.value ? '正在切换模式…' : '正在打开 infOS 客户端…', removing: '正在移除连接…', revoking: '正在退出登录…', certificate: '正在核对连接…' }[step.value] || '正在处理…')))
const progressDetail = computed(() => tr(decoding.value ? '只读取配对信息，下一步由你确认连接。' : ['authorizing', 'switching'].includes(step.value) ? '完成后会自动打开对应模式。' : '请稍等，当前操作完成后就可以继续。'))
const recoveryHint = computed(() => tr(({ network: '请确认电脑上的 infOS 正在运行，再检查手机和电脑的网络与地址。', credentials: '请在电脑端生成新的配对信息，再回来重试。', scan: '也可以在电脑端复制配对链接，粘贴到这里，不需要相机。', version: '请更新电脑端 infOS 和本机 PeroperoChat 后再试。', 'local-draft': '本机内容还在。先处理当前草稿或回复，再回来连接电脑。' })[failureKind.value] || ''))
let disposed = false, listVersion = 0
async function focusStage() { await nextTick(); if (!disposed && formOpen.value && !scanning.value) { stageHeading.value?.scrollIntoView({ block: 'nearest', behavior: 'auto' }); stageHeading.value?.focus({ preventScroll: true }) } }
function goToStage(value) { if (locked.value) return; pairingStage.value = value; error.value = ''; failureKind.value = ''; focusStage() }
function beforeClose() { if (!formOpen.value) return true; if (pairingStage.value > 0) goToStage(pairingStage.value - 1); else closeForm(); return false }
function startManual() { inputOrigin.value = 'manual'; manualOpen.value = true; goToStage(2) }
async function usePairingLink() { if (locked.value) return; goToStage(1); showLinkInput.value = true; await nextTick(); linkInput.value?.focus() }
function reportFailure(failure, fallback = '') {
  if (disposed) return
  const code = failure?.code
  failureKind.value = ['LOCAL_DRAFT_PENDING', 'LOCAL_REPLY_RUNNING'].includes(code) ? 'local-draft' : ['REMOTE_NETWORK', 'REMOTE_HTTP'].includes(code) ? 'network' : ['REMOTE_AUTH_REQUIRED', 'CREDENTIAL_UNAVAILABLE', 'REMOTE_PAIRING_REJECTED'].includes(code) ? 'credentials' : ['REMOTE_INCOMPATIBLE', 'REMOTE_PROTOCOL'].includes(code) ? 'version' : fallback
  error.value = tr(failure?.message || '这一步还没完成，请再试一次。')
  nextTick(() => { if (!disposed) { errorPanel.value?.scrollIntoView({ block: 'nearest', behavior: 'auto' }); errorPanel.value?.focus({ preventScroll: true }) } })
}
function returnToDraft() { if (!locked.value) emit('close') }
async function openInstallGuide() {
  try { await openExternal('https://github.com/YoKONCy/infOS') }
  catch { reportFailure({ message: '无法打开浏览器。请在电脑浏览器中访问 github.com/YoKONCy/infOS，查看安装说明。' }) }
}
async function applyPairingLink(raw) {
  const version = scanVersion
  const link = await remoteConnections.decodePairing(raw.trim())
  if (disposed || version !== scanVersion) return
  if (editingId.value && endpoint.value !== link.endpoint) throw Object.assign(new Error('这是另一台电脑的配对信息。请返回模式页，选择「连接另一台电脑」。'), { code: 'PAIRING_TARGET_MISMATCH' })
  endpoint.value = link.endpoint; secret.value = link.code; method.value = 'pair'; allowInsecure.value = false
  formOpen.value = true; manualOpen.value = false; inputOrigin.value = 'link'; pairingStage.value = 2
  pairingLink.value = ''; showLinkInput.value = false
  focusStage()
}
async function readPairingLink() {
  if (locked.value || !pairingLink.value.trim()) return
  const version = ++scanVersion
  decoding.value = true; error.value = ''; failureKind.value = ''
  try { await applyPairingLink(pairingLink.value) } catch (failure) { if (!disposed && version === scanVersion) reportFailure(failure) }
  finally { if (!disposed && version === scanVersion) decoding.value = false }
}
async function scanPairing() {
  if (locked.value) return
  const version = ++scanVersion
  scanning.value = true; error.value = ''; failureKind.value = ''
  try {
    scanner = await import('@tauri-apps/plugin-barcode-scanner')
    if (disposed || version !== scanVersion) return
    const permission = await scanner.requestPermissions()
    if (disposed || version !== scanVersion) return
    if (permission !== 'granted') throw Object.assign(new Error('还没有相机权限。可以改用配对链接，也可以在系统设置中允许相机访问。'), { code: 'CAMERA_PERMISSION' })
    document.body.classList.add('ppc-pairing-scanning')
    const result = await scanner.scan({ windowed: true, formats: [scanner.Format.QRCode], cameraDirection: 'back' })
    if (!disposed && version === scanVersion) await applyPairingLink(result.content)
  } catch (failure) { if (!disposed && version === scanVersion) reportFailure(['CAMERA_PERMISSION', 'INVALID_PAIRING_QR', 'PAIRING_TARGET_MISMATCH'].includes(failure?.code) ? failure : { message: '暂时无法完成扫码，可以改用配对链接或手动填写。' }, 'scan') }
  finally { if (version === scanVersion) { document.body.classList.remove('ppc-pairing-scanning'); if (!disposed) { scanning.value = false; if (!error.value) focusStage() } } }
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
    }
  }
  catch (failure) { if (!disposed && version === listVersion) listError.value = failure.message }
  finally { if (!disposed && version === listVersion) profilesLoading.value = false }
}
async function run(action) {
  if (locked.value) return
  busy.value = true; error.value = ''; failureKind.value = ''; savedName.value = ''; certificateNotice.value = ''; step.value = 'preparing'
  try { await action() } catch (failure) { reportFailure(failure) } finally { if (!disposed) busy.value = false }
}
async function prepare() { await props.prepareSwitch?.(); if (disposed) throw new Error('连接面板已关闭') }
async function enter(profile) { await run(async () => { await prepare(); if (!await verifyCertificate(profile)) return; step.value = 'switching'; await switchMode({ mode: 'remote', profileId: profile.id }); emit('close') }) }
async function save() {
  if (!endpoint.value.trim() || !secret.value.trim() || insecureEndpoint.value && !allowInsecure.value) return
  await run(async () => {
    await prepare(); step.value = 'certificate'
    const inspection = await remoteConnections.inspectCertificate(endpoint.value.trim(), editingId.value, allowInsecure.value)
    if (disposed || inspection.requiresConfirmation && !await confirmCertificate(inspection)) return
    if (disposed) return
    step.value = 'authorizing'
    let profile
    try {
      profile = await remoteConnections.save({ endpoint: endpoint.value.trim(), method: method.value,
        token: method.value === 'bearer' ? secret.value : '', code: method.value === 'bearer' ? '' : method.value === 'pair' ? secret.value.trim() : secret.value,
        certificateFingerprint: inspection.requiresConfirmation ? inspection.certificate.fingerprint : null,
        name: deviceName.value, remember: remember.value, profileId: editingId.value, allowInsecure: allowInsecure.value })
    } finally { secret.value = '' }
    if (disposed) return
    // 保存与进入分别处理，切换失败也保留卡片，避免再次消耗一次性配对码。
    listVersion++; profilesLoading.value = false
    profiles.value = [...profiles.value.filter(item => item.id !== profile.id), profile]
    savedName.value = profile.displayName || 'infOS'; editingId.value = profile.id; formOpen.value = false; pairingLink.value = ''
    step.value = 'switching'; await refreshMode()
    if (disposed) return
    await switchMode({ mode: 'remote', profileId: profile.id })
    emit('close')
  })
}
async function verifyCertificate(profile) {
  if (!profile.endpoint.startsWith('https:')) return true
  step.value = 'certificate'
  const inspection = await remoteConnections.inspectCertificate(profile.endpoint, profile.id, false)
  if (disposed) return false
  if (inspection.requiresConfirmation) {
    if (!await confirmCertificate(inspection) || disposed) return false
    await remoteConnections.trustCertificate(profile.id, inspection.certificate.fingerprint)
    await refreshMode(); await loadProfiles()
  }
  return !disposed
}
async function checkCertificate(profile) {
  await run(async () => { await prepare(); if (await verifyCertificate(profile)) certificateNotice.value = tr('证书已核对') })
}
async function reauthorize(profile) {
  scanVersion++
  editingId.value = profile.id; endpoint.value = profile.endpoint; secret.value = ''; method.value = 'pair'
  allowInsecure.value = false; error.value = ''; failureKind.value = ''; pairingLink.value = ''; showLinkInput.value = false; savedName.value = ''; certificateNotice.value = ''
  formOpen.value = true; manualOpen.value = false; pairingStage.value = 1; inputOrigin.value = 'manual'; expandedProfile.value = null; removing.value = null
  await focusStage()
}
async function newConnection() {
  scanVersion++
  editingId.value = null; endpoint.value = ''; secret.value = ''; allowInsecure.value = false; savedName.value = ''
  method.value = 'pair'; pairingLink.value = ''; showLinkInput.value = false; error.value = ''; failureKind.value = ''; certificateNotice.value = ''
  formOpen.value = true; manualOpen.value = false; pairingStage.value = 0; inputOrigin.value = 'manual'; removing.value = null
  await focusStage()
}
function closeForm() { scanVersion++; formOpen.value = false; manualOpen.value = false; showLinkInput.value = false; secret.value = ''; pairingLink.value = ''; error.value = ''; failureKind.value = ''; nextTick(() => connectionSection.value?.querySelector('.connections-start')?.focus()) }
function revealInvalidField(event) {
  // 展开必填字段所在的折叠区，让系统校验能定位到实际输入框。
  for (let parent = event.target.parentElement; parent && parent !== pairingForm.value; parent = parent.parentElement) if (parent.tagName === 'DETAILS') parent.open = true
}
function profileName(profile) {
  const name = profile.displayName || 'infOS'
  return /^infos-node-[a-z\d-]{8,}$/i.test(name) ? `infOS · ${name.slice(-6)}` : name
}
function expired(profile) { return !!profile.device && profile.device.expiresAt <= Date.now() }
function needsPairing(profile) { return expired(profile) || activeId.value === profile.id && remoteState.connection === 'auth_required' }
function profileState(profile) { return tr(needsPairing(profile) ? '需要重新配对' : activeId.value !== profile.id ? '已配对' : remoteState.connection === 'online' ? '已连接' : ['connecting', 'reconnecting'].includes(remoteState.connection) ? '连接中' : '暂时离线') }
function expiry(value) { const date = new Date(value); return Number.isNaN(date.getTime()) ? '未提供' : date.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' }) }
async function logout() { await run(async () => { await prepare(); step.value = 'revoking'; await remoteClient.logoutDevice(currentToken()); logoutConfirm.value = false; if (currentProfile.value) await reauthorize(currentProfile.value) }) }
async function standalone() { await run(async () => { await prepare(); step.value = 'switching'; await switchMode({ mode: 'standalone' }); emit('close') }) }
async function remove(id) { await run(async () => { step.value = 'removing'; await remoteConnections.remove(id); await loadProfiles(); removing.value = null; if (editingId.value === id) newConnection() }) }
onMounted(loadProfiles)
onBeforeUnmount(() => { disposed = true; finishCertificate(false); listVersion++; secret.value = ''; pairingLink.value = ''; cancelScan() })
</script>
<style>
body.ppc-pairing-scanning { background: transparent !important; }
body.ppc-pairing-scanning > * { visibility: hidden; }
body.ppc-pairing-scanning > .remote-scanner-controls { visibility: visible; }
</style>
<style scoped>
/* 两种模式先讲清用途，配对时只显示当前一步，沿用主题与系统动效偏好。 */
.connections-panel { min-width: 0; color: var(--ui-ink); font-size: var(--ui-text, 14px); line-height: 1.75; }
.connections-panel :is(button, input, textarea) { font: inherit; }
.connections-panel button { cursor: pointer; touch-action: manipulation; -webkit-tap-highlight-color: transparent; }
.connections-panel button:disabled { cursor: default; opacity: .5; }
.connections-panel :is(button, summary, input, textarea):focus-visible { outline: 2px solid var(--ui-accent); outline-offset: 3px; }
.connections-intro { margin: 2px 0 19px; }
.connections-current { display: inline-flex; align-items: center; gap: 6px; color: var(--ui-muted); font-size: 11px; }
.connections-current > i { width: 6px; height: 6px; border-radius: 50%; background: var(--ui-accent); }
.connections-intro h2 { margin: 9px 0; font-size: 22px; font-weight: 700; line-height: 1.5; letter-spacing: -.025em; }
.connections-intro p { margin: 0; color: var(--ui-muted); font-size: 13px; }
.connections-panel .connections-spaces { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; margin: 0 0 14px; }
.connections-spaces > article { display: flex; flex-direction: column; align-items: flex-start; gap: 5px; min-width: 0; padding: 15px 12px; border: 1px solid var(--ui-line); border-radius: 17px; background: var(--ui-paper); }
.connections-spaces > article.selected { border-color: var(--ui-contour); background: linear-gradient(145deg, var(--ui-paper), var(--ui-soft)); }
.connections-space-icon { display: grid; place-items: center; width: 34px; height: 34px; margin-bottom: 7px; border-radius: 11px; background: var(--ui-soft); color: var(--ui-accent); }
.connections-space-icon > svg { width: 22px; height: 22px; }
.connections-spaces strong { font-size: 14px; font-weight: 650; }
.connections-spaces small { color: var(--ui-muted); font-size: 11px; }
.connections-spaces p { margin: 3px 0 10px; color: var(--ui-muted); font-size: 12px; }
.connections-current-label, .connections-mode-note { display: flex; align-items: center; gap: 5px; min-height: 31px; margin-top: auto; font-size: 11px; color: var(--ui-accent); }
.connections-current-label > svg { width: 14px; height: 14px; }
.connections-mode-note { color: var(--ui-muted); }
.connections-spaces article > button { display: flex; align-items: center; gap: 5px; min-height: 38px; margin-top: auto; padding: 5px 0; border: 0; background: transparent; color: var(--ui-accent); font-size: 12px; text-align: left; }
.connections-spaces article > button > svg { width: 13px; height: 13px; flex: none; }
.connections-data-note { display: flex; align-items: flex-start; gap: 9px; margin: 14px 0 24px; padding: 12px; border-radius: 13px; background: var(--ui-blue-soft); color: var(--ui-muted); font-size: 12px; }
.connections-data-note > svg { width: 17px; height: 17px; flex: none; margin-top: 3px; color: var(--ui-blue); }
.connections-heading { display: flex; align-items: center; justify-content: space-between; gap: 10px; margin: 0 0 10px; }
.connections-heading h3 { margin: 0; font-size: 14px; font-weight: 650; }
.connections-icon-button { display: grid; place-items: center; flex: none; width: 40px; height: 40px; padding: 9px; border: 0; border-radius: 11px; background: transparent; color: var(--ui-accent); }
.connections-icon-button > svg { width: 19px; height: 19px; }
.connections-empty { color: var(--ui-muted); font-size: 12px; }
.connections-list { display: grid; gap: 12px; min-width: 0; }
.connections-panel .connection-card { min-width: 0; border: 1px solid var(--ui-line); border-radius: 17px; overflow: hidden; background: var(--ui-paper); box-shadow: none; }
.connection-card.is-current { border-color: var(--ui-contour); }
.connection-identity { display: grid; grid-template-columns: 32px minmax(0, 1fr); align-items: center; gap: 5px 11px; min-width: 0; padding: 15px 13px 12px; }
.connections-panel .connection-glyph { display: grid; place-items: center; width: 32px; height: 34px; border-radius: 11px; color: var(--ui-accent); background: var(--ui-soft); }
.connection-glyph > svg { width: 20px; height: 20px; }
.connection-identity > div { min-width: 0; display: grid; gap: 3px; }
.connections-panel .connection-identity strong { min-width: 0; overflow-wrap: anywhere; font-size: 14px; font-weight: 650; }
.connections-panel .connection-identity small { color: var(--ui-muted); font-size: 11px; overflow-wrap: anywhere; }
.connection-state { grid-column: 2; justify-self: start; padding: 1px 7px; border-radius: 6px; background: var(--ui-soft); color: var(--ui-accent); font-size: 10px; }
.connection-state.needs-pairing { color: var(--ui-amber); background: var(--ui-amber-soft); }
.connection-pairing-help { margin: 0 13px 12px 56px; color: var(--ui-muted); font-size: 11px; }
.connections-panel .connection-actions { display: flex; align-items: center; justify-content: space-between; gap: 10px; padding: 7px 10px; border-top: 1px solid var(--ui-line); background: var(--ui-inset); }
.connections-panel .connection-enter { display: inline-flex; align-items: center; justify-content: center; gap: 8px; min-height: 40px; padding: 6px 11px; border: 0; border-radius: 11px; color: var(--ui-accent); background: var(--ui-soft); font-size: 12px; }
.connection-enter > svg { width: 14px; height: 14px; flex: none; }
.connections-panel .connection-enter:disabled { background: transparent; color: var(--ui-muted); opacity: 1; }
.connection-more { display: grid; place-items: center; width: 40px; height: 40px; flex: none; border: 0; border-radius: 10px; color: var(--ui-muted); background: transparent; }
.connection-more > svg { width: 19px; height: 19px; }
.connection-details { border-top: 1px solid var(--ui-line); padding: 13px; }
.connection-details dl { display: grid; gap: 8px; margin: 0; font-size: 12px; }
.connection-details dl > div { display: grid; grid-template-columns: auto minmax(0, 1fr); gap: 12px; }
.connection-details dt { color: var(--ui-muted); }
.connection-details dd { margin: 0; overflow-wrap: anywhere; }
.connection-detail-actions { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 13px; }
.connection-detail-actions button { display: flex; align-items: center; gap: 5px; min-height: 40px; padding: 6px 9px; border: 1px solid var(--ui-line); border-radius: 10px; color: var(--ui-ink); background: var(--ui-paper); font-size: 11px; }
.connection-detail-actions svg { width: 15px; height: 15px; }
.connections-panel .connections-submit { display: flex; align-items: center; justify-content: center; gap: 9px; width: 100%; min-height: 48px; padding: 11px 14px; border: 1px solid var(--ui-accent); border-radius: 13px; background: var(--ui-action, var(--ui-accent)); color: var(--ui-on-accent); font-size: 13px; font-weight: 600; line-height: 1.6; }
.connections-submit > svg { width: 18px; height: 18px; flex: none; }
.connections-start { margin-top: 19px; }
.connections-start-note { margin: 9px 0 21px; text-align: center; color: var(--ui-muted); font-size: 11px; }
.connections-panel summary { display: flex; align-items: center; gap: 8px; min-height: 46px; list-style: none; cursor: pointer; font-size: 12px; color: var(--ui-muted); }
.connections-panel summary::-webkit-details-marker { display: none; }
.connections-panel summary > svg { width: 16px; height: 16px; flex: none; }
.connections-panel summary > svg:last-child { width: 12px; height: 12px; margin-left: auto; transition: transform 180ms; }
.connections-panel details[open] > summary > svg:last-child { transform: rotate(180deg); }
.connections-help, .connections-settings { margin-top: 14px; border-top: 1px solid var(--ui-line); }
.connections-help p, .connections-help ul { margin: 2px 0 12px; color: var(--ui-muted); font-size: 12px; line-height: 1.85; }
.connections-help ul { padding-left: 19px; }
.connections-help li { margin: 8px 0; }
.connections-help-actions { display: flex; flex-wrap: wrap; gap: 4px 20px; }
.connections-text { display: inline-flex; align-items: center; gap: 6px; min-height: 40px; padding: 5px 0; border: 0; background: transparent; color: var(--ui-accent); font-size: 12px !important; text-align: left; }
.connections-text > svg { width: 14px; height: 14px; flex: none; }
.connections-steps { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); list-style: none; gap: 8px; padding: 0 0 22px; margin: 0; }
.connections-steps li { position: relative; display: flex; flex-direction: column; align-items: center; gap: 7px; font-size: 11px; color: var(--ui-muted); text-align: center; }
.connections-steps li:not(:last-child)::after { content: ''; position: absolute; top: 14px; left: calc(50% + 22px); right: calc(-50% + 14px); height: 1px; background: var(--ui-line); }
.connections-steps li > span { display: grid; place-items: center; width: 28px; height: 28px; border-radius: 50%; background: var(--ui-inset); border: 1px solid var(--ui-line); font-size: 12px; font-variant-numeric: tabular-nums; }
.connections-steps li > span > svg { width: 15px; height: 15px; }
.connections-steps li[data-state='current'] { color: var(--ui-accent); font-weight: 600; }
.connections-steps li[data-state='current'] > span { color: var(--ui-on-accent); background: var(--ui-action); border-color: var(--ui-action); }
.connections-steps li[data-state='done'] > span { color: var(--ui-mint); background: var(--ui-mint-soft); border-color: var(--ui-mint-edge); }
.connections-step-heading { display: flex; align-items: center; gap: 4px; margin: 0 0 15px; }
.connections-step-heading > button { margin-left: -9px; }
.connections-step-heading h3 { margin: 0; font-size: 18px; font-weight: 650; line-height: 1.6; outline: none; }
.connections-device-scene { display: flex; align-items: center; justify-content: center; gap: 22px; height: 100px; margin-bottom: 19px; border: 1px solid var(--ui-blue-edge); border-radius: 20px; background: linear-gradient(130deg, var(--ui-blue-soft), var(--ui-paper)); color: var(--ui-blue); }
.connections-device-scene > svg:first-child { width: 53px; height: 53px; }
.connections-device-scene > svg:last-child { width: 35px; height: 35px; }
.connections-device-scene > span { display: flex; gap: 7px; }
.connections-device-scene i { width: 4px; height: 4px; border-radius: 50%; background: currentColor; opacity: .5; }
.connections-preparation { display: grid; gap: 18px; list-style: none; padding: 0; margin: 0 0 25px; }
.connections-preparation li { display: flex; align-items: baseline; gap: 11px; }
.connections-preparation b { display: grid; place-items: center; width: 22px; height: 22px; flex: none; border-radius: 8px; background: var(--ui-soft); color: var(--ui-accent); font-size: 11px; }
.connections-preparation li > span { display: grid; gap: 5px; }
.connections-preparation strong { font-size: 13px; font-weight: 600; }
.connections-preparation small { font-size: 12px; color: var(--ui-muted); }
.connections-step-note { margin: 0 0 17px; color: var(--ui-muted); font-size: 12px; }
.connections-methods { display: grid; gap: 11px; }
.connections-methods > button { display: flex; align-items: center; gap: 12px; min-width: 0; min-height: 85px; padding: 16px 13px; border: 1px solid var(--ui-line); border-radius: 17px; background: var(--ui-paper); color: var(--ui-ink); text-align: left; }
.connections-methods > button.connections-method-primary { border-color: var(--ui-blue-edge); background: var(--ui-blue-soft); }
.connections-methods button > svg:first-child { width: 27px; height: 27px; flex: none; color: var(--ui-blue); }
.connections-methods button > svg:last-child { width: 14px; height: 14px; flex: none; color: var(--ui-muted); }
.connections-methods button > span { display: grid; gap: 5px; min-width: 0; flex: 1; }
.connections-methods strong { font-size: 14px; font-weight: 600; }
.connections-methods small { font-size: 11px; color: var(--ui-muted); }
.connections-link { margin: 15px 0; padding: 13px; border: 1px solid var(--ui-line); border-radius: 15px; background: var(--ui-paper); }
.connections-link > button { display: flex; align-items: center; gap: 7px; min-height: 43px; margin: 8px 0 0 auto; padding: 8px 10px; border: 0; border-radius: 10px; background: var(--ui-soft); color: var(--ui-accent); font-size: 12px; }
.connections-link > button > svg { width: 14px; height: 14px; }
.connections-manual-entry { display: flex; align-items: center; gap: 8px; width: 100%; min-height: 47px; margin-top: 17px; padding: 9px 2px; border: 0; background: transparent; color: var(--ui-muted); text-align: left; font-size: 12px !important; }
.connections-manual-entry > svg { width: 16px; height: 16px; flex: none; }
.connections-manual-entry > svg:last-child { width: 12px; height: 12px; margin-left: auto; }
.connections-pair-target { display: flex; align-items: flex-start; gap: 12px; padding: 16px 13px; margin: 12px 0; border: 1px solid var(--ui-mint-edge); border-radius: 16px; background: var(--ui-mint-soft); }
.connections-pair-target > svg { width: 29px; height: 29px; flex: none; margin-top: 3px; color: var(--ui-mint); }
.connections-pair-target > span { display: grid; gap: 7px; min-width: 0; }
.connections-pair-target small { color: var(--ui-muted); font-size: 11px; }
.connections-pair-target strong { font-size: 14px; font-weight: 600; overflow-wrap: anywhere; user-select: text; }
.connections-pair-target em { display: flex; align-items: center; gap: 5px; color: var(--ui-mint); font-size: 11px; font-style: normal; }
.connections-pair-target em > svg { width: 13px; height: 13px; }
.connections-fields { display: grid; gap: 15px; min-width: 0; margin: 0; padding: 6px 0; border: 0; }
.connections-field { display: grid; gap: 7px; min-width: 0; font-size: 12px; color: var(--ui-ink); }
.connections-panel .connections-field :is(input, textarea) { display: block; width: 100%; min-width: 0; min-height: 46px; margin: 0; padding: 10px 12px; border: 1px solid var(--ui-line); border-radius: 12px; background: var(--ui-paper); color: var(--ui-ink); font-size: 16px; line-height: 1.65; }
.connections-field textarea { resize: vertical; }
.connections-field :is(input, textarea)::placeholder { color: var(--ui-muted); font-size: 13px; }
.connections-field input:disabled { opacity: .65; }
.connections-panel .connections-hint { margin: 0; font-size: 11px; font-weight: 400; color: var(--ui-muted); line-height: 1.8; }
.connections-advanced { border-block: 1px dashed var(--ui-line); }
.connections-advanced[open] { padding-bottom: 12px; }
.connections-confirm-fields { margin-top: 14px; }
.connections-check { display: flex; align-items: flex-start; gap: 10px; min-width: 0; min-height: 44px; font-size: 12px; line-height: 1.8; }
.connections-check > span { min-width: 0; overflow-wrap: anywhere; }
.connections-check small { display: block; margin-top: 4px; color: var(--ui-muted); font-size: 11px; }
.connections-check input { appearance: auto; -webkit-appearance: auto; flex: none; width: 19px; height: 19px; min-height: 0; margin: 2px 0 0; padding: 0; accent-color: var(--ui-accent); }
.connections-network-consent { display: grid; gap: 9px; padding: 13px; border: 1px solid var(--ui-amber-edge); border-radius: 14px; background: var(--ui-amber-soft); }
.connections-network-consent > strong { display: flex; align-items: center; gap: 7px; color: var(--ui-amber); font-size: 12px; font-weight: 600; }
.connections-network-consent > strong > svg { width: 16px; height: 16px; }
.connections-network-consent > p { margin: 0; font-size: 11px; color: var(--ui-muted); }
.connections-network-consent > label { margin-top: 4px; }
.connections-later { display: block; min-height: 44px; margin: 15px auto 0; padding: 9px 12px; border: 0; border-radius: 10px; background: transparent; color: var(--ui-muted); font-size: 11px !important; }
.connections-recovery { padding: 14px; margin: 0 0 20px; border: 1px solid var(--ui-amber-edge); border-radius: 15px; background: var(--ui-amber-soft); outline: none; }
.connections-recovery > strong { display: flex; align-items: center; gap: 7px; font-size: 13px; font-weight: 600; }
.connections-recovery > strong > svg { width: 17px; height: 17px; color: var(--ui-amber); }
.connections-recovery p { margin: 8px 0 0; font-size: 12px; overflow-wrap: anywhere; }
.connections-recovery-hint { color: var(--ui-muted); }
.connections-pending, .connections-saved-notice { display: flex; align-items: flex-start; gap: 7px; margin: 0 0 15px; color: var(--ui-muted); font-size: 12px; }
.connections-saved-notice { padding: 12px; border-radius: 12px; background: var(--ui-mint-soft); color: var(--ui-mint); }
.connections-pending > svg, .connections-saved-notice > svg { width: 16px; height: 16px; flex: none; margin-top: 3px; }
.connections-progress { display: flex; align-items: center; gap: 10px; margin: 0 0 18px; padding: 12px; border-radius: 13px; background: var(--ui-soft); color: var(--ui-accent); }
.connections-progress > svg { width: 19px; height: 19px; flex: none; }
.connections-progress > span { display: grid; gap: 4px; }
.connections-progress strong { font-size: 12px; font-weight: 500; }
.connections-progress small { font-size: 11px; color: var(--ui-muted); }
.connections-online { display: flex; justify-content: space-between; gap: 8px; font-size: 12px; }
.connections-online small { color: var(--ui-muted); font-size: 11px; }
.connections-confirm { padding: 12px; margin-top: 12px; border: 1px solid var(--ui-line); border-radius: 12px; background: var(--ui-inset); }
.connections-confirm p { margin: 0 0 10px; font-size: 12px; }
.connections-confirm > div { display: flex; justify-content: flex-end; gap: 9px; }
.connections-confirm button { min-height: 40px; padding: 8px 12px; border: 1px solid var(--ui-line); border-radius: 10px; background: var(--ui-paper); color: var(--ui-ink); font-size: 12px; }
.connections-confirm .is-danger { color: var(--room-danger); }
.remote-scanner-controls { position: fixed; bottom: max(24px, env(safe-area-inset-bottom)); left: 16px; right: 16px; z-index: 99999; display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 14px; border-radius: 16px; background: #1b243beb; color: white; font-size: 12px; }
.remote-scanner-controls button { flex: none; min-height: 44px; background: white; color: #33466b; border: 0; padding: 10px 15px; border-radius: 12px; }
.connection-step-enter-active { transition: opacity 160ms, transform 190ms var(--ui-ease); }
.connection-step-leave-active { transition: opacity 100ms; pointer-events: none; }
.connection-step-enter-from { opacity: 0; transform: translateY(6px); }
.connection-step-leave-to { opacity: 0; }
.connections-methods > button { transition: transform 160ms, border-color 160ms; }
.connections-methods > button:active:not(:disabled) { transform: scale(.985); }
@media (hover: hover) { .connections-methods > button:hover:not(:disabled) { border-color: var(--ui-accent); } }
@media (prefers-reduced-motion: reduce) { .connection-step-enter-active, .connection-step-leave-active, .connections-methods > button, .connections-panel summary > svg { transition: none; } }
:global([data-reduce-motion='true']) :is(.connection-step-enter-active, .connection-step-leave-active, .connections-methods > button, .connections-panel summary > svg) { transition: none; }
@media (max-width: 350px) { .connections-intro h2 { font-size: 20px; } .connections-spaces > article { padding: 13px 10px; } .connections-steps { gap: 5px; } }
</style>
