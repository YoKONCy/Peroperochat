<template>
  <RoomDrawer :title="message ? '消息操作' : '回退会话'" :busy="busy" @close="$emit('close')"><p v-if="error" class="room-error">{{ error }}</p><p v-if="unresolved" class="room-error">操作结果待核对。<button @click="$emit('pending')">前往通知</button></p>
    <template v-if="message"><form @submit.prevent="operation = 'edit'"><label class="remote-field">消息正文<textarea v-model="content" rows="8" maxlength="131072" required :disabled="busy" /></label><button class="room-primary-button" :disabled="busy || unresolved || !content.trim() || content === message.content">保存编辑</button></form><div class="remote-manager-actions"><button :disabled="busy" @click="copy">复制正文</button><button :disabled="busy || unresolved" @click="operation = 'delete'">删除此消息</button><button :disabled="busy || unresolved" @click="operation = 'pair'">删除此轮对话</button><button :disabled="busy || unresolved" @click="preview">回退到这轮之前</button></div></template>
    <button v-else-if="!previewData" class="room-secondary-button" :disabled="busy" @click="preview">预览整个会话的回退范围</button>
    <div v-if="previewData" class="remote-manager-card"><h2>回退预览</h2><p class="remote-help">按服务端清单处理对话和文件；不保证撤销已执行的所有外部操作。</p><RemoteValue :value="previewData" /><button class="room-danger-button" :disabled="busy || unresolved" @click="operation = 'rewind'">确认回退</button></div>
    <RoomDrawer v-if="operation" title="确认消息操作" :busy="busy" @close="operation = null"><p v-if="error" class="room-error">{{ error }}</p><p>{{ { edit: '保存对这条消息的修改？', delete: '删除这条消息？', pair: '删除此消息所在的整轮对话？', rewind: '按预览范围回退对话与相关文件？' }[operation] }}</p><button class="room-primary-button" :disabled="busy || unresolved" @click="submit">确认</button></RoomDrawer>
  </RoomDrawer>
</template>
<script setup>
import { ref } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RemoteValue from './RemoteValue.vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { performRemote } from '../../stores/remote'
const props = defineProps({ threadId: { type: String, required: true }, message: Object })
const emit = defineEmits(['close', 'updated', 'pending'])
const { read, run, guard, busy, error, unresolved, fail } = useRemoteManagement()
const content = ref(props.message?.content || ''), operation = ref(null), previewData = ref(null)
async function action(value) { try { const data = await performRemote(value); guard(); return data } catch (failure) { fail(failure); throw failure } }
function rewind(kind) { const messageId = props.message ? Number(props.message.messageId) : null; if (messageId !== null && (!Number.isSafeInteger(messageId) || messageId <= 0)) throw new Error('消息身份无效，无法回退'); return { kind, id: props.threadId, messageId, wholeThread: !props.message } }
async function preview() { await run(async () => { const input = rewind('rewind_preview'); previewData.value = await read('thread.rewind_preview', { id: props.threadId, body: { wholeThread: input.wholeThread, ...(input.messageId ? { messageId: input.messageId } : {}) } }) }) }
async function submit() { await run(async () => { const op = operation.value; if (op === 'rewind') { if (!previewData.value) throw new Error('请先读取回退预览'); await action(rewind('rewind')) } else await action({ kind: op === 'edit' ? 'edit_message' : op === 'pair' ? 'delete_message_pair' : 'delete_message', threadId: props.threadId, id: String(props.message.messageId), ...(op === 'edit' ? { content: content.value, expectedRevision: props.message.revision } : {}) }); emit('updated'); emit('close') }) }
async function copy() { try { await navigator.clipboard.writeText(content.value) } catch { error.value = '复制未完成，可长按选择正文' } }
</script>
