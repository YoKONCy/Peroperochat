<template>
  <RoomDrawer :title="`${agent.name} 的日记`" subtitle="记忆与共同经历" :tone="tone" @close="$emit('close')">
    <form class="journal-search" @submit.prevent="reload">
      <input v-model="keyword" type="search" placeholder="找找某件事、某个词…" aria-label="搜索日记" />
      <input v-model="date" type="date" aria-label="筛选日记日期" />
    </form>
    <p v-if="error" class="room-error" role="alert">{{ error }} <button class="room-text-button" @click="reload">重试</button></p>
    <p v-if="loading && !entries.length" class="journal-loading" role="status">正在读取日记…</p>
    <div v-else-if="!loading && !entries.length && !error" class="room-empty"><RoomIcon name="diary" /><p>{{ keyword || date ? '没有找到相关日记' : '还没有日记' }}<br />{{ keyword || date ? '试试其他关键词或日期。' : '角色写下的日记会显示在这里。' }}</p></div>
    <section v-for="group in groups" :key="group.date" class="journal-day">
      <h3><span>{{ formatDate(group.date) }}</span><span class="journal-date-line" /></h3>
      <article v-for="entry in group.entries" :key="entry.id" class="journal-entry">
        <form v-if="editing?.id === entry.id" class="journal-editor" @submit.prevent="save(entry)">
          <label>日期<input v-model="editing.date" type="date" required /></label>
          <label>日记<textarea v-model="editing.content" rows="7" maxlength="16000" required /></label>
          <div class="journal-editor-actions"><button type="button" class="room-text-button" :disabled="saving" @click="editing = null">取消</button><button class="room-primary-button" :disabled="saving || !editing.content.trim()">{{ saving ? '正在保存…' : '保存修订' }}</button></div>
        </form>
        <template v-else>
          <p class="journal-body" :class="{ 'is-folded': !expandedIds.has(entry.id) }">{{ entry.content }}</p>
          <button v-if="entry.content.length > 180" class="journal-read-more" @click="toggleExpanded(entry.id)">{{ expandedIds.has(entry.id) ? '收起' : '继续读' }}</button>
          <div v-if="entry.tags.length" class="journal-tags"><span v-for="tag in entry.tags" :key="tag">{{ tag }}</span></div>
          <div class="journal-actions">
            <button v-if="entry.sourceMessageIds?.length" @click="$emit('source', entry.sourceMessageIds)"><RoomIcon name="chat" />那时的对话</button>
            <span class="journal-actions-spacer" />
            <button :disabled="saving" @click="edit(entry)">修订</button><button :disabled="saving" @click="remove(entry)">删除</button>
          </div>
        </template>
      </article>
    </section>
    <button v-if="hasMore && entries.length" class="journal-load-more room-text-button" :disabled="loading" @click="loadMore">{{ loading ? '正在加载…' : '加载更多日记' }}</button>
    <p v-if="entries.length && !hasMore" class="journal-end">已显示全部日记</p>
  </RoomDrawer>
</template>

<script setup>
import { ref, computed, watch, onBeforeUnmount } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { queryDiary, writeDiary, deleteDiary } from '../../services/standalone'
import RoomDrawer from './RoomDrawer.vue'
import RoomIcon from './RoomIcon.vue'
const props = defineProps({ agent: { type: Object, required: true }, tone: { type: String, default: 'pero' } })
const emit = defineEmits(['close', 'source', 'updated'])
const entries = ref([])
const keyword = ref('')
const date = ref('')
const loading = ref(false)
const saving = ref(false)
const error = ref('')
const hasMore = ref(false)
const editing = ref(null)
const expandedIds = ref(new Set())
let requestVersion = 0
let debounce = null
let disposed = false
const pageSize = 40
const groups = computed(() => {
  const result = []
  for (const entry of entries.value) {
    if (result.at(-1)?.date !== entry.date) result.push({ date: entry.date, entries: [] })
    result.at(-1).entries.push(entry)
  }
  return result
})
function formatDate(value) { return new Date(`${value}T12:00:00`).toLocaleDateString('zh-CN', { month: 'long', day: 'numeric', weekday: 'short', year: 'numeric' }) }
async function fetchPage(append) {
  const version = ++requestVersion
  const agentId = props.agent.id
  loading.value = true
  error.value = ''
  if (!append) { entries.value = []; hasMore.value = false }
  try {
    const result = await queryDiary(agentId, { keyword: keyword.value.trim(), from: date.value || null, to: date.value || null, limit: pageSize, offset: append ? entries.value.length : 0 })
    if (disposed || version !== requestVersion || agentId !== props.agent.id) return
    entries.value = append ? [...entries.value, ...result] : result
    hasMore.value = result.length === pageSize
  } catch (failure) { if (!disposed && version === requestVersion) error.value = failure.message }
  finally { if (!disposed && version === requestVersion) loading.value = false }
}
function reload() { clearTimeout(debounce); editing.value = null; expandedIds.value = new Set(); fetchPage(false) }
function loadMore() { if (!loading.value) fetchPage(true) }
function toggleExpanded(id) { const next = new Set(expandedIds.value); if (next.has(id)) next.delete(id); else next.add(id); expandedIds.value = next }
function edit(entry) { editing.value = { id: entry.id, content: entry.content, date: entry.date } }
async function save(entry) {
  if (saving.value) return
  const agentId = props.agent.id
  const draft = { id: entry.id, expectedRevision: entry.revision, content: editing.value.content, date: editing.value.date, tags: entry.tags }
  saving.value = true
  try {
    await writeDiary(agentId, draft)
    if (disposed || agentId !== props.agent.id) return
    editing.value = null; emit('updated'); await fetchPage(false)
  } catch (failure) {
    if (!disposed) { ElMessage.error(failure.message); if (failure.code === 'STALE_DIARY') await fetchPage(false) }
  } finally { if (!disposed) saving.value = false }
}
async function remove(entry) {
  const agentId = props.agent.id
  try { await ElMessageBox.confirm('删除后，这篇日记将不再用于后续聊天。', '删除这篇日记？', { confirmButtonText: '删除', cancelButtonText: '保留', type: 'warning' }) }
  catch { return }
  if (disposed || agentId !== props.agent.id || saving.value) return
  saving.value = true
  try { await deleteDiary(agentId, entry.id); if (!disposed) { emit('updated'); await fetchPage(false) } }
  catch (failure) { if (!disposed) ElMessage.error(failure.message) }
  finally { if (!disposed) saving.value = false }
}
watch([keyword, date], () => { clearTimeout(debounce); requestVersion++; debounce = setTimeout(reload, 300) })
watch(() => props.agent.id, () => { requestVersion++; entries.value = []; reload() }, { immediate: true })
onBeforeUnmount(() => { disposed = true; requestVersion++; clearTimeout(debounce) })
</script>

<style scoped>
.journal-search { display: flex; flex-wrap: wrap; gap: 9px; margin-bottom: 24px; }
.journal-search input { border: 1px solid var(--room-line); padding: 11px 12px; border-radius: 5px; min-width: 0; color: var(--room-ink); background: white; font-size: 13px; }
.journal-search input[type='search'] { flex: 1 1 190px; }
.journal-search input[type='date'] { flex: 0 1 150px; }
.journal-search input:focus, .journal-editor input:focus, .journal-editor textarea:focus { outline: 2px solid var(--room-accent); outline-offset: 2px; }
.journal-loading { color: var(--room-muted); text-align: center; padding: 35px; }
.journal-day h3 { display: flex; align-items: center; gap: 12px; font-size: 12px; font-weight: 700; color: var(--room-accent); margin: 24px 0 14px; }
.journal-date-line { height: 1px; background: var(--room-line); flex: 1; }
.journal-entry { background: white; border: 1px solid var(--room-line); border-left: 2px solid var(--room-accent-bright); border-radius: 3px 12px 3px 3px; padding: 15px; margin: 12px 0; }
.journal-body { font-size: 14px; white-space: pre-wrap; overflow-wrap: anywhere; line-height: 1.75; margin: 0; }
.journal-body.is-folded { display: -webkit-box; -webkit-line-clamp: 7; -webkit-box-orient: vertical; overflow: hidden; }
.journal-tags { display: flex; flex-wrap: wrap; gap: 7px; margin-top: 15px; }
.journal-tags > span { color: var(--room-accent); font-size: 11px; background: var(--room-accent-soft); border-radius: 6px; padding: 4px 7px; }
.journal-actions { display: flex; align-items: center; flex-wrap: wrap; gap: 8px; margin-top: 16px; padding-top: 11px; border-top: 1px solid var(--room-line); }
.journal-actions button, .journal-read-more { border: 0; background: transparent; padding: 6px 0; font-size: 11px; color: var(--room-muted); display: flex; align-items: center; gap: 5px; }
.journal-actions button:first-child { color: var(--room-accent); }
.journal-actions svg { width: 13px; height: 13px; }
.journal-actions-spacer { flex: 1; }
.journal-read-more { color: var(--room-accent); margin-top: 6px; }
.journal-editor { display: flex; flex-direction: column; gap: 15px; }
.journal-editor label { display: flex; flex-direction: column; gap: 8px; font-size: 12px; color: var(--room-muted); }
.journal-editor input, .journal-editor textarea { border: 1px solid var(--room-line); background: white; color: var(--room-ink); padding: 11px; border-radius: 5px; line-height: 1.8; resize: vertical; max-height: 360px; font-size: 13px; }
.journal-editor-actions { display: flex; justify-content: flex-end; gap: 8px; }
.journal-load-more { display: block; margin: 24px auto 0; }
.journal-end { font-size: 11px; color: var(--room-muted); text-align: center; margin: 30px 0 4px; letter-spacing: .05em; }
@supports (-webkit-touch-callout: none) { .journal-search input, .journal-editor input, .journal-editor textarea { font-size: 16px; } }
</style>
