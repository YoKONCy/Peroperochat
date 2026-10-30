<template>
  <RoomDrawer :title="tr(isMemory ? '编辑记忆' : '编辑事实')" variant="archive" :busy="busy || memoryActionBusy" :before-close="beforeClose" @close="$emit('close')">
    <div class="archive-edit-scroll">
    <form ref="form" class="archive-editor" @submit.prevent="save">
      <div class="archive-edit-context"><RemoteIcon :name="isMemory ? 'memory' : 'document'" /><span>{{ tr('保存为新版本，原来的记录仍会保留。') }}</span></div>
      <p v-if="!editable" class="archive-edit-hint">{{ tr('这条记录已发生变化，请返回列表刷新后再编辑。') }}</p>
      <label class="remote-field"><span>{{ tr('正文') }}</span><textarea v-model="draft.text" rows="7" required maxlength="20000" :disabled="locked" /></label>
      <template v-if="isMemory">
        <div class="archive-edit-scores"><label v-for="item in scores" :key="item.key" class="remote-field"><span>{{ tr(item.label) }}</span><input v-model.number="draft[item.key]" type="number" min="0" max="10" step="1" required :disabled="locked" /></label></div>
        <details class="archive-edit-tags"><summary><RemoteIcon name="memory" />{{ tr('人物、地点与情绪') }}</summary><p class="archive-edit-hint">{{ tr('每行一项，留空可清除。') }}</p><label v-for="item in tagFields" :key="item.key" class="remote-field"><span>{{ tr(item.label) }}</span><textarea v-model="draft[item.key]" rows="2" :disabled="locked" /></label></details>
        <label class="archive-edit-vector"><span>{{ tr('同时更新相似记忆检索') }}</span><RoomSwitch v-model="draft.regenerateVector" :label="tr('同时更新相似记忆检索')" :disabled="locked" /></label>
        <p class="archive-edit-hint">{{ tr(draft.regenerateVector ? '开启后会重新整理检索信息，可能需要多等一会儿。' : '关闭时只更新文字搜索，相似记忆仍可能参考旧内容。') }}</p>
      </template>
      <template v-else><label class="remote-field"><span>{{ tr('事实来源') }}</span><input v-model="draft.source" maxlength="2000" :disabled="locked" /></label><label class="remote-field"><span>{{ tr('可信度（0–1，可留空）') }}</span><input v-model="draft.confidence" type="number" min="0" max="1" step="any" :disabled="locked" /></label></template>
      <RemoteFeedback :message="error" />
      <p v-if="conflict" class="archive-edit-hint" role="status">{{ tr('这条记录已被修改。草稿还在，请复制正文后返回列表刷新。') }}</p>
      <div v-if="unresolved" class="archive-edit-recovery"><p>{{ tr('保存结果还在确认中，先保留草稿。') }}</p><RemoteAction icon="bell" @click="$emit('pending')">{{ tr('查看保存结果') }}</RemoteAction></div>
    </form>
    </div>
    <template #footer><div class="archive-edit-footer"><span>{{ tr(dirty ? '有未保存的修改' : '未修改') }}<small v-if="isMemory && dirty">{{ t('保存修改后可归档或恢复记忆。', [], 'memory-actions') }}</small></span><RemoteMemoryActions v-if="isMemory" ref="memoryActions" class="archive-edit-archive" :record="original" :disabled="locked || dirty" :allow-edit="false" :allow-delete="false" :show-status="false" @changed="memoryChanged" @busy="memoryActionBusy = $event" @pending="$emit('pending')" /><RemoteAction class="archive-edit-save" icon="check" tone="primary" :busy="busy" :disabled="locked || !dirty" @click="save">{{ tr('保存修改') }}</RemoteAction></div></template>
    <RoomDrawer v-if="confirmDiscard" :title="tr('放弃尚未保存的修改？')" @close="cancelDiscard"><div class="remote-purpose-actions"><RemoteAction icon="back" @click="cancelDiscard">{{ tr('继续编辑') }}</RemoteAction><RemoteAction icon="trash" tone="danger" @click="discard">{{ tr('放弃修改') }}</RemoteAction></div></RoomDrawer>
  </RoomDrawer>
</template>
<script setup>
import { computed, ref, onBeforeUnmount } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RoomSwitch from '../companion/RoomSwitch.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import RemoteMemoryActions from './RemoteMemoryActions.vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { t } from '../../i18n/remote'

const props = defineProps({ kind: { type: String, required: true }, record: { type: Object, required: true } })
const emit = defineEmits(['close', 'saved', 'changed', 'pending'])
const tr = (key, values) => t(key, values, 'archive-edit')
const { write, run, guard, error, busy, unresolved } = useRemoteManagement()
const isMemory = props.kind === 'memories'
const original = JSON.parse(JSON.stringify(props.record))
const editable = !!original.id && (isMemory ? ['active', 'archived'].includes(original.status) : original.status === 'active')
const list = values => Array.isArray(values) ? values.join('\n') : ''
const draft = ref({ text: (isMemory ? original.narrative : original.statement) || '', importance: original.importance ?? 5, valence: original.affect?.valence ?? 5, arousal: original.affect?.arousal ?? 0, tones: list(original.affect?.tones), participants: list(original.participants), places: list(original.places), objects: list(original.objects), topics: list(original.topics), source: original.source || '', confidence: original.confidence ?? '', regenerateVector: false })
const scores = [{ key: 'importance', label: '重要度' }, { key: 'valence', label: '情绪明朗度' }, { key: 'arousal', label: '情绪强烈度' }]
const tagFields = [{ key: 'tones', label: '情绪标签' }, { key: 'participants', label: '人物' }, { key: 'places', label: '地点' }, { key: 'objects', label: '物品' }, { key: 'topics', label: '主题' }]
const snapshot = () => JSON.stringify(draft.value), initial = snapshot()
const dirty = computed(() => snapshot() !== initial)
const form = ref(null), conflict = ref(false), confirmDiscard = ref(false)
const memoryActions = ref(null), memoryActionBusy = ref(false)
const locked = computed(() => busy.value || memoryActionBusy.value || memoryActions.value?.unresolved || unresolved.value || conflict.value || !editable)
let lastBody = '', operationId = '', navigationDecision = null
function beforeClose() { if (dirty.value) { confirmDiscard.value = true; return false } return true }
function cancelDiscard() { confirmDiscard.value = false; navigationDecision?.(false); navigationDecision = null }
function discard() { confirmDiscard.value = false; navigationDecision?.(true); navigationDecision = null; emit('close') }
async function prepareLeave() {
  await memoryActions.value?.prepareLeave?.()
  if (busy.value) throw new Error(tr('正在保存，请稍等。'))
  if (!dirty.value) return
  confirmDiscard.value = true
  if (!await new Promise(resolve => { navigationDecision = resolve })) throw Object.assign(new Error(''), { code: 'UI_CANCELLED' })
}
function memoryChanged(change) { emit('changed', change); emit('close') }
defineExpose({ prepareLeave })
onBeforeUnmount(() => { navigationDecision?.(false); navigationDecision = null })
function parseTags(value) { return [...new Set(value.split(/\r?\n/).map(item => item.trim()).filter(Boolean))] }
async function save() {
  if (locked.value || !dirty.value || !form.value?.reportValidity()) return
  await run(async () => {
    const value = draft.value, text = value.text.trim()
    if (!text) throw new Error(tr('正文不能为空。'))
    const body = isMemory ? { narrative: text, importance: Number(value.importance), affect: { tones: parseTags(value.tones), valence: Number(value.valence), arousal: Number(value.arousal) }, participants: parseTags(value.participants), places: parseTags(value.places), objects: parseTags(value.objects), topics: parseTags(value.topics), expectedStatus: original.status, regenerateVector: value.regenerateVector } : { statement: text, source: value.source.trim(), ...(String(value.confidence).trim() === '' ? {} : { confidence: Number(value.confidence) }) }
    if (isMemory && [body.affect.tones, body.participants, body.places, body.objects, body.topics].some(items => items.length > 100 || items.some(item => [...item].length > 200))) throw new Error(tr('每组最多 100 项，每项最多 200 字。'))
    // 相同草稿沿用服务端操作编号；网络中断后先核对，不盲目生成第二次修订。
    const serialized = JSON.stringify(body)
    if (serialized !== lastBody) { operationId = crypto.randomUUID(); lastBody = serialized }
    try {
      const saved = await write(`${props.kind}.edit`, { id: original.id, body: { ...body, operationId } })
      guard()
      if (!saved?.id) { unresolved.value = true; throw Object.assign(new Error(tr('保存结果还在确认中，先保留草稿。')), { outcome: 'unknown' }) }
      emit('saved', saved)
    } catch (failure) {
      if (failure.code === 'REMOTE_CONFLICT') conflict.value = true
      throw failure
    }
  })
}
</script>
<style scoped>
/* 滚动区独立铺满抽屉，表单在其中居中，不参与通用编辑弹层的横向分配。 */
.archive-edit-scroll { display: block; width: 100%; height: 100%; min-width: 0; min-height: 0; padding: 20px 18px 28px; overflow-y: auto; overflow-x: hidden; overscroll-behavior: contain; scrollbar-width: thin; }
.archive-editor { display: grid; grid-template-columns: minmax(0, 1fr); gap: 18px; width: 100%; max-width: 680px; min-width: 0; margin: 0 auto; padding: 0; text-align: start; }
.archive-editor > * { min-width: 0; max-width: 100%; }
.archive-editor .remote-field > textarea, .archive-editor .remote-field > input { display: block; width: 100%; min-width: 0; max-width: 100%; box-sizing: border-box; }
.archive-editor > .remote-field > textarea { min-height: 210px; resize: vertical; }
.archive-editor textarea { resize: vertical; line-height: 1.8; }
.archive-edit-vector { display: grid; grid-template-columns: minmax(0, 1fr) auto; align-items: center; gap: 12px; font-size: var(--ui-label); color: var(--ui-ink); line-height: 1.7; }
.archive-edit-context { display: flex; width: 100%; align-items: center; gap: 10px; padding: 13px; border: 1px solid var(--ui-violet-edge); border-radius: 16px 16px 16px 6px; background: var(--ui-violet-soft); color: var(--ui-violet); font-size: var(--ui-caption); line-height: 1.7; }
.archive-edit-context > span { min-width: 0; overflow-wrap: anywhere; }
.archive-edit-context > svg { width: 22px; height: 22px; flex: none; }
.archive-edit-scores { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 9px; align-items: end; }
.archive-edit-scores .remote-field { min-width: 0; font-size: var(--ui-caption); }
.archive-edit-tags { border: 1px solid var(--ui-line); border-radius: 16px; padding: 14px; background: var(--ui-paper); }
.archive-edit-tags summary { display: flex; align-items: center; gap: 8px; color: var(--ui-violet); cursor: pointer; font-size: var(--ui-label); line-height: 1.7; }
.archive-edit-tags summary > svg { width: 19px; height: 19px; flex: none; }
.archive-edit-tags[open] .remote-field { margin-top: 12px; }
.archive-edit-hint { margin: 0; font-size: var(--ui-caption); line-height: 1.7; color: var(--ui-muted); }
.archive-edit-tags[open] > .archive-edit-hint { margin-top: 12px; }
.archive-edit-recovery { border-left: 3px solid var(--ui-amber); padding: 10px 12px; background: var(--ui-amber-soft); border-radius: 12px; }
.archive-edit-recovery p { font-size: var(--ui-caption); line-height: 1.7; }
.archive-edit-footer { display: grid; grid-template-columns: minmax(0, 1fr) auto; align-items: center; gap: 10px 12px; width: 100%; max-width: 680px; min-width: 0; margin: 0 auto; }
.archive-edit-footer > span { grid-column: 1 / -1; min-width: 0; overflow-wrap: anywhere; font-size: var(--ui-caption); color: var(--ui-muted); }
.archive-edit-footer > span > small { display: block; margin-top: 3px; font-size: inherit; line-height: 1.7; }
.archive-edit-save { grid-column: 2; min-width: 0; }
.archive-edit-archive { min-width: 0; }
</style>
