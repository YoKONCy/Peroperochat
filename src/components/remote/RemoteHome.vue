<template>
  <section class="remote-home">
    <div class="remote-home-hero" :class="{ 'avatar-only': !cg || !showCg }">
      <button v-if="cg && showCg" class="remote-hero-art" :aria-label="t('展开{0}的 CG', [agent?.name || t('角色')])" @click="$emit('art')"><img :src="cg" :alt="t('{0}的 CG', [agent?.name || t('角色')])" @error="imageFailed = true" /></button>
      <div v-else class="remote-home-avatar"><RemoteAvatar :id="agent?.id" :label="agent?.name" /><RemoteIcon :name="decoration" artwork /></div>
      <div class="remote-home-connection"><button @click="$emit('connections')"><strong>{{ profile?.displayName || 'infOS' }}</strong><small><i :class="{ online }" />{{ connectionText }}</small></button><button class="room-icon-button" :aria-label="t('显示偏好')" @click="$emit('appearance')"><RemoteIcon name="settings" /></button></div>
      <div class="remote-home-greeting"><span>{{ agent?.name || t('你的伙伴') }}<RemoteIcon v-if="decoration !== 'none'" :name="decoration" artwork /></span><p>{{ state?.vibe || state?.mood || agent?.description || t('从这里，继续上次的对话。') }}</p></div>
    </div>
    <div class="remote-home-content">
      <button class="remote-resume-card" @click="$emit(selection ? 'resume' : 'threads')"><RemoteIcon class="remote-resume-art" name="chat" /><span class="remote-eyebrow">{{ t(selection ? '继续上次会话' : '开始一段对话') }}</span><h2>{{ selection?.title || t('选择你的伙伴') }}</h2><span class="remote-resume-arrow"><RemoteIcon name="arrow" /></span></button>
      <!-- 待处理事项紧跟主操作，不再排在任务与文件之后。 -->
      <button class="remote-home-inbox" @click="$emit('inbox')"><RemoteIcon name="bell" /><span>{{ t('通知与待处理事项') }}</span><b v-if="inboxCount">{{ inboxCount > 99 ? '99+' : inboxCount }}</b><RemoteIcon name="arrow" /></button>
      <div class="renewal-home-shortcuts"><button type="button" data-ui-tone="rose" @click="$emit('groups')"><RemoteIcon name="people" /><span>{{ tr('一起聊') }}</span></button><button type="button" data-ui-tone="violet" @click="$emit('memories')"><RemoteIcon name="memory" /><span>{{ tr('记住的小事') }}</span></button><button type="button" data-ui-tone="blue" @click="$emit('library')"><RemoteIcon name="folder" /><span>{{ tr('找文件') }}</span></button></div>
      <RemoteFeedback :message="error" :action-label="t('重试')" @action="reload" /><button v-if="error" class="room-text-button" @click="reload">{{ t('重新加载首页') }}</button>
      <div class="remote-home-section-title"><h3><RemoteIcon name="diamond" />{{ t('进行中的任务') }}</h3><button class="room-text-button" @click="$emit('tasks')">{{ t('查看全部') }}<RemoteIcon name="arrow" /></button></div>
      <p v-if="loading && !tasks.length" class="remote-help">{{ t('正在读取任务…') }}</p><p v-else-if="!tasks.length" class="remote-help">{{ online ? t('现在没有进行中的任务。') : t('连上 infOS 后，就能看到任务进展。') }}</p>
      <button v-for="task in tasks" :key="task.id" class="remote-home-task" @click="$emit('detail', { title: task.title || t('任务详情'), query: { kind: 'task_projection', id: task.id }, taskId: task.id })"><span class="remote-task-icon"><RemoteIcon name="tasks" /></span><span><strong>{{ task.title || t('远程任务') }}</strong><small>{{ remoteStatus(task.status) }}</small><span v-if="typeof task.currentStage === 'string' && task.currentStage" class="remote-task-stage">{{ task.currentStage }}</span></span><RemoteIcon name="arrow" /></button>
      <div class="remote-home-section-title"><h3><RemoteIcon name="diamond" />{{ t('会话里的文件') }}</h3><button class="room-text-button" @click="$emit('library')">{{ t('查看全部') }}<RemoteIcon name="arrow" /></button></div>
      <button v-for="file in files" :key="file.path" class="remote-home-file" @click="$emit('detail', { title: file.name, query: { kind: 'workspace_file', agentId: selection.agentId, threadId: selection.threadId, path: file.path } })"><RemoteFileCover :name="file.name" /><span><strong>{{ file.name }}</strong><small>{{ t('保存在当前 infOS') }}</small></span><RemoteIcon name="open" /></button>
      <p v-if="!files.length" class="remote-help">{{ selection ? t('会话中的文件会收在这里。') : t('选择会话后，可以查看它的文件。') }}</p>
      <!-- 运行模式切换属于低频操作，放在页面末尾。 -->
      <ModeSwitchEntry remote class="remote-home-mode-entry" @open="$emit('connections')" />
      <div class="remote-home-signature"><RemoteIcon v-if="decoration !== 'none'" :name="decoration" artwork /><small>{{ t('同一个世界，随时相见。') }}</small></div>
    </div>
  </section>
</template>
<script setup>
import RemoteFeedback from './RemoteFeedback.vue'
import { t } from '../../i18n/remote'
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
defineEmits(['art', 'connections', 'appearance', 'resume', 'threads', 'groups', 'memories', 'tasks', 'library', 'inbox', 'detail'])
const tr = key => t(key, [], 'renewal')
const token = currentToken(), loading = ref(false), error = ref(''), queries = ref([]), imageFailed = ref(false)
let disposed = false, version = 0
const online = computed(() => remoteState.connection === 'online')
// 与工作区连接提示使用同一套说法。
const connectionText = computed(() => t(remoteState.initializing ? '正在连接' : ({ online: '已连接', auth_required: '需要重新登录', incompatible: '暂时无法连接', reconnecting: '正在重连', connecting: '正在连接' }[remoteState.connection] || '暂时离线')))
const cg = computed(() => imageFailed.value ? '' : props.image)
const tasks = computed(() => newestTasks(queries.value.filter(query => query.kind === 'tasks').flatMap(query => remoteItems(remoteViews.get(JSON.stringify(query))?.data))).filter(task => task.category === 'agent_task' && activeTaskStatuses.includes(task.status)).slice(0, 3))
const files = computed(() => { const query = queries.value.find(query => query.kind === 'workspace_tree'), data = query && remoteViews.get(JSON.stringify(query))?.data; return remoteItems(data?.nodes || data).filter(file => file.type !== 'directory').slice(0, 3) })
async function reload() {
  const request = ++version; loading.value = true; error.value = ''
  /** @type {import('../../types/ui').DataRecord[]} */
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
