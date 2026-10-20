<template>
  <div class="floating-dialogue" role="status" aria-live="polite" aria-atomic="true" :aria-label="`${speaker}：${text}`">
    <span class="floating-dialogue-copy" aria-hidden="true" :style="{ '--letter-step': `${letterStep}ms` }"><span v-for="(letter, index) in letters" :key="index" class="floating-dialogue-letter" :style="letter.style">{{ letter.text }}</span></span>
  </div>
</template>

<script setup>
import { computed } from 'vue'
const props = defineProps({ text: { type: String, required: true }, speaker: { type: String, default: '伙伴' } })
const segmenter = typeof Intl.Segmenter === 'function' ? new Intl.Segmenter(undefined, { granularity: 'grapheme' }) : null
const letters = computed(() => {
  const result = [], tilts = [-3, 1, 3, -1, 2, -2], lifts = [1, -1, 0, 2, -1, 1]
  // 限制展示节点数量；组合字符一起出现，较长台词也不创建整篇文本的动画节点。
  for (const unit of segmenter ? segmenter.segment(props.text) : props.text) {
    if (result.length === 120) { result.push({ text: '…', style: { '--letter-index': 120, '--letter-tilt': '0deg', '--letter-lift': '0px' } }); break }
    const index = result.length
    result.push({ text: typeof unit === 'string' ? unit : unit.segment, style: { '--letter-index': index, '--letter-tilt': `${tilts[index % tilts.length]}deg`, '--letter-lift': `${lifts[index % lifts.length]}px` } })
  }
  return result
})
const letterStep = computed(() => Math.min(38, 1200 / Math.max(1, letters.value.length - 1)))
</script>

<style scoped>
.floating-dialogue { position: absolute; z-index: 4; left: 50%; bottom: 7%; width: min(80vw, 320px); transform: translateX(-50%); pointer-events: none; text-align: center; color: var(--room-accent); }
.floating-dialogue-copy { display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden; padding: 6px 9px; font-family: 'Yuanti SC', 'Hiragino Maru Gothic ProN', 'Microsoft YaHei', sans-serif; font-size: clamp(16px, 4.5vw, 19px); font-weight: 600; line-height: 1.85; letter-spacing: .06em; white-space: pre-wrap; overflow-wrap: anywhere; -webkit-text-stroke: 2px #fffffff5; paint-order: stroke fill; text-shadow: 0 0 7px white, 0 2px 3px #ffffffb8; }
.floating-dialogue-letter { display: inline-block; transform: translateY(var(--letter-lift)) rotate(var(--letter-tilt)); animation: floating-letter-pop .52s cubic-bezier(.22,.85,.35,1.2) both; animation-delay: calc(var(--letter-index) * var(--letter-step)); }
@keyframes floating-letter-pop { 0% { opacity: 0; transform: translateY(12px) scale(.75) rotate(calc(var(--letter-tilt) - 5deg)); } 65% { opacity: 1; transform: translateY(-4px) scale(1.07) rotate(var(--letter-tilt)); } 100% { opacity: 1; transform: translateY(var(--letter-lift)) rotate(var(--letter-tilt)); } }
.room-floating-leave-active { transition: opacity .25s, translate .25s; }
.room-floating-leave-to { opacity: 0; translate: 0 5px; }
@media (prefers-reduced-motion: reduce) { .floating-dialogue-letter { animation: none; } .room-floating-leave-active { transition: none; } }
</style>
