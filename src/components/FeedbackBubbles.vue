<template>
  <Teleport :to="feedbackSurface || 'body'">
    <div id="app-feedback-layer" class="feedback-layer companion-surface" data-ui="soft" :class="{ 'remote-themed': current?.remote, 'standalone-design': !current?.remote }" :data-remote-theme="current?.remote ? theme?.mode : undefined" :data-reduce-motion="theme?.reduceMotion" :style="[viewportStyle, theme?.style]" aria-live="polite" aria-atomic="true">
      <Transition name="little-feedback" mode="out-in">
        <div v-if="current" :key="current.id" class="little-feedback" :data-tone="current.tone" @keydown.esc.stop.prevent="closeFeedback(current?.id, true)" @pointerenter="holdFeedback('pointer')" @pointerleave="resumeFeedback('pointer')" @focusin="holdFeedback('focus')" @focusout="resumeFeedback('focus')">
          <span class="feedback-seal" aria-hidden="true"><RemoteIcon :name="icon" /><i /></span>
          <span class="feedback-copy">{{ current.message }}</span>
          <button v-if="current.actionLabel" type="button" class="feedback-action" @click="act">{{ current.actionLabel }}</button>
          <button type="button" class="feedback-dismiss" aria-label="收起提示" @click="closeFeedback(current?.id, true)"><RemoteIcon name="close" /></button>
          <span class="feedback-orbit" aria-hidden="true"><i /><i /></span>
        </div>
      </Transition>
    </div>
  </Teleport>
</template>
<script setup>
import { computed, onMounted, onBeforeUnmount } from 'vue'
import RemoteIcon from './remote/RemoteIcon.vue'
import { useRoomViewport } from '../composables/useRoomViewport'
import { activeFeedback as current, feedbackSurface, holdFeedback, resumeFeedback, closeFeedback, clearFeedback } from '../services/appFeedback'
const props = defineProps({ fallbackTheme: Object })
const { viewportStyle } = useRoomViewport()
const theme = computed(() => current.value?.theme?.() || props.fallbackTheme)
const icon = computed(() => ({ success: 'check', warning: 'alert', error: 'alert', info: 'bell' })[current.value?.tone] || 'bell')
function act() { const item = current.value; if (!item) return; closeFeedback(item.id, true, true); item.onAction?.() }
function visibility() { if (document.hidden) holdFeedback('hidden'); else resumeFeedback('hidden') }
onMounted(() => { document.addEventListener('visibilitychange', visibility); visibility() })
onBeforeUnmount(() => { document.removeEventListener('visibilitychange', visibility); clearFeedback(); resumeFeedback('hidden') })
</script>
<style scoped>
.feedback-layer{position:fixed;inset:var(--room-top,0px) 0 auto;height:var(--room-height,100dvh);z-index:4000;display:flex;align-items:flex-end;justify-content:center;padding:16px 22px calc(env(safe-area-inset-bottom) + clamp(72px,calc(var(--room-height,100dvh) * .2),172px));pointer-events:none;background:transparent;font-family:"Segoe UI","PingFang SC","Microsoft YaHei UI",system-ui,sans-serif}
.little-feedback{--bubble-accent:var(--accent,var(--room-accent,#6a9cbd));--bubble-paper:var(--surface,var(--room-surface,#fff));--bubble-ink:var(--ink,var(--room-ink,#34465e));position:relative;display:flex;align-items:center;gap:9px;max-width:min(100%,350px);width:fit-content;min-height:48px;padding:6px 5px 6px 10px;border:1px solid color-mix(in srgb,var(--bubble-accent) 30%,var(--bubble-paper));border-radius:17px 17px 8px 17px;background:color-mix(in srgb,var(--bubble-paper) 95%,var(--bubble-accent));box-shadow:0 8px 28px -12px color-mix(in srgb,var(--bubble-ink) 28%,transparent),inset 0 1px 0 color-mix(in srgb,var(--bubble-paper) 85%,transparent);color:var(--bubble-ink);pointer-events:auto;transform-origin:70% 100%}
.little-feedback::after{content:"";position:absolute;right:28px;bottom:-4px;width:7px;height:7px;border-right:1px solid color-mix(in srgb,var(--bubble-accent) 30%,var(--bubble-paper));border-bottom:1px solid color-mix(in srgb,var(--bubble-accent) 30%,var(--bubble-paper));border-radius:0 0 2px;transform:rotate(45deg);background:color-mix(in srgb,var(--bubble-paper) 95%,var(--bubble-accent))}
.little-feedback[data-tone=error]{--bubble-accent:var(--danger,#ad617c)}.little-feedback[data-tone=warning]{--bubble-accent:#aa813f}.little-feedback[data-tone=success]{--bubble-accent:var(--success,#438d80)}
.feedback-seal{position:relative;display:grid;place-items:center;flex:none;width:27px;height:29px;border-radius:9px 9px 9px 4px;background:color-mix(in srgb,var(--bubble-accent) 12%,var(--bubble-paper));color:var(--bubble-accent)}.feedback-seal>svg{width:17px;height:17px}.feedback-seal>i{position:absolute;right:-2px;top:-1px;width:4px;height:4px;transform:rotate(45deg);border:1px solid var(--bubble-accent);background:var(--bubble-paper);border-radius:1px}
.feedback-copy{flex:1;min-width:0;max-height:min(30dvh,160px);overflow-y:auto;scrollbar-width:thin;overflow-wrap:anywhere;font-size:12px;line-height:1.7;letter-spacing:.01em;font-weight:500}
.little-feedback button{display:grid;place-items:center;flex:none;min-width:36px;min-height:36px;border:0;border-radius:11px;background:transparent;color:inherit;font:inherit;cursor:pointer;touch-action:manipulation;-webkit-tap-highlight-color:transparent;transition:background-color 120ms,transform 120ms}.little-feedback button:active{transform:scale(.94);background:color-mix(in srgb,var(--bubble-accent) 12%,transparent)}.little-feedback button:focus-visible{outline:2px solid var(--bubble-accent);outline-offset:1px}.feedback-dismiss{padding:9px;opacity:.65}.feedback-dismiss>svg{width:13px;height:13px}.little-feedback .feedback-action{padding-inline:7px;color:var(--bubble-accent);font-size:11px;font-weight:600}
.feedback-orbit{position:absolute;right:11px;top:-5px;pointer-events:none}.feedback-orbit>i{position:absolute;width:5px;height:5px;border-radius:2px;border:1px solid var(--bubble-accent);background:var(--bubble-paper);transform:rotate(45deg)}.feedback-orbit>i+ i{right:9px;top:3px;width:3px;height:3px;opacity:.6}
.little-feedback-enter-active{animation:little-feedback-pop 300ms cubic-bezier(.2,.8,.2,1) both}.little-feedback-leave-active{pointer-events:none;transition:opacity 150ms,transform 150ms}.little-feedback-leave-to{opacity:0;transform:translateY(6px) scale(.97)}
@keyframes little-feedback-pop{0%{opacity:0;transform:translateY(10px) scale(.93)}65%{opacity:1;transform:translateY(-2px) scale(1.015)}100%{opacity:1;transform:none}}
@media(prefers-reduced-motion:reduce){.little-feedback-enter-active{animation:none}.little-feedback-leave-active,.little-feedback button{transition:none}}
[data-reduce-motion=true] .little-feedback-enter-active{animation:none}[data-reduce-motion=true] .little-feedback-leave-active,[data-reduce-motion=true] .little-feedback button{transition:none}
</style>
