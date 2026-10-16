<template>
  <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.65" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path :d="paths[name] || paths.chat" />
  </svg>
</template>

<script setup>
defineProps({ name: { type: String, default: 'chat' } })
const paths = {
  globe: 'M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0ZM2 12h20M12 2a18 18 0 0 1 0 20 18 18 0 0 1 0-20Z',
  model: 'M7 7h10v10H7ZM9 2v5M15 2v5M9 17v5M15 17v5M2 9h5M2 15h5M17 9h5M17 15h5',
  chat: 'M20 11.5a8 8 0 0 1-8 8H5l-3 2v-10a9 9 0 0 1 18 0ZM7 10h10M7 14h6',
  diary: 'M4 3h13a2 2 0 0 1 2 2v16H6a3 3 0 0 1-3-3V5a2 2 0 0 1 1-2ZM3 17h16M8 7h6M8 10h4',
  bell: 'M18 8a6 6 0 0 0-12 0v5l-2 4h16l-2-4V8ZM10 21h4M12 2V1',
  profile: 'M16 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0ZM4 21v-2a8 8 0 0 1 16 0v2',
  settings: 'M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8ZM9 3h6l1 3 3 1 2 5-2 5-3 1-1 3H9l-1-3-3-1-2-5 2-5 3-1 1-3Z',
  menu: 'M5 7h14M5 12h14M5 17h14',
  more: 'M5 12h.01M12 12h.01M19 12h.01',
  close: 'm6 6 12 12M6 18 18 6',
  down: 'm6 9 6 6 6-6',
  up: 'm6 15 6-6 6 6',
  back: 'm14 5-7 7 7 7',
  arrow: 'm9 5 7 7-7 7',
  send: 'm21 3-7 18-4-7-7-4 18-7ZM10 14 21 3',
  stop: 'M7 5h10a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2Z',
  image: 'M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2ZM3 17l6-6 4 4 3-3 5 5M15 7h.01',
  copy: 'M8 8h12v13H8ZM16 8V3H3v13h5',
  refresh: 'M20 8a8 8 0 1 0 0 8M20 3v5h-5',
  trash: 'M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7M14 10v7',
  expand: 'M8 3H3v5M16 3h5v5M21 16v5h-5M8 21H3v-5',
  collapse: 'M3 8h5V3M16 3v5h5M21 16h-5v5M8 21v-5H3',
  check: 'm5 12 4 4L19 6',
  wardrobe: 'M4 3h16v18H4ZM12 3v18M9 10v4M15 10v4',
  hanger: 'M10 6a2 2 0 1 1 3 1.73L12 8.5V10L3 16a1.6 1.6 0 0 0 .9 3h16.2a1.6 1.6 0 0 0 .9-3l-9-6',
  search: 'M17 10a7 7 0 1 1-14 0 7 7 0 0 1 14 0Zm-2 5 6 6',
  plus: 'M12 5v14M5 12h14',
  download: 'M12 3v12m-5-5 5 5 5-5M4 16v5h16v-5',
  upload: 'M12 16V4m-5 5 5-5 5 5M4 16v5h16v-5',
  edit: 'm14 5 5 5M4 20l1-6L16 3a2 2 0 0 1 3 0l2 2a2 2 0 0 1 0 3L10 19l-6 1Z',
  camera: 'M8 5l1-2h6l1 2h4a2 2 0 0 1 2 2v12H2V7a2 2 0 0 1 2-2h4Zm8 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0Z',
  help: 'M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0ZM9 8a3 3 0 0 1 6 0c0 2-3 2-3 5M12 17h.01',
  document: 'M14 2H4v20h16V8l-6-6Zm0 0v6h6M8 12h8M8 16h6',
  code: 'm8 6-6 6 6 6m8-12 6 6-6 6M14 3l-4 18',
  power: 'M12 2v10M6 5a9 9 0 1 0 12 0',
  scan: 'M8 3H3v5m13-5h5v5M3 16v5h5m13-5v5h-5M7 7h3v3H7Zm7 0h3v3h-3ZM7 14h3v3H7Zm7 0h3v3h-3Z',
  eye: 'M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12Zm13 0a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z',
  connection: 'M9 15l6-6M7 13l-2 2a4 4 0 0 0 6 6l3-3M10 6l3-3a4 4 0 0 1 6 6l-2 2',
  sliders: 'M4 6h16M4 12h16M4 18h16M8 3v6M16 9v6M10 15v6',
  group: 'M14 7a3 3 0 1 1-6 0 3 3 0 0 1 6 0ZM3 21v-2a8 8 0 0 1 16 0v2M18 4a3 3 0 0 1 0 6M21 21v-2a8 8 0 0 0-3-6',
  game: 'M7 7h10a4 4 0 0 1 4 4v6a3 3 0 0 1-5 2l-2-2h-4l-2 2a3 3 0 0 1-5-2v-6a4 4 0 0 1 4-4ZM8 10v5M5.5 12.5h5M16 11h.01M18 14h.01',
  remote: 'M5 8a10 10 0 0 1 14 0M8 11a6 6 0 0 1 8 0M11 14a2 2 0 0 1 2 0M12 18h.01',
  sun: 'M16 12a4 4 0 1 1-8 0 4 4 0 0 1 8 0ZM12 2v2M12 20v2M2 12h2M20 12h2M5 5l1 1M18 18l1 1M5 19l1-1M18 6l1-1',
}
</script>
