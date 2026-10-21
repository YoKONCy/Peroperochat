<template>
  <main class="remote-workspace remote-themed" :data-remote-theme="theme.mode" :data-decoration="theme.decoration" :data-bubble="theme.bubble" :data-reduce-motion="theme.reduceMotion" :style="[viewportStyle, theme.style]" :inert="drawer || detail || resource ? true : undefined">
    <header v-if="tab !== 'home'" class="remote-header">
      <button class="remote-avatar-button" :aria-label="`${agent?.name || '角色'}资料`" @click="drawer = 'character'"><RemoteAvatar :id="agent?.id" :label="agent?.name" /><span class="remote-presence" :class="{ online }" /></button>
      <div class="remote-heading"><button class="remote-server" @click="drawer = 'connections'">infOS<span>/ {{ profile?.displayName || '远程连接' }}</span><RoomIcon name="down" /></button><button class="remote-thread-title" @click="showThreads">{{ selection?.title || agent?.name || '选择会话' }}<RoomIcon name="down" /></button></div>
      <button class="room-icon-button" aria-label="展开角色 CG" @click="drawer = 'art'"><RoomIcon name="expand" /></button>
      <button class="room-icon-button remote-inbox-button" :aria-label="`通知与待处理事项${inboxCount ? `，${inboxCount} 项` : ''}`" @click="changeTab('inbox')"><RoomIcon name="bell" /><i v-if="inboxCount">{{ inboxCount > 99 ? '99+' : inboxCount }}</i></button>
      <button class="room-icon-button" aria-label="会话操作" @click="drawer = 'actions'"><RoomIcon name="more" /></button>
    </header>
    <Transition name="remote-status"><div v-if="!online || !remoteState.ready" class="remote-connection-state" role="status" :data-state="remoteState.connection"><RoomIcon :name="needsConnectionSettings ? 'help' : 'remote'" /><span><strong>{{ connectionLabel }}</strong><small>{{ remoteState.initializing ? '正在核对设备并恢复服务端数据，已有草稿继续保留。' : remoteState.error || '保留当前空间，连接恢复后只刷新数据，不自动重发操作。' }}</small></span><button v-if="needsConnectionSettings" @click="drawer = 'connections'">{{ remoteState.connection === 'auth_required' ? '重新授权' : '管理连接' }}</button><button v-else :disabled="reconnecting || remoteState.initializing" @click="reconnect">{{ reconnecting || remoteState.initializing ? '连接中…' : '重试' }}</button></div></Transition>
    <p v-if="error" class="remote-page-error" role="alert">{{ error }}<button aria-label="收起提示" @click="error = ''"><RoomIcon name="close" /></button></p>
    <p v-if="storageError" class="remote-page-error" role="alert">{{ storageError }}<button @click="retryPersist">重新保存</button></p>

    <RemoteHome v-if="tab === 'home'" :agent="agent" :state="characterState" :profile="profile" :selection="selection" :image="cg" :show-cg="display.showCg" :decoration="theme.decoration" :inbox-count="inboxCount" @art="drawer = 'art'" @connections="drawer = 'connections'" @appearance="drawer = 'appearance'" @resume="changeTab('chat')" @threads="showThreads" @tasks="changeTab('tasks')" @library="changeTab('library')" @inbox="changeTab('inbox')" @detail="detail = $event" />
    <section v-show="tab === 'chat'" class="remote-conversation">
      <RemoteQueue v-if="selection" :key="selection.threadId" :thread-id="selection.threadId" :agent-id="selection.agentId" :visible="tab === 'chat'" @updated="refreshConversation" />
      <div v-if="selection && isCached" class="remote-cache-note">缓存内容 · 更新完成前可能与服务端不同</div>
      <div ref="transcript" class="remote-transcript" aria-label="会话消息" @scroll.passive="onScroll" @click="openLink">
        <div v-if="!selection" class="remote-chat-empty"><span class="remote-empty-emblem"><RoomIcon name="chat" /></span><span class="remote-eyebrow">从这里，继续陪伴</span><h1>{{ remoteState.agents.length ? '选一条对话，接着聊吧' : '正在等你的伙伴上线' }}</h1><p>这里的对话、记忆与任务，保存在你连接的 infOS 中。</p><button class="room-primary-button" @click="showThreads">选择会话</button><button class="room-text-button" @click="drawer = 'connections'">管理连接</button></div>
        <template v-else>
          <button v-if="oldestSnapshot?.hasMoreBefore" class="remote-load-history" :disabled="loadingHistory" @click="older">{{ loadingHistory ? '正在读取…' : '查看更早的消息' }}</button>
          <p v-if="loading && !rows.length" class="room-empty">正在恢复对话…</p>
          <p v-else-if="!rows.length" class="room-empty">{{ online ? '对话从这一句开始。' : '此会话尚无可用缓存，连接恢复后会继续读取。' }}</p>
          <article v-for="row in rows" :key="row.surface.surfaceId" class="remote-message" :class="{ 'is-user': row.role === 'user', 'is-panel': !row.surface.messageId && row.surface.state !== 'open' }" :data-anchor="row.surface.surfaceId" :data-cursor="row.cursor || ''">
            <div class="remote-message-label"><RemoteAvatar class="remote-message-avatar" v-if="row.role !== 'user'" :id="row.surface.principalId || agent?.id" :label="agent?.name" /><span>{{ row.role === 'user' ? '你' : agent?.name || '角色' }}<RoomIcon v-if="row.role !== 'user' && theme.decoration !== 'none'" name="bow" /></span><time v-if="row.timestamp">{{ time(row.timestamp) }}</time><button v-if="row.message && row.surface.state !== 'open'" class="remote-message-tools" aria-label="编辑或回滚消息" :disabled="!online || busy" @click="messageAction = row.message; drawer = 'message'"><RoomIcon name="more" /></button></div>
            <RemoteSurface :surface="row.surface" :disabled="!online" @resource="openResource" @updated="refreshConversation" @resize="contentResized" />
          </article>

          <div v-if="pendingForThread.length" class="remote-pending-summary"><RoomIcon name="help" /><span>有 {{ pendingForThread.length }} 项发送结果待核对</span><button @click="pending = pendingForThread[0]; drawer = 'pending'">查看</button></div>
        </template>
      </div>
      <button v-if="selection && (!atEnd || historyWindow)" class="remote-jump-latest" @click="latest"><RoomIcon name="down" />{{ busy && !historyWindow ? '回复还在继续 · 回到底部' : '回到最新' }}</button>
      <form v-if="selection" class="remote-composer" @submit.prevent="send">
        <RemoteComposerTools ref="composerTools" :key="selection.threadId" :thread-id="selection.threadId" :agent-id="selection.agentId" :content="draft" :blocked="!online || !verified || uploading || !!sending" :running="busy" @settings="drawer = 'thread-settings'" @voice="drawer = 'voice-input'" @new-thread="quickNewConversation" @clear="draft = ''; draftChanged()" @workbench="openManagement('workbench')" @changed="refreshConversation" @capabilities="inputCapabilities = $event" @busy="controlsBusy = $event" @pending="showManagementPending" @restore="restoreRegeneration" />
        <div class="remote-composer-modes"><button v-for="item in interactionModes" :key="item.id" type="button" :class="{ selected: interactionMode === item.id }" :disabled="!online || !verified || busy || controlsBusy || changingInteraction" @click="setInteraction(item.id)"><RoomIcon :name="item.icon" />{{ item.label }}</button><button type="button" :disabled="uploading || !!sending" @click="drawer = 'image-mode'">{{ imageModeOptions.find(item => item.value === imageMode)?.label || '自动识图' }}<RoomIcon name="down" /></button></div>
        <div v-if="workspaceContext" class="remote-draft-attachments"><span><RoomIcon name="files" />工作台 · {{ workspaceContextLabel }}<button type="button" aria-label="移除工作台引用" @click="workspaceContext = null; draftChanged()"><RoomIcon name="close" /></button></span></div>
        <div v-if="attachments.length" class="remote-draft-attachments"><span v-for="(file, index) in attachments" :key="file.id"><RoomIcon name="document" />{{ file.name }}<button type="button" :aria-label="`移除附件 ${file.name}`" @click="attachments.splice(index, 1); draftChanged()"><RoomIcon name="close" /></button></span></div>
        <div class="remote-composer-line"><button type="button" class="room-icon-button" :disabled="!online || !verified || uploading || attachments.length >= 5" aria-label="添加附件" @click="fileInput.click()"><RoomIcon name="plus" /></button><textarea ref="composer" v-model="draft" :disabled="!sessionReady" rows="1" maxlength="131072" :placeholder="`和 ${agent?.name || '伙伴'} 接着聊…`" @input="draftChanged" @keydown="composeKey" /><button v-if="busy" type="button" class="remote-send" aria-label="停止远程回复" :disabled="stopping || !online || sending?.phase === 'preparing'" @click="stop"><RoomIcon name="stop" /></button><button type="submit" class="remote-send" :disabled="!canSend" :aria-label="busy ? '加入消息队列' : '发送消息'"><RoomIcon name="send" /></button></div>
        <span class="remote-composer-status">{{ uploading ? '附件正在上传…' : !verified ? '等待服务端确认会话' : agent?.isEnabled === false ? '角色已停用，可以继续阅读历史' : busy ? '角色正在回复，离开页面后服务端仍可继续执行' : '对话保存在当前 infOS' }}</span>
        <input ref="fileInput" type="file" hidden @change="attach" />
      </form>
    </section>

    <div v-if="tab === 'manage'" class="remote-section-body"><RemoteManager ref="manager" :key="`${selection?.agentId || remoteState.activeAgentId}:${selection?.threadId}`" :agent-id="selection?.agentId || remoteState.activeAgentId" :thread-id="selection?.threadId" :initial-section="managementSection" @pending="showManagementPending" @resource="openResource" @thread="fromReference" @context="useWorkbenchContext" /></div>
    <div v-if="!['chat', 'home', 'manage'].includes(tab)" class="remote-section-body"><button v-if="tab === 'library'" class="room-text-button" @click="openManagement('memories')"><RoomIcon name="edit" />资料管理、导入与编辑</button><RemoteCollection :key="tab" :kind="tab" :agent-id="selection?.agentId || remoteState.activeAgentId" :thread-id="selection?.threadId" @resource="openResource" @detail="detail = $event" @thread="fromReference" @pending="pending = $event; drawer = 'pending'" @create-task="drawer = 'dispatch'" /></div>
    <nav v-if="!keyboardOpen" class="remote-navigation" aria-label="远程客户端导航"><button v-for="item in tabs" :key="item.id" :aria-current="tab === item.id ? 'page' : undefined" :class="{ selected: tab === item.id }" @click="changeTab(item.id)"><RoomIcon v-if="tab === item.id && theme.decoration !== 'none'" class="remote-nav-ornament" :name="theme.decoration" artwork /><span><RoomIcon :name="item.icon" /></span>{{ item.label }}</button></nav>

    <RoomDrawer v-if="drawer === 'appearance'" title="显示偏好" subtitle="主题、配色与本机角色外观" :tone="tone" @close="drawer = null"><RemoteAppearance :display="display" :character="appearance" :agent="agent" :ready="themeReady" :saving="themeSaving" :save-error="themeError" :save-character="saveCharacter" @change="changeDisplay" @save="persistTheme" /></RoomDrawer>
    <RoomDrawer v-if="drawer === 'art'" :title="agent?.name || '角色展示'" subtitle="CG 与角色资料" :tone="tone" @close="drawer = null"><img v-if="cg" class="remote-cg-full" :src="cg" :alt="`${agent?.name || '角色'}的 CG`" /><RemoteAvatar v-else class="remote-cg-avatar" :id="agent?.id" :label="agent?.name" /><p class="remote-help">{{ characterState?.vibe || characterState?.mood || agent?.description || '这个角色尚未提供 CG，可以在本机显示偏好中选择图片。' }}</p><button class="room-text-button" @click="drawer = 'appearance'">调整角色展示</button></RoomDrawer>
    <RoomDrawer v-if="drawer === 'dispatch'" title="新的远程任务" subtitle="由当前 infOS 执行" :tone="tone" :busy="dispatching" @close="drawer = null"><p v-if="error" class="room-error">{{ error }}</p><p v-if="dispatchUnknown" class="remote-notice">提交结果尚未确认，请先到通知页核对，避免重复派发。<button class="room-text-button" @click="drawer = null; changeTab('inbox')">去核对</button></p><form class="remote-connection-form" @submit.prevent="dispatchTask"><label>任务标题<input v-model="taskTitle" maxlength="120" :disabled="dispatching" placeholder="可选，方便之后查找" /></label><label>希望完成什么<textarea v-model="taskInstruction" required rows="5" maxlength="16000" :disabled="dispatching" placeholder="描述任务目标和要求…" /></label><p class="remote-help">执行角色：{{ agent?.name || '未选择角色' }}。离开手机后，任务仍可在服务端继续执行。</p><button class="room-primary-button" :disabled="!online || dispatching || dispatchUnknown || !agent?.id || !taskInstruction.trim()">{{ dispatching ? '正在提交…' : '提交任务' }}</button></form></RoomDrawer>

    <RoomDrawer v-if="drawer === 'threads'" title="会话" subtitle="找到上次的话题，也可以开始新的对话" :tone="tone" @close="drawer = null"><label v-if="remoteState.agents.length > 12" class="remote-search"><RoomIcon name="search" /><input v-model="agentSearch" placeholder="查找角色" /></label><div class="remote-agent-list"><button v-for="item in visibleAgents" :key="item.id" :class="{ selected: pickerAgent === item.id }" @click="pickerAgent = item.id; threadPage = 1; loadThreads()"><RemoteAvatar :id="item.id" :label="item.name" /><span>{{ item.name }}{{ item.isEnabled === false ? ' · 已停用' : '' }}</span></button></div><div class="remote-list-toolbar"><button class="room-text-button" :disabled="!online || creating || !pickerAgent" @click="createConversation"><RoomIcon name="plus" />新建会话</button><button class="room-icon-button" :disabled="threadLoading" aria-label="刷新会话列表" @click="loadThreads"><RoomIcon name="refresh" /></button></div><p v-if="threadError" class="room-error">{{ threadError }}</p><p v-if="threadLoading && !threads.length" class="room-empty">正在读取会话…</p><p v-else-if="!threads.length" class="room-empty">{{ pickerAgent ? '这个角色还没有可用会话。' : '连接恢复后，可选择角色与会话。' }}</p><div class="remote-record-list"><button v-for="item in threads" :key="item.id" class="remote-record" :class="{ selected: selection?.threadId === item.id }" @click="selectConversation({ threadId: item.id, agentId: item.agentId || pickerAgent, title: item.title || '新的对话' })"><RoomIcon name="chat" /><span><strong>{{ item.title || '新的对话' }}</strong><small>{{ time(item.updatedAt || item.createdAt) }}</small></span><RoomIcon name="arrow" /></button></div><div class="remote-pagination"><button :disabled="threadLoading || threadPage === 1" @click="threadPage--; loadThreads()">上一页</button><span>{{ threadPage }}</span><button :disabled="threadLoading || !moreThreads" @click="threadPage++; loadThreads()">下一页</button></div></RoomDrawer>

    <RoomDrawer v-if="drawer === 'character'" :title="agent?.name || '角色'" subtitle="来自 infOS 的伙伴" :tone="tone" @close="drawer = null"><RemoteCharacter :agent="agent" :state="characterState" :image="cg" @collapse="drawer = null" /><p>{{ agent?.description || '选择会话后，可以查看角色信息。' }}</p><dl class="remote-metadata"><template v-for="(value, key) in agent?.publicProfile || {}" :key="key"><dt>{{ profileLabels[key] || key }}</dt><dd>{{ value }}</dd></template><template v-if="characterState?.mood"><dt>心情</dt><dd>{{ characterState.mood }}</dd></template><template v-if="characterState?.mind"><dt>此刻的想法</dt><dd>{{ characterState.mind }}</dd></template></dl><button class="room-primary-button" @click="drawer = 'appearance'">显示偏好与角色外观</button></RoomDrawer>

    <RoomDrawer v-if="drawer === 'actions'" title="会话操作" :subtitle="selection?.title || '当前连接'" :tone="tone" @close="drawer = null"><div class="remote-action-menu"><button :disabled="!selection || !online" @click="drawer = 'thread-settings'"><RoomIcon name="settings" />模型、工具与执行设置</button><button :disabled="!selection || busy || !online" @click="messageAction = null; drawer = 'rewind'"><RoomIcon name="refresh" />回退会话与相关文件</button><button @click="drawer = null; openManagement('workbench')"><RoomIcon name="files" />工作区与终端</button><button @click="drawer = null; openManagement()"><RoomIcon name="settings" />管理与工作台</button><button :disabled="!selection" @click="refreshConversation"><RoomIcon name="refresh" />刷新当前会话</button><button :disabled="!selection" @click="drawer = null; detail = { title: '行动清单', query: { kind: 'plan', threadId: selection.threadId } }"><RoomIcon name="document" />查看行动清单</button><button :disabled="!selection" @click="renameTitle = selection.title; drawer = 'rename'"><RoomIcon name="edit" />修改会话名称</button><button :disabled="!selection" @click="drawer = 'delete'"><RoomIcon name="trash" />删除远程会话</button><button :disabled="!selection || busy || uploading" @click="drawer = 'reset'"><RoomIcon name="refresh" />重置本机会话记录</button><button @click="drawer = 'connections'"><RoomIcon name="connection" />连接与运行模式</button></div></RoomDrawer>
    <RoomDrawer v-if="drawer === 'rename'" title="修改会话名称" :busy="mutating" :tone="tone" @close="drawer = null"><p v-if="error" class="room-error">{{ error }}</p><form class="remote-connection-form" @submit.prevent="renameConversation"><label>会话名称<input v-model="renameTitle" required maxlength="100" /></label><button class="room-primary-button" :disabled="mutating || !online || !renameTitle.trim()">保存</button></form></RoomDrawer>
    <RoomDrawer v-if="drawer === 'delete'" title="删除这条远程会话？" :busy="mutating" :tone="tone" @close="drawer = null"><p v-if="error" class="room-error">{{ error }}</p><p>将按 infOS 的删除规则处理「{{ selection?.title }}」。该操作会影响其他客户端。</p><button class="room-danger-button" :disabled="mutating || !online" @click="deleteConversation">确认删除会话</button></RoomDrawer>
    <RoomDrawer v-if="drawer === 'pending' && pending" title="核对发送结果" subtitle="连接中断不代表服务端停止了执行" :tone="tone" :busy="reviewing" @close="drawer = null; reviewNote = ''; reviewed = false"><p v-if="pending.context?.operation" class="remote-notice">{{ managementOperationLabel(pending.context.operation) }}</p><p class="remote-help">{{ pending.context?.input?.content || pending.context?.input?.instruction || '这项操作的结果尚未确认。' }}</p><p v-if="error" class="room-error">{{ error }}</p><p v-if="remoteState.pendingError" class="room-error">{{ remoteState.pendingError }}</p><div class="remote-actions"><button class="room-secondary-button" @click="reconcile">重新核对</button><button v-if="pending.context?.input?.threadId" class="room-secondary-button" @click="fromReference({ threadId: pending.context.input.threadId, agentId: pending.context.input.agentId || selection?.agentId }); drawer = null">查看对应会话</button></div><template v-if="remotePendingRequests.has(pending.requestId)"><label class="remote-checkbox"><input v-model="reviewed" type="checkbox" />我已查看服务端状态并完成核对</label><textarea v-model="reviewNote" rows="3" maxlength="2048" placeholder="写下核对结果，例如已在会话中看到回复…" /><button class="room-primary-button" :disabled="!reviewed || !reviewNote.trim() || reviewing || pending.state !== 'unknown'" @click="review">保存核对记录</button></template><p v-else class="remote-help">此操作已经完成核对。</p></RoomDrawer>
    <RoomDrawer v-if="drawer === 'reset'" title="重置本机会话记录？" :tone="tone" @close="drawer = null"><p>将清除此会话在手机上的草稿、附件草稿和阅读位置，远程消息仍由 infOS 保存。</p><p v-if="error" class="room-error">{{ error }}</p><button class="room-danger-button" :disabled="busy || uploading" @click="resetSession">确认重置</button></RoomDrawer>
    <RoomDrawer v-if="drawer === 'image-mode'" title="图片识别方式" subtitle="用于接下来发送的图片" :tone="tone" @close="drawer = null"><RemoteSelect v-model="imageMode" label="图片识别方式" :options="imageModeOptions" /></RoomDrawer>
    <RemoteConnections v-if="drawer === 'connections'" :prepare-switch="prepareModeSwitch" @close="drawer = null" />
    <RemoteThreadSettings v-if="drawer === 'thread-settings' && selection" :key="selection.threadId" :thread-id="selection.threadId" :agent-id="selection.agentId" @close="drawer = null" @updated="refreshConversation" @pending="showManagementPending" />
    <RemoteMessageActions v-if="(drawer === 'message' || drawer === 'rewind') && selection" :key="`${selection.threadId}:${messageAction?.messageId || 'all'}`" :thread-id="selection.threadId" :agent-id="selection.agentId" :blocked="busy || !online" :message="drawer === 'rewind' ? null : messageAction" @close="drawer = null" @updated="conversationUpdated" @pending="showManagementPending" />
    <RemoteVoiceInput v-if="drawer === 'voice-input' && selection" :key="selection.threadId" :blocked="!online || !verified" @close="drawer = null" @transcript="applyTranscript" @pending="showManagementPending" @configure="drawer = null; openManagement('voice')" />
    <RemoteDetail v-if="detail" :key="JSON.stringify(detail.query)" :detail="detail" @close="detail = null" @resource="openResource" @thread="fromReference" />
    <RemoteResource v-if="resource" :key="JSON.stringify(resource)" :resource="resourceInput" :name="resource.name" @close="resource = null" @resource="openResource" />
  </main>
</template>

<script setup>
import { computed, ref, reactive, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import { open as openExternal } from '@tauri-apps/plugin-shell'
import RoomIcon from '../components/remote/RemoteIcon.vue'
import RoomDrawer from '../components/companion/RoomDrawer.vue'
import RemoteAvatar from '../components/remote/RemoteAvatar.vue'
import RemoteCharacter from '../components/remote/RemoteCharacter.vue'
import RemoteConnections from '../components/remote/RemoteConnections.vue'
import RemoteCollection from '../components/remote/RemoteCollection.vue'
import RemoteDetail from '../components/remote/RemoteDetail.vue'
import RemoteResource from '../components/remote/RemoteResource.vue'
import RemoteSurface from '../components/remote/RemoteSurface.vue'
import RemoteHome from '../components/remote/RemoteHome.vue'
import RemoteQueue from '../components/remote/RemoteQueue.vue'
import RemoteAppearance from '../components/remote/RemoteAppearance.vue'
import RemoteManager from '../components/remote/RemoteManager.vue'
import RemoteThreadSettings from '../components/remote/RemoteThreadSettings.vue'
import RemoteMessageActions from '../components/remote/RemoteMessageActions.vue'
import RemoteComposerTools from '../components/remote/RemoteComposerTools.vue'
import RemoteVoiceInput from '../components/remote/RemoteVoiceInput.vue'
import RemoteSelect from '../components/remote/RemoteSelect.vue'
import { useRemoteTheme, defaultCharacterAppearance, characterCg } from '../services/remoteTheme'
import { useRoomViewport } from '../composables/useRoomViewport'
import { currentToken, assertToken } from '../stores/mode'
import { remoteClient, remoteConnections } from '../services/remote'
import { readPresentation, writePresentation } from '../services/remotePresentation'
import { uploadRemoteFile } from '../services/remoteMedia'
import { remoteState, remoteViews, remoteExecutions, remotePendingRequests, remoteNotifications, readRemote, sendRemoteChat, performRemote, initializeRemote, reconnectRemote, resumeRemote, reconcileRemotePendingRequests, reviewRemoteRequest, refreshRemoteNotifications, refreshRemotePendingRequests } from '../stores/remote'
import { remoteItems } from '../utils/remoteMarkdown'
import { managementOperationLabel } from '../services/remoteManagement'
import '../styles/remote.css'
import '../styles/remote-theme.css'
import '../styles/remote-management.css'
import '../styles/remote-interactions.css'
import '../styles/remote-conversation.css'
import { resolveRemoteResource } from '../services/remoteResources'

const token = currentToken()
const managementSection = ref(null), messageAction = ref(null), manager = ref(null)
const imageMode = ref('auto')
const inputCapabilities = ref(null), composerTools = ref(null), controlsBusy = ref(false)
const workspaceContext = ref(null)
const workspaceContextLabel = computed(() => [workspaceContext.value?.filePath, workspaceContext.value?.terminalId ? `终端 ${workspaceContext.value.terminalId}` : null].filter(Boolean).join(' · ').slice(-90))
const imageModeOptions = computed(() => [{ value: 'auto', label: '自动识图', description: '由 infOS 按当前模型与服务配置处理' }, { value: 'native', label: '原生视觉', description: inputCapabilities.value?.effective?.enableVision ? '当前模型支持视觉' : '当前模型未声明视觉能力', disabled: !!inputCapabilities.value && !inputCapabilities.value.effective?.enableVision }, { value: 'relay', label: '图片转述', description: inputCapabilities.value?.relayAvailable ? '转述服务已配置' : '尚未确认有可用的转述服务', disabled: inputCapabilities.value?.relayAvailable === false }])
const { keyboardOpen, viewportStyle } = useRoomViewport()
const selection = ref(null), tab = ref('chat'), drawer = ref(null), detail = ref(null), resource = ref(null), profile = ref(null)
const draft = ref(''), attachments = ref([]), pages = ref([]), loading = ref(false), loadingHistory = ref(false), verified = ref(false), sending = ref(null), sessionReady = ref(false)
const error = ref(''), storageError = ref(''), uploading = ref(false), stopping = ref(false), reconnecting = ref(false), creating = ref(false), mutating = ref(false)
const transcript = ref(null), composer = ref(null), fileInput = ref(null), atEnd = ref(true), historyWindow = ref(false)
const appearance = reactive(defaultCharacterAppearance())
const { display, theme, ready: themeReady, saving: themeSaving, error: themeError, restore: restoreTheme, save: persistTheme } = useRemoteTheme(appearance)
const cg = computed(() => characterCg(agent.value, appearance))
const changingInteraction = ref(false), taskTitle = ref(''), taskInstruction = ref(''), dispatching = ref(false)
const dispatchRequestId = ref(null), dispatchUnknown = ref(false)
const interactionModes = [{ id: 'minimal', label: '极简', icon: 'chat' }, { id: 'chat', label: '聊天', icon: 'chat' }, { id: 'work', label: '工作', icon: 'tasks' }]
const pickerAgent = ref(null), threadPage = ref(1), threadLoading = ref(false), threadError = ref(''), threadQuery = ref(null), renameTitle = ref(''), agentSearch = ref('')
const pending = ref(null), reviewing = ref(false), reviewed = ref(false), reviewNote = ref('')
const tabs = [{ id: 'home', label: '首页', icon: 'home' }, { id: 'chat', label: '会话', icon: 'chat' }, { id: 'tasks', label: '任务', icon: 'tasks' }, { id: 'library', label: '资料', icon: 'files' }, { id: 'manage', label: '管理', icon: 'settings' }]
const profileLabels = { gender: '性别', identity: '身份', appearance: '外貌', personality: '性格' }
const emptySession = () => ({ draft: '', attachments: [], anchor: null, offset: 0, atEnd: true, cursor: null, editId: crypto.randomUUID(), sendRequestId: null, workspaceContext: null })
let reading = emptySession(), disposed = false, version = 0, listVersion = 0, suppressScroll = false, draftTimer = null, scrollTimer = null, opening = null, verificationTimer = null, projectionTimer = null, readingVersion = 0, scrollRestore = 0
const observedFrames = new Set()
const online = computed(() => remoteState.connection === 'online')
const agent = computed(() => {
  const id = selection.value?.agentId || pickerAgent.value || remoteState.activeAgentId
  if (!id) return null
  const detail = remoteViews.get(JSON.stringify({ kind: 'agent', id }))?.data
  return { id, name: id, ...detail, ...remoteState.agents.find(item => item.id === id) }
})
const tone = computed(() => theme.value.mode === 'dark' ? 'nana' : 'pero')
const interactionMode = computed(() => selection.value ? remoteViews.get(JSON.stringify({ kind: 'thread', id: selection.value.threadId }))?.data?.thread?.interactionMode || 'chat' : 'chat')
const characterQuery = computed(() => agent.value ? { kind: 'agent_state', id: agent.value.id, threadId: selection.value?.agentId === agent.value.id ? selection.value.threadId : null } : null)
const characterState = computed(() => characterQuery.value ? remoteViews.get(JSON.stringify(characterQuery.value))?.data : null)
const snapshots = computed(() => pages.value.map(page => ({ ...page, data: remoteViews.get(JSON.stringify(page.query))?.data || page.snapshot })).filter(page => page.data))
const oldestSnapshot = computed(() => snapshots.value[0]?.data)
const isCached = computed(() => pages.value.some(page => { const view = remoteViews.get(JSON.stringify(page.query)); return view?.cached || !view && page.snapshot }))
const serverExecution = computed(() => remoteItems(remoteViews.get(JSON.stringify({ kind: 'active_executions' }))?.data).find(item => item.threadId === selection.value?.threadId && ['running', 'paused'].includes(item.state)))
const busy = computed(() => !!selection.value && (!!serverExecution.value || sending.value?.threadId === selection.value.threadId || [...remoteExecutions.values()].some(item => item.threadId === selection.value.threadId && item.state === 'observing')))
const canSend = computed(() => online.value && verified.value && agent.value?.isEnabled !== false && !controlsBusy.value && !sending.value && !changingInteraction.value && !uploading.value && !loading.value && !!draft.value.trim())
const visibleAgents = computed(() => { const keyword = agentSearch.value.trim().toLowerCase(); return remoteState.agents.filter(item => !keyword || `${item.name} ${item.id}`.toLowerCase().includes(keyword)).slice(0, 24) })
const threads = computed(() => remoteItems(threadQuery.value && remoteViews.get(JSON.stringify(threadQuery.value))?.data))
const moreThreads = computed(() => { const data = threadQuery.value && remoteViews.get(JSON.stringify(threadQuery.value))?.data; return typeof data?.total === 'number' ? threadPage.value * 30 < data.total : threads.value.length >= 30 })
const pendingForThread = computed(() => [...remotePendingRequests.values()].filter(item => item.state === 'unknown' && item.context?.input?.threadId === selection.value?.threadId))
const inboxCount = computed(() => remoteNotifications.size + [...remotePendingRequests.values()].filter(item => item.state === 'unknown').length + ['approvals', 'questions'].reduce((sum, kind) => sum + remoteItems(remoteViews.get(JSON.stringify({ kind }))?.data).length, 0))
const needsConnectionSettings = computed(() => ['auth_required', 'incompatible'].includes(remoteState.connection))
const connectionLabel = computed(() => ({ auth_required: '设备授权需要更新', incompatible: '当前实例或协议需要处理', reconnecting: '正在重新连接 infOS', connecting: '正在连接你的 infOS', offline: '暂时离线，当前空间仍保留', online: '正在同步角色与会话' }[remoteState.connection] || '正在连接 infOS'))
const resourceInput = computed(() => resource.value)
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
  const output = [...descriptors.values()].map(surface => ({ surface, message: messages.get(surface.messageId), role: messages.get(surface.messageId)?.role || 'assistant', timestamp: messages.get(surface.messageId)?.timestamp, cursor: cursors.get(surface.surfaceId) || null }))
  return [...output.filter(row => row.surface.messageId || row.surface.state === 'open'), ...output.filter(row => !row.surface.messageId && row.surface.state !== 'open')]
})
function guard(request = version) { assertToken(token, 'remote'); if (disposed || request !== version) throw new Error('会话已切换') }
async function loadCharacterAppearance() {
  const id = agent.value?.id, request = version
  if (!id) return
  Object.assign(appearance, defaultCharacterAppearance())
  try {
    const saved = await readPresentation({ kind: 'character', agentId: id }, token); guard(request)
    if (agent.value?.id !== id) return
    Object.assign(appearance, defaultCharacterAppearance(), saved || {}, { expanded: false })
    await Promise.allSettled([readRemote({ kind: 'agent', id }), readRemote({ kind: 'agent_state', id, threadId: selection.value?.agentId === id ? selection.value.threadId : null })])
  } catch (failure) { if (!disposed && request === version) storageError.value = failure.message }
}
async function saveCharacter(value) {
  const id = agent.value?.id, request = version
  if (!id) throw new Error('请先选择角色')
  const saved = await writePresentation({ kind: 'character', agentId: id }, value, token)
  guard(request)
  if (agent.value?.id !== id) throw new Error('角色已切换，外观已保存到原角色')
  Object.assign(appearance, saved.data)
}
function changeDisplay({ key, value }) {
  if (!['theme', 'palette', 'showCg', 'reduceMotion', 'roleStyle'].includes(key)) return
  display[key] = value
  persistTheme()
}
async function setInteraction(mode) {
  if (!selection.value || changingInteraction.value || busy.value || !online.value || !verified.value) return
  const request = version, id = selection.value.threadId
  changingInteraction.value = true; error.value = ''
  try { await performRemote({ kind: 'interaction_mode', id, mode }); guard(request); await readRemote({ kind: 'thread', id }, { allowCached: false }); guard(request) }
  catch (failure) { if (!disposed && request === version) { error.value = failure.message; verified.value = false; verifySelected() } }
  finally { if (!disposed) changingInteraction.value = false }
}
async function dispatchTask() {
  if (dispatching.value || dispatchUnknown.value || !online.value || !agent.value?.id || !taskInstruction.value.trim()) return
  const request = version, id = agent.value.id
  dispatching.value = true; error.value = ''
  try {
    dispatchRequestId.value ||= crypto.randomUUID()
    await performRemote({ kind: 'dispatch_task', agentId: id, instruction: taskInstruction.value.trim(), title: taskTitle.value.trim() || null, targetThreadId: selection.value?.threadId || null }, { requestId: dispatchRequestId.value }); guard(request)
    dispatchRequestId.value = null
    taskTitle.value = ''; taskInstruction.value = ''; drawer.value = null; await changeTab('tasks')
  } catch (failure) {
    if (!disposed && request === version) {
      error.value = failure.message
      dispatchUnknown.value = !['not_sent', 'rejected', 'failed'].includes(failure.outcome)
      if (!dispatchUnknown.value) dispatchRequestId.value = null
    }
  }
  finally { if (!disposed) dispatching.value = false }
}
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
  const current = { ...reading, workspaceContext: workspaceContext.value, draft: draft.value, attachments: attachments.value.map(item => ({ id: item.id, name: item.name })) }
  try { await writePresentation(sessionScope(), current, token); guard(); storageError.value = '' }
  catch (failure) { if (!disposed) storageError.value = `草稿或阅读位置尚未保存：${failure.message}`; throw failure }
}
async function prepareModeSwitch() {
  await manager.value?.prepareLeave?.()
  if (uploading.value || dispatching.value || mutating.value || changingInteraction.value) throw new Error('当前操作尚未完成，请稍后再切换运行模式')
  if (taskInstruction.value.trim() && !dispatchUnknown.value) throw new Error('还有未提交的任务说明，请先提交或清空后再切换')
  await persistCurrent()
  if (themeReady.value) { await persistTheme(); if (themeError.value) throw new Error(themeError.value) }
  guard()
}
function persistLater() { clearTimeout(draftTimer); draftTimer = setTimeout(() => persistCurrent().catch(() => {}), 280) }
function retryPersist() { persistCurrent().catch(() => {}) }
function draftChanged() { reading.editId = crypto.randomUUID(); reading.sendRequestId = null; persistLater(); if (composer.value) { composer.value.style.height = 'auto'; composer.value.style.height = `${Math.min(composer.value.scrollHeight, 132)}px` } }
function onScroll() { if (suppressScroll) return; readingVersion++; captureReading(); clearTimeout(scrollTimer); scrollTimer = setTimeout(() => persistCurrent().catch(() => {}), 450) }

async function restoreScroll(saved = reading) {
  const request = version, restore = ++scrollRestore
  suppressScroll = true
  await nextTick()
  if (disposed || request !== version || restore !== scrollRestore) { if (restore === scrollRestore) suppressScroll = false; return }
  const list = transcript.value
  if (list) {
    if (saved.atEnd) list.scrollTop = list.scrollHeight
    else {
      const anchor = [...list.querySelectorAll('[data-anchor]')].find(element => element.dataset.anchor === saved.anchor)
      if (anchor) list.scrollTop += anchor.getBoundingClientRect().top - list.getBoundingClientRect().top - saved.offset
      else if (rows.value.length) error.value = '上次阅读的消息暂未找到，可以继续加载历史或回到最新。'
    }
  }
  await nextTick(); if (restore === scrollRestore) suppressScroll = false
}
async function selectConversation(item) {
  if (!item.threadId || !item.agentId) return
  try { await manager.value?.prepareLeave?.() } catch (failure) { error.value = failure.message; return }
  if (uploading.value) { error.value = '附件正在上传，完成后再切换会话'; return }
  inputCapabilities.value = null
  const request = ++version
  const promise = (async () => {
    if (selection.value) await persistCurrent()
    guard(request)
    selection.value = { agentId: item.agentId, threadId: item.threadId, title: item.title || '继续对话' }; tab.value = 'chat'; drawer.value = null; detail.value = null; verified.value = false; sessionReady.value = false; loading.value = true; loadingHistory.value = false; error.value = ''; pages.value = []; draft.value = ''; attachments.value = []; workspaceContext.value = null
    Object.assign(appearance, defaultCharacterAppearance())
    try {
      const [saved, character] = await Promise.all([readPresentation(sessionScope(item), token), readPresentation({ kind: 'character', agentId: item.agentId }, token)])
      guard(request)
      reading = saved || emptySession(); draft.value = reading.draft; attachments.value = reading.attachments || []; workspaceContext.value = reading.workspaceContext || null; atEnd.value = reading.atEnd; historyWindow.value = !!reading.cursor; sessionReady.value = true
      Object.assign(appearance, defaultCharacterAppearance(), character || {}, { expanded: false })
      await writePresentation({ kind: 'home' }, selection.value, token); guard(request)
      const currentQuery = query(reading.atEnd ? null : reading.cursor)
      pages.value = [{ query: currentQuery, snapshot: null }]
      const cached = await readRemote(currentQuery, { cachedOnly: true }); guard(request)
      if (cached) { pages.value[0].snapshot = cached; await restoreScroll({ ...reading }) }
      if (remoteState.initializing) { await initializeRemote(); guard(request) }
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
      readRemote({ kind: 'agent_state', id: item.agentId, threadId: item.threadId }).catch(() => {})
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
async function changeTab(value) { const request = version; captureReading(); try { if (tab.value === 'manage' && value !== 'manage') await manager.value?.prepareLeave?.(); await persistCurrent(); guard(request); tab.value = value; if (value === 'manage') managementSection.value = null; if (value === 'chat') { await restoreScroll(reading); guard(request); if (online.value) refreshConversation() } } catch (failure) { if (!disposed) error.value = failure.message } }
async function openManagement(section = null) { try { await manager.value?.prepareLeave?.() } catch (failure) { error.value = failure.message; return }; await changeTab('manage'); if (tab.value === 'manage') managementSection.value = section }
async function useWorkbenchContext(input) {
  const request = version
  try {
    if (input.threadId !== selection.value?.threadId || input.agentId !== selection.value?.agentId) throw new Error('工作台引用与当前会话不匹配')
    await manager.value?.prepareLeave?.(); guard(request)
    workspaceContext.value = { ...(workspaceContext.value || {}), ...input.workspaceContext }
    draftChanged(); await persistCurrent(); guard(request); await changeTab('chat')
  } catch (failure) { if (!disposed) error.value = failure.message }
}
async function quickNewConversation() { pickerAgent.value = selection.value?.agentId || remoteState.activeAgentId; await createConversation(); if (threadError.value) error.value = threadError.value }
async function conversationUpdated(change) { if (change?.kind === 'rewind') await latest(); else await refreshConversation(); await composerTools.value?.refresh?.() }
function applyTranscript(text) { if (!selection.value || !sessionReady.value) return; const next = [draft.value.trimEnd(), String(text).trim()].filter(Boolean).join('\n'); if (next.length > 131072) { error.value = '文字过长，请缩短后再加入'; return }; draft.value = next; draftChanged(); drawer.value = null }
async function restoreRegeneration(recovery) {
  const request = version
  try {
    guard(request)
    if (recovery.threadId !== selection.value?.threadId || recovery.agentId !== selection.value?.agentId) throw new Error('恢复记录不属于当前会话')
    if (draft.value.trim() || attachments.value.length) throw new Error('输入框已有草稿，请先发送或清空，避免覆盖')
    draft.value = recovery.input.content
    attachments.value = (recovery.input.attachments || []).map(file => ({ id: file.id, name: file.name || file.originalName || '附件' }))
    workspaceContext.value = null; draftChanged(); await persistCurrent(); guard(request); await composerTools.value?.clearRecovery?.()
  } catch (failure) { if (!disposed && request === version) error.value = failure.message }
}
async function showManagementPending() {
  try { await refreshRemotePendingRequests(); guard(); const item = [...remotePendingRequests.values()].find(item => item.state === 'unknown'); if (item) { pending.value = item; drawer.value = 'pending' } else error.value = '暂无待核对操作，可刷新对应管理页面查看服务端状态' }
  catch (failure) { if (!disposed) error.value = failure.message }
}
async function send() {
  if (!canSend.value) return
  const request = version, item = { ...selection.value }, content = draft.value, files = attachments.value.map(file => ({ ...file })), imageRecognition = imageMode.value, workspaceReference = workspaceContext.value ? JSON.parse(JSON.stringify(workspaceContext.value)) : null
  error.value = ''
  const observationId = crypto.randomUUID(), editId = reading.editId || crypto.randomUUID()
  sending.value = { observationId, threadId: item.threadId, content, attachments: files, phase: 'preparing' }
  reading.editId = editId; reading.sendRequestId = observationId
  try {
    await persistCurrent(); guard(request)
    if (historyWindow.value) await latest()
    guard(request); if (sending.value?.observationId === observationId) sending.value.phase = 'observing'
    await sendRemoteChat({ threadId: item.threadId, agentId: item.agentId, content, attachmentIds: files.map(file => file.id), imageMode: imageRecognition, workspaceContext: workspaceReference }, { requestId: observationId })
    assertToken(token, 'remote')
    const saved = await readPresentation(sessionScope(item), token)
    assertToken(token, 'remote')
    if (!disposed && selection.value?.threadId === item.threadId && reading.editId === editId && saved && saved.sendRequestId === null) { reading = saved; draft.value = saved.draft; attachments.value = saved.attachments; workspaceContext.value = saved.workspaceContext || null }
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
async function openResource(value) {
  const request = version
  try {
    const file = await resolveRemoteResource(value, { agentId: selection.value?.agentId, threadId: selection.value?.threadId }, token)
    guard(request)
    if (file.kind === 'knowledge') { detail.value = { kind: 'document', query: { kind: 'document', id: file.id }, title: file.name || '文档' }; return }
    resource.value = { ...file, agentId: file.agentId || selection.value?.agentId, threadId: file.threadId || selection.value?.threadId }
  } catch (failure) { if (!disposed && request === version) error.value = failure.message }
}
let contentFrame = null
function contentResized() {
  if (contentFrame !== null || disposed || tab.value !== 'chat') return
  contentFrame = requestAnimationFrame(() => {
    contentFrame = null
    if (disposed || tab.value !== 'chat' || loading.value || !transcript.value) return
    // 异步排版与媒体加载也遵循阅读位置；不能在用户翻历史时抢走滚动。
    if (atEnd.value && !historyWindow.value) transcript.value.scrollTop = transcript.value.scrollHeight
    else if (reading.anchor) {
      const anchor = [...transcript.value.querySelectorAll('[data-anchor]')].find(element => element.dataset.anchor === reading.anchor)
      if (anchor) transcript.value.scrollTop += anchor.getBoundingClientRect().top - transcript.value.getBoundingClientRect().top - reading.offset
    }
  })
}
async function fromReference(value) { if (!value.threadId || !value.agentId) { error.value = '此事项没有可打开的会话'; return }; detail.value = null; await selectConversation(value) }
async function renameConversation() { if (!selection.value || mutating.value) return; const request = version, item = { ...selection.value }, title = renameTitle.value.trim(); mutating.value = true; try { await performRemote({ kind: 'rename_thread', id: item.threadId, title }); guard(request); selection.value.title = title; await writePresentation({ kind: 'home' }, selection.value, token); guard(request); drawer.value = null } catch (failure) { if (!disposed && request === version) error.value = failure.message } finally { if (!disposed) mutating.value = false } }
async function deleteConversation() { if (!selection.value || mutating.value) return; const request = version, item = { ...selection.value }; mutating.value = true; try { await persistCurrent(); guard(request); await performRemote({ kind: 'delete_thread', id: item.threadId }); guard(request); await writePresentation({ kind: 'home' }, null, token); guard(request); selection.value = null; sessionReady.value = false; pages.value = []; draft.value = ''; attachments.value = []; workspaceContext.value = null; drawer.value = null; await showThreads() } catch (failure) { if (!disposed && request === version) error.value = failure.message } finally { if (!disposed) mutating.value = false } }
async function reconnect() { if (reconnecting.value) return; const request = version; reconnecting.value = true; try { await reconnectRemote(); guard(request); if (selection.value && !opening) await selectConversation({ ...selection.value }); else if (drawer.value === 'threads') await loadThreads() } catch (failure) { if (!disposed && request === version) error.value = failure.message } finally { if (!disposed) reconnecting.value = false } }
async function reconcile() { reviewing.value = true; try { await reconcileRemotePendingRequests(); guard(); if (selection.value) await refreshConversation() } catch (failure) { if (!disposed) error.value = failure.message } finally { if (!disposed) reviewing.value = false } }
async function review() {
  if (!pending.value || !reviewed.value) return
  const requestId = pending.value.requestId
  reviewing.value = true
  try {
    await reviewRemoteRequest(requestId, reviewNote.value, { confirmed: true }); guard()
    if (requestId === dispatchRequestId.value) { dispatchUnknown.value = false; dispatchRequestId.value = null; taskTitle.value = ''; taskInstruction.value = '' }
    drawer.value = null; reviewed.value = false; reviewNote.value = ''
  } catch (failure) { if (!disposed) error.value = failure.message }
  finally { if (!disposed) reviewing.value = false }
}
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
let openingNotice = false
async function consumeNotificationOpen() {
  if (disposed || document.hidden || remoteState.restoringView || openingNotice) return
  openingNotice = true
  const request = version
  try {
    const result = await remoteClient.consumeNotificationOpen(token); guard(request)
    if (!result.data) return
    await manager.value?.prepareLeave?.(); guard(request)
    await persistCurrent(); guard(request)
    drawer.value = null; detail.value = null; resource.value = null; tab.value = 'inbox'
    await refreshRemoteNotifications(); guard(request)
    await nextTick(); guard(request)
    document.getElementById(`remote-notice-${JSON.stringify([result.data.principalId, result.data.notificationId])}`)?.scrollIntoView({ block: 'center' })
  } catch (failure) { if (!disposed && request === version) error.value = failure.message }
  finally { openingNotice = false }
}
function resume() { if (!document.hidden && !disposed) { resumeRemote().catch(() => {}); consumeNotificationOpen() } }
function saveOnHide() { persistCurrent().catch(() => {}) }
function visibility() { if (document.hidden) saveOnHide(); else resume() }
async function warmInbox() { try { guard(); await Promise.allSettled(['approvals', 'questions', 'tools', 'active_executions'].map(kind => readRemote({ kind }))) } catch { /* 连接提示保留在主界面，列表本身保留各自的读取错误。 */ } }

watch(() => remoteState.activeAgentId, () => { if (!selection.value && themeReady.value) loadCharacterAppearance() })
watch(imageModeOptions, options => { if (options.find(item => item.value === imageMode.value)?.disabled) imageMode.value = 'auto' })
watch(() => pending.value?.requestId, () => { reviewed.value = false; reviewNote.value = '' })
watch(drawer, value => { if (value !== 'pending') { reviewed.value = false; reviewNote.value = '' } })
watch(() => [...remotePendingRequests.keys()].join('|'), async () => {
  if (!sessionReady.value || !selection.value || !reading.sendRequestId || remotePendingRequests.has(reading.sendRequestId)) return
  const request = version, editId = reading.editId
  try {
    const saved = await readPresentation(sessionScope(), token); guard(request)
    if (saved && reading.editId === editId && saved.editId !== editId && saved.sendRequestId === null) { reading = saved; draft.value = saved.draft; attachments.value = saved.attachments; workspaceContext.value = saved.workspaceContext || null }
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
  remoteState.restoringView = true
  document.addEventListener('visibilitychange', visibility)
  window.addEventListener('online', resume); window.addEventListener('focus', resume); window.addEventListener('pageshow', resume); window.addEventListener('pagehide', saveOnHide)
  const request = version
  try {
    const [profiles, home] = await Promise.all([remoteConnections.list(), readPresentation({ kind: 'home' }, token), restoreTheme()])
    assertToken(token, 'remote'); if (disposed) return
    profile.value = profiles.find(item => item.id === token.target.profileId)
    guard(request)
    if (home) await selectConversation(home); else { tab.value = 'home'; pickerAgent.value = remoteState.activeAgentId || remoteState.agents[0]?.id || null; await loadCharacterAppearance() }
    if (online.value) warmInbox()
  } catch (failure) { if (!disposed && request === version) error.value = failure.message }
  finally { if (!disposed) { remoteState.restoringView = false; consumeNotificationOpen() } }
})
onBeforeUnmount(() => {
  if (contentFrame !== null) cancelAnimationFrame(contentFrame)
  clearTimeout(draftTimer); clearTimeout(scrollTimer); clearTimeout(verificationTimer); clearTimeout(projectionTimer)
  if (selection.value && sessionReady.value) { captureReading(); writePresentation(sessionScope(), { ...reading, workspaceContext: workspaceContext.value, draft: draft.value, attachments: attachments.value.map(file => ({ id: file.id, name: file.name })) }, token).catch(() => {}) }
  disposed = true; version++; listVersion++; document.removeEventListener('visibilitychange', visibility)
  window.removeEventListener('online', resume); window.removeEventListener('focus', resume); window.removeEventListener('pageshow', resume); window.removeEventListener('pagehide', saveOnHide)
})
</script>
