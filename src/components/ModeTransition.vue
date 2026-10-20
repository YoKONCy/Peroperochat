<template>
  <section ref="panel" class="mode-transition" :class="{ 'is-remote': remote, 'is-paused': !busy }" :aria-busy="busy" role="dialog" aria-modal="true" aria-label="切换空间" tabindex="-1" @keydown="trapFocus">
    <header class="mode-brand"><svg viewBox="0 0 40 24" fill="none" aria-hidden="true"><path d="m18 11-12-7v16l12-7m4-2 12-7v16l-12-7M12 7l-3 5 3 5m16-10 3 5-3 5" /><path d="m20 8 4 4-4 4-4-4Z" /></svg><span>PeroperoChat</span></header>
    <div class="mode-transition-content">
      <div class="mode-art" aria-hidden="true">
        <div class="mode-art-halo" />
        <svg class="mode-letter-scene" viewBox="0 0 320 260" fill="none" stroke-linecap="round" stroke-linejoin="round" focusable="false">
          <ellipse class="mode-ground" cx="160" cy="229" rx="69" ry="7" />
          <ellipse class="mode-orbit-line" cx="160" cy="133" rx="132" ry="83" transform="rotate(-18 160 133)" />
          <path class="mode-orbit-arc" d="M36 148c5 54 109 90 207 40" />
          <g class="mode-satellite"><path class="mode-jewel" d="m279 117 6 9-6 9-6-9Z" /><circle class="mode-moon-dot" cx="57" cy="144" r="4" /></g>
          <g class="mode-twinkle mode-twinkle-one"><path class="mode-star" d="m65 60 3 10 10 3-10 3-3 10-3-10-10-3 10-3Z" /><path class="mode-spark" d="M42 99v7m-3.5-3.5h7" /></g>
          <g class="mode-twinkle mode-twinkle-two"><path class="mode-star" d="m253 172 3 8 8 3-8 3-3 8-3-8-8-3 8-3Z" /><circle class="mode-spark" cx="276" cy="157" r="2" /></g>
          <g class="mode-twinkle mode-twinkle-three"><path class="mode-spark" d="M238 45v10m-5-5h10M91 209v7m-3.5-3.5h7" /><path class="mode-jewel" d="m217 216 3 5-3 5-3-5Z" /></g>
          <g class="mode-letter-float">
            <g transform="rotate(-7 160 137)">
              <rect class="mode-glass-back" x="84" y="58" width="152" height="153" rx="23" />
              <path class="mode-glass-edge" d="M97 101V81a10 10 0 0 1 10-10h50m58 94v23a10 10 0 0 1-10 10h-22" />
              <path class="mode-envelope-back" d="m97 139 54-39a15 15 0 0 1 18 0l54 39v56a9 9 0 0 1-9 9H106a9 9 0 0 1-9-9Z" />
              <g class="mode-paper-slip">
                <rect class="mode-paper" x="115" y="81" width="90" height="103" rx="11" />
                <path class="mode-paper-corner" d="M185 81v12a6 6 0 0 0 6 6h14" />
                <path class="mode-letter-star" d="m150 100 3.5 10.5 11 .5-8.5 7 2.5 10.5-8.5-6-9 6 3-10.5-8.5-7 11-.5Z" />
                <path class="mode-handwriting" d="M133 144c6-5 8 5 13 0s7 3 11 0 7-2 11 0 7 1 13-1m-45 12c8-3 11 3 18 0s10 2 16 0" />
                <circle class="mode-paper-dot" cx="181" cy="114" r="2.5" />
              </g>
              <path class="mode-envelope-front" d="m97 140 57 36a11 11 0 0 0 12 0l57-36v55a9 9 0 0 1-9 9H106a9 9 0 0 1-9-9Z" />
              <path class="mode-envelope-seam" d="m99 198 43-32m79 32-43-32" />
              <path class="mode-ribbon-tail mode-tail-left" d="m154 177-14 30 11-3 4 9 8-32" />
              <path class="mode-ribbon-tail mode-tail-right" d="m166 177 13 29-10-4-5 9-6-30" />
              <g class="mode-letter-bow">
                <path class="mode-bow-left" d="M155 174c-13-16-29-15-29-7 0 11 17 17 29 12Z" />
                <path class="mode-bow-right" d="M165 174c13-16 29-15 29-7 0 11-17 17-29 12Z" />
                <path class="mode-bow-fold" d="m133 166 15 9-14 1m53-10-15 9 14 1" />
                <rect class="mode-bow-knot" x="154" y="169" width="12" height="14" rx="5" />
                <path class="mode-knot-shine" d="m160 172 2 4-2 4-2-4Z" />
              </g>
            </g>
          </g>
          <g class="mode-little-moon"><path class="mode-moon" d="M260 73a15 15 0 0 1-15-23 16 16 0 1 0 15 23Z" /><path class="mode-spark" d="m262 52 1.5 4.5L268 58l-4.5 1.5L262 64l-1.5-4.5L256 58l4.5-1.5Z" /></g>
        </svg>
      </div>
      <p class="mode-destination">{{ remote ? 'infOS' : '独立空间' }}</p>
      <h1 role="status">{{ title }}</h1>
      <div v-if="busy" class="mode-loading-dots" aria-hidden="true"><i /><i /><i /></div>
      <p v-else class="mode-transition-error" role="alert">{{ error || '暂时没能到达，再试一次吧。' }}</p>
      <div v-if="!busy" class="mode-transition-actions"><button class="room-primary-button" @click="retry">{{ modeState.restoreError ? '再读一次' : '再试一次' }}</button><button v-if="modeState.restoreError && isStandalone" class="room-secondary-button" @click="router.push({ path: '/settings', query: { section: 'data' } })">数据备份与恢复</button><button v-if="remote" class="room-secondary-button" @click="returnStandalone">先回独立空间</button></div>
    </div>
    <footer class="mode-tip" aria-live="off"><span class="mode-tip-label"><svg viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="m8 1 2 5 5 2-5 2-2 5-2-5-5-2 5-2Z" /></svg>TIPS</span><Transition name="mode-tip" mode="out-in"><p :key="tip">{{ tip }}</p></Transition></footer>
  </section>
</template>
<script setup>
import { computed, ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { modeState, modeTransition, isStandalone, initializeMode, refreshMode, switchMode, retryModeData } from '../stores/mode'
const props = defineProps({ waiting: Boolean })
const router = useRouter(), panel = ref(null)
const failure = ref(''), acting = ref(false), tip = ref('')
let tipTimer = null, previousFocus = null
const remote = computed(() => (modeTransition.target || modeState.requestedTarget || modeState.snapshot?.selectedTarget)?.mode === 'remote')
const busy = computed(() => props.waiting || acting.value || modeState.loading || modeState.restoring || !!modeState.snapshot && ['activating', 'quiescing'].includes(modeState.snapshot.phase))
const error = computed(() => failure.value || modeState.restoreError || modeState.error)
const title = computed(() => busy.value ? remote.value ? '去另一边，见你' : '回到熟悉的小天地' : '这里稍微卡住了')
const commonTips = ['有话慢慢说，今天还很长。', '累了的话，就休息一小会儿。', '聊天里的小事，也可以很重要。', '换个配色，也换一点心情。']
const remoteTips = ['上次的话题，会在另一边等你。', '手机收起来，远程任务也能继续。', '想换个心情，可以试试显示偏好。']
const localTips = ['房间里的日记，只属于这个空间。', '想动一动？可以切换到 Live2D。', '不动的时候，静态立绘也很好看。']
function pickTip() { const tips = [...commonTips, ...(remote.value ? remoteTips : localTips)].filter(value => value !== tip.value); tip.value = tips[Math.floor(Math.random() * tips.length)] }
watch(remote, pickTip, { immediate: true })
function trapFocus(event) {
  if (event.key !== 'Tab') return
  const targets = [...panel.value.querySelectorAll('button:not(:disabled)')].filter(element => element.getClientRects().length)
  const first = targets[0], last = targets.at(-1)
  if (!first) { event.preventDefault(); panel.value.focus({ preventScroll: true }) }
  else if (event.shiftKey && (document.activeElement === first || document.activeElement === panel.value)) { event.preventDefault(); last.focus() }
  else if (!event.shiftKey && (document.activeElement === last || document.activeElement === panel.value)) { event.preventDefault(); first.focus() }
}
function keepFocus(event) { if (panel.value && !panel.value.contains(event.target)) panel.value.focus({ preventScroll: true }) }
async function run(action) { if (busy.value) return; acting.value = true; failure.value = ''; try { await action() } catch (error) { failure.value = error.message } finally { acting.value = false } }
function retry() { return run(async () => { if (modeState.restoreError) { retryModeData(); return }; await initializeMode(); const state = await refreshMode(); if (state.phase !== 'stable') await switchMode(state.selectedTarget) }) }
function returnStandalone() { return run(() => switchMode({ mode: 'standalone' })) }
onMounted(() => { previousFocus = document.activeElement; panel.value?.focus({ preventScroll: true }); document.addEventListener('focusin', keepFocus, true); tipTimer = setInterval(() => { if (!document.hidden) pickTip() }, 6000) })
onBeforeUnmount(() => { clearInterval(tipTimer); document.removeEventListener('focusin', keepFocus, true); if (previousFocus?.isConnected && !previousFocus.closest('[inert]')) previousFocus.focus({ preventScroll: true }) })
</script>
<style scoped>
.mode-transition { --switch-accent: #7d9ccc; --switch-secondary: #bba5d9; --switch-ink: #455574; --switch-muted: #8a98b1; --switch-paper: #fcfdff; --switch-soft: #e9f0fc; --switch-ribbon: #d5c8ee; position: fixed; inset: 0; z-index: 9000; display: flex; flex-direction: column; align-items: center; overflow-y: auto; padding: calc(28px + env(safe-area-inset-top)) 24px calc(24px + env(safe-area-inset-bottom)); color: var(--switch-ink); background: radial-gradient(ellipse at 12% 20%, #e5efff 0, transparent 52%), radial-gradient(ellipse at 94% 80%, #eee7fb 0, transparent 52%), #f8faff; text-align: center; outline: 0; isolation: isolate; -webkit-tap-highlight-color: transparent; }
.mode-transition.is-remote { --switch-accent: #9b8ac5; --switch-secondary: #93b8dd; --switch-ribbon: #c6d8ef; }
.mode-brand { display: flex; align-items: center; gap: 9px; color: var(--switch-muted); font-size: 10px; letter-spacing: 1.4px; flex: none; opacity: .8; }.mode-brand svg { width: 24px; height: 16px; stroke: var(--switch-accent); stroke-width: 1.2; }
.mode-transition-content { width: min(100%, 380px); flex: 1; display: flex; flex-direction: column; justify-content: center; align-items: center; padding: 12px 0 30px; }
.mode-art { position: relative; width: min(100%, 320px); flex: none; }.mode-art-halo { position: absolute; inset: 15% 10% 8%; border: 1px solid #ffffffcf; border-radius: 50%; background: #ffffff4d; box-shadow: inset 0 0 35px #ffffff90, 0 0 50px #d4def42b; backdrop-filter: blur(8px); transform: rotate(-18deg); z-index: -1; }
.mode-letter-scene { display: block; width: 100%; height: auto; overflow: visible; stroke-width: 1.45; }
.mode-ground { fill: var(--switch-accent); fill-opacity: .09; animation: mode-shadow 3.8s ease-in-out infinite; transform-origin: 160px 229px; }
.mode-orbit-line { stroke: var(--switch-accent); stroke-opacity: .22; stroke-dasharray: 2 7; stroke-width: 1; }.mode-orbit-arc { stroke: var(--switch-secondary); stroke-opacity: .25; stroke-width: 1; }
.mode-satellite { transform-origin: 160px 133px; animation: mode-orbit 16s linear infinite; }.mode-jewel { fill: var(--switch-secondary); fill-opacity: .6; stroke: white; }.mode-moon-dot { fill: var(--switch-accent); fill-opacity: .25; stroke: white; stroke-width: 2; }.mode-star { fill: var(--switch-paper); stroke: var(--switch-secondary); }.mode-spark { stroke: var(--switch-accent); stroke-opacity: .65; stroke-width: 1.2; }
.mode-letter-float { animation: mode-letter-float 3.8s ease-in-out infinite; }.mode-glass-back { fill: var(--switch-paper); fill-opacity: .48; stroke: var(--switch-accent); stroke-opacity: .2; }.mode-glass-edge { stroke: white; stroke-opacity: .95; stroke-width: 2; }
.mode-envelope-back { fill: var(--switch-soft); stroke: var(--switch-accent); stroke-opacity: .65; }.mode-paper-slip { animation: mode-letter-slip 3.8s ease-in-out infinite; }.mode-paper { fill: var(--switch-paper); stroke: var(--switch-accent); stroke-opacity: .6; }.mode-paper-corner { stroke: var(--switch-accent); stroke-opacity: .25; }.mode-letter-star { fill: var(--switch-secondary); fill-opacity: .25; stroke: var(--switch-secondary); stroke-width: 1.15; }.mode-handwriting { stroke: var(--switch-accent); stroke-opacity: .5; stroke-width: 1.8; }.mode-paper-dot { fill: var(--switch-accent); fill-opacity: .3; }
.mode-envelope-front { fill: var(--switch-soft); stroke: var(--switch-accent); stroke-opacity: .7; }.mode-envelope-seam { stroke: var(--switch-accent); stroke-opacity: .28; }.mode-ribbon-tail { fill: var(--switch-ribbon); stroke: var(--switch-accent); stroke-opacity: .55; }.mode-tail-left { transform-origin: 157px 177px; animation: mode-tail-left 2.5s ease-in-out infinite; }.mode-tail-right { transform-origin: 163px 177px; animation: mode-tail-right 2.5s ease-in-out -.6s infinite; }
.mode-bow-left, .mode-bow-right { fill: var(--switch-ribbon); stroke: var(--switch-accent); stroke-opacity: .75; }.mode-bow-fold { stroke: white; stroke-opacity: .8; stroke-width: 1.1; }.mode-bow-knot { fill: var(--switch-accent); stroke: var(--switch-paper); stroke-width: 1.4; }.mode-knot-shine { fill: var(--switch-paper); fill-opacity: .9; }
.mode-little-moon { animation: mode-moon-drift 5s ease-in-out infinite; }.mode-moon { fill: var(--switch-secondary); fill-opacity: .19; stroke: var(--switch-secondary); stroke-width: 1.1; }.mode-twinkle { animation: mode-twinkle 2.8s ease-in-out infinite; }.mode-twinkle-two { animation-delay: -.9s; }.mode-twinkle-three { animation-delay: -1.8s; }
.mode-destination { margin: 10px 0 8px; padding: 4px 14px; color: var(--switch-accent); font-size: 10px; letter-spacing: 1.3px; border: 1px solid color-mix(in srgb, var(--switch-accent) 20%, transparent); border-radius: 20px; background: color-mix(in srgb, var(--switch-paper) 65%, transparent); }.mode-transition h1 { font-size: 19px; line-height: 1.6; font-weight: 500; letter-spacing: .8px; margin: 2px 0 0; }
.mode-loading-dots { display: flex; align-items: center; gap: 7px; height: 34px; margin-top: 9px; }.mode-loading-dots i { width: 4px; height: 4px; border-radius: 50%; background: var(--switch-accent); opacity: .35; animation: mode-dot 1.4s ease-in-out infinite; }.mode-loading-dots i:nth-child(2) { animation-delay: .16s; }.mode-loading-dots i:nth-child(3) { animation-delay: .32s; }
.mode-tip { width: min(100%, 310px); min-height: 66px; flex: none; color: var(--switch-muted); }.mode-tip-label { display: inline-flex; align-items: center; gap: 6px; font-size: 8px; letter-spacing: 2px; color: var(--switch-accent); }.mode-tip-label svg { width: 11px; height: 11px; stroke: currentColor; stroke-width: 1; }.mode-tip p { margin: 10px 0 0; font-size: 11px; line-height: 1.9; letter-spacing: .25px; text-wrap: balance; }.mode-tip-enter-active, .mode-tip-leave-active { transition: opacity .25s, transform .25s; }.mode-tip-enter-from { opacity: 0; transform: translateY(4px); }.mode-tip-leave-to { opacity: 0; transform: translateY(-4px); }
.mode-transition-error { max-width: 300px; margin: 14px 0 0; font-size: 11px; line-height: 1.9; color: #a76986; overflow-wrap: anywhere; }.mode-transition-actions { display: flex; flex-direction: column; gap: 9px; width: min(100%, 230px); margin-top: 22px; }.mode-transition-actions button { min-height: 40px; padding: 10px 15px; border: 1px solid color-mix(in srgb, var(--switch-accent) 30%, transparent); border-radius: 13px; font: inherit; font-size: 12px; background: var(--switch-paper); color: var(--switch-accent); }.mode-transition-actions .room-primary-button { background: var(--switch-accent); color: var(--switch-paper); }.mode-transition-actions button:focus-visible { outline: 2px solid var(--switch-secondary); outline-offset: 3px; }.is-paused .mode-letter-scene * { animation-play-state: paused; }
@keyframes mode-letter-float { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-7px); } }@keyframes mode-letter-slip { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-3px); } }@keyframes mode-shadow { 50% { transform: scaleX(.87); opacity: .6; } }@keyframes mode-orbit { to { transform: rotate(360deg); } }@keyframes mode-tail-left { 50% { transform: rotate(6deg); } }@keyframes mode-tail-right { 50% { transform: rotate(-7deg); } }@keyframes mode-moon-drift { 50% { transform: translateY(4px); } }@keyframes mode-twinkle { 50% { opacity: .3; } }@keyframes mode-dot { 50% { opacity: .9; transform: translateY(-3px); } }
@media (prefers-color-scheme: dark) { .mode-transition { --switch-accent: #a99bcc; --switch-secondary: #86a9cc; --switch-ink: #e4deee; --switch-muted: #a199b4; --switch-paper: #302c43; --switch-soft: #3c3852; --switch-ribbon: #756a9e; background: radial-gradient(ellipse at 10% 10%, #292e45, transparent 55%), radial-gradient(ellipse at 95% 90%, #32273f, transparent 50%), #211f2d; }.mode-transition.is-remote { --switch-accent: #baa5d8; --switch-ribbon: #6d7b9d; }.mode-art-halo { border-color: #d7cdeb14; background: #e5dbff04; box-shadow: inset 0 0 35px #c4b5e510, 0 0 50px #40345820; }.mode-glass-edge { stroke-opacity: .18; } }
@media (max-height: 620px) { .mode-transition { padding-top: calc(16px + env(safe-area-inset-top)); }.mode-art { width: min(65vw, 240px); }.mode-transition-content { padding-bottom: 12px; }.mode-tip { min-height: 55px; } }
@media (prefers-reduced-motion: reduce) { .mode-letter-scene *, .mode-loading-dots i { animation: none !important; }.mode-tip-enter-active, .mode-tip-leave-active { transition: none; } }
</style>
