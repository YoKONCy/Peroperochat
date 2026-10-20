(() => {
  // 图标只取自本文件内的 SVG 路径，不接收用户输入或外部图标代码。
  const paths = {
    menu: '<path d="M4 6h16M4 12h16M4 18h12"/>',
    back: '<path d="m14 5-7 7 7 7"/>',
    next: '<path d="m9 5 7 7-7 7"/>',
    down: '<path d="m6 9 6 6 6-6"/>',
    close: '<path d="m6 6 12 12M18 6 6 18"/>',
    chat: '<path d="M7 3h10a5 5 0 0 1 5 5v5a5 5 0 0 1-5 5H9l-6 4v-7a5 5 0 0 1-1-3V8a5 5 0 0 1 5-5Z"/><path class="detail" d="M7 9h10M7 13h6"/>',
    hanger: '<path d="M9 5a3 3 0 1 1 4 2.8V10L3 17a2 2 0 0 0 1 3h16a2 2 0 0 0 1-3l-8-7"/>',
    journal: '<path d="M6 3h14v18H6a3 3 0 0 1-3-3V6a3 3 0 0 1 3-3Zm0 0v18"/><path class="detail" d="M10 8h6m-6 4h6m-6 4h4"/>',
    image: '<rect x="3" y="3" width="18" height="18" rx="3"/><circle cx="8" cy="8" r="1.5"/><path d="m3 17 5-5 4 4 4-6 5 7"/>',
    send: '<path d="m3 3 18 8-8 3-3 8Zm0 0 10 11"/>',
    bell: '<path d="M6 10a6 6 0 0 1 12 0v5l2 3H4l2-3Zm4 11h4M12 2v2"/>',
    check: '<path d="m5 12 4 4L19 6"/>',
    model: '<path d="m12 2 9 5v10l-9 5-9-5V7Zm-9 5 9 5 9-5m-9 5v10"/>',
    prompt: '<path d="M5 3h13v14H5Zm3 18h13V7M8 7h7m-7 4h4"/>',
    palette: '<path d="M12 3a9 9 0 1 0 0 18h1a2 2 0 0 0 1-4c-2-1-1-3 1-3h3a3 3 0 0 0 3-3 9 9 0 0 0-9-8Z"/><circle cx="7" cy="11" r=".7"/><circle cx="10" cy="7" r=".7"/><circle cx="15" cy="7.5" r=".7"/>',
    motion: '<circle cx="12" cy="12" r="9"/><path d="m10 8 6 4-6 4Z"/>',
    database: '<ellipse cx="12" cy="5" rx="8" ry="3"/><path d="M4 5v7c0 4 16 4 16 0V5M4 12v7c0 4 16 4 16 0v-7"/>',
    settings: '<path d="m9 3 1-1h4l1 2 2 1 2-.5 2 4L20 10v3l1 2-2 4-2-.5-2 1-1 2h-4l-1-2-2-1-2 .5-2-4L4 13v-3L3 8l2-4 2 .5Z"/><circle cx="12" cy="12" r="3"/>',
    diamond: '<path d="m12 4 6 8-6 8-6-8Z"/>',
    clock: '<circle cx="12" cy="12" r="9"/><path d="M12 6v6l4 2"/>',
    plus: '<path d="M12 5v14M5 12h14"/>',
    home: '<path d="m2 10 10-8 10 8M5 8v13h5v-7h4v7h5V8"/>',
    wifi: '<path d="M2 8a16 16 0 0 1 20 0M5 12a11 11 0 0 1 14 0m-11 4a6 6 0 0 1 8 0"/><circle cx="12" cy="20" r="1"/>',
    battery: '<rect x="2" y="6" width="18" height="12" rx="3"/><path d="M23 10v4"/><path class="fill" d="M5 9h12v6H5Z"/>',
    signal: '<path d="M4 19v-4m5 4v-7m5 7V8m5 11V4"/>',
  }
  window.LocalIcons = { render: name => `<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.65" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">${paths[name] || paths.diamond}</svg>` }
})()
