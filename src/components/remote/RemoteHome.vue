<template>
  <section class="remote-home">
    <div class="remote-home-hero" :class="{ 'avatar-only': !cg || !showCg }">
      <button v-if="cg && showCg" class="remote-hero-art" :aria-label="`展开${agent?.name || '角色'}的 CG`" @click="$emit('art')"><img :src="cg" :alt="`${agent?.name || '角色'}的 CG`" @error="imageFailed = true" /></button>
      <div v-else class="remote-home-avatar"><RemoteAvatar :id="agent?.id" :label="agent?.name" /><RemoteIcon :name="decoration" artwork /></div>
      <div class="remote-home-connection"><button @click="$emit('connections')"><strong>{{ profile?.displayName || 'infOS' }}</strong><small><i :class="{ online }" />{{ connectionText }}</small></button><button class="room-icon-button" aria-label="显示偏好" @click="$emit('appearance')"><RemoteIcon name="settings" /></button></div>
      <div class="remote-home-greeting"><span>{{ agent?.name || '你的伙伴' }}<RemoteIcon v-if="decoration !== 'none'" :name="decoration" artwork /></span><p>{{ state?.vibe || state?.mood || agent?.description || '从这里，继续上次的对话。' }}</p></div>
    </div>
    <div class="remote-home-content">
      <ModeSwitchEntry remote class="remote-home-mode-entry" @open="$emit('connections')" />
      <button class="remote-resume-card" @click="$emit(selection ? 'resume' : 'threads')"><RemoteIcon class="remote-resume-art" name="landscape" artwork /><span class="remote-eyebrow"><RemoteIcon name="diamond" />{{ selection ? '继续上次会话' : '开始一段对话' }}</span><h2>{{ selection?.title || '选择你的伙伴' }}</h2><p>{{ selection ? '会话、任务和记忆，与你连接的 infOS 同步。' : '选择角色和会话，把熟悉的陪伴带在身边。' }}</p><span class="remote-resume-arrow"><RemoteIcon name="arrow" /></span></button>
      <RemoteFeedback :message="error" action-label="重试" @action="reload" /><button v-if="error" class="room-text-button" @click="reload">重新加载首页</button>
      <div class="remote-home-section-title"><h3><RemoteIcon name="diamond" />进行中的任务</h3><button class="room-text-button" @click="$emit('tasks')">查看全部<RemoteIcon name="arrow" /></button></div>
      <p v-if="loading && !tasks.length" class="remote-help">正在读取任务…</p><p v-else-if="!tasks.length" class="remote-help">{{ online ? '现在没有进行中的任务。' : '连上 infOS 后，就能看到任务进展。' }}</p>
      <button v-for="task in tasks" :key="task.id" class="remote-home-task" @click="$emit('detail', { title: task.title || '任务详情', query: { kind: 'task_projection', id: task.id }, taskId: task.id })"><span class="remote-task-icon"><RemoteIcon name="tasks" /></span><span><strong>{{ task.title || '远程任务' }}</strong><small>{{ remoteStatus(task.status) }}</small><span v-if="typeof task.currentStage === 'string' && task.currentStage" class="remote-task-stage">{{ task.currentStage }}</span></span><RemoteIcon name="arrow" /></button>
      <div class="remote-home-section-title"><h3><RemoteIcon name="diamond" />会话里的文件</h3><button class="room-text-button" @click="$emit('library')">查看全部<RemoteIcon name="arrow" /></button></div>
      <button v-for="file in files" :key="file.path" class="remote-home-file" @click="$emit('detail', { title: file.name, query: { kind: 'workspace_file', agentId: selection.agentId, threadId: selection.threadId, path: file.path } })"><RemoteFileCover :name="file.name" /><span><strong>{{ file.name }}</strong><small>保存在当前 infOS</small></span><RemoteIcon name="open" /></button>
      <p v-if="!files.length" class="remote-help">{{ selection ? '会话中的文件会收在这里。' : '选择会话后，可以查看它的文件。' }}</p>
      <button class="remote-home-inbox" @click="$emit('inbox')"><RemoteIcon name="bell" /><span>通知与待处理事项</span><b v-if="inboxCount">{{ inboxCount > 99 ? '99+' : inboxCount }}</b><RemoteIcon name="arrow" /></button>
      <div class="remote-home-signature"><RemoteIcon v-if="decoration !== 'none'" :name="decoration" artwork /><small>同一个世界，随时相见。</small></div>
    </div>
  </section>
</template>
<script setup>
import RemoteFeedback from './RemoteFeedback.vue'
import { computed, ref, watch, onBeforeUnmount } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
import ModeSwitchEntry from '../ModeSwitchEntry.vue'
import RemoteAvatar from './RemoteAvatar.vue'
import RemoteFileCover from './RemoteFileCover.vue'
import { readRemote, remoteViews, remoteState } from '../../stores/remote'
import { currentToken, assertToken } from '../../stores/mode'
import { activeTaskStatuses, newestTasks } from '../../utils/taskCenter'
import { remoteItems, remoteStatus } from '../../utils/remoteMarkdown'
const props = defineProps({ agent: Object, state: Object, profile: Object, selection: Object, image: String, showCg: Boolean, decoration: { type: String, default: 'ribbon' }, inboxCount: Number })
defineEmits(['art', 'connections', 'appearance', 'resume', 'threads', 'tasks', 'library', 'inbox', 'detail'])
const token = currentToken(), loading = ref(false), error = ref(''), queries = ref([]), imageFailed = ref(false)
let disposed = false, version = 0
const online = computed(() => remoteState.connection === 'online')
const connectionText = computed(() => remoteState.initializing ? '正在连接' : ({ online: '已连接', auth_required: '需要重新授权', incompatible: '需要检查连接', reconnecting: '正在重连', connecting: '正在连接' }[remoteState.connection] || '暂时离线'))
const cg = computed(() => imageFailed.value ? '' : props.image)
const tasks = computed(() => newestTasks(queries.value.filter(query => query.kind === 'tasks').flatMap(query => remoteItems(remoteViews.get(JSON.stringify(query))?.data))).filter(task => task.category === 'agent_task' && activeTaskStatuses.includes(task.status)).slice(0, 3))
const files = computed(() => { const query = queries.value.find(query => query.kind === 'workspace_tree'), data = query && remoteViews.get(JSON.stringify(query))?.data; return remoteItems(data?.nodes || data).filter(file => file.type !== 'directory').slice(0, 3) })
async function reload() {
  const request = ++version; loading.value = true; error.value = ''
  const next = activeTaskStatuses.map(status => ({ kind: 'tasks', category: 'agent_task', agentId: props.agent?.id || null, status, page: 1, pageSize: 3 }))
  if (props.selection) next.push({ kind: 'workspace_tree', agentId: props.selection.agentId, threadId: props.selection.threadId, path: '.' })
  queries.value = next
  try { const results = await Promise.allSettled(next.map(query => readRemote(query))); assertToken(token, 'remote'); if (!disposed && request === version) error.value = results.filter(result => result.status === 'rejected').map(result => result.reason.message).join('；') }
  catch (failure) { if (!disposed && request === version) error.value = failure.message }
  finally { if (!disposed && request === version) loading.value = false }
}
watch(() => [props.agent?.id, props.selection?.threadId], reload, { immediate: true })
watch(() => props.image, () => { imageFailed.value = false })
onBeforeUnmount(() => { disposed = true; version++ })
</script>
