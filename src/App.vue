<template>
  <div class="app companion-surface" :class="{ 'standalone-design': isStandalone }" :data-tone="tone" :data-reduce-motion="isStandalone && !displayPreferences.motion" :style="isStandalone ? localStyle : undefined">
    <div v-if="contentReady" class="app-mode-content" :inert="transitionVisible ? true : undefined" :aria-hidden="transitionVisible ? true : undefined">
      <router-view v-if="isStandalone" v-slot="{ Component }"><Transition name="app-route" mode="out-in"><component :is="Component" :key="route.path" /></Transition></router-view>
      <RemoteWorkspace v-else-if="isRemote" :key="`${modeState.snapshot.installationId}:${modeState.snapshot.activationEpoch}:${modeState.snapshot.activeTarget.profileId}`" />
    </div>
    <Transition name="mode-curtain"><ModeTransition v-if="transitionVisible" :waiting="transitionWaiting" /></Transition>
    <FeedbackBubbles :fallback-theme="{ style: localStyle, reduceMotion: !displayPreferences.motion }" />
  </div>
</template>

<script setup>
import { computed, provide, watch } from 'vue'
import { useRoute } from 'vue-router'
import { activeAgentId } from './stores/agents'
import { isStandalone, isRemote, modeState } from './stores/mode'
import { remoteState } from './stores/remote'
import { useModeTransition } from './composables/useModeTransition'
import RemoteWorkspace from './pages/RemoteWorkspace.vue'
import ModeTransition from './components/ModeTransition.vue'
import FeedbackBubbles from './components/FeedbackBubbles.vue'
import { clearFeedback } from './services/appFeedback'
import './styles/companion.css'
import './styles/controls.css'
import './styles/standalone.css'
import { appearanceStyle, displayPreferences, LOCAL_APPEARANCE } from './stores/appearance'
const tone = computed(() => isStandalone.value && activeAgentId.value === 'nana' ? 'nana' : 'pero')
const route = useRoute()
const localDataRecovery = computed(() => isStandalone.value && route.path === '/settings' && route.query.section === 'data')
const contentReady = computed(() => isRemote.value || isStandalone.value && !modeState.restoring && (!modeState.restoreError || localDataRecovery.value))
const transitionLoading = computed(() => modeState.loading || modeState.restoring || !!modeState.snapshot && ['activating', 'quiescing'].includes(modeState.snapshot.phase) || isRemote.value && (remoteState.initializing || remoteState.restoringView))
const transitionNeeded = computed(() => !contentReady.value || modeState.loading || modeState.restoring)
const { visible: transitionVisible, waiting: transitionWaiting } = useModeTransition(transitionNeeded, transitionLoading)
const localStyle = computed(() => appearanceStyle(activeAgentId.value))
provide(LOCAL_APPEARANCE, computed(() => isStandalone.value ? { style: localStyle.value, reduceMotion: !displayPreferences.value.motion } : null))
watch(tone, value => { document.documentElement.dataset.tone = value }, { immediate: true })
watch(() => [route.fullPath, modeState.snapshot?.activationEpoch, modeState.snapshot?.phase], clearFeedback, { flush: 'sync' })
</script>

<style>
:root { color-scheme: light }
* { box-sizing: border-box }
body { margin: 0; background-color: var(--room-page); }
.mode-curtain-leave-active { transition: opacity .22s ease; pointer-events: none; }
.mode-curtain-leave-to { opacity: 0; }
.app-route-enter-active{transition:opacity 170ms ease,transform 210ms cubic-bezier(.2,.75,.25,1)}
.app-route-leave-active{pointer-events:none;transition:opacity 110ms ease,transform 130ms ease}
.app-route-enter-from{opacity:0;transform:translateY(7px)}
.app-route-leave-to{opacity:0;transform:translateY(-3px)}
@media(prefers-reduced-motion:reduce){.app-route-enter-active,.app-route-leave-active{transition:none;transform:none}}
[data-reduce-motion=true] .app-route-enter-active,[data-reduce-motion=true] .app-route-leave-active{transition:none;transform:none}
@media (prefers-reduced-motion: reduce) { .mode-curtain-leave-active { transition: none; } }
.app { 
  min-height: 100vh; 
  min-height: 100dvh; 
  position: relative; 
  overflow-x: clip;
  background: var(--room-page);
}

/* Markdown 基础样式 */
.markdown-body {
  line-height: 1.5;
  word-wrap: break-word;
}
.markdown-body p { margin-top: 0; margin-bottom: 8px; }
.markdown-body p:last-child { margin-bottom: 0; }
.markdown-body strong { font-weight: 600; color: inherit; }
.markdown-body ul, .markdown-body ol { padding-left: 20px; margin-top: 0; margin-bottom: 8px; }
.markdown-body li { margin-bottom: 4px; }
.markdown-body code {
  padding: 0.2em 0.4em;
  margin: 0;
  font-size: 85%;
  background-color: rgba(0, 0, 0, 0.1);
  border-radius: 6px;
  font-family: ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, Liberation Mono, monospace;
}
.markdown-body pre {
  padding: 12px;
  overflow: auto;
  font-size: 85%;
  line-height: 1.45;
  background-color: rgba(0, 0, 0, 0.15);
  border-radius: 8px;
  margin-bottom: 12px;
}
.markdown-body pre code {
  padding: 0;
  background-color: transparent;
  font-size: 100%;
}
.markdown-body h1, .markdown-body h2, .markdown-body h3 { margin-top: 16px; margin-bottom: 8px; font-weight: 600; line-height: 1.25; }
.markdown-body h1 { font-size: 1.5em; }
.markdown-body h2 { font-size: 1.25em; }
.markdown-body h3 { font-size: 1.1em; }

</style>
