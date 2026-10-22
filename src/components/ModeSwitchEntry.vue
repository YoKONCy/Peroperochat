<template>
  <button type="button" class="mode-switch-entry" :class="{ 'mode-switch-entry--remote': remote }" :aria-label="remote ? '当前为 infOS 客户端，打开运行模式切换' : '当前为独立空间，切换到 infOS 客户端'" @click="$emit('open')">
    <svg class="mode-switch-art" viewBox="0 0 56 56" fill="none" aria-hidden="true">
      <path class="mode-door-back" d="M9 15 30 8v34l-21 6Z" />
      <path class="mode-door-front" d="m23 12 22 6v29l-22-6Z" />
      <path class="mode-door-line" d="m9 15 21-7v34l-21 6Zm14-3 22 6v29l-22-6Z" />
      <path class="mode-door-travel" d="M16 28h23m-5-5 5 5-5 5" />
      <path class="mode-door-star" d="m43 5 1.5 4.5L49 11l-4.5 1.5L43 17l-1.5-4.5L37 11l4.5-1.5Z" />
    </svg>
    <span class="mode-switch-copy"><small><i />{{ remote ? 'infOS 客户端' : '独立陪伴空间' }}</small><strong>{{ remote ? '切换回独立空间' : '前往 infOS' }}</strong></span>
    <span class="mode-switch-destination" aria-hidden="true"><svg viewBox="0 0 24 24" fill="none"><path d="M4 8h15m-4-4 4 4-4 4M20 16H5m4-4-4 4 4 4" /></svg></span>
  </button>
</template>

<script setup>
defineProps({ remote: Boolean })
defineEmits(['open'])
</script>

<style scoped>
.mode-switch-entry { --switch-accent: var(--room-accent, #467fc5); position: relative; display: flex; align-items: center; gap: 10px; width: 100%; min-height: 67px; padding: 8px 14px 8px 9px; color: var(--room-ink, #2b3c56); background: linear-gradient(115deg, #fffffff0, color-mix(in srgb, var(--room-accent-soft, #eaf4ff) 87%, white)); border: 1px solid #ffffffdb; border-radius: 16px 22px 16px 8px; box-shadow: inset 0 1px 0 #ffffffd9, 0 4px 16px #355a8e0c; text-align: left; cursor: pointer; }
.mode-switch-entry--remote { --switch-accent: var(--accent); color: var(--ink); border-color: var(--bevel); background: linear-gradient(115deg, var(--surface), var(--wash)); box-shadow: inset 0 1px 0 var(--edge-light); }
.mode-switch-art { flex: none; width: 46px; height: 46px; overflow: visible; }
.mode-door-back { fill: var(--switch-accent); opacity: .1; }.mode-door-front { fill: var(--switch-accent); opacity: .06; }.mode-door-line { stroke: var(--switch-accent); stroke-width: 1.25; stroke-linejoin: round; opacity: .65; }
.mode-door-travel { stroke: var(--switch-accent); stroke-width: 1.6; stroke-linecap: round; stroke-linejoin: round; transition: transform .18s ease; }.mode-door-star { fill: var(--switch-accent); opacity: .65; }
.mode-switch-copy { display: grid; gap: 6px; min-width: 0; flex: 1; }.mode-switch-copy small { display: flex; align-items: center; gap: 5px; font-size: 9px; opacity: .65; }.mode-switch-copy i { width: 4px; height: 4px; border-radius: 50%; background: var(--switch-accent); }.mode-switch-copy strong { font-size: 14px; font-weight: 600; }
.mode-switch-destination { display: grid; place-items: center; flex: none; width: 28px; height: 28px; color: var(--switch-accent); }.mode-switch-destination svg { width: 22px; height: 22px; stroke: currentColor; stroke-width: 1.5; stroke-linecap: round; stroke-linejoin: round; }
.mode-switch-entry:focus-visible { outline: 2px solid var(--switch-accent); outline-offset: 3px; }.mode-switch-entry:active .mode-door-travel { transform: translateX(3px); }
@media (prefers-reduced-motion: reduce) { .mode-door-travel { transition: none; } }
</style>
