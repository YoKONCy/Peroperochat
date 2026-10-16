<template>
  <div class="companion-home companion-surface" :data-tone="tone" :class="[`companion-home--${panelMode}`, { 'is-keyboard': keyboardOpen }]" :style="roomStyle" @keydown="onHomeKeydown">
    <div class="room-main" :inert="drawer ? true : undefined">
      <div class="room-scenery" aria-hidden="true"><Transition name="room-change"><img :key="tone" :src="`/rooms/${tone}.svg`" alt="" class="room-background" /></Transition><div class="room-vignette" /></div>
      <header class="room-header">
        <button class="room-selector" aria-label="切换角色与房间" @click="drawer = 'roles'">
          <CharacterAvatar :key="activeAgentId" class="header-avatar" :src="agent.avatar" :label="agent.name" />
          <span :key="`label-${activeAgentId}`" class="room-selector-label"><span class="room-selector-name"><strong>{{ agent.name }}</strong><CompanionEmblem /></span><span class="room-selector-status"><i />{{ projection.vibe || '陪伴中' }}</span></span><RoomIcon name="down" />
        </button>
        <nav class="room-header-actions" aria-label="房间导航">
          <button class="room-icon-button" aria-label="打开房间菜单" title="菜单" @click="drawer = 'menu'"><RoomIcon name="menu" /></button>
        </nav>
      </header>
      <div class="room-character-stage"><Live2DWidget :quiet="panelMode !== 'rest' || keyboardOpen" :interactive="panelMode !== 'full' && !keyboardOpen && !drawer" @projection="onProjection" @open-chat="panelMode = 'peek'" /></div>
      <Transition name="room-actions"><nav v-show="panelMode === 'rest' && !keyboardOpen" class="room-quick-actions" aria-label="角色互动">
        <button aria-label="展开聊天记录" @click="panelMode = 'peek'"><span><RoomIcon name="chat" /></span><small>聊天</small></button>
        <button aria-label="打开衣柜" @click="drawer = 'wardrobe'"><span><RoomIcon name="hanger" /></span><small>换装</small></button>
        <button aria-label="打开日记" @click="drawer = 'journal'"><span><RoomIcon name="diary" /></span><small>日记</small></button>
      </nav></Transition>
      <ReminderStrip v-if="panelMode === 'rest' && !keyboardOpen && (reminderTotal || reminderError)" :entries="reminders" :total="reminderTotal" :error="reminderError" @open="drawer = 'reminders'" />
      <div v-if="loadError" class="room-load-error" role="alert"><span>{{ loadError }}</span><button @click="loadRoomData">重试</button></div>
      <div v-if="actionError" class="room-action-error" role="alert"><span>{{ actionError }}</span><button aria-label="关闭提示" @click="actionError = ''"><RoomIcon name="close" /></button></div>
      <RoomChat :key="activeAgentId" ref="chatPanel" v-model="draft.text" v-model:mode="panelMode" :agent="agent" :messages="messages" :busy="isLoading" :activity="activityText" :image="draft.image" :keyboard-open="keyboardOpen" :jump-id="jumpId" @focus="onComposerFocus" @send="send" @stop="conversation.stop" @attach="fileInput?.click()" @remove-image="draft.image = null" @copy="copy" @delete="removeMessage" @regenerate="regenerate" @preview-image="openImage" />
      <input ref="fileInput" class="room-hidden-input" type="file" accept="image/*" aria-label="选择聊天图片" tabindex="-1" @change="attachImage" />
    </div>

    <RoomDrawer v-if="drawer === 'roles'" title="选择伙伴" subtitle="切换角色与房间" :tone="tone" @close="drawer = null">
      <div class="room-role-list">
        <button v-for="role in agentList" :key="role.id" :data-role-tone="role.id === 'nana' ? 'nana' : 'pero'" :class="{ 'is-current': role.id === activeAgentId }" :disabled="switching" @click="chooseRole(role.id)">
          <CharacterAvatar class="role-avatar" :src="role.avatar" :label="role.name" />
          <span class="role-label"><strong>{{ role.name }}</strong><span>{{ role.description || '你的专属伙伴' }}</span><span v-if="role.tags?.length" class="role-tags"><b v-for="tag in role.tags.slice(0, 3)" :key="tag">{{ tag }}</b></span></span>
          <RoomIcon :name="role.id === activeAgentId ? 'check' : 'arrow'" />
        </button>
      </div>
      <template #footer><button class="room-text-button" @click="navigate('/roles')">管理角色</button></template>
    </RoomDrawer>

    <RoomDrawer v-if="drawer === 'menu'" title="菜单" :subtitle="agent.name" :tone="tone" @close="drawer = null">
      <div class="room-menu-list">
        <button @click="openFullChat"><RoomIcon name="chat" /><span><strong>聊天记录</strong><small>查看完整对话</small></span><RoomIcon name="arrow" /></button>
        <button @click="drawer = 'journal'"><RoomIcon name="diary" /><span><strong>日记</strong><small>查看和管理记忆</small></span><RoomIcon name="arrow" /></button>
        <button @click="drawer = 'reminders'"><RoomIcon name="bell" /><span><strong>提醒</strong><small>{{ reminderTotal ? `${reminderTotal} 件待办的小事` : '安排时间，查看或取消提醒' }}</small></span><RoomIcon name="arrow" /></button>
        <button @click="drawer = 'profile'"><RoomIcon name="profile" /><span><strong>关于 {{ agent.name }}</strong><small>此刻的心情与角色资料</small></span><RoomIcon name="arrow" /></button>
        <button @click="drawer = 'wardrobe'"><RoomIcon name="hanger" /><span><strong>衣柜</strong><small>更换角色服装</small></span><RoomIcon name="arrow" /></button>
        <button @click="navigate('/settings')"><RoomIcon name="settings" /><span><strong>设置</strong><small>模型连接与聊天偏好</small></span><RoomIcon name="arrow" /></button>
      </div>
      <details class="room-other-spaces"><summary>其他空间</summary><div class="room-secondary-links"><button @click="navigate('/group')"><RoomIcon name="group" />家庭群聊</button><button @click="navigate('/game/blackjack')"><RoomIcon name="game" />21 点小游戏</button><button @click="drawer = 'legacy'"><RoomIcon name="diary" />旧话题 <span v-if="topics.length">{{ topics.length }}</span></button><button @click="openRemote"><RoomIcon name="remote" />远程连接</button></div></details>
    </RoomDrawer>

    <RoomDrawer v-if="drawer === 'profile'" :title="`关于 ${agent.name}`" subtitle="角色资料" :tone="tone" @close="drawer = null">
      <div class="room-profile-identity"><CharacterAvatar class="profile-avatar" :src="agent.avatar" :label="agent.name" /><div><h3>{{ agent.name }}</h3><p>{{ projection.mood || agent.default_state?.mood || '在这里陪你' }}</p></div></div>
      <div class="room-profile-note"><span>此刻的想法</span><p>{{ projection.mind || agent.default_state?.mind || '想听听你今天发生的事。' }}</p></div>
      <div v-if="projection.environment?.location || projection.environment?.weather" class="room-profile-environment"><RoomIcon name="sun" /><span>{{ [projection.environment?.location, projection.environment?.weather].filter(Boolean).join(' · ') }}</span></div>
      <details class="room-persona"><summary>角色设定</summary><p>{{ agent.persona_prompt }}</p></details>
      <template #footer><button class="room-text-button" @click="navigate('/roles')">编辑角色资料</button></template>
    </RoomDrawer>

    <RoomJournal v-if="drawer === 'journal'" :key="agent.id" :agent="agent" :tone="tone" @close="drawer = null" @source="openSource" @updated="refreshProjection" />
    <RoomWardrobe v-if="drawer === 'wardrobe'" :key="agent.id" :agent="agent" :tone="tone" :texture="projection.modelTexturesId" @close="drawer = null" />
    <RoomReminders v-if="drawer === 'reminders'" :key="agent.id" :agent="agent" :tone="tone" @close="drawer = null" @updated="reloadReminders" />
    <RoomDrawer v-if="drawer === 'image'" title="对话里的图片" :tone="tone" @close="drawer = null"><img :src="imagePreview" class="room-image-preview" alt="对话附件大图" /></RoomDrawer>

    <RoomDrawer v-if="drawer === 'legacy'" title="旧话题" :tone="tone" @close="drawer = null">
      <div v-if="legacyError" class="room-error">{{ legacyError }} <button class="room-text-button" @click="loadRoomData">重试</button></div>
      <div v-if="!topics.length && !legacyError" class="room-empty">暂时没有旧话题。</div>
      <h3 v-if="topics.length" class="legacy-heading">话题</h3>
      <article v-for="topic in topics" :key="topic.id" class="legacy-entry"><p v-if="topic.revealed">{{ topic.topic }}</p><button v-else class="room-text-button" @click="revealTopic(topic)">揭开这个话题</button><button class="room-text-button" @click="removeTopic(topic)">删除话题</button></article>
    </RoomDrawer>
  </div>
</template>

<script setup>
import { computed, reactive, ref, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import Live2DWidget from '../components/Live2DWidget.vue'
import RoomChat from '../components/companion/RoomChat.vue'
import RoomDrawer from '../components/companion/RoomDrawer.vue'
import RoomJournal from '../components/companion/RoomJournal.vue'
import RoomWardrobe from '../components/companion/RoomWardrobe.vue'
import RoomIcon from '../components/companion/RoomIcon.vue'
import CharacterAvatar from '../components/companion/CharacterAvatar.vue'
import CompanionEmblem from '../components/companion/CompanionEmblem.vue'
import RoomReminders from '../components/companion/RoomReminders.vue'
import ReminderStrip from '../components/companion/ReminderStrip.vue'
import { useReminders } from '../composables/useReminders'
import { AGENTS, activeAgentId, agentList, setActiveAgentId } from '../stores/agents'
import { useLocalConversation } from '../stores/conversations'
import { useRoomViewport } from '../composables/useRoomViewport'
import { localInvoke, publishCharacterState } from '../services/localChat'
import { getTopics, deleteTopic, updateTopicStatus, setAppMode } from '../api'
import { visibleMessage } from '../utils/messageContent'
import '../styles/companion.css'

const router = useRouter()
const route = useRoute()
const conversation = useLocalConversation()
const { messages, isLoading, activityText } = conversation
const { keyboardOpen, viewportStyle } = useRoomViewport()
const agent = computed(() => AGENTS[activeAgentId.value] || { id: activeAgentId.value, name: '伙伴' })
const tone = computed(() => activeAgentId.value === 'nana' ? 'nana' : 'pero')
const drafts = reactive(new Map())
const drawer = ref(null)
const projection = ref({})
const chatPanel = ref(null)
const fileInput = ref(null)
const jumpId = ref(null)
const imagePreview = ref('')
const switching = ref(false)
const loadError = ref('')
const actionError = ref('')
const legacyError = ref('')
const { entries: reminders, total: reminderTotal, error: reminderError, reload: reloadReminders } = useReminders(activeAgentId)
const topics = ref([])
let disposed = false
let loadVersion = 0
let projectionVersion = 0
let imageReader = null
let loadScheduled = false
const draft = computed(() => drafts.get(activeAgentId.value))
const panelMode = computed({ get: () => draft.value.mode, set: value => { draft.value.mode = value; if (value === 'rest') chatPanel.value?.blur() } })
const roomStyle = computed(() => ({ ...viewportStyle.value, '--reminder-space': reminderTotal.value || reminderError.value ? '62px' : '0px', '--dock-height': keyboardOpen.value || panelMode.value === 'full' ? 'calc(100% - 96px - env(safe-area-inset-top))' : panelMode.value === 'peek' ? 'min(calc(100% - 96px - env(safe-area-inset-top)), max(280px, 43%))' : `calc(${messages.value.some(message => message.role === 'assistant' && message.status === 'active') || isLoading.value ? 126 : 86}px + env(safe-area-inset-bottom))` }))

async function loadRoomData() {
  const id = activeAgentId.value
  const version = ++loadVersion
  const results = await Promise.allSettled([conversation.load(), getTopics(id)])
  if (disposed || version !== loadVersion || id !== activeAgentId.value) return
  loadError.value = results[0].status === 'rejected' ? results[0].reason.message || '对话暂时没有加载出来' : ''
  if (results[1].status === 'fulfilled') topics.value = results[1].value
  legacyError.value = results[1].status === 'rejected' ? '旧话题暂时无法读取。' : ''
}
function scheduleRoomLoad() {
  if (loadScheduled) return
  loadScheduled = true
  nextTick(() => { loadScheduled = false; if (!disposed) loadRoomData() })
}
function onProjection(state) { if (state.agentId === activeAgentId.value) projection.value = state }
async function refreshProjection() {
  const id = activeAgentId.value
  const version = ++projectionVersion
  try {
    const state = await localInvoke('local_character_projection', { agentId: id })
    if (!disposed && version === projectionVersion && id === activeAgentId.value) { onProjection(state); publishCharacterState(state) }
  } catch (error) { if (!disposed && id === activeAgentId.value) console.warn('刷新角色展示失败：', error) }
}
async function chooseRole(id) {
  if (switching.value) return
  switching.value = true
  try { if (id !== activeAgentId.value) await setActiveAgentId(id); drawer.value = null }
  catch (error) { ElMessage.error(error.message) }
  finally { switching.value = false }
}
function onComposerFocus() { actionError.value = ''; jumpId.value = null; if (panelMode.value === 'rest') panelMode.value = 'peek' }
function openFullChat() { drawer.value = null; jumpId.value = null; panelMode.value = 'full' }
function navigate(path) { drawer.value = null; chatPanel.value?.blur(); router.push(path) }
function openRemote() { drawer.value = null; setAppMode('remote') }
function openImage(url) { if (url) { imagePreview.value = url; drawer.value = 'image' } }
function onHomeKeydown(event) {
  if (event.key !== 'Escape' || event.isComposing || drawer.value) return
  if (panelMode.value !== 'rest') { event.preventDefault(); chatPanel.value?.blur(); panelMode.value = panelMode.value === 'full' ? 'peek' : 'rest' }
}
async function send() {
  const id = activeAgentId.value
  const target = draft.value
  const text = target.text.trim()
  const image = target.image
  if ((!text && !image) || isLoading.value) return
  const content = image ? [{ type: 'text', text: text || '这张图片里有什么？' }, { type: 'image_url', image_url: { url: image } }] : text
  target.text = ''; target.image = null
  jumpId.value = null
  if (target.mode === 'rest') target.mode = 'peek'
  actionError.value = ''
  try {
    const result = await conversation.send(content, { agentId: id })
    if (!disposed && id === activeAgentId.value) {
      if (result.state?.diaryWrites > 0) ElMessage.success({ message: '日记已更新', duration: 2500 })
      if (result.state?.diarySyncPending) actionError.value = '对话已保存，日记正在等待存储恢复。'
      if (result.state?.backgroundSyncPending) actionError.value = '提醒已保存，系统调度暂未登记成功，将继续重试。'
    }
  } catch (error) {
    if (!disposed && id === activeAgentId.value && error.code !== 'CANCELLED') actionError.value = error.message
    if (!disposed && !error.persisted && !target.text && !target.image) { target.text = text; target.image = image }
  }
}
async function regenerate(index) {
  const id = activeAgentId.value
  if (isLoading.value) return
  actionError.value = ''
  try { await conversation.regenerate(index) }
  catch (error) { if (!disposed && id === activeAgentId.value && error.code !== 'CANCELLED') actionError.value = error.message }
}
async function removeMessage(index) {
  const id = activeAgentId.value
  const message = messages.value[index]
  if (!message || isLoading.value) return
  try { await ElMessageBox.confirm('这一轮的输入、回复，以及由它产生的日记会一并删除。', '删除这轮对话？', { confirmButtonText: '删除', cancelButtonText: '保留', type: 'warning' }) }
  catch { return }
  if (disposed || id !== activeAgentId.value || isLoading.value) return
  try { await conversation.removeById(message.id, true); await refreshProjection() }
  catch (error) { if (!disposed && id === activeAgentId.value) actionError.value = error.message }
}
async function copy(message) {
  const text = visibleMessage(message.content)
  try { await navigator.clipboard.writeText(text); ElMessage.success('已复制') }
  catch {
    const previous = document.activeElement
    const area = document.createElement('textarea')
    area.value = text; area.style.cssText = 'position:fixed;opacity:0;pointer-events:none;'
    document.body.appendChild(area); area.select()
    const copied = document.execCommand('copy')
    area.remove(); previous?.focus({ preventScroll: true })
    if (copied) ElMessage.success('已复制')
    else ElMessage.error('复制失败，请长按选择文字')
  }
}
function attachImage(event) {
  const file = event.target.files?.[0]
  event.target.value = ''
  if (!file) return
  if (!['image/jpeg', 'image/png', 'image/webp', 'image/gif'].includes(file.type) || file.size > 10 * 1024 * 1024) { ElMessage.error('请选择不超过 10MB 的 JPG、PNG、WebP 或 GIF 图片'); return }
  imageReader?.abort()
  const target = draft.value
  const reader = new FileReader()
  imageReader = reader
  reader.onload = () => { if (!disposed && imageReader === reader) { target.image = reader.result; if (target.mode === 'rest') target.mode = 'peek' } }
  reader.onerror = () => { if (!disposed && imageReader === reader) ElMessage.error('图片没有读出来，请再试一次') }
  reader.readAsDataURL(file)
}
async function openSource(ids) {
  const message = [...messages.value].reverse().find(item => ids.includes(item.id))
  if (!message) { ElMessage.info('原始对话当前不可见'); return }
  drawer.value = null; jumpId.value = null; panelMode.value = 'full'
  await nextTick(); jumpId.value = message.id
}
async function revealTopic(topic) {
  const id = activeAgentId.value
  try { await updateTopicStatus(topic.id, true); if (!disposed && id === activeAgentId.value) topic.revealed = true }
  catch (error) { ElMessage.error(error.message) }
}
async function removeTopic(topic) {
  const id = activeAgentId.value
  try { await ElMessageBox.confirm('删除后将无法继续查看这个话题。', '删除这个话题？', { confirmButtonText: '删除', cancelButtonText: '保留' }) }
  catch { return }
  if (disposed || id !== activeAgentId.value) return
  try { await deleteTopic(topic.id); if (!disposed && id === activeAgentId.value) topics.value = topics.value.filter(item => item.id !== topic.id) }
  catch (error) { ElMessage.error(error.message) }
}
function onAgentUpdated() { scheduleRoomLoad() }
watch(activeAgentId, id => {
  if (!drafts.has(id)) drafts.set(id, { text: '', image: null, mode: 'rest' })
  drawer.value = null; projection.value = {}; jumpId.value = null; actionError.value = ''; loadError.value = ''; topics.value = []; projectionVersion++
  scheduleRoomLoad()
}, { immediate: true, flush: 'sync' })
watch(drawer, value => { if (value) chatPanel.value?.blur() })
onMounted(async () => {
  if (route.query.panel === 'journal') {
    drawer.value = 'journal'
    const query = { ...route.query }; delete query.panel
    router.replace({ path: '/', query })
  }
  window.addEventListener('ppc:agent-switched', onAgentUpdated)

})
onBeforeUnmount(() => { disposed = true; loadVersion++; projectionVersion++; imageReader?.abort(); window.removeEventListener('ppc:agent-switched', onAgentUpdated) })
</script>

<style scoped>
.companion-home { position: fixed; inset: var(--room-top, 0px) 0 auto; height: var(--room-height, 100dvh); overflow: hidden; background: #dbf2ff; }
.companion-home[data-tone='nana'] { background: #dcd2f6; }
.room-main { position: relative; height: 100%; overflow: hidden; }
.room-scenery { position: absolute; inset: 0; overflow: hidden; pointer-events: none; }
.room-background { position: absolute; height: 100%; width: 100%; object-fit: cover; object-position: center 48%; transition: transform .4s; }
.room-vignette { position: absolute; inset: 0; background: linear-gradient(180deg, #eefbff40, transparent 25%, transparent 75%, #b9e6ff55); }
[data-tone='nana'] .room-vignette { background: linear-gradient(180deg, #efe5ff30, transparent 25%, transparent 75%, #c7a5eb55); }
.room-change-enter-active, .room-change-leave-active { transition: opacity .55s; }
.room-change-enter-from, .room-change-leave-to { opacity: 0; }
.room-header { position: absolute; z-index: 30; left: 0; right: 0; top: 0; display: flex; align-items: center; justify-content: space-between; padding: calc(12px + env(safe-area-inset-top)) 14px 8px; gap: 10px; }
.room-selector { position: relative; display: flex; gap: 10px; align-items: center; background: #ffffffed; border: 1px solid #ffffffcc; border-bottom-color: var(--room-line); border-radius: 3px 12px 3px 3px; padding: 5px 12px 5px 5px; max-width: calc(100% - 52px); box-shadow: 0 3px 12px #28354d0a; text-align: left; }
.room-selector::after { content: ''; position: absolute; left: 5px; bottom: -1px; width: 42px; height: 2px; background: var(--room-accent); }
.header-avatar, .role-avatar, .profile-avatar { flex: none; background: var(--room-accent-soft); color: var(--room-accent); }
.header-avatar { width: 42px; height: 46px; clip-path: polygon(0 0, calc(100% - 6px) 0, 100% 6px, 100% 100%, 6px 100%, 0 calc(100% - 6px)); }
.room-selector-label { display: flex; flex-direction: column; gap: 5px; min-width: 0; }
.room-selector-name { display: flex; align-items: center; gap: 12px; }
.room-selector-name strong { font-size: 16px; font-weight: 650; letter-spacing: .3px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.room-selector-name svg { width: 21px; height: 18px; flex: none; color: var(--room-accent); opacity: .75; }
.room-selector-status { display: flex; align-items: center; gap: 5px; color: var(--room-muted); font-size: 10px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 130px; }
.room-selector-status i { width: 4px; height: 4px; border-radius: 50%; background: var(--room-accent); flex: none; }
.room-selector > svg { width: 12px; height: 12px; margin-left: 3px; color: var(--room-muted); }
.header-avatar, .room-selector-label { animation: room-content-in .3s var(--room-ease); }
.room-selector-name > svg { transition: transform .3s var(--room-spring); }
.room-selector:active .room-selector-name > svg { transform: rotate(-8deg) scale(.94); }
.room-header-actions { display: flex; gap: 7px; }
.room-header-actions button { background: #fffffff0; border: 1px solid #ffffffcc; border-bottom-color: var(--room-line); border-radius: 4px 10px 4px 4px; width: 38px; height: 40px; color: var(--room-ink); box-shadow: 0 3px 12px #28354d08; }
.room-header-actions button svg { width: 18px; height: 18px; }
.room-header-actions button:hover { border-bottom-color: var(--room-accent); }
/* 原生模型画布的上方有透明留白，按可视区域对齐人物，而不是对齐画布。 */
.room-character-stage { --actor-size: min(148vw, calc(var(--room-height, 100dvh) * .96), 920px); --actor-top: max(calc(90px + env(safe-area-inset-top) - var(--actor-size) * .2), calc(var(--room-height, 100dvh) * .5 - var(--actor-size) * .625 - 8px)); position: absolute; z-index: 5; left: 46%; top: var(--actor-top); width: var(--actor-size); aspect-ratio: 1; transform: translateX(-50%); transition: width .32s, top .32s, left .32s, opacity .2s; }
.companion-home--peek .room-character-stage { --actor-top: calc(90px - var(--actor-size) * .2 + env(safe-area-inset-top)); }
.companion-home--full .room-character-stage, .is-keyboard .room-character-stage { opacity: 0; pointer-events: none; }
.companion-home--peek .room-background, .companion-home--full .room-background { transform: scale(1.03) translateY(-1%); }
.room-quick-actions { position: absolute; z-index: 25; top: calc(91px + env(safe-area-inset-top)); right: 12px; display: flex; flex-direction: column; gap: 10px; }
.room-quick-actions::before { content: ''; position: absolute; left: 50%; top: 18px; bottom: 18px; width: 1px; background: #ffffffa8; z-index: -1; }
.room-quick-actions button { border: 0; padding: 3px; min-width: 44px; display: flex; flex-direction: column; align-items: center; gap: 4px; background: transparent; color: var(--room-ink); }
.room-quick-actions button > span { display: grid; place-items: center; width: 34px; height: 36px; border: 1px solid #ffffffd9; border-bottom: 2px solid var(--room-accent-bright); border-radius: 3px 9px 3px 3px; background: #fffffff0; color: var(--room-ink); transition: border-color .18s; }
.room-quick-actions button:hover > span { border-bottom-color: var(--room-accent); }
.room-quick-actions small { font-size: 10px; font-weight: 500; line-height: 1.5; text-shadow: 0 1px 4px white, 0 0 8px white; }
.room-quick-actions svg { width: 18px; height: 18px; }
.room-quick-actions button > span, .room-quick-actions svg { transition: transform .24s var(--room-spring), border-color .18s, background .18s; }
.room-quick-actions button:active > span { transform: translateY(1px) scale(.94); background: var(--room-accent-soft); }
.room-actions-enter-active { transition: opacity .3s; }
.room-actions-leave-active { transition: opacity .12s; pointer-events: none; }
.room-actions-enter-from, .room-actions-leave-to { opacity: 0; }
.room-actions-enter-active button { animation: room-action-in .26s var(--room-spring) both; }
.room-actions-enter-active button:nth-child(2) { animation-delay: .025s; }
.room-actions-enter-active button:nth-child(3) { animation-delay: .05s; }
@keyframes room-action-in { from { opacity: 0; transform: translateX(6px) scale(.94); } to { opacity: 1; transform: translateX(0) scale(1); } }
@media (hover: hover) {
  .room-selector:hover .room-selector-name > svg { transform: rotate(-6deg); }
  .room-quick-actions button:hover > span { transform: translateY(-2px); }
  .room-quick-actions button:hover svg { transform: rotate(-5deg); }
}
.room-hidden-input { position: absolute; width: 1px; height: 1px; overflow: hidden; opacity: 0; pointer-events: none; }
.room-load-error, .room-action-error { position: absolute; z-index: 35; top: calc(88px + env(safe-area-inset-top)); left: 22px; right: 22px; display: flex; align-items: center; justify-content: space-between; gap: 12px; background: #fff8f3f5; border: 1px solid #e9c5b6; color: #885547; box-shadow: var(--room-shadow); padding: 10px 14px; border-radius: 13px; font-size: 12px; line-height: 1.6; }
.room-action-error { top: auto; bottom: calc(var(--dock-height) + 10px); }
.room-load-error button, .room-action-error button { border: 0; background: transparent; padding: 5px; flex: none; }
.room-action-error svg { width: 16px; height: 16px; }
.room-role-list { display: flex; flex-direction: column; gap: 10px; }
.room-role-list > button { display: flex; align-items: center; gap: 16px; padding: 12px; border: 1px solid var(--room-line); border-radius: 3px 12px 3px 3px; background: var(--room-surface); text-align: left; width: 100%; }
.room-role-list > button.is-current { border-color: var(--room-accent); background: var(--room-subtle); }
.role-avatar { width: 62px; height: 76px; border-radius: 3px 9px 3px 3px; }
.role-label { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 7px; }
.role-label strong { font-size: 17px; font-weight: 600; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
.role-label > span { font-size: 11px; color: var(--room-muted); line-height: 1.7; }
.role-tags { display: flex; flex-wrap: wrap; gap: 5px; }
.role-tags b { padding: 2px 6px; background: var(--room-accent-soft); border-radius: 5px; color: var(--room-accent); font-size: 10px; font-weight: 400; }
[data-role-tone='nana'] .role-tags b { color: #9963b8; }
.room-role-list svg { width: 17px; height: 17px; color: var(--room-accent); }
.room-menu-list { display: flex; flex-direction: column; }
.room-menu-list > button { display: flex; align-items: center; gap: 14px; border: 0; border-bottom: 1px solid var(--room-line); background: transparent; border-radius: 0; padding: 18px 2px; width: 100%; text-align: left; }
.room-menu-list > button > svg:first-child { width: 21px; height: 21px; margin: 0 5px; color: var(--room-accent); }
.room-menu-list > button > span { flex: 1; display: flex; flex-direction: column; gap: 7px; }
.room-menu-list strong { font-weight: 500; font-size: 14px; }
.room-menu-list small { color: var(--room-muted); font-size: 11px; }
.room-menu-list > button > svg:last-child { width: 14px; height: 14px; color: var(--room-muted); }
.room-other-spaces { margin-top: 26px; font-size: 12px; color: var(--room-muted); }
.room-other-spaces summary { padding: 10px 0; cursor: pointer; }
.room-secondary-links { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-top: 12px; }
.room-secondary-links button { display: flex; align-items: center; gap: 9px; padding: 14px 10px; border: 1px solid var(--room-line); border-radius: 12px; background: transparent; font-size: 12px; }
.room-secondary-links svg { width: 17px; height: 17px; }
.room-secondary-links button > span { margin-left: auto; }
.room-profile-identity { display: flex; align-items: center; gap: 20px; padding: 8px 0 25px; }
.profile-avatar { width: 82px; height: 100px; border-radius: 3px 12px 3px 3px; }
.room-profile-identity h3 { font-size: 25px; font-weight: 500; margin: 0 0 8px; }
.room-profile-identity p { font-size: 13px; color: var(--room-accent); margin: 0; }
.room-profile-note { border-radius: 3px 12px 3px 3px; border-left: 2px solid var(--room-accent-bright); padding: 16px; background: var(--room-accent-soft); }
.room-profile-note > span { color: var(--room-muted); font-size: 11px; }
.room-profile-note p { margin: 10px 0 0; line-height: 1.9; font-size: 14px; white-space: pre-wrap; }
.room-profile-environment { display: flex; align-items: center; gap: 10px; color: var(--room-muted); margin: 22px 0; font-size: 12px; line-height: 1.7; }
.room-profile-environment svg { width: 17px; height: 17px; }
.room-persona { margin-top: 25px; color: var(--room-muted); font-size: 12px; }
.room-persona summary { padding: 12px 0; cursor: pointer; }
.room-persona p { white-space: pre-wrap; line-height: 1.85; max-height: 300px; overflow-y: auto; overflow-wrap: anywhere; padding: 12px; background: white; border: 1px solid var(--room-line); border-radius: 12px; }
.legacy-heading { font-size: 13px; color: var(--room-muted); font-weight: 500; margin: 25px 0 10px; }
.legacy-entry { padding: 16px; border: 1px solid var(--room-line); border-radius: 14px; margin-bottom: 10px; line-height: 1.8; }
.legacy-entry time { font-size: 11px; color: var(--room-muted); }
.legacy-entry p { margin: 8px 0; white-space: pre-wrap; }
.room-image-preview { display: block; width: 100%; height: auto; border-radius: 14px; }
@media (min-width: 960px) {
  .room-header { padding: calc(22px + env(safe-area-inset-top)) 28px 12px; }
  .room-character-stage, .companion-home--peek .room-character-stage { left: calc((100% - 480px) / 2); --actor-size: min(76vw, calc(var(--room-height, 100dvh) * 1.1), 1000px); --actor-top: max(calc(100px + env(safe-area-inset-top) - var(--actor-size) * .2), calc(var(--room-height, 100dvh) * .5 - var(--actor-size) * .625 + 8px)); opacity: 1; pointer-events: auto; }
  .companion-home--full .room-character-stage { opacity: 1; pointer-events: auto; }
  .room-quick-actions { right: 28px; top: calc(97px + env(safe-area-inset-top)); flex-direction: row; gap: 13px; }
  .room-quick-actions::before { display: none; }
  .room-action-error { left: auto; right: 28px; width: 420px; }
  .room-load-error { left: auto; right: 28px; width: 420px; }
}
@media (max-width: 360px) { .room-header { padding-left: 12px; padding-right: 12px; } .room-selector-status { max-width: 110px; } .room-quick-actions { right: 10px; gap: 12px; } }
@media (max-height: 540px) and (max-width: 959px) { .room-character-stage { --actor-size: min(118vw, calc(var(--room-height, 100dvh) * 1.25)); --actor-top: calc(82px - var(--actor-size) * .2); } .room-quick-actions { top: 90px; gap: 8px; } .room-quick-actions button > span { width: 32px; height: 34px; } }
</style>
