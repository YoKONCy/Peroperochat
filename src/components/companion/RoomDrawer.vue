<template>
  <Teleport to="body">
    <Transition name="room-drawer" appear @after-leave="$emit('close')">
      <div v-if="visible" class="room-drawer-overlay companion-surface" :class="[`room-drawer-overlay--${variant}`, { 'remote-themed': remoteTheme, 'standalone-design': !remoteTheme && localTheme }]" :data-tone="tone" :data-remote-theme="remoteTheme?.mode" :data-decoration="remoteTheme?.decoration" :data-bubble="remoteTheme?.bubble" :data-reduce-motion="remoteTheme ? remoteTheme.reduceMotion : localTheme?.reduceMotion" :style="[viewportStyle, remoteTheme ? remoteTheme.style : localTheme?.style]" @click.self="requestClose">
        <section ref="panel" class="room-drawer-panel" role="dialog" aria-modal="true" :aria-label="title" tabindex="-1" @keydown="handleKeydown">
          <RoomAccent v-if="!remoteTheme && localTheme" class="drawer-accent" kind="orbit" />
          <header class="room-drawer-header">
            <RemoteIcon v-if="remoteTheme && remoteTheme.decoration !== 'none'" class="drawer-emblem" :name="remoteTheme.decoration" artwork />
            <CompanionEmblem v-else-if="!remoteTheme" class="drawer-emblem" />
            <div><span v-if="subtitle" class="room-drawer-eyebrow">{{ subtitle }}</span><h2>{{ title }}</h2></div>
            <button class="room-icon-button" aria-label="关闭" :disabled="busy" @click="requestClose"><component :is="remoteTheme ? RemoteIcon : RoomIcon" name="close" /></button>
          </header>
          <div class="room-drawer-content"><slot /></div>
          <footer v-if="$slots.footer" class="room-drawer-footer"><slot name="footer" /></footer>
        </section>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref, inject, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { REMOTE_THEME } from '../../services/remoteTheme'
import { LOCAL_APPEARANCE } from '../../stores/appearance'
import RoomIcon from './RoomIcon.vue'
import RoomAccent from './RoomAccent.vue'
import CompanionEmblem from './CompanionEmblem.vue'
import RemoteIcon from '../remote/RemoteIcon.vue'
import { useRoomViewport } from '../../composables/useRoomViewport'
import { registerFeedbackSurface } from '../../services/appFeedback'
const props = defineProps({ title: { type: String, required: true }, subtitle: { type: String, default: '' }, tone: { type: String, default: 'pero' }, variant: { type: String, default: 'default' }, busy: Boolean, beforeClose: Function })
defineEmits(['close'])
const panel = ref(null)
const visible = ref(true)
const { viewportStyle } = useRoomViewport()
const remoteTheme = inject(REMOTE_THEME, null)
const localTheme = inject(LOCAL_APPEARANCE, null)
let previousFocus = null, unregisterFeedback = null
function requestClose() { if (!props.busy && props.beforeClose?.() !== false) visible.value = false }
function handleKeydown(event) {
  if (event.key === 'Escape') { event.stopPropagation(); requestClose(); return }
  if (event.key !== 'Tab') return
  const targets = Array.from(panel.value.querySelectorAll('button:not(:disabled), a[href], input:not(:disabled), textarea:not(:disabled), select:not(:disabled), summary, [tabindex="0"]')).filter(el => el.getClientRects().length && !el.closest('[inert]'))
  const first = targets[0]
  const last = targets[targets.length - 1]
  if (!first) { event.preventDefault(); panel.value.focus(); return }
  if (event.shiftKey && (document.activeElement === first || document.activeElement === panel.value)) { event.preventDefault(); last.focus() }
  else if (!event.shiftKey && (document.activeElement === last || document.activeElement === panel.value)) { event.preventDefault(); first.focus() }
}
onMounted(async () => { previousFocus = document.activeElement; await nextTick(); if (panel.value) { unregisterFeedback = registerFeedbackSurface(panel.value); panel.value.focus({ preventScroll: true }) } })
onBeforeUnmount(() => { unregisterFeedback?.(); if (previousFocus?.isConnected) previousFocus.focus({ preventScroll: true }) })
</script>

<style scoped>
.room-drawer-overlay { position: fixed; inset: var(--room-top, 0px) 0 auto; height: var(--room-height, 100dvh); z-index: 1500; background: #202c424f; display: flex; align-items: flex-end; justify-content: center; padding-top: max(24px, env(safe-area-inset-top)); }
.room-drawer-panel { width: min(100%, 520px); max-height: calc(var(--room-height, 100dvh) - max(24px, env(safe-area-inset-top))); display: flex; flex-direction: column; background: var(--room-surface); border: 1px solid var(--room-line); border-bottom: 0; border-radius: 5px 18px 0 0; box-shadow: 0 18px 60px #16243b1f; padding-bottom: env(safe-area-inset-bottom); outline: 0; }
.room-drawer-header { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 20px 18px 18px; flex: none; }
.drawer-emblem { width: 26px; height: 24px; color: var(--room-accent); flex: none; }
.room-drawer-header > div { flex: 1; min-width: 0; }
.room-drawer-eyebrow { font-size: 10px; color: var(--room-muted); font-weight: 400; }
.room-drawer-header h2 { margin: 5px 0 0; font-weight: 650; font-size: 16px; }
.room-drawer-header > .room-icon-button { width: 34px; height: 34px; color: var(--room-muted); background: var(--room-subtle); border-radius: 5px; }
.room-drawer-content { padding: 0 18px 20px; overflow-y: auto; overscroll-behavior: contain; min-height: 0; }
.room-drawer-footer { padding: 12px 18px; border-top: 1px solid var(--room-line); }
.room-drawer-overlay--wardrobe { backdrop-filter: none; background: linear-gradient(180deg, transparent 35%, #202d4320); }
.room-drawer-overlay--wardrobe .room-drawer-panel { max-height: min(48%, 430px); }
.room-drawer-overlay--wardrobe .room-drawer-header { padding-top: 18px; padding-bottom: 14px; }
.room-drawer-overlay--editor .room-drawer-panel { width: min(100%, 760px); height: calc(var(--room-height, 100dvh) - max(24px, env(safe-area-inset-top))); padding-bottom: 0; }
.room-drawer-overlay--editor .room-drawer-header { border-bottom: 1px solid var(--room-line); padding-top: 20px; padding-bottom: 18px; }
.room-drawer-overlay--editor .room-drawer-content { display: flex; flex: 1; padding: 0; overflow: hidden; }
.room-drawer-panel { transform-origin: bottom center; }
.room-drawer-enter-active { transition: opacity .28s; }
.room-drawer-leave-active { transition: opacity .18s; }
.room-drawer-enter-active .room-drawer-panel { transition: transform .28s var(--room-spring); }
.room-drawer-leave-active .room-drawer-panel { transition: transform .18s var(--room-ease); pointer-events: none; }
.room-drawer-enter-from, .room-drawer-leave-to { opacity: 0; }
.room-drawer-enter-from .room-drawer-panel, .room-drawer-leave-to .room-drawer-panel { transform: translateY(24px) scale(.985); }
.room-drawer-enter-active .drawer-emblem { animation: drawer-bow .36s var(--room-ease); }
.room-drawer-header > button svg { transition: transform .2s var(--room-spring); }
@keyframes drawer-bow { 0% { transform: rotate(-10deg) scale(.92); } 65% { transform: rotate(4deg); } 100% { transform: rotate(0) scale(1); } }
@media (hover: hover) { .room-drawer-header > button:hover:not(:disabled) svg { transform: rotate(90deg); } }
@media (min-width: 760px) { .room-drawer-overlay { align-items: center; padding: 32px; } .room-drawer-panel { border-radius: 5px 18px 5px 5px; border-bottom: 1px solid var(--room-line); max-height: min(850px, calc(var(--room-height, 100dvh) - 64px)); } }
@media (min-width: 960px) { .room-drawer-overlay--wardrobe { justify-content: flex-end; align-items: flex-end; padding: 24px 28px; } .room-drawer-overlay--wardrobe .room-drawer-panel { width: 420px; max-height: min(65%, 540px); } }
@media (min-width: 760px) and (max-width: 959px) { .room-drawer-overlay--wardrobe { align-items: flex-end; padding: 0; } .room-drawer-overlay--wardrobe .room-drawer-panel { border-radius: 5px 18px 0 0; } }
</style>
