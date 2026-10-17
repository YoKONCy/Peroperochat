<template>
  <div class="group-space companion-surface" :data-tone="tone" :style="viewportStyle">
    <div class="group-scenery" aria-hidden="true"><img :src="`/rooms/${tone}.svg`" alt="" /><div /></div>
    <div class="group-main" :inert="showMembers ? true : undefined">
      <CompanionHeader title="家庭群聊" kicker="大家的小客厅" :show-partner="false" back-label="回到房间" @back="router.push('/')">
        <button class="room-icon-button" aria-label="设置成员与回应顺序" :aria-expanded="showMembers" :disabled="pageLoading || !membersReady || busy" @click="openMembers"><RoomIcon name="sliders" /></button>
      </CompanionHeader>

      <section class="group-roster" aria-label="参与群聊的伙伴">
        <div class="roster-avatars"><CharacterAvatar v-for="member in enabledMembers" :key="member.id" :src="agents[member.id]?.avatar" :label="member.name" :data-tone="member.id === 'nana' ? 'nana' : 'pero'" /></div>
        <div class="roster-caption"><strong>{{ enabledMembers.length ? enabledMembers.map(member => member.name).join('、') : '还没有参与的伙伴' }}</strong><span>{{ isLoading || submitting ? '正在聊着呢' : '按成员顺序，一起聊聊今天' }}</span></div>
        <button class="room-text-button roster-edit" :disabled="pageLoading || !membersReady || busy" @click="openMembers">成员<RoomIcon name="arrow" /></button>
      </section>

      <p v-if="loadError" class="room-error group-error" role="alert">{{ loadError }} <button class="room-text-button" :disabled="pageLoading || busy" @click="load">重新读取</button></p>
      <p v-if="actionError" class="room-error group-error" role="alert">{{ actionError }} <button class="room-icon-button" aria-label="关闭提示" @click="actionError = ''"><RoomIcon name="close" /></button></p>

      <main ref="chatList" class="group-timeline" aria-label="家庭群聊消息" @scroll="onScroll">
        <div class="group-messages">
          <button v-if="messages.length > displayCount" class="group-earlier room-text-button" @click="loadMore"><RoomIcon name="up" />查看更早的对话</button>
          <div v-if="!messages.length" class="group-empty">
            <div class="empty-portraits"><CharacterAvatar v-for="member in enabledMembers.slice(0, 3)" :key="member.id" :src="agents[member.id]?.avatar" :label="member.name" /><CompanionEmblem /></div>
            <h2>{{ pageLoading ? '大家正在过来…' : '在这里，和大家一起聊' }}</h2>
            <p>{{ enabledMembers.length ? '日常小事、突然的灵感，都可以说给大家听。' : '打开成员设置，邀请伙伴参与回应。' }}</p>
          </div>

          <article v-for="message in displayedMessages" :key="message.id" v-memo="[message.content, message.reasoning, message.status, message.error, message.name, message.timestamp, selectedId === message.id, busy, agents[message.agentId]?.avatar, agents[message.agentId]?.name]" class="group-message" :class="{ 'is-user': message.role === 'user' }" :data-speaker="message.agentId === 'nana' ? 'nana' : 'pero'">
            <CharacterAvatar v-if="message.role !== 'user'" class="group-message-avatar" :src="agents[message.agentId]?.avatar" :label="speakerName(message)" />
            <div class="group-message-column">
              <div class="group-byline"><strong>{{ speakerName(message) }}</strong><time>{{ formatTime(message.timestamp) }}</time><button class="room-icon-button" :aria-label="`操作${speakerName(message)}的消息`" :aria-expanded="selectedId === message.id" @click="selectedId = selectedId === message.id ? null : message.id"><RoomIcon name="more" /></button></div>
              <div class="group-bubble">
                <details v-if="message.reasoning" class="group-reasoning"><summary>{{ message.status === 'pending' ? '正在思考' : '思考过程' }}</summary><div>{{ message.reasoning }}</div></details>
                <div v-if="message.content === '__loading__'" class="group-typing" aria-label="正在思考"><i /><i /><i /></div>
                <div v-else class="markdown-body" v-html="roomMarkdown(message.content)" />
                <p v-if="message.error && visibleMessage(message.content) !== message.error" class="group-message-error">{{ message.error }}</p>
                <small v-if="['cancelled', 'interrupted'].includes(message.status)" class="group-message-status">{{ message.status === 'cancelled' ? '已停止' : '上次回复中断了' }}</small>
              </div>
              <Transition name="room-soft"><div v-if="selectedId === message.id" class="group-message-actions"><button @click="copyMessage(message)"><RoomIcon name="copy" />复制</button><button :disabled="busy" @click="deleteMessage(message.id)"><RoomIcon name="trash" />删除</button></div></Transition>
            </div>
          </article>
        </div>
      </main>

      <div class="group-dock">
        <button v-if="!following && messages.length" class="group-latest room-secondary-button" @click="scrollToLatest(true)"><RoomIcon name="down" />回到最新</button>
        <Transition name="room-soft"><div v-if="isLoading" class="group-activity" role="status"><CharacterAvatar v-if="processingAgent" :src="processingAgent.avatar" :label="processingAgent.name" /><span>{{ processingAgent?.name || '大家' }} · {{ activityText || '正在回应你…' }}</span><span class="group-typing" aria-hidden="true"><i /><i /><i /></span></div></Transition>
        <form class="group-composer" @submit.prevent="sendMessage">
          <CompanionEmblem class="composer-emblem" />
          <textarea ref="textarea" v-model="input" rows="1" maxlength="100000" placeholder="和大家说点什么…" aria-label="群聊消息" enterkeyhint="send" @input="resizeInput" @keydown="handleComposerKey" />
          <button v-if="isLoading" type="button" class="group-send is-stop" aria-label="停止本轮群聊" @click="conversation.stop"><RoomIcon name="stop" /></button>
          <button v-else class="group-send" :disabled="busy || pageLoading || !!loadError || !membersReady || !enabledMembers.length || !input.trim()" aria-label="发送群聊消息"><RoomIcon name="send" /></button>
        </form>
        <p class="group-composer-note"><span>{{ enabledMembers.length }} 位伙伴参与回应</span><span>Shift + Enter 换行</span></p>
      </div>
    </div>

    <RoomDrawer v-if="showMembers" title="客厅里的伙伴" subtitle="选择参与成员，调整回应顺序" :tone="tone" :busy="savingMembers" @close="showMembers = false">
      <p class="members-hint">从上到下依次回应。关闭开关后，伙伴仍保留在成员列表中。</p>
      <div class="group-member-list">
        <article v-for="(member, index) in memberDraft" :key="member.id" class="group-member" :class="{ 'is-paused': !member.enabled }">
          <span class="member-order">{{ String(index + 1).padStart(2, '0') }}</span><CharacterAvatar :src="agents[member.id]?.avatar" :label="member.name" />
          <div class="member-identity"><strong>{{ member.name }}</strong><small>{{ member.enabled ? '参与回应' : '暂时旁听' }}</small></div>
          <div class="member-move"><button class="room-icon-button" :disabled="savingMembers || index === 0" :aria-label="`上移${member.name}`" @click="moveMember(index, -1)"><RoomIcon name="up" /></button><button class="room-icon-button" :disabled="savingMembers || index === memberDraft.length - 1" :aria-label="`下移${member.name}`" @click="moveMember(index, 1)"><RoomIcon name="down" /></button></div>
          <label class="member-toggle"><input v-model="member.enabled" type="checkbox" role="switch" :disabled="savingMembers" :aria-label="`${member.name}参与回应`" /><span aria-hidden="true" /></label>
        </article>
      </div>
      <p v-if="membersError" class="room-error" role="alert">{{ membersError }}</p>
      <template #footer><div class="members-footer"><span>{{ memberDraft.filter(member => member.enabled).length }} 位伙伴参与</span><button class="room-primary-button" :disabled="savingMembers || !membersChanged" @click="saveMembers">{{ savingMembers ? '正在保存…' : '保存成员设置' }}</button></div></template>
    </RoomDrawer>
  </div>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import CompanionHeader from '../components/companion/CompanionHeader.vue'
import CharacterAvatar from '../components/companion/CharacterAvatar.vue'
import CompanionEmblem from '../components/companion/CompanionEmblem.vue'
import RoomDrawer from '../components/companion/RoomDrawer.vue'
import RoomIcon from '../components/companion/RoomIcon.vue'
import { useRoomViewport } from '../composables/useRoomViewport'
import { AGENTS as agents, activeAgentId } from '../stores/agents'
import { useLocalConversation } from '../stores/conversations'
import { localInvoke } from '../services/localChat'
import { roomMarkdown } from '../utils/roomMarkdown'
import { visibleMessage } from '../utils/messageContent'

const router = useRouter()
const { viewportStyle } = useRoomViewport()
const tone = computed(() => activeAgentId.value === 'nana' ? 'nana' : 'pero')
const conversation = useLocalConversation('group')
const { messages, isLoading, processingAgentId, activityText } = conversation
const members = ref([]), memberDraft = ref([]), showMembers = ref(false), savingMembers = ref(false), membersReady = ref(false)
const pageLoading = ref(false), submitting = ref(false), deleting = ref(false), loadError = ref(''), actionError = ref(''), membersError = ref('')
const input = ref(''), textarea = ref(null), chatList = ref(null), displayCount = ref(80), following = ref(true), selectedId = ref(null)
const busy = computed(() => isLoading.value || submitting.value || deleting.value)
const enabledMembers = computed(() => members.value.filter(member => member.enabled))
const processingAgent = computed(() => agents[processingAgentId.value])
const displayedMessages = computed(() => messages.value.slice(-displayCount.value))
const memberValues = list => list.map(({ id, enabled }) => ({ id, enabled }))
const membersChanged = computed(() => JSON.stringify(memberValues(memberDraft.value)) !== JSON.stringify(memberValues(members.value)))
let disposed = false, loadVersion = 0

function speakerName(message) { return message.role === 'user' ? '你' : agents[message.agentId]?.name || message.name || '伙伴' }
function formatTime(timestamp) { return new Date(timestamp).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }) }
async function load() {
  if (pageLoading.value) return
  const version = ++loadVersion
  pageLoading.value = true; loadError.value = ''; membersReady.value = false
  const results = await Promise.allSettled([conversation.load(), localInvoke('local_group_settings')])
  if (disposed || version !== loadVersion) return
  if (results[1].status === 'fulfilled') { members.value = results[1].value; membersReady.value = true }
  loadError.value = results.filter(result => result.status === 'rejected').map(result => result.reason.message || '群聊暂时无法读取').join('；')
  pageLoading.value = false
  await scrollToLatest()
}
function openMembers() {
  if (!membersReady.value || busy.value) return
  textarea.value?.blur(); memberDraft.value = members.value.map(member => ({ ...member })); membersError.value = ''; showMembers.value = true
}
function moveMember(index, offset) {
  const target = index + offset
  if (savingMembers.value || target < 0 || target >= memberDraft.value.length) return
  const reordered = [...memberDraft.value]
  const [member] = reordered.splice(index, 1)
  reordered.splice(target, 0, member)
  memberDraft.value = reordered
}
async function saveMembers() {
  if (savingMembers.value || !membersChanged.value) return
  savingMembers.value = true; membersError.value = ''
  try {
    // 前端只提交顺序与开关草稿，角色有效性和最终成员投影由 Rust 返回。
    const result = await localInvoke('local_group_settings', { members: memberValues(memberDraft.value) })
    if (!disposed) { members.value = result; showMembers.value = false }
  } catch (error) { if (!disposed) membersError.value = error.message || '成员设置没有保存成功，请重试' }
  finally { if (!disposed) savingMembers.value = false }
}
function resizeInput() {
  if (!textarea.value) return
  textarea.value.style.height = 'auto'
  textarea.value.style.height = `${Math.min(textarea.value.scrollHeight, 112)}px`
}
function handleComposerKey(event) {
  if (event.key !== 'Enter' || event.shiftKey || event.isComposing || event.keyCode === 229) return
  event.preventDefault(); sendMessage()
}
async function sendMessage() {
  const value = input.value.trim()
  if (!value || busy.value || pageLoading.value || loadError.value || !membersReady.value || !enabledMembers.value.length) return
  submitting.value = true; following.value = true; selectedId.value = null; actionError.value = ''; input.value = ''
  nextTick(resizeInput)
  try {
    const result = await conversation.sendGroup(value)
    if (!disposed && result.state?.backgroundSyncPending) actionError.value = '提醒已保存，系统调度正在等待恢复。'
  } catch (error) {
    if (!disposed && error.code !== 'CANCELLED') actionError.value = error.message || '本轮回应没有完成，请重试'
    if (!disposed && !error.persisted && !input.value) { input.value = value; nextTick(resizeInput) }
  } finally { if (!disposed) submitting.value = false }
}
async function deleteMessage(id) {
  selectedId.value = null
  if (busy.value) return
  deleting.value = true
  try {
    await ElMessageBox.confirm('确认删除这条消息？关联数据将由后端同步清理。', '删除群聊消息', { confirmButtonText: '删除', cancelButtonText: '保留', type: 'warning' })
    if (!disposed && !isLoading.value) await conversation.removeById(id, false)
  } catch (error) { if (!disposed && error !== 'cancel' && error !== 'close') actionError.value = error.message || '消息没有删除成功，请重试' }
  finally { if (!disposed) deleting.value = false }
}
async function copyMessage(message) {
  selectedId.value = null
  const text = visibleMessage(message.content)
  try { await navigator.clipboard.writeText(text) }
  catch {
    if (disposed) return
    const previous = document.activeElement, area = document.createElement('textarea')
    area.value = text; area.style.cssText = 'position:fixed;opacity:0;pointer-events:none;'
    let copied = false
    try { document.body.appendChild(area); area.select(); copied = document.execCommand('copy') }
    catch { copied = false }
    finally { area.remove(); previous?.focus({ preventScroll: true }) }
    if (!copied) { ElMessage.error('复制失败，请长按选择文字'); return }
  }
  if (!disposed) ElMessage.success('已复制')
}
function onScroll() {
  if (chatList.value) following.value = chatList.value.scrollHeight - chatList.value.scrollTop - chatList.value.clientHeight < 72
}
async function scrollToLatest(force = false) {
  if (!force && !following.value) return
  await nextTick()
  if (disposed || (!force && !following.value)) return
  if (chatList.value) chatList.value.scrollTop = chatList.value.scrollHeight
  if (force) following.value = true
}
async function loadMore() {
  if (!chatList.value) return
  const before = chatList.value.scrollHeight, top = chatList.value.scrollTop
  following.value = false; displayCount.value += 80
  await nextTick()
  if (!disposed && chatList.value) chatList.value.scrollTop = top + chatList.value.scrollHeight - before
}
watch(() => [messages.value.at(-1)?.id, messages.value.at(-1)?.content, messages.value.at(-1)?.reasoning, isLoading.value], () => scrollToLatest())
onMounted(() => { resizeInput(); load() })
onBeforeUnmount(() => { disposed = true; loadVersion++ })
</script>

<style scoped>
.group-space { position: fixed; inset: var(--room-top, 0px) 0 auto; height: var(--room-height, 100dvh); overflow: hidden; background: var(--room-page); }
.group-main { position: relative; z-index: 1; display: flex; flex-direction: column; height: 100%; }
.group-scenery { position: absolute; inset: 0; pointer-events: none; }
.group-scenery > img { width: 100%; height: 100%; object-fit: cover; opacity: .2; }
.group-scenery > div { position: absolute; inset: 0; background: linear-gradient(180deg, var(--room-page), transparent 45%, var(--room-page)); }
.group-roster { display: flex; align-items: center; gap: 13px; padding: 13px max(16px, calc((100% - 760px) / 2)); border-bottom: 1px solid var(--room-line); background: var(--room-glass); flex: none; }
.roster-avatars { display: flex; flex: none; max-width: 38%; overflow-x: auto; padding: 2px; scrollbar-width: none; }
.roster-avatars .character-avatar { width: 33px; height: 39px; border: 2px solid var(--room-surface); border-radius: 3px 9px 3px 3px; margin-left: -5px; background: var(--room-accent-soft); }
.roster-avatars .character-avatar:first-child { margin-left: 0; }
.roster-caption { display: flex; flex: 1; min-width: 0; flex-direction: column; gap: 4px; }
.roster-caption strong { font-size: 12px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.roster-caption > span { font-size: 10px; color: var(--room-muted); }
.roster-edit { display: inline-flex; align-items: center; gap: 3px; padding-right: 0; font-size: 11px; flex: none; }
.roster-edit svg { width: 12px; height: 12px; }
.group-error { display: flex; align-items: center; justify-content: space-between; gap: 8px; flex: none; margin: 8px max(14px, calc((100% - 760px) / 2)) 0; }
.group-error .room-icon-button { width: 28px; height: 28px; }
.group-timeline { flex: 1; min-height: 0; overflow-y: auto; overscroll-behavior: contain; scrollbar-width: thin; scrollbar-color: var(--room-line) transparent; }
.group-messages { width: min(100%, 760px); margin: 0 auto; padding: 22px 16px 12px; }
.group-earlier { display: flex; align-items: center; gap: 5px; margin: 0 auto 16px; font-size: 11px; }
.group-earlier svg { width: 14px; height: 14px; }
.group-empty { text-align: center; color: var(--room-muted); padding: clamp(36px, 10vh, 100px) 16px 28px; }
.empty-portraits { position: relative; display: inline-flex; align-items: flex-end; gap: 10px; padding: 0 12px 15px; }
.empty-portraits .character-avatar { width: 61px; height: 76px; border-radius: 4px 15px 4px 4px; border: 3px solid white; box-shadow: 0 4px 12px #28354d0a; transform: rotate(-5deg); }
.empty-portraits .character-avatar:nth-child(2) { transform: translateY(-8px) rotate(5deg); }
.empty-portraits > svg { position: absolute; right: -2px; bottom: 4px; width: 30px; height: 28px; color: var(--room-accent); }
.group-empty h2 { font-size: 16px; font-weight: 600; color: var(--room-ink); margin: 13px 0 9px; }
.group-empty p { font-size: 12px; line-height: 1.8; margin: 0; }
.group-message { --speaker-accent: #418ec6; --speaker-soft: #e9f3fb; --speaker-line: #dce6f0; display: flex; align-items: flex-start; gap: 9px; margin-bottom: 22px; }
.group-message[data-speaker='nana'] { --speaker-accent: #986ab9; --speaker-soft: #f2eaf9; --speaker-line: #e8dff1; }
.group-message-avatar { width: 32px; height: 40px; border-radius: 3px 10px 3px 3px; border-bottom: 2px solid var(--speaker-accent); margin-top: 6px; }
.group-message-column { min-width: 0; max-width: calc(100% - 56px); }
.group-byline { display: flex; align-items: center; gap: 8px; min-height: 24px; margin: 0 0 4px; }
.group-byline strong { font-size: 11px; font-weight: 600; color: var(--speaker-accent); }
.group-byline time { font-size: 9px; color: var(--room-muted); }
.group-byline .room-icon-button { margin-left: auto; width: 30px; height: 28px; color: var(--room-muted); }
.group-byline svg { width: 16px; height: 16px; }
.group-bubble { position: relative; padding: 12px 15px; border: 1px solid var(--speaker-line); border-radius: 4px 14px 14px; background: var(--room-surface); box-shadow: 0 2px 0 var(--speaker-soft); overflow-wrap: anywhere; }
.group-bubble::before { content: ''; position: absolute; left: -5px; top: 11px; width: 8px; height: 8px; transform: rotate(45deg); border-left: 1px solid var(--speaker-line); border-bottom: 1px solid var(--speaker-line); background: var(--room-surface); }
.group-message.is-user { justify-content: flex-end; padding-left: 30px; }
.is-user .group-byline { justify-content: flex-end; }
.is-user .group-byline strong { color: var(--room-muted); }
.is-user .group-byline .room-icon-button { margin-left: 0; }
.is-user .group-message-column { max-width: 100%; }
.is-user .group-bubble { border-color: transparent; border-radius: 14px 4px 14px 14px; background: var(--room-user-bubble); box-shadow: none; }
.is-user .group-bubble::before { display: none; }
.group-reasoning { color: var(--room-muted); font-size: 12px; margin-bottom: 8px; }
.group-reasoning summary { cursor: pointer; padding: 4px 0; }
.group-reasoning > div { max-height: 220px; overflow-y: auto; white-space: pre-wrap; line-height: 1.8; padding: 8px 0; }
.group-message-error { color: var(--room-danger); font-size: 12px; margin: 8px 0 0; line-height: 1.7; }
.group-message-status { display: block; color: var(--room-muted); font-size: 10px; margin-top: 7px; }
.group-message-actions { display: flex; gap: 4px; width: fit-content; padding: 4px; margin-top: 8px; border: 1px solid var(--room-line); border-radius: 4px 10px 4px 4px; background: var(--room-surface); box-shadow: var(--room-shadow); }
.group-message-actions button { display: flex; gap: 5px; align-items: center; border: 0; border-radius: 4px; background: transparent; padding: 8px; font-size: 11px; }
.group-message-actions button:last-child { color: var(--room-danger); }
.group-message-actions svg { width: 13px; height: 13px; }
.is-user .group-message-actions { margin-left: auto; }
.group-dock { position: relative; flex: none; padding: 10px max(12px, calc((100% - 760px) / 2)) calc(8px + env(safe-area-inset-bottom)); background: var(--room-glass); border-top: 1px solid var(--room-line); }
.group-latest { position: absolute; top: -46px; right: max(16px, calc((100% - 730px) / 2)); display: flex; align-items: center; gap: 5px; min-height: 32px; padding: 7px 10px; font-size: 11px; }
.group-latest svg { width: 13px; height: 13px; }
.group-activity { display: flex; align-items: center; gap: 7px; padding: 0 5px 9px; font-size: 11px; color: var(--room-muted); }
.group-activity .character-avatar { width: 20px; height: 24px; border-radius: 2px 5px 2px 2px; }
.group-activity > span:first-of-type { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.group-typing { display: inline-flex; align-items: center; gap: 4px; min-height: 18px; }
.group-typing i { width: 4px; height: 4px; background: var(--room-accent); border-radius: 50%; animation: group-breathe 1.2s infinite; }
.group-typing i:nth-child(2) { animation-delay: .15s; }
.group-typing i:nth-child(3) { animation-delay: .3s; }
.group-composer { display: flex; align-items: flex-end; gap: 7px; padding: 6px; border: 1px solid var(--room-line); border-bottom: 2px solid var(--room-accent-bright); border-radius: 4px 12px 4px 4px; background: var(--room-surface); transition: border-color .2s, box-shadow .2s; }
.group-composer:focus-within { border-bottom-color: var(--room-accent); box-shadow: 0 0 0 3px var(--room-accent-soft); }
.composer-emblem { align-self: center; margin: 0 6px; width: 22px; height: 21px; color: var(--room-accent); opacity: .75; }
.group-composer textarea { flex: 1; min-width: 0; min-height: 42px; max-height: 112px; resize: none; border: 0; outline: 0; background: transparent; color: var(--room-ink); font: inherit; font-size: 14px; line-height: 22px; padding: 10px 0; }
.group-composer textarea::placeholder { color: var(--room-muted); }
.group-send { display: grid; place-items: center; flex: none; width: 42px; height: 42px; border: 0; border-radius: 3px 8px 3px 3px; background: var(--room-action); color: white !important; }
.group-send:disabled, .group-send.is-stop { background: var(--room-accent-soft); color: var(--room-accent) !important; }
.group-send svg { transition: transform .22s var(--room-spring); }
.group-send:active:not(:disabled):not(.is-stop) svg { transform: translate(2px, -2px) scale(.93); }
.group-composer-note { display: flex; justify-content: space-between; margin: 6px 5px 0; color: var(--room-muted); font-size: 9px; }
.members-hint { color: var(--room-muted); font-size: 11px; line-height: 1.7; margin: 0 0 14px; }
.group-member { display: flex; align-items: center; gap: 10px; padding: 13px 0; border-bottom: 1px solid var(--room-line); }
.member-order { color: var(--room-muted); font-size: 10px; font-variant-numeric: tabular-nums; }
.group-member .character-avatar { width: 36px; height: 44px; border-radius: 3px 9px 3px 3px; }
.member-identity { display: flex; flex: 1; min-width: 0; flex-direction: column; gap: 5px; }
.member-identity strong { font-size: 13px; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.member-identity small { font-size: 10px; color: var(--room-accent); }
.is-paused .member-identity small { color: var(--room-muted); }
.is-paused .character-avatar { opacity: .6; }
.member-move { display: flex; flex: none; gap: 2px; }
.member-move .room-icon-button { width: 32px; height: 36px; color: var(--room-muted); }
.member-move svg { width: 15px; height: 15px; }
.member-toggle { position: relative; display: inline-flex; align-items: center; width: 36px; height: 36px; flex: none; cursor: pointer; }
.member-toggle input { position: absolute; inset: 0; width: 100%; height: 100%; margin: 0; opacity: 0; cursor: pointer; }
.member-toggle > span { width: 36px; height: 20px; border-radius: 6px; background: var(--room-line); padding: 2px; pointer-events: none; transition: background .18s; }
.member-toggle > span::after { content: ''; display: block; width: 16px; height: 16px; border-radius: 4px; background: white; transition: transform .2s var(--room-spring); }
.member-toggle input:checked + span { background: var(--room-accent); }
.member-toggle input:checked + span::after { transform: translateX(16px); }
.member-toggle input:focus-visible + span { outline: 2px solid var(--room-accent); outline-offset: 4px; }
.member-toggle input:disabled + span { opacity: .5; }
.members-footer { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
.members-footer > span { color: var(--room-muted); font-size: 11px; }
@keyframes group-breathe { 0%, 100% { opacity: .3; } 50% { opacity: 1; } }
@media (hover: hover) { .group-message-actions button:hover { background: var(--room-accent-soft); } }
@media (max-width: 380px) { .group-roster { gap: 8px; padding-left: 12px; padding-right: 12px; } .group-member { gap: 7px; } .member-move .room-icon-button { width: 28px; } .composer-emblem { display: none; } }
@media (max-height: 500px) { .group-roster { padding-top: 7px; padding-bottom: 7px; } .group-composer-note { display: none; } }
@supports (-webkit-touch-callout: none) { .group-composer textarea { font-size: 16px; } }
</style>
