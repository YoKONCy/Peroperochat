<template>
  <div class="app companion-surface" :data-tone="tone">
    <router-view v-if="isStandalone" />
    <RemoteWorkspace v-else-if="isRemote" :key="`${modeState.snapshot.installationId}:${modeState.snapshot.activationEpoch}:${modeState.snapshot.activeTarget.profileId}`" />
    <p v-else role="status">{{ modeState.error || '正在恢复运行模式…' }}</p>
  </div>
</template>

<script setup>
import { computed, watch } from 'vue'
import { activeAgentId } from './stores/agents'
import { isStandalone, isRemote, modeState } from './stores/mode'
import RemoteWorkspace from './pages/RemoteWorkspace.vue'
import './styles/companion.css'
import './styles/controls.css'
const tone = computed(() => isStandalone.value && activeAgentId.value === 'nana' ? 'nana' : 'pero')
watch(tone, value => { document.documentElement.dataset.tone = value }, { immediate: true })
</script>

<style>
:root { color-scheme: light }
* { box-sizing: border-box }
body { margin: 0; background-color: var(--room-page); }
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
