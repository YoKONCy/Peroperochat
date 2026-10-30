<template>
  <div class="memory-entry-actions" :class="{ 'is-archive-only': !allowEdit && !allowDelete }" role="group" :aria-label="tr('记忆操作')">
    <span v-if="showStatus" class="memory-status-label" :data-archived="record.status === 'archived'"><RemoteIcon :name="record.status === 'archived' ? 'archive' : 'memory'" />{{ tr(record.status === 'archived' ? '已归档' : '当前记忆') }}</span>
    <div v-if="editable" class="memory-entry-buttons">
      <RemoteAction v-if="allowEdit" icon="edit" tone="primary" class="memory-edit-button" :disabled="locked" @click="$emit('edit')">{{ tr('编辑记忆') }}</RemoteAction>
      <RemoteAction v-if="record.status === 'active'" icon="archive" class="memory-archive-button" :disabled="locked" @click="open('archive')">{{ tr('归档记忆') }}</RemoteAction>
      <RemoteAction v-else icon="restore" class="memory-archive-button" :disabled="locked" @click="open('restore')">{{ tr('恢复归档') }}</RemoteAction>
      <RemoteAction v-if="allowDelete" icon="trash" tone="danger" class="memory-delete-button" :disabled="locked" :aria-label="tr('删除')" :title="tr('删除')" @click="open('delete')" />
    </div>
    <RemoteAction v-if="unresolved && !confirmation" icon="bell" @click="$emit('pending')">{{ tr('查看操作结果') }}</RemoteAction>
    <RoomDrawer v-if="confirmation" :title="tr(copy.title)" :busy="busy" @close="confirmation = null">
      <div class="memory-operation-confirm" :data-danger="confirmation.action === 'delete'">
        <span class="memory-confirm-symbol"><RemoteIcon :name="copy.icon" /></span>
        <p>{{ tr(copy.description) }}</p>
        <blockquote>{{ confirmation.summary || tr('一段记忆') }}</blockquote>
        <RemoteFeedback :message="error" />
        <RemoteAction v-if="unresolved" icon="bell" @click="$emit('pending')">{{ tr('查看操作结果') }}</RemoteAction>
      </div>
      <template #footer><div class="memory-confirm-footer"><RemoteAction icon="back" :disabled="busy" @click="confirmation = null">{{ tr('取消') }}</RemoteAction><RemoteAction :icon="copy.icon" :tone="confirmation.action === 'delete' ? 'danger' : 'primary'" :busy="busy" :disabled="locked" @click="apply">{{ tr(copy.label) }}</RemoteAction></div></template>
    </RoomDrawer>
  </div>
</template>
<script setup>
import { computed, ref, watch } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { remoteState } from '../../stores/remote'
import { t } from '../../i18n/remote'
const props = defineProps({ record: { type: Object, required: true }, disabled: Boolean, allowEdit: { type: Boolean, default: true }, allowDelete: { type: Boolean, default: true }, showStatus: { type: Boolean, default: true } })
const emit = defineEmits(['edit', 'changed', 'pending', 'busy'])
const { token, read, write, run, guard, busy, error, unresolved } = useRemoteManagement()
const tr = key => t(key, [], 'memory-actions')
const editable = computed(() => !!props.record.id && ['active', 'archived'].includes(props.record.status))
const locked = computed(() => props.disabled || busy.value || unresolved.value || remoteState.connection !== 'online' || !editable.value)
const confirmation = ref(null)
const descriptions = {
  archive: { title: '归档这条记忆？', label: '确认归档', icon: 'archive', description: '移到“已归档”，需要时可以恢复。相关聊天会保留。' },
  restore: { title: '恢复这条记忆？', label: '恢复到当前记忆', icon: 'restore', description: '恢复后，角色会再次参考这条记忆。' },
  delete: { title: '永久删除这条记忆？', label: '永久删除', icon: 'trash', description: '删除后无法恢复。只删除这条记忆，相关聊天会保留。' },
}
const copy = computed(() => descriptions[confirmation.value?.action] || descriptions.archive)
function prepareLeave() { if (busy.value) throw new Error(tr('正在处理，请稍等。')) }
defineExpose({ prepareLeave, unresolved })
watch(busy, value => emit('busy', value), { flush: 'sync' })
watch(() => props.record.id, () => { confirmation.value = null })
function open(action) {
  if (locked.value || !descriptions[action]) return
  error.value = ''
  confirmation.value = { action, id: String(props.record.id), status: props.record.status, summary: String(props.record.summary || props.record.narrative || '').slice(0, 160) }
}
async function apply() {
  if (locked.value || !confirmation.value) return
  const target = { ...confirmation.value }
  let change = null
  await run(async () => {
    const current = await read('memories.get', { id: target.id }); guard()
    if (String(current?.id) !== target.id || current.status !== target.status) throw new Error(tr('这条记忆已发生变化，请刷新后再操作。'))
    // 删除仅针对记忆条目，不向服务端请求删除来源聊天。
    await write(`memories.${target.action}`, { id: target.id, ...(target.action === 'delete' ? { params: { deleteSource: false } } : {}) }); guard()
    confirmation.value = null
    change = { token, id: target.id, action: target.action, agentId: current.agentId }
  })
  if (change) {
    try { guard() } catch { return }
    window.dispatchEvent(new CustomEvent('ppc:memories-changed', { detail: change })); emit('changed', change)
  }
}
</script>
