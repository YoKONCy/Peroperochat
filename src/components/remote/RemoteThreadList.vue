<template>
  <div class="conversation-thread-list">
    <article v-for="item in threads" :key="item.id" class="conversation-thread-row" :class="{ 'is-current': item.id === selectedId, 'is-holding': holdingId === item.id }">
      <button type="button" class="conversation-thread-open" :disabled="disabled" :aria-current="item.id === selectedId ? 'true' : undefined"
        @pointerdown="startHold($event, item)" @pointermove="moveHold" @pointerup="cancelHold" @pointercancel="cancelHold" @pointerleave="cancelHold"
        @contextmenu.prevent="showActions(item)" @click="open($event, item)">
        <RemoteAvatar :id="item.agentId || agentId" :label="agents.find(agent => agent.id === (item.agentId || agentId))?.name" />
        <span><strong>{{ item.title || t('新的对话') }}</strong><small>{{ remoteDate(item.updatedAt || item.createdAt) }}</small></span>
      </button>
      <button type="button" class="conversation-thread-more" :disabled="disabled || !online" :aria-label="t('管理会话：{0}', [item.title || t('新的对话')])" :title="t('会话管理')" @click="showActions(item)"><RoomIcon name="more" /></button>
    </article>
  </div>
</template>

<script setup>
import { ref, onBeforeUnmount, watch } from 'vue'
import RemoteAvatar from './RemoteAvatar.vue'
import RoomIcon from '../companion/RoomIcon.vue'
import { t, remoteDate } from '../../i18n/remote'

const props = defineProps({ threads: { type: Array, default: () => [] }, agents: { type: Array, default: () => [] }, agentId: String, selectedId: String, online: Boolean, disabled: Boolean })
const emit = defineEmits(['open', 'manage'])
const holdingId = ref(null)
let timer = null, pointer = null, suppressedClick = null
function cancelHold() { clearTimeout(timer); timer = null; pointer = null; holdingId.value = null }
function showActions(item) {
  cancelHold()
  if (!props.disabled && props.online) emit('manage', item)
}
function startHold(event, item) {
  cancelHold()
  suppressedClick = null
  if (props.disabled || !props.online || !event.isPrimary || event.button !== 0) return
  pointer = { id: event.pointerId, x: event.clientX, y: event.clientY }
  holdingId.value = item.id
  timer = setTimeout(() => {
    // 长按只打开操作，不让松手后补发的点击进入会话。
    suppressedClick = item.id
    showActions(item)
  }, 550)
}
function moveHold(event) {
  if (pointer && (event.pointerId !== pointer.id || Math.hypot(event.clientX - pointer.x, event.clientY - pointer.y) > 8)) cancelHold()
}
function open(event, item) {
  cancelHold()
  if (event.detail && suppressedClick === item.id) { suppressedClick = null; return }
  suppressedClick = null
  if (!props.disabled) emit('open', item)
}
watch(() => [props.threads, props.disabled, props.online], cancelHold)
onBeforeUnmount(cancelHold)
</script>

<style scoped>
.conversation-thread-row{display:flex;align-items:center;gap:6px;border-bottom:1px solid var(--room-line);border-radius:12px;transition:background .15s ease}
.conversation-thread-row.is-current{background:var(--room-accent-soft)}
.conversation-thread-open{display:flex;align-items:center;gap:11px;padding:15px 8px;min-width:0;flex:1;border:0;background:none;color:var(--room-ink);font:inherit;text-align:left;user-select:none;-webkit-touch-callout:none}
.conversation-thread-open>.character-avatar{width:44px;height:44px;min-width:44px;flex:none;border-radius:14px}
.conversation-thread-open>span:not(.character-avatar){flex:1;min-width:0}
.conversation-thread-open strong{font-size:14px;display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-weight:600}
.conversation-thread-open small{display:block;color:var(--room-muted);font-size:11px;margin-top:6px}
.conversation-thread-more{display:grid;place-items:center;width:44px;height:44px;flex:none;margin-right:3px;border:1px solid var(--room-line);border-radius:13px;background:var(--room-surface);color:var(--room-accent);transition:transform .15s ease,background .15s ease}
.conversation-thread-more>svg{width:20px;height:20px}
.conversation-thread-more:active{transform:scale(.94);background:var(--room-accent-soft)}
.conversation-thread-row.is-holding{background:var(--room-accent-soft)}
.conversation-thread-open:focus-visible,.conversation-thread-more:focus-visible{outline:2px solid var(--room-accent);outline-offset:2px}
.conversation-thread-row button:disabled{opacity:.45;cursor:default}
@media(prefers-reduced-motion:reduce){.conversation-thread-row,.conversation-thread-more{transition:none}}
:global([data-reduce-motion=true]) .conversation-thread-row,:global([data-reduce-motion=true]) .conversation-thread-more{transition:none}
</style>
