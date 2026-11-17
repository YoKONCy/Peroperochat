<template>
  <svg class="journal-object" viewBox="0 0 56 46" fill="none" stroke="var(--draw-ink)" stroke-width="1.35" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
    <g v-if="kind === 'theme'"><g v-for="{ angle, color } in blades" :key="angle" :transform="`rotate(${angle} 24 37)`">
    <path d="M18.4 6.4q.2-1.4 1.8-1.5l8.9.2q1.2.1 1.3 1.6l-.6 30.2q-.1 4.4-5.7 4.4-5.3 0-5.5-4.4Z" fill="var(--draw-paper)"/>
    <path d="m20.3 7.4 8.3.1-.3 18.1-7.8-.1Z" :fill="color" stroke="none"/>
    <path d="m20.8 28.8 6.8.2" stroke-width=".8"/>
  </g><circle cx="24" cy="37" r="1.8" fill="var(--draw-gold)" /></g>
    <g v-else-if="kind === 'profile'" transform="rotate(-6 28 26)">
    <path d="M21 12 19 2.5l4.3-.5 3.8 11.2M27 13.1 31.5 2l4.3 1.7-6.6 10.1" fill="var(--draw-blue)"/>
    <rect x="6" y="12.6" width="44" height="28.5" rx="3" fill="var(--draw-paper)"/>
    <rect x="8.4" y="15.1" width="39.3" height="23.3" rx="1.4" stroke-width=".6"/>
    <image :href="portrait" x="11" y="18.2" width="15.7" height="17.5" preserveAspectRatio="xMidYMid slice"/>
    <path d="M30.4 20.5h11m-11 4.4h8m-8.1 7.8h2.2m1.8 0h1m1.8 0h3.3m1.4 0h1" stroke-width="1.4"/>
    <rect x="22.8" y="11" width="8.6" height="3.8" rx="1.4" fill="var(--draw-sky)"/>
  </g>
    <g v-else transform="rotate(8 29 23)">
    <path d="m12 8 30.8-2.2 1.8 31.9-29.5 2.4q-4 .1-4.1-3.5Z" fill="var(--draw-blue)"/>
    <path d="m14.2 32.9 27.3-2.2.3 6.2-27.2 2.2q-3.3.2-3.5-2.9-.1-2.7 3.1-3.3Z" fill="var(--draw-paper)"/>
    <path d="m34 32.3 5.1-.4.5 10.8-2.8-2.4-2.2 2.8Z" fill="var(--draw-blue)"/>
    <path d="M13.3 6.4 40.4 4l1.7 27.6-28.2 2.3q-1.9.2-2.8 1.7L9.6 10.4q-.1-3.6 3.7-4Z" fill="var(--draw-paper)"/>
    <path d="m16 6.9 1.6 25m-7.2-19.6 5-.4m-4.7 7.4 5-.4m-4.6 7.6 5-.4" stroke-width="1.6"/>
    <path d="m22 14.1 13-1 .3 5.2-13.1 1Z" fill="var(--draw-sky)" stroke-width=".8"/>
    <path d="m23.7 23.7 9.3-.8m-18.4 14 16.1-1.3" stroke-width=".8"/>
  </g>
  </svg>
</template>
<script setup>
import portrait from '../../assets/journal/pero-sketch.webp'
defineProps({ kind: String })
const blades = [
  { angle: 49, color: 'var(--draw-rose)' }, { angle: 23, color: 'var(--draw-gold)' },
  { angle: -4, color: 'var(--draw-blue)' }, { angle: -32, color: 'var(--draw-sky)' },
]
</script>
