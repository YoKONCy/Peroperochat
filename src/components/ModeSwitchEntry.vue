<template>
  <button type="button" class="mode-switch-entry" :class="{ 'mode-switch-entry--remote': remote }" :aria-label="tr(remote ? '当前使用 infOS 客户端，管理连接或返回独立空间' : '当前独立运行，了解如何连接电脑上的 infOS')" @click="$emit('open')">
    <svg class="mode-switch-art" viewBox="0 0 56 56" fill="none" aria-hidden="true">
      <path class="mode-door-back" d="M9 15 30 8v34l-21 6Z" />
      <path class="mode-door-front" d="m23 12 22 6v29l-22-6Z" />
      <path class="mode-door-line" d="m9 15 21-7v34l-21 6Zm14-3 22 6v29l-22-6Z" />
      <path class="mode-door-travel" d="M16 28h23m-5-5 5 5-5 5" />
      <path class="mode-door-star" d="m43 5 1.5 4.5L49 11l-4.5 1.5L43 17l-1.5-4.5L37 11l4.5-1.5Z" />
    </svg>
    <span class="mode-switch-copy"><small><i />{{ tr(remote ? '当前：infOS 客户端' : '当前：本机独立运行') }}</small><strong>{{ tr(remote ? '管理连接与切换模式' : '连接电脑上的 infOS') }}</strong><span>{{ tr(remote ? '切换电脑，或回到本机独立空间' : '使用电脑上的角色、对话和文件') }}</span></span>
    <span class="mode-switch-destination" aria-hidden="true"><svg viewBox="0 0 24 24" fill="none"><path d="M4 8h15m-4-4 4 4-4 4M20 16H5m4-4-4 4 4 4" /></svg></span>
  </button>
</template>

<script setup>
import { t } from '../i18n/remote'
defineProps({ remote: Boolean })
defineEmits(['open'])
const tr = key => t(key, [], 'connections')
</script>

<style scoped>
.mode-switch-entry { --switch-accent: var(--room-accent, #467fc5); position: relative; display: flex; align-items: center; gap: 10px; width: 100%; min-height: 67px; padding: 8px 14px 8px 9px; color: var(--room-ink, #2b3c56); background: linear-gradient(115deg, #fffffff0, color-mix(in srgb, var(--room-accent-soft, #eaf4ff) 87%, white)); border: 1px solid #ffffffdb; border-radius: 16px 22px 16px 8px; box-shadow: inset 0 1px 0 #ffffffd9, 0 4px 16px #355a8e0c; text-align: left; cursor: pointer; }
.mode-switch-entry--remote { --switch-accent: var(--accent); color: var(--ink); border-color: var(--bevel); background: linear-gradient(115deg, var(--surface), var(--wash)); box-shadow: inset 0 1px 0 var(--edge-light); }
.mode-switch-art { flex: none; width: 46px; height: 46px; overflow: visible; }
.mode-door-back { fill: var(--switch-accent); opacity: .1; }.mode-door-front { fill: var(--switch-accent); opacity: .06; }.mode-door-line { stroke: var(--switch-accent); stroke-width: 1.25; stroke-linejoin: round; opacity: .65; }
.mode-door-travel { stroke: var(--switch-accent); stroke-width: 1.6; stroke-linecap: round; stroke-linejoin: round; transition: transform .18s ease; }.mode-door-star { fill: var(--switch-accent); opacity: .65; }
.mode-switch-copy { display: grid; gap: 6px; min-width: 0; flex: 1; }.mode-switch-copy small { display: flex; align-items: center; gap: 5px; font-size: 9px; opacity: .65; }.mode-switch-copy i { width: 4px; height: 4px; border-radius: 50%; background: var(--switch-accent); }.mode-switch-copy strong { font-size: 14px; font-weight: 600; }
.mode-switch-copy > span { font-size: 11px; color: var(--ui-muted, var(--room-muted)); line-height: 1.6; overflow-wrap: anywhere; }
.mode-switch-entry { min-height: 88px; padding-block: 12px; }
.mode-switch-copy { gap: 4px; }
.mode-switch-copy small { font-size: 10px; line-height: 1.4; opacity: 1; color: var(--ui-muted, var(--room-muted)); }
.mode-switch-copy strong { line-height: 1.45; }
.mode-switch-destination { display: grid; place-items: center; flex: none; width: 28px; height: 28px; color: var(--switch-accent); }.mode-switch-destination svg { width: 22px; height: 22px; stroke: currentColor; stroke-width: 1.5; stroke-linecap: round; stroke-linejoin: round; }
.mode-switch-entry:focus-visible { outline: 2px solid var(--switch-accent); outline-offset: 3px; }.mode-switch-entry:active .mode-door-travel { transform: translateX(3px); }
@media (max-width: 380px) { .mode-switch-entry { gap: 8px; } .mode-switch-art { width: 36px; height: 36px; } .mode-switch-destination { width: 22px; } }
@media (prefers-reduced-motion: reduce) { .mode-door-travel { transition: none; } }
</style>
