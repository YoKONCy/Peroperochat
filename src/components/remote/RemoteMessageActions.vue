<template>
  <RoomDrawer :title="view === 'edit' ? '编辑消息' : plan ? '保存并重新生成' : previewData ? '确认回滚' : message ? '消息操作' : '回滚会话'" :busy="busy" :before-close="beforeClose" @close="emit('close')">
    <RemoteFeedback :message="error" /><RemoteFeedback :message="notice" tone="success" />
    <button v-if="unresolved" class="room-text-button" @click="$emit('pending')">操作结果待核对，前往通知</button>
    <div v-if="message && view === 'menu'" class="remote-action-menu">
      <button type="button" data-ui-tone="blue" @click="copy"><RemoteIcon name="copy" /><span><strong>复制正文</strong><small>复制这条消息的文字</small></span></button>
      <button type="button" data-ui-tone="violet" :disabled="busy || blocked" @click="view = 'edit'"><RemoteIcon name="edit" /><span><strong>编辑消息</strong><small>仅保存，或保存后重新生成</small></span><RemoteIcon name="next" /></button>
      <button type="button" data-ui-tone="amber" :disabled="busy || blocked || unresolved" @click="preview"><RemoteIcon name="refresh" /><span><strong>回滚到此处</strong><small>撤回本轮及后续对话，预览相关文件变更</small></span><RemoteIcon name="next" /></button>
    </div>
    <form v-if="view === 'edit'" @submit.prevent="saveOnly">
      <label class="remote-field">消息正文<textarea v-model="content" rows="8" maxlength="131072" required :disabled="busy" /></label>
      <p class="remote-help">仅保存会保留后续对话；重新生成会替换这之后的内容。</p>
      <div class="remote-purpose-actions"><RemoteAction icon="back" :disabled="busy" @click="view = 'menu'">返回</RemoteAction><RemoteAction icon="check" type="submit" :disabled="busy || blocked || unresolved || !content.trim() || content === message.content">仅保存</RemoteAction><RemoteAction icon="refresh" tone="primary" :disabled="busy || blocked || unresolved || !content.trim()" @click="prepare">保存并重新生成</RemoteAction></div>
    </form>
    <RemoteAction v-if="!message && !previewData" icon="refresh" :disabled="busy || blocked" @click="preview">查看会话回滚范围</RemoteAction>
    <template v-if="previewData"><RemoteRewindPreview :preview="previewData" :regenerating="!!plan" /><div class="remote-purpose-actions"><RemoteAction v-if="!plan" icon="back" :disabled="busy" @click="previewData = null; view = 'menu'">返回</RemoteAction><RemoteAction icon="refresh" :tone="plan ? 'primary' : 'danger'" :disabled="busy || blocked || unresolved" @click="confirm">{{ plan ? phase === 'rewound' ? '重试生成' : '回滚并重新生成' : '回滚并撤回' }}</RemoteAction></div></template>
    <RemoteAction v-if="plan && !previewData" icon="refresh" :disabled="busy || unresolved" @click="confirm">重新读取回滚预览</RemoteAction>
    <p v-if="discarding" class="remote-inline-confirm">有未保存的编辑，确认离开？<button @click="emit('close')">放弃编辑</button><button @click="discarding = false">继续编辑</button></p>
  </RoomDrawer>
</template>
<script setup>
import RemoteFeedback from './RemoteFeedback.vue'
import { ref, onBeforeUnmount } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RemoteAction from './RemoteAction.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteRewindPreview from './RemoteRewindPreview.vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { performRemote, refreshRemotePendingRequests } from '../../stores/remote'
import { remoteClient } from '../../services/remote'
const props = defineProps({ threadId: { type: String, required: true }, agentId: String, message: Object, blocked: Boolean })
const emit = defineEmits(['close', 'updated', 'pending'])
const { token, read, run, guard, busy, error, unresolved, fail } = useRemoteManagement()
const content = ref(props.message?.content || ''), view = ref('menu'), previewData = ref(null), plan = ref(null), phase = ref(''), notice = ref(''), discarding = ref(false)
async function action(value) { guard(); try { return await performRemote(value, { token }) } catch (failure) { fail(failure); throw failure } }
async function refreshPending() { await refreshRemotePendingRequests().catch(() => {}) }
function target(kind) { const id = props.message ? Number(props.message.messageId) : null; if (id !== null && (!Number.isSafeInteger(id) || id <= 0)) throw new Error('消息身份无效，请刷新后再操作'); return { kind, id: props.threadId, messageId: id, wholeThread: !props.message } }
async function preview() { await run(async () => { const input = target('rewind'); previewData.value = await read('thread.rewind_preview', { id: props.threadId, body: { wholeThread: input.wholeThread, ...(input.messageId ? { messageId: input.messageId } : {}) } }); view.value = 'review' }) }
function changed(kind = 'edit') { emit('updated', { kind }); window.dispatchEvent(new CustomEvent('ppc:conversation-changed', { detail: { token, threadId: props.threadId } })) }
async function saveOnly() { await run(async () => { const input = target('rewind'); await action({ kind: 'edit_message', threadId: props.threadId, id: String(input.messageId), content: content.value, expectedRevision: props.message.revision }); guard(); changed(); emit('close') }) }
async function prepare() {
  await run(async () => {
    try {
      const input = target('rewind')
      const result = await remoteClient.prepareRegeneration({ threadId: props.threadId, agentId: props.agentId || '', messageId: input.messageId, content: content.value, revision: props.message.revision }, token)
      guard(); plan.value = result.data.planId; phase.value = result.data.state; previewData.value = result.data.preview || null; view.value = 'review'; notice.value = result.data.message || '消息修订已保存，请确认重新生成范围。'; changed()
    } catch (failure) { fail(failure); throw failure }
    finally { await refreshPending() }
  })
}
async function confirm() {
  await run(async () => {
    if (plan.value) {
      try {
        const { data } = await remoteClient.commitRegeneration(plan.value, token); guard()
        phase.value = data.state
        if (data.state === 'review') { previewData.value = data.preview; notice.value = '回滚清单已更新，请重新确认。'; return }
        if (data.state === 'rewound') { notice.value = data.message; changed('rewind'); return }
        plan.value = null; changed('rewind'); emit('close')
      } catch (failure) { fail(failure); throw failure }
      finally { await refreshPending() }
    } else {
      if (!previewData.value) throw new Error('请先读取回滚清单')
      try {
        const { data } = await remoteClient.confirmRewind({ threadId: props.threadId, wholeThread: !props.message, messageId: props.message ? Number(props.message.messageId) : null, expected: previewData.value }, token)
        guard(); if (data.state === 'review') { previewData.value = data.preview; notice.value = '回滚范围已变化，请重新确认。'; return }
        changed('rewind'); emit('close')
      } catch (failure) { fail(failure); throw failure }
      finally { await refreshPending() }
    }
  })
}
async function copy() { try { await navigator.clipboard.writeText(props.message?.content || ''); notice.value = '正文已复制' } catch { error.value = '复制未完成，可长按选择正文' } }
function beforeClose() { if (props.message && !plan.value && content.value !== props.message.content) { discarding.value = true; return false } return true }
onBeforeUnmount(() => { if (plan.value && ['saved', 'review'].includes(phase.value)) remoteClient.discardRegeneration(plan.value, token).catch(() => {}) })
</script>
