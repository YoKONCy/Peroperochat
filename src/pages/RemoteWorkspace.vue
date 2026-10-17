<template>
  <main class="remote-workspace" :data-palette="appearance.palette" :style="viewportStyle" :inert="drawer || detail || resource ? true : undefined">
    <header class="remote-header">
      <button class="remote-avatar-button" :aria-label="`${agent?.name || '角色'}资料`" @click="drawer = 'character'"><RemoteAvatar :id="agent?.id" :label="agent?.name" /><span class="remote-presence" :class="{ online }" /></button>
      <div class="remote-heading"><button class="remote-server" @click="drawer = 'connections'">infOS<span>/ {{ profile?.displayName || '远程连接' }}</span><RoomIcon name="down" /></button><button class="remote-thread-title" @click="showThreads">{{ selection?.title || agent?.name || '选择会话' }}<RoomIcon name="down" /></button></div>
      <button class="room-icon-button" aria-label="打开会话列表" @click="showThreads"><RoomIcon name="chat" /></button>
      <button class="room-icon-button" aria-label="会话操作" @click="drawer = 'actions'"><RoomIcon name="more" /></button>
    </header>
    <div v-if="!online" class="remote-connection-state" role="status"><RoomIcon name="remote" /><span>{{ connectionLabel }}</span><button :disabled="reconnecting" @click="reconnect">{{ reconnecting ? '连接中…' : '重新连接' }}</button></div>
    <p v-if="error" class="remote-page-error" role="alert">{{ error }}<button aria-label="收起提示" @click="error = ''"><RoomIcon name="close" /></button></p>
    <p v-if="storageError" class="remote-page-error" role="alert">{{ storageError }}<button @click="retryPersist">重新保存</button></p>

    <section v-show="tab === 'chat'" class="remote-conversation">
      <Transition name="remote-welcome" @after-enter="settleWelcome" @after-leave="settleWelcome"><RemoteCharacter v-if="welcomeVisible" :agent="agent" :state="characterState" :model="appearance.model" @collapse="collapseCharacter" @profile="drawer = 'character'" /></Transition>
      <div v-if="selection && isCached" class="remote-cache-note">缓存内容 · 更新完成前可能与服务端不同</div>
      <div ref="transcript" class="remote-transcript" aria-label="会话消息" @scroll.passive="onScroll" @wheel.passive="readingGesture" @touchmove.passive="readingGesture" @click="openLink">
        <div v-if="!selection" class="remote-chat-empty"><span class="remote-empty-emblem"><RoomIcon name="chat" /></span><span class="remote-eyebrow">从这里，继续陪伴</span><h1>{{ remoteState.agents.length ? '选一条对话，接着聊吧' : '正在等你的伙伴上线' }}</h1><p>这里的对话、记忆与任务，保存在你连接的 infOS 中。</p><button class="room-primary-button" @click="showThreads">选择会话</button><button class="room-text-button" @click="drawer = 'connections'">管理连接</button></div>
        <template v-else>
          <button v-if="oldestSnapshot?.hasMoreBefore" class="remote-load-history" :disabled="loadingHistory" @click="older">{{ loadingHistory ? '正在读取…' : '查看更早的消息' }}</button>
          <p v-if="loading && !rows.length" class="room-empty">正在恢复对话…</p>
          <p v-else-if="!rows.length" class="room-empty">{{ online ? '对话从这一句开始。' : '此会话尚无可用缓存，连接恢复后会继续读取。' }}</p>
          <article v-for="row in rows" :key="row.surface.surfaceId" class="remote-message" :class="{ 'is-user': row.role === 'user', 'is-panel': !row.surface.messageId && row.surface.state !== 'open' }" :data-anchor="row.surface.surfaceId" :data-cursor="row.cursor || ''">
            <div class="remote-message-label"><span>{{ row.role === 'user' ? '你' : agent?.name || '角色' }}</span><time v-if="row.timestamp">{{ time(row.timestamp) }}</time></div>
            <RemoteSurface :surface="row.surface" :disabled="!online" @resource="openResource" @updated="refreshConversation" />
          </article>

          <div v-if="pendingForThread.length" class="remote-pending-summary"><RoomIcon name="help" /><span>有 {{ pendingForThread.length }} 项发送结果待核对</span><button @click="pending = pendingForThread[0]; drawer = 'pending'">查看</button></div>
        </template>
      </div>
      <button v-if="selection && (!atEnd || historyWindow)" class="remote-jump-latest" @click="latest"><RoomIcon name="down" />回到最新</button>
      <form v-if="selection" class="remote-composer" @submit.prevent="send">
        <div v-if="attachments.length" class="remote-draft-attachments"><span v-for="(file, index) in attachments" :key="file.id"><RoomIcon name="document" />{{ file.name }}<button type="button" :aria-label="`移除附件 ${file.name}`" @click="attachments.splice(index, 1); draftChanged()"><RoomIcon name="close" /></button></span></div>
        <div class="remote-composer-line"><button type="button" class="room-icon-button" :disabled="!online || !verified || uploading || attachments.length >= 5" aria-label="添加附件" @click="fileInput.click()"><RoomIcon name="plus" /></button><textarea ref="composer" v-model="draft" :disabled="!sessionReady" rows="1" maxlength="131072" :placeholder="`和 ${agent?.name || '伙伴'} 接着聊…`" @input="draftChanged" @focus="inputFocused = true; collapseCharacter()" @blur="inputFocused = false" @keydown="composeKey" /><button v-if="busy" type="button" class="remote-send" aria-label="停止远程回复" :disabled="stopping || !online || sending?.phase === 'preparing'" @click="stop"><RoomIcon name="stop" /></button><button v-else type="submit" class="remote-send" :disabled="!canSend" aria-label="发送消息"><RoomIcon name="send" /></button></div>
        <span class="remote-composer-status">{{ uploading ? '附件正在上传…' : !verified ? '等待服务端确认会话' : agent?.isEnabled === false ? '角色已停用，可以继续阅读历史' : busy ? '角色正在回复，离开页面后服务端仍可继续执行' : '对话保存在当前 infOS' }}</span>
        <input ref="fileInput" type="file" hidden @change="attach" />
      </form>
    </section>

    <div v-if="tab !== 'chat'" class="remote-section-body"><RemoteCollection :key="tab" :kind="tab" :agent-id="selection?.agentId || remoteState.activeAgentId" :thread-id="selection?.threadId" @detail="detail = $event" @thread="fromReference" @pending="pending = $event; drawer = 'pending'" /></div>
    <nav v-if="!keyboardOpen" class="remote-navigation" aria-label="远程客户端导航"><button v-for="item in tabs" :key="item.id" :aria-current="tab === item.id ? 'page' : undefined" :class="{ selected: tab === item.id }" @click="changeTab(item.id)"><span><RoomIcon :name="item.icon" /><i v-if="item.id === 'inbox' && inboxCount">{{ inboxCount > 99 ? '99+' : inboxCount }}</i></span>{{ item.label }}</button></nav>

    <RoomDrawer v-if="drawer === 'threads'" title="会话" subtitle="找到上次的话题，也可以开始新的对话" :tone="tone" @close="drawer = null"><label v-if="remoteState.agents.length > 12" class="remote-search"><RoomIcon name="search" /><input v-model="agentSearch" placeholder="查找角色" /></label><div class="remote-agent-list"><button v-for="item in visibleAgents" :key="item.id" :class="{ selected: pickerAgent === item.id }" @click="pickerAgent = item.id; threadPage = 1; loadThreads()"><RemoteAvatar :id="item.id" :label="item.name" /><span>{{ item.name }}{{ item.isEnabled === false ? ' · 已停用' : '' }}</span></button></div><div class="remote-list-toolbar"><button class="room-text-button" :disabled="!online || creating || !pickerAgent" @click="createConversation"><RoomIcon name="plus" />新建会话</button><button class="room-icon-button" :disabled="threadLoading" aria-label="刷新会话列表" @click="loadThreads"><RoomIcon name="refresh" /></button></div><p v-if="threadError" class="room-error">{{ threadError }}</p><p v-if="threadLoading && !threads.length" class="room-empty">正在读取会话…</p><p v-else-if="!threads.length" class="room-empty">{{ pickerAgent ? '这个角色还没有可用会话。' : '连接恢复后，可选择角色与会话。' }}</p><div class="remote-record-list"><button v-for="item in threads" :key="item.id" class="remote-record" :class="{ selected: selection?.threadId === item.id }" @click="selectConversation({ threadId: item.id, agentId: item.agentId || pickerAgent, title: item.title || '新的对话' })"><RoomIcon name="chat" /><span><strong>{{ item.title || '新的对话' }}</strong><small>{{ time(item.updatedAt || item.createdAt) }}</small></span><RoomIcon name="arrow" /></button></div><div class="remote-pagination"><button :disabled="threadLoading || threadPage === 1" @click="threadPage--; loadThreads()">上一页</button><span>{{ threadPage }}</span><button :disabled="threadLoading || !moreThreads" @click="threadPage++; loadThreads()">下一页</button></div></RoomDrawer>

    <RoomDrawer v-if="drawer === 'character'" :title="agent?.name || '角色'" subtitle="来自 infOS 的伙伴" :tone="tone" @close="drawer = null"><RemoteCharacter :agent="agent" :state="characterState" :model="appearance.model" @collapse="drawer = null" /><p>{{ agent?.description || '选择会话后，可以查看角色信息。' }}</p><dl class="remote-metadata"><template v-for="(value, key) in agent?.publicProfile || {}" :key="key"><dt>{{ profileLabels[key] || key }}</dt><dd>{{ value }}</dd></template><template v-if="characterState?.mood"><dt>心情</dt><dd>{{ characterState.mood }}</dd></template><template v-if="characterState?.mind"><dt>此刻的想法</dt><dd>{{ characterState.mind }}</dd></template></dl><section v-if="agent?.id" class="remote-appearance"><h3>本机显示</h3><p v-if="storageError" class="room-error">{{ storageError }}</p><p class="remote-help">仅影响这条连接中的角色展示。</p><label>迎接区<select v-model="appearance.expanded" @change="saveAppearance"><option :value="true">展开迎接区</option><option :value="false">保持紧凑头像</option></select></label><label>展示形象<select v-model="appearance.model" @change="saveAppearance"><option :value="null">使用服务端头像</option><option :value="0">Pio · 默认形象</option><option :value="1">Pio · 校服形象</option><option :value="3">Pio · 冬日形象</option></select></label><label>主题色<select v-model="appearance.palette" @change="saveAppearance"><option value="sky">天青</option><option value="violet">浅紫</option><option value="rose">珊瑚</option></select></label></section></RoomDrawer>

    <RoomDrawer v-if="drawer === 'actions'" title="会话操作" :subtitle="selection?.title || '当前连接'" :tone="tone" @close="drawer = null"><div class="remote-action-menu"><button :disabled="!selection" @click="refreshConversation"><RoomIcon name="refresh" />刷新当前会话</button><button :disabled="!selection" @click="drawer = null; detail = { title: '行动清单', query: { kind: 'plan', threadId: selection.threadId } }"><RoomIcon name="document" />查看行动清单</button><button :disabled="!selection" @click="renameTitle = selection.title; drawer = 'rename'"><RoomIcon name="edit" />修改会话名称</button><button :disabled="!selection" @click="drawer = 'delete'"><RoomIcon name="trash" />删除远程会话</button><button :disabled="!selection || busy || uploading" @click="drawer = 'reset'"><RoomIcon name="refresh" />重置本机会话记录</button><button @click="drawer = 'connections'"><RoomIcon name="connection" />连接与运行模式</button></div></RoomDrawer>
    <RoomDrawer v-if="drawer === 'rename'" title="修改会话名称" :busy="mutating" :tone="tone" @close="drawer = null"><p v-if="error" class="room-error">{{ error }}</p><form class="remote-connection-form" @submit.prevent="renameConversation"><label>会话名称<input v-model="renameTitle" required maxlength="200" /></label><button class="room-primary-button" :disabled="mutating || !online || !renameTitle.trim()">保存</button></form></RoomDrawer>
    <RoomDrawer v-if="drawer === 'delete'" title="删除这条远程会话？" :busy="mutating" :tone="tone" @close="drawer = null"><p v-if="error" class="room-error">{{ error }}</p><p>将按 infOS 的删除规则处理「{{ selection?.title }}」。该操作会影响其他客户端。</p><button class="room-danger-button" :disabled="mutating || !online" @click="deleteConversation">确认删除会话</button></RoomDrawer>
    <RoomDrawer v-if="drawer === 'pending' && pending" title="核对发送结果" subtitle="连接中断不代表服务端停止了执行" :tone="tone" :busy="reviewing" @close="drawer = null; reviewNote = ''; reviewed = false"><p class="remote-help">{{ pending.context?.input?.content || pending.context?.input?.instruction || '这项操作的结果尚未确认。' }}</p><p v-if="error" class="room-error">{{ error }}</p><p v-if="remoteState.pendingError" class="room-error">{{ remoteState.pendingError }}</p><div class="remote-actions"><button class="room-secondary-button" @click="reconcile">重新核对</button><button v-if="pending.context?.input?.threadId" class="room-secondary-button" @click="fromReference({ threadId: pending.context.input.threadId, agentId: pending.context.input.agentId || selection?.agentId }); drawer = null">查看对应会话</button></div><template v-if="remotePendingRequests.has(pending.requestId)"><label class="remote-checkbox"><input v-model="reviewed" type="checkbox" />我已查看服务端状态并完成核对</label><textarea v-model="reviewNote" rows="3" maxlength="2048" placeholder="写下核对结果，例如已在会话中看到回复…" /><button class="room-primary-button" :disabled="!reviewed || !reviewNote.trim() || reviewing || pending.state !== 'unknown'" @click="review">保存核对记录</button></template><p v-else class="remote-help">此操作已经完成核对。</p></RoomDrawer>
    <RoomDrawer v-if="drawer === 'reset'" title="重置本机会话记录？" :tone="tone" @close="drawer = null"><p>将清除此会话在手机上的草稿、附件草稿和阅读位置，远程消息仍由 infOS 保存。</p><p v-if="error" class="room-error">{{ error }}</p><button class="room-danger-button" :disabled="busy || uploading" @click="resetSession">确认重置</button></RoomDrawer>
    <RemoteConnections v-if="drawer === 'connections'" :prepare-switch="persistCurrent" @close="drawer = null" />
    <RemoteDetail v-if="detail" :key="JSON.stringify(detail.query)" :detail="detail" @close="detail = null" @resource="openResource" @thread="fromReference" />
    <RemoteResource v-if="resource" :key="JSON.stringify(resource)" :resource="resourceInput" :name="resource.name" @close="resource = null" />
  </main>
</template>

<script setup>
import { computed, ref, reactive, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import { open as openExternal } from '@tauri-apps/plugin-shell'
import RoomIcon from '../components/companion/RoomIcon.vue'
import RoomDrawer from '../components/companion/RoomDrawer.vue'
import RemoteAvatar from '../components/remote/RemoteAvatar.vue'
import RemoteCharacter from '../components/remote/RemoteCharacter.vue'
import RemoteConnections from '../components/remote/RemoteConnections.vue'
import RemoteCollection from '../components/remote/RemoteCollection.vue'
import RemoteDetail from '../components/remote/RemoteDetail.vue'
import RemoteResource from '../components/remote/RemoteResource.vue'
import RemoteSurface from '../components/remote/RemoteSurface.vue'
import { useRoomViewport } from '../composables/useRoomViewport'
import { currentToken, assertToken } from '../stores/mode'
import { remoteClient, remoteConnections } from '../services/remote'
import { readPresentation, writePresentation } from '../services/remotePresentation'
import { uploadRemoteFile } from '../services/remoteMedia'
import { remoteState, remoteViews, remoteExecutions, remotePendingRequests, readRemote, sendRemoteChat, performRemote, reconnectRemote, reconcileRemotePendingRequests, reviewRemoteRequest } from '../stores/remote'
import { remoteItems } from '../utils/remoteMarkdown'
import '../styles/remote.css'

const token = currentToken()
const { keyboardOpen, viewportStyle } = useRoomViewport()
const selection = ref(null), tab = ref('chat'), drawer = ref(null), detail = ref(null), resource = ref(null), profile = ref(null)
const draft = ref(''), attachments = ref([]), pages = ref([]), loading = ref(false), loadingHistory = ref(false), verified = ref(false), sending = ref(null), sessionReady = ref(false), welcomeSuppressed = ref(false)
const error = ref(''), storageError = ref(''), uploading = ref(false), stopping = ref(false), reconnecting = ref(false), creating = ref(false), mutating = ref(false)
const transcript = ref(null), composer = ref(null), fileInput = ref(null), inputFocused = ref(false), atEnd = ref(true), historyWindow = ref(false)
const appearance = reactive({ expanded: true, model: null, palette: 'sky' })
const pickerAgent = ref(null), threadPage = ref(1), threadLoading = ref(false), threadError = ref(''), threadQuery = ref(null), renameTitle = ref(''), agentSearch = ref('')
const pending = ref(null), reviewing = ref(false), reviewed = ref(false), reviewNote = ref('')
const tabs = [{ id: 'chat', label: '会话', icon: 'chat' }, { id: 'tasks', label: '任务', icon: 'model' }, { id: 'inbox', label: '通知', icon: 'bell' }, { id: 'library', label: '资料', icon: 'diary' }]
const profileLabels = { gender: '性别', identity: '身份', appearance: '外貌', personality: '性格' }
const emptySession = () => ({ draft: '', attachments: [], anchor: null, offset: 0, atEnd: true, cursor: null, editId: crypto.randomUUID(), sendRequestId: null })
let reading = emptySession(), disposed = false, version = 0, listVersion = 0, suppressScroll = false, draftTimer = null, scrollTimer = null, opening = null, verificationTimer = null, projectionTimer = null, readingVersion = 0, scrollRestore = 0
const observedFrames = new Set()
const online = computed(() => remoteState.connection === 'online')
const agent = computed(() => {
  const id = selection.value?.agentId || pickerAgent.value || remoteState.activeAgentId
  if (!id) return null
  const detail = remoteViews.get(JSON.stringify({ kind: 'agent', id }))?.data
  return { id, name: id, ...detail, ...remoteState.agents.find(item => item.id === id) }
})
const tone = computed(() => appearance.palette === 'violet' ? 'nana' : 'pero')
const characterState = computed(() => agent.value ? remoteViews.get(JSON.stringify({ kind: 'agent_state', id: agent.value.id }))?.data : null)
const welcomeVisible = computed(() => !!selection.value && parseFloat(viewportStyle.value['--room-height']) >= 520 && appearance.expanded && !welcomeSuppressed.value && !historyWindow.value && tab.value === 'chat' && !drawer.value && !detail.value && !resource.value && !keyboardOpen.value && !inputFocused.value)
const snapshots = computed(() => pages.value.map(page => ({ ...page, data: remoteViews.get(JSON.stringify(page.query))?.data || page.snapshot })).filter(page => page.data))
const oldestSnapshot = computed(() => snapshots.value[0]?.data)
const isCached = computed(() => pages.value.some(page => { const view = remoteViews.get(JSON.stringify(page.query)); return view?.cached || !view && page.snapshot }))
const busy = computed(() => !!selection.value && (sending.value?.threadId === selection.value.threadId || [...remoteExecutions.values()].some(item => item.threadId === selection.value.threadId && item.state === 'observing')))
const canSend = computed(() => online.value && verified.value && agent.value?.isEnabled !== false && !busy.value && !uploading.value && !loading.value && !!draft.value.trim())
const visibleAgents = computed(() => { const keyword = agentSearch.value.trim().toLowerCase(); return remoteState.agents.filter(item => !keyword || `${item.name} ${item.id}`.toLowerCase().includes(keyword)).slice(0, 24) })
const threads = computed(() => remoteItems(threadQuery.value && remoteViews.get(JSON.stringify(threadQuery.value))?.data))
const moreThreads = computed(() => { const data = threadQuery.value && remoteViews.get(JSON.stringify(threadQuery.value))?.data; return typeof data?.total === 'number' ? threadPage.value * 30 < data.total : threads.value.length >= 30 })
const pendingForThread = computed(() => [...remotePendingRequests.values()].filter(item => item.state === 'unknown' && item.context?.input?.threadId === selection.value?.threadId))
const inboxCount = computed(() => [...remotePendingRequests.values()].filter(item => item.state === 'unknown').length + ['approvals', 'questions'].reduce((sum, kind) => sum + remoteItems(remoteViews.get(JSON.stringify({ kind }))?.data).length, 0))
const connectionLabel = computed(() => ({ auth_required: '凭据需要更新，请管理连接', incompatible: '当前连接需要处理，请查看连接设置', reconnecting: '正在重新连接 infOS…', offline: '暂时离线，保留当前对话' }[remoteState.connection] || '正在连接 infOS…'))
const resourceInput = computed(() => { if (!resource.value) return null; const { name, ...input } = resource.value; return input })
const rows = computed(() => {
  const messages = new Map(), descriptors = new Map(), cursors = new Map()
  for (const page of snapshots.value) {
    for (const message of page.data.messages || []) messages.set(message.messageId, message)
    for (const surface of page.data.surfaces || []) { descriptors.set(surface.surfaceId, surface); cursors.set(surface.surfaceId, page.query.beforeCursor) }
  }
  if (!historyWindow.value) for (const execution of remoteExecutions.values()) {
    if (execution.threadId !== selection.value?.threadId) continue
    for (const surface of execution.presentation.surfaces.values()) {
      if (execution.state === 'observing') descriptors.set(surface.surfaceId, surface)
      // 最终投影读取失败时，保留已经收到的持久消息，待服务端刷新接替。
      else if (execution.needsRefresh && surface.messageId && !descriptors.has(surface.surfaceId)) descriptors.set(surface.surfaceId, surface)
    }
  }
  const output = [...descriptors.values()].map(surface => ({ surface, role: messages.get(surface.messageId)?.role || 'assistant', timestamp: messages.get(surface.messageId)?.timestamp, cursor: cursors.get(surface.surfaceId) || null }))
  return [...output.filter(row => row.surface.messageId || row.surface.state === 'open'), ...output.filter(row => !row.surface.messageId && row.surface.state !== 'open')]
})
function guard(request = version) { assertToken(token, 'remote'); if (disposed || request !== version) throw new Error('会话已切换') }
function time(value) { if (!value) return ''; const date = new Date(value); return Number.isNaN(date.getTime()) ? '' : date.toLocaleString([], { month: 'numeric', day: 'numeric', hour: '2-digit', minute: '2-digit' }) }
function query(cursor = null) { return { kind: 'conversation', id: selection.value.threadId, pageSize: 60, beforeCursor: cursor } }
function sessionScope(item = selection.value) { return { kind: 'session', agentId: item.agentId, threadId: item.threadId } }
function captureReading() {
  const list = transcript.value
  if (!list || !selection.value || suppressScroll || tab.value !== 'chat') return
  const end = list.scrollHeight - list.scrollTop - list.clientHeight < 72
  const top = list.getBoundingClientRect().top
  const visible = [...list.querySelectorAll('[data-anchor]')].find(element => element.getBoundingClientRect().bottom > top + 4)
  reading = { ...reading, atEnd: end && !historyWindow.value, anchor: visible?.dataset.anchor || null, offset: visible ? visible.getBoundingClientRect().top - top : 0, cursor: visible?.dataset.cursor || null }
  atEnd.value = end
}
async function persistCurrent() {
  clearTimeout(draftTimer); clearTimeout(scrollTimer)
  if (!selection.value || !sessionReady.value) return
  guard(); captureReading()
  const current = { ...reading, draft: draft.value, attachments: attachments.value.map(item => ({ id: item.id, name: item.name })) }
  try { await writePresentation(sessionScope(), current, token); guard(); storageError.value = '' }
  catch (failure) { if (!disposed) storageError.value = `草稿或阅读位置尚未保存：${failure.message}`; throw failure }
}
function persistLater() { clearTimeout(draftTimer); draftTimer = setTimeout(() => persistCurrent().catch(() => {}), 280) }
function retryPersist() { persistCurrent().catch(() => {}) }
function draftChanged() { reading.editId = crypto.randomUUID(); reading.sendRequestId = null; persistLater(); if (composer.value) { composer.value.style.height = 'auto'; composer.value.style.height = `${Math.min(composer.value.scrollHeight, 132)}px` } }
function onScroll() { if (suppressScroll) return; readingVersion++; captureReading(); clearTimeout(scrollTimer); scrollTimer = setTimeout(() => persistCurrent().catch(() => {}), 450) }
function readingGesture() { if (!suppressScroll && appearance.expanded) collapseCharacter() }
async function restoreScroll(saved = reading) {
  const request = version, restore = ++scrollRestore
  suppressScroll = true
  await nextTick()
  if (disposed || request !== version || restore !== scrollRestore) { if (restore === scrollRestore) suppressScroll = false; return }
  const list = transcript.value
  if (list) {
    const anchor = [...list.querySelectorAll('[data-anchor]')].find(element => element.dataset.anchor === saved.anchor)
    if (!saved.atEnd && anchor) list.scrollTop += anchor.getBoundingClientRect().top - list.getBoundingClientRect().top - saved.offset
    else if (saved.atEnd) list.scrollTop = list.scrollHeight
    else if (rows.value.length) error.value = '上次阅读的消息暂未找到，可以继续加载历史或回到最新。'
  }
  await nextTick(); if (restore === scrollRestore) suppressScroll = false
}
async function saveAppearance() {
  if (!agent.value) return
  const request = version
  if (appearance.expanded) welcomeSuppressed.value = false
  try { await writePresentation({ kind: 'character', agentId: agent.value.id }, { ...appearance }, token); guard(request) }
  catch (failure) { if (!disposed && request === version) storageError.value = `角色显示偏好尚未保存：${failure.message}` }
}
function collapseCharacter() { if (!appearance.expanded) return; captureReading(); appearance.expanded = false; saveAppearance(); restoreScroll({ ...reading }) }
function settleWelcome() { if (sessionReady.value && tab.value === 'chat') restoreScroll({ ...reading }) }
async function selectConversation(item) {
  if (!item.threadId || !item.agentId) return
  if (uploading.value) { error.value = '附件正在上传，完成后再切换会话'; return }
  const request = ++version
  const promise = (async () => {
    if (selection.value) await persistCurrent()
    guard(request)
    selection.value = { agentId: item.agentId, threadId: item.threadId, title: item.title || '继续对话' }; tab.value = 'chat'; drawer.value = null; detail.value = null; verified.value = false; sessionReady.value = false; loading.value = true; loadingHistory.value = false; error.value = ''; pages.value = []; draft.value = ''; attachments.value = []
    try {
      const [saved, character] = await Promise.all([readPresentation(sessionScope(item), token), readPresentation({ kind: 'character', agentId: item.agentId }, token)])
      guard(request)
      reading = saved || emptySession(); draft.value = reading.draft; attachments.value = reading.attachments || []; atEnd.value = reading.atEnd; historyWindow.value = !!reading.cursor; welcomeSuppressed.value = !reading.atEnd && !!reading.anchor; sessionReady.value = true
      Object.assign(appearance, character || { expanded: true, model: null, palette: 'sky' })
      await writePresentation({ kind: 'home' }, selection.value, token); guard(request)
      const currentQuery = query(reading.atEnd ? null : reading.cursor)
      pages.value = [{ query: currentQuery, snapshot: null }]
      const cached = await readRemote(currentQuery, { cachedOnly: true }); guard(request)
      if (cached) { pages.value[0].snapshot = cached; await restoreScroll({ ...reading }) }
      if (remoteState.activeAgentId !== item.agentId) remoteClient.connect(item.agentId, token).catch(failure => { if (!disposed && request === version) error.value = failure.message })
      const results = await Promise.allSettled([readRemote(currentQuery), readRemote({ kind: 'thread', id: item.threadId }, { allowCached: false })])
      guard(request)
      const metadata = results[1]
      if (metadata.status === 'fulfilled') {
        const thread = metadata.value?.thread
        if (!thread || thread.agentId !== item.agentId) throw new Error('此会话的角色信息已改变，请重新选择会话')
        verified.value = true; selection.value.title = thread.title || '新的对话'
        await writePresentation({ kind: 'home' }, selection.value, token); guard(request)
      } else if (metadata.reason?.code === 'REMOTE_NOT_FOUND' || metadata.reason?.code === 'REMOTE_AUTH_REQUIRED') {
        if (metadata.reason.code === 'REMOTE_NOT_FOUND') { await writePresentation({ kind: 'home' }, null, token); guard(request); selection.value = null; pages.value = []; drawer.value = 'threads'; pickerAgent.value = item.agentId; loadThreads() }
        throw metadata.reason
      }
      if (results[0].status === 'rejected') throw results[0].reason
      if (pages.value[0]) pages.value[0].snapshot = results[0].value
      await restoreScroll({ ...reading }); guard(request)
      readRemote({ kind: 'agent_state', id: item.agentId }).catch(() => {})
      readRemote({ kind: 'agent', id: item.agentId }).catch(() => {})
      if (online.value) remoteClient.subscribe({ kind: 'thread', id: item.threadId }, token).catch(failure => { if (!disposed && request === version) error.value = failure.message })
    } catch (failure) { if (!disposed && request === version) error.value = failure.message }
    finally { if (!disposed && request === version) loading.value = false }
  })()
  opening = promise
  try { await promise } catch (failure) { if (!disposed && request === version) error.value = failure.message } finally { if (opening === promise) opening = null }
}
async function showThreads() { drawer.value = 'threads'; pickerAgent.value = selection.value?.agentId || remoteState.activeAgentId || remoteState.agents[0]?.id || null; await loadThreads() }
async function loadThreads() {
  if (!pickerAgent.value) return
  const request = ++listVersion; threadLoading.value = true; threadError.value = ''
  const next = { kind: 'threads', agentId: pickerAgent.value, page: threadPage.value, pageSize: 30 }; threadQuery.value = next
  try { guard(); await readRemote(next); guard() } catch (failure) { if (!disposed && request === listVersion) threadError.value = failure.message } finally { if (!disposed && request === listVersion) threadLoading.value = false }
}
async function createConversation() {
  if (creating.value) return
  const request = version
  creating.value = true
  try { guard(request); const agentId = pickerAgent.value; const result = await performRemote({ kind: 'create_thread', agentId, title: '新的对话' }); guard(request); if (!result?.thread?.id) throw new Error('服务端未返回新会话身份'); await selectConversation({ threadId: result.thread.id, agentId: result.thread.agentId || agentId, title: result.thread.title }) }
  catch (failure) { if (!disposed && request === version) threadError.value = failure.message } finally { if (!disposed) creating.value = false }
}
async function refreshConversation() {
  if (!selection.value) return
  const saved = { ...reading }, request = version, readingRequest = readingVersion, current = query(pages.value.at(-1)?.query.beforeCursor || null)
  try { guard(request); const data = await readRemote(current, { allowCached: false }); guard(request); const page = pages.value.find(page => JSON.stringify(page.query) === JSON.stringify(current)); if (page) page.snapshot = data; await restoreScroll(readingRequest === readingVersion ? saved : { ...reading }) }
  catch (failure) { if (!disposed && request === version) error.value = failure.message }
}
async function older() {
  if (loadingHistory.value || !oldestSnapshot.value?.beforeCursor) return
  const saved = { ...reading }, request = version, readingRequest = readingVersion, next = query(oldestSnapshot.value.beforeCursor)
  loadingHistory.value = true
  try { const data = await readRemote(next); guard(request); if (!pages.value.some(page => JSON.stringify(page.query) === JSON.stringify(next))) pages.value.unshift({ query: next, snapshot: data }); if (pages.value.length > 4) { pages.value.pop(); historyWindow.value = true }; await restoreScroll(readingRequest === readingVersion ? saved : { ...reading }) }
  catch (failure) { if (!disposed && request === version) error.value = failure.message } finally { if (!disposed && request === version) loadingHistory.value = false }
}
async function latest() {
  if (!selection.value) return
  const request = version
  historyWindow.value = false; reading = { ...reading, atEnd: true, anchor: null, offset: 0, cursor: null }; atEnd.value = true
  const next = query(); pages.value = [{ query: next, snapshot: null }]
  await refreshConversation()
  if (!disposed && request === version) await restoreScroll(reading)
}
async function changeTab(value) { const request = version; captureReading(); try { await persistCurrent(); guard(request); if (value !== 'chat') collapseCharacter(); tab.value = value; if (value === 'chat') { await restoreScroll(reading); guard(request); if (online.value) refreshConversation() } } catch { /* 保存失败或会话切换时保留当前页面与草稿。 */ } }
async function send() {
  if (!canSend.value) return
  const request = version, item = { ...selection.value }, content = draft.value, files = attachments.value.map(file => ({ ...file }))
  error.value = ''
  const observationId = crypto.randomUUID(), editId = reading.editId || crypto.randomUUID()
  sending.value = { observationId, threadId: item.threadId, content, attachments: files, phase: 'preparing' }
  reading.editId = editId; reading.sendRequestId = observationId
  try {
    await persistCurrent(); guard(request)
    if (historyWindow.value) await latest()
    guard(request); collapseCharacter(); if (sending.value?.observationId === observationId) sending.value.phase = 'observing'
    await sendRemoteChat({ threadId: item.threadId, agentId: item.agentId, content, attachmentIds: files.map(file => file.id), imageMode: null }, { requestId: observationId })
    assertToken(token, 'remote')
    const saved = await readPresentation(sessionScope(item), token)
    assertToken(token, 'remote')
    if (!disposed && selection.value?.threadId === item.threadId && reading.editId === editId && saved && saved.sendRequestId === null) { reading = saved; draft.value = saved.draft; attachments.value = saved.attachments }
  } catch (failure) { if (!disposed && request === version) error.value = failure.message }
  finally { if (!disposed && sending.value?.observationId === observationId) sending.value = null; if (!disposed && request === version && atEnd.value) restoreScroll({ ...reading, atEnd: true }) }
}
function composeKey(event) { if (event.key === 'Enter' && !event.shiftKey && !event.isComposing && !('ontouchstart' in window)) { event.preventDefault(); send() } }
async function stop() { if (stopping.value || !selection.value) return; const request = version; stopping.value = true; try { await performRemote({ kind: 'stop_chat', threadId: selection.value.threadId }); guard(request); await refreshConversation() } catch (failure) { if (!disposed && request === version) error.value = failure.message } finally { if (!disposed) stopping.value = false } }
async function attach(event) {
  const file = event.target.files?.[0]; event.target.value = ''; if (!file || uploading.value || !selection.value) return
  const request = version, item = { ...selection.value }; uploading.value = true
  try { const result = await uploadRemoteFile(item.threadId, file); guard(request); if (!result?.id) throw new Error('服务端未返回附件身份'); attachments.value.push({ id: result.id, name: file.name }); reading.editId = crypto.randomUUID(); reading.sendRequestId = null; await persistCurrent() }
  catch (failure) { if (!disposed && request === version) error.value = failure.message } finally { if (!disposed && request === version) uploading.value = false }
}
function openResource(value) { resource.value = value }
async function fromReference(value) { if (!value.threadId || !value.agentId) { error.value = '此事项没有可打开的会话'; return }; detail.value = null; await selectConversation(value) }
async function renameConversation() { if (!selection.value || mutating.value) return; const request = version, item = { ...selection.value }, title = renameTitle.value.trim(); mutating.value = true; try { await performRemote({ kind: 'rename_thread', id: item.threadId, title }); guard(request); selection.value.title = title; await writePresentation({ kind: 'home' }, selection.value, token); guard(request); drawer.value = null } catch (failure) { if (!disposed && request === version) error.value = failure.message } finally { if (!disposed) mutating.value = false } }
async function deleteConversation() { if (!selection.value || mutating.value) return; const request = version, item = { ...selection.value }; mutating.value = true; try { await persistCurrent(); guard(request); await performRemote({ kind: 'delete_thread', id: item.threadId }); guard(request); await writePresentation({ kind: 'home' }, null, token); guard(request); selection.value = null; sessionReady.value = false; pages.value = []; draft.value = ''; attachments.value = []; drawer.value = null; await showThreads() } catch (failure) { if (!disposed && request === version) error.value = failure.message } finally { if (!disposed) mutating.value = false } }
async function reconnect() { if (reconnecting.value) return; const request = version; reconnecting.value = true; try { await reconnectRemote(); guard(request); if (selection.value && !opening) await selectConversation({ ...selection.value }); else if (drawer.value === 'threads') await loadThreads() } catch (failure) { if (!disposed && request === version) error.value = failure.message } finally { if (!disposed) reconnecting.value = false } }
async function reconcile() { reviewing.value = true; try { await reconcileRemotePendingRequests(); guard(); if (selection.value) await refreshConversation() } catch (failure) { if (!disposed) error.value = failure.message } finally { if (!disposed) reviewing.value = false } }
async function review() { if (!pending.value || !reviewed.value) return; reviewing.value = true; try { await reviewRemoteRequest(pending.value.requestId, reviewNote.value, { confirmed: true }); guard(); drawer.value = null; reviewed.value = false; reviewNote.value = '' } catch (failure) { if (!disposed) error.value = failure.message } finally { if (!disposed) reviewing.value = false } }
async function resetSession() {
  if (!selection.value || busy.value || uploading.value) return
  const item = { ...selection.value }, request = version
  try {
    clearTimeout(draftTimer); clearTimeout(scrollTimer)
    await writePresentation(sessionScope(item), null, token); guard(request)
    sessionReady.value = false
    await selectConversation(item)
  } catch (failure) { if (!disposed && request === version) error.value = failure.message }
}
async function openLink(event) { const link = event.target.closest('a[href]'); if (!link) return; event.preventDefault(); try { const url = new URL(link.href); if (['http:', 'https:'].includes(url.protocol) && !url.username && !url.password) await openExternal(url.href) } catch (failure) { error.value = failure.message } }
function visibility() { if (document.hidden) persistCurrent().catch(() => {}); else if (online.value && selection.value) refreshConversation() }
async function warmInbox() { try { guard(); await Promise.allSettled(['approvals', 'questions'].map(kind => readRemote({ kind }))) } catch { /* 连接提示保留在主界面，列表本身保留各自的读取错误。 */ } }
watch(keyboardOpen, value => { if (value) collapseCharacter() })
watch(() => pending.value?.requestId, () => { reviewed.value = false; reviewNote.value = '' })
watch(drawer, value => { if (value !== 'pending') { reviewed.value = false; reviewNote.value = '' } })
watch(() => [...remotePendingRequests.keys()].join('|'), async () => {
  if (!sessionReady.value || !selection.value || !reading.sendRequestId || remotePendingRequests.has(reading.sendRequestId)) return
  const request = version, editId = reading.editId
  try {
    const saved = await readPresentation(sessionScope(), token); guard(request)
    if (saved && reading.editId === editId && saved.editId !== editId && saved.sendRequestId === null) { reading = saved; draft.value = saved.draft; attachments.value = saved.attachments }
  } catch (failure) { if (!disposed && request === version) storageError.value = failure.message }
})
watch(() => [...remoteExecutions.values()].map(item => `${item.requestId}:${item.revision}`).join('|'), () => {
  if (historyWindow.value || !selection.value || disposed) return
  for (const execution of remoteExecutions.values()) {
    const frame = execution.frame
    if (execution.threadId !== selection.value.threadId || !frame || observedFrames.has(frame.operationId)) continue
    observedFrames.add(frame.operationId)
    if (observedFrames.size > 256) observedFrames.delete(observedFrames.values().next().value)
    if (frame.operation?.type === 'surface.open' || ['approval', 'input'].includes(frame.operation?.node?.kind)) {
      clearTimeout(projectionTimer)
      projectionTimer = setTimeout(() => { if (!disposed) refreshConversation() }, 150)
    }
  }
})
watch(() => rows.value.map(row => `${row.surface.surfaceId}:${row.surface.revision}`).join('|'), async () => { if (atEnd.value && !historyWindow.value && !loading.value && tab.value === 'chat') await restoreScroll({ ...reading, atEnd: true }) })
async function verifySelected() {
  clearTimeout(verificationTimer)
  if (!selection.value || !online.value || disposed) return
  if (opening) { verificationTimer = setTimeout(verifySelected, 500); return }
  const request = version, item = { ...selection.value }
  try {
    guard(request)
    const data = await readRemote({ kind: 'thread', id: item.threadId }, { allowCached: false }); guard(request)
    if (data?.thread?.agentId !== item.agentId) throw new Error('会话角色信息已改变，请重新选择')
    verified.value = true
    await refreshConversation(); guard(request)
    await remoteClient.subscribe({ kind: 'thread', id: item.threadId }, token)
  } catch (failure) {
    if (!disposed && request === version) {
      verified.value = false; error.value = failure.message
      if (failure.code === 'REMOTE_NOT_FOUND') {
        try { await persistCurrent(); guard(request); await writePresentation({ kind: 'home' }, null, token); guard(request); selection.value = null; sessionReady.value = false; pages.value = []; await showThreads() }
        catch (saveFailure) { if (!disposed && request === version) error.value = saveFailure.message }
      }
    }
  }
}
watch(() => remoteState.ready, ready => { if (ready) warmInbox(); if (ready && selection.value) verifySelected(); else if (ready && drawer.value === 'threads' && !pickerAgent.value) showThreads() })
onMounted(async () => {
  document.addEventListener('visibilitychange', visibility)
  const request = version
  try {
    const [profiles, home] = await Promise.all([remoteConnections.list(), readPresentation({ kind: 'home' }, token)])
    assertToken(token, 'remote'); if (disposed) return
    profile.value = profiles.find(item => item.id === token.target.profileId)
    guard(request)
    if (home) await selectConversation(home); else await showThreads()
    if (online.value) warmInbox()
  } catch (failure) { if (!disposed && request === version) error.value = failure.message }
})
onBeforeUnmount(() => {
  clearTimeout(draftTimer); clearTimeout(scrollTimer); clearTimeout(verificationTimer); clearTimeout(projectionTimer)
  if (selection.value && sessionReady.value) { captureReading(); writePresentation(sessionScope(), { ...reading, draft: draft.value, attachments: attachments.value.map(file => ({ id: file.id, name: file.name })) }, token).catch(() => {}) }
  disposed = true; version++; listVersion++; document.removeEventListener('visibilitychange', visibility)
})
</script>
