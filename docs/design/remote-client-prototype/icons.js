(() => {
  // 功能图标共享二十四格画布、双色填充和端点规范；装饰插画使用独立比例。
  const paths = {
    home: '<path class="icon-wash" d="m3.5 10 8.5-7 8.5 7v10.5h-6v-6h-5v6h-6Z"/><path d="m2 10 10-8 10 8M4.5 9v11h5v-6h5v6h5V9"/><path class="icon-detail" d="m9.5 7 2.5-2 2.5 2M17 11v3"/><path class="icon-jewel" d="m12 6 1.3 1.7L12 9.4l-1.3-1.7Z"/>',
    chat: '<path class="icon-wash" d="M4 3.5h13l3.5 4V16L17 19.5H8l-4 3v-6L2.5 15V6Z"/><path d="M6 3.5h11l3.5 4V15L17 18.5H9l-5 3v-6H2.5V7Z"/><path class="icon-detail" d="m17 3.5-.5 4h4M7 9h7m-7 4h5"/><path class="icon-jewel" d="m17 11 1.3 1.7-1.3 1.7-1.3-1.7Z"/>',
    tasks: '<path class="icon-wash" d="M5.5 4.5h13v16h-13Z"/><path d="M8 4H4.5v17h15V4H16M9 2h6v4H9Z"/><path class="icon-detail" d="M8 10h8m-8 4h3m-3 4h3"/><path d="m13 16 1.5 1.5L18 14"/><path class="icon-jewel" d="m7 3 1 1-1 1-1-1Z"/>',
    files: '<path class="icon-wash" d="M6 2.5h9l5 5v14H6Z"/><path d="M6 2.5h9l5 5v14H6ZM3 6v15h13"/><path class="icon-detail" d="M15 2.5v5h5M9.5 11h7m-7 3h7m-7 3h3"/><path class="icon-jewel" d="m16 16 1.4 1.7-1.4 1.7-1.4-1.7Z"/>',
    back: '<path d="m14.5 5-7 7 7 7"/><path class="icon-detail" d="M8 12h12"/>',
    next: '<path d="m9.5 5 7 7-7 7"/><path class="icon-detail" d="M4 12h12"/>',
    down: '<path d="m6 9 6 6 6-6"/>',
    close: '<path d="m6.5 6.5 11 11m0-11-11 11"/>',
    more: '<path class="icon-jewel" d="m4 10 2 2-2 2-2-2Zm8 0 2 2-2 2-2-2Zm8 0 2 2-2 2-2-2Z"/>',
    settings: '<path class="icon-wash" d="m12 2 8.7 5v10L12 22l-8.7-5V7Z"/><path d="m9 3 .5-1h5l.5 2 2 1 2-.5 2.5 4-1.5 1.5v3l1.5 1.5-2.5 4-2-.5-2 1-.5 2h-5L9 20l-2-1-2 .5-2.5-4L4 14v-3L2.5 9.5l2.5-4 2 .5 2-1Z"/><path class="icon-detail" d="m12 8 4 4-4 4-4-4Z"/><circle cx="12" cy="12" r="1.4"/>',
    search: '<circle class="icon-wash" cx="10.5" cy="10.5" r="7"/><circle cx="10.5" cy="10.5" r="7"/><path d="m16 16 5 5"/><path class="icon-detail" d="M6.5 10.5a4 4 0 0 1 4-4m8 11 2 2"/>',
    send: '<path class="icon-wash" d="m3 3 18 7.5-7.5 3L10.5 21Z"/><path d="m3 3 18 7.5-7.5 3L10.5 21Zm0 0 10.5 10.5"/><path class="icon-detail" d="m16 8 2 1m-9 7 1 2"/>',
    attach: '<path d="m8 14 7-7a3 3 0 0 1 4.2 4.2l-9 9a5 5 0 0 1-7-7L13.5 3a3.5 3.5 0 0 1 5 5L8 18.5a1.4 1.4 0 0 1-2-2L15 7.5"/>',
    expand: '<path d="M9 3H3v6m12-6h6v6M3 15v6h6m12-6v6h-6"/><path class="icon-detail" d="m9 15 6-6m-6 0h6v6"/>',
    check: '<path d="m5 12 4 4L19 6"/>',
    sun: '<circle class="icon-wash" cx="12" cy="12" r="5"/><circle cx="12" cy="12" r="4"/><path d="M12 1.5v2m0 17v2M1.5 12h2m17 0h2M4.5 4.5 6 6m12 12 1.5 1.5m0-15L18 6M6 18l-1.5 1.5"/><path class="icon-detail" d="M10 10a3 3 0 0 1 3-1"/>',
    moon: '<path class="icon-wash" d="M20.5 14a8.5 8.5 0 0 1-11-11 9 9 0 1 0 11 11Z"/><path d="M20.5 14a8.5 8.5 0 0 1-11-11 9 9 0 1 0 11 11Z"/><path class="icon-jewel" d="m17 3 1 2.5 2.5 1-2.5 1L17 10l-1-2.5-2.5-1 2.5-1Z"/>',
    route: '<path class="icon-wash" d="m5 2 3 3-3 3-3-3Zm14 14 3 3-3 3-3-3Z"/><path d="m5 2 3 3-3 3-3-3Zm14 14 3 3-3 3-3-3ZM5 8v7a4 4 0 0 0 4 4h7M8 5h7a4 4 0 0 1 0 8h-4"/><path class="icon-detail" d="M11 10v6"/>',
    globe: '<circle class="icon-wash" cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="9"/><ellipse cx="12" cy="12" rx="4" ry="9"/><path class="icon-detail" d="M3 12h18M5 6.5c4 2 10 2 14 0M5 17.5c4-2 10-2 14 0"/>',
    folder: '<path class="icon-wash" d="M3 5h6l3 3h9v12H3Z"/><path d="M3 9V5h6l3 3h9v12H3Z"/><path class="icon-detail" d="M3 11h18m-13 5h6"/><path class="icon-jewel" d="m17 14 1 1.5-1 1.5-1-1.5Z"/>',
    download: '<path class="icon-wash" d="M4 17h16v4H4Z"/><path d="M12 2.5v12m-4-4 4 4 4-4M4 17v4h16v-4"/><path class="icon-detail" d="M8 19h8"/>',
    open: '<path class="icon-wash" d="M4 8h12v12H4Z"/><path d="M14 3h7v7m0-7L10 14M10 4H4v16h16v-6"/>',
    memory: '<path class="icon-wash" d="M6 3h14v18H6a3 3 0 0 1-3-3V6Z"/><path d="M6 3h14v18H6a3 3 0 0 1-3-3V6a3 3 0 0 1 3-3Zm0 0v18"/><path class="icon-detail" d="M10 14h6m-6 3h4"/><path class="icon-jewel" d="m13 6 2.5 3-2.5 3-2.5-3Z"/>',
    shield: '<path class="icon-wash" d="m12 2 8.5 4v6c0 5-8.5 10-8.5 10S3.5 17 3.5 12V6Z"/><path d="m12 2 8.5 4v6c0 5-8.5 10-8.5 10S3.5 17 3.5 12V6Z"/><path class="icon-detail" d="m12 5 5.5 2.5V12"/><path d="m8 12 3 3 5-6"/>',
    wifi: '<path d="M2 8a16 16 0 0 1 20 0M5 12a11 11 0 0 1 14 0m-11 4a6 6 0 0 1 8 0"/><circle cx="12" cy="20" r=".7"/>',
    battery: '<rect x="2" y="6" width="18" height="12" rx="3"/><path d="M23 10v4"/><path d="M5 9h12v6H5Z" fill="currentColor" stroke="none"/>',
    signal: '<path d="M4 18v-3m5 3v-6m5 6V8m5 10V4" stroke-width="2.6"/>',
    refresh: '<path d="M20 7A9 9 0 1 0 21 15M20 2v6h-6"/><path class="icon-detail" d="M6.5 15a5 5 0 0 0 5 3"/>',
    alert: '<path class="icon-wash" d="m12 3 10 18H2Z"/><path d="m12 3 10 18H2ZM12 9v5m0 3v.5"/>',
    image: '<path class="icon-wash" d="M4 3h13l4 4v14H3V4Z"/><path d="M4 3h13l4 4v14H3V4ZM17 3v4h4"/><circle cx="8" cy="8" r="1.5"/><path class="icon-detail" d="m3 17 5-5 4 4 4-6 5 7"/>',
    palette: '<path class="icon-wash" d="M12 3a9 9 0 1 0 0 18h1a2 2 0 0 0 1-4c-2-1-1-3 1-3h3a3 3 0 0 0 3-3 9 9 0 0 0-9-8Z"/><path d="M12 3a9 9 0 1 0 0 18h1a2 2 0 0 0 1-4c-2-1-1-3 1-3h3a3 3 0 0 0 3-3 9 9 0 0 0-9-8Z"/><circle class="icon-jewel" cx="7" cy="11" r="1"/><circle class="icon-jewel" cx="10" cy="7" r="1"/><circle class="icon-jewel" cx="15" cy="7.5" r="1"/>',
    bow: '<path class="icon-wash" d="m11 10-8-6-1 2v12l2 1 7-5m2-4 8-6 1 2v12l-2 1-7-5"/><path d="m11 10-8-6-1 2v12l2 1 7-5m2-4 8-6 1 2v12l-2 1-7-5m-1-6 4 4-4 4-4-4 4-4Z"/><path class="icon-detail" d="m3 7 6 5-6 4m18-9-6 5 6 4m-11 0-3 5 5-2 5 2-3-5"/>',
    orbit: '<path class="icon-wash" d="m12 4 4 8-4 8-4-8Z"/><path d="m12 4 4 8-4 8-4-8Z"/><ellipse class="icon-detail" cx="12" cy="12" rx="11" ry="3" transform="rotate(-25 12 12)"/><path class="icon-jewel" d="m20 3 .8 2.2L23 6l-2.2.8L20 9l-.8-2.2L17 6l2.2-.8Z"/>',
    diamond: '<path class="icon-wash" d="m12 2 8 10-8 10-8-10Z"/><path d="m12 2 8 10-8 10-8-10Z"/><path class="icon-jewel" d="m12 8 3 4-3 4-3-4Z"/>',
    sound: '<path class="icon-wash" d="m4 9 5-5v16l-5-5H1V9Z"/><path d="m4 9 5-5v16l-5-5H1V9Zm10-2a7 7 0 0 1 0 10m4-13a11 11 0 0 1 0 16"/>',
    copy: '<rect class="icon-wash" x="8" y="8" width="13" height="13" rx="2"/><rect x="8" y="8" width="13" height="13" rx="2"/><path d="M16 8V3H3v13h5"/>',
  };
  const art = {
    ribbon: { box: '0 0 100 44', paths: '<path class="art-soft" d="m44 18-28-15-6 4v27l8 5 26-14m12-7L84 3l6 4v27l-8 5-26-14"/><path class="art-fold" d="m11 7 27 16-20 12m71-28L62 23l20 12M41 25 31 42l19-8 19 8-10-17"/><path class="art-line" d="m44 18-28-15-6 4v27l8 5 26-14m12-7L84 3l6 4v27l-8 5-26-14M16 4l24 19-23 10m67-29L60 23l23 10m-42-8L31 42l19-8 19 8-10-17"/><path class="art-jewel" d="m50 9 9 13-9 13-9-13Z"/><path class="art-light" d="m50 12 5 10-5 9-5-9Z"/><path class="art-hairline" d="M2 22h6m84 0h6"/>' },
    constellation: { box: '0 0 100 44', paths: '<ellipse class="art-hairline" cx="50" cy="22" rx="44" ry="10" transform="rotate(-12 50 22)"/><path class="art-line" d="M3 8 32 17 50 9l18 18 29 9M3 36l29-9L50 35l18-18L97 8"/><path class="art-soft" d="m50 8 11 14-11 14-11-14Z"/><path class="art-line" d="m50 8 11 14-11 14-11-14Z"/><path class="art-jewel" d="m50 16 5 6-5 6-5-6Zm34-14 1.5 4.5L90 8l-4.5 1.5L84 14l-1.5-4.5L78 8l4.5-1.5Z"/><circle class="art-light" cx="15" cy="30" r="2"/><circle class="art-jewel" cx="72" cy="16" r="1.5"/>' },
    landscape: { box: '0 0 180 120', paths: '<circle class="art-soft" cx="136" cy="32" r="23"/><circle class="art-line" cx="136" cy="32" r="28"/><path class="art-hairline" d="M55 32h39m57 25h22M90 12h20M3 88h25"/><path class="art-soft" d="m0 112 48-59 24 27 27-45 81 77Z"/><path class="art-fold" d="m30 120 42-40 20 20 27-36 61 56Z"/><path class="art-line" d="m0 112 48-59 24 27 27-45 81 77M48 53l10 32m41-50 7 48M0 112h180"/><path class="art-hairline" d="m66 120 22-12 29-2 13-9m-10 23 19-8 34-1"/><path class="art-jewel" d="m25 34 2 6 6 2-6 2-2 6-2-6-6-2 6-2Z"/>' },
    horizon: { box: '0 0 280 95', paths: '<path class="art-hairline" d="M0 80h280M22 85h66m105 0h57M56 75l38-45 27 35 29-51 55 61 29-29 34 34M94 30l8 28m48-44 10 34m74-2 6 19"/><path class="art-soft" d="m58 79 36-41 27 32 29-46 49 55Z"/><circle class="art-hairline" cx="210" cy="24" r="16"/><path class="art-line" d="M15 79h14m218-69h20m-10-10v20"/><path class="art-jewel" d="m39 43 2 5 5 2-5 2-2 5-2-5-5-2 5-2Z"/>' },
    'cover-blue': { box: '0 0 76 96', paths: '<path class="paper-back" d="M4 5h47l21 21v66H4Z"/><path class="paper-base" d="M1 1h49l21 21v69H1Z"/><path class="paper-fold" d="M50 1v21h21"/><circle class="paper-sun" cx="48" cy="38" r="10"/><path class="paper-mountain" d="m1 71 23-31 16 21 10-11 21 25v16H1Z"/><path class="paper-front" d="m1 83 23-15 14 8 16-12 17 17v10H1Z"/><path class="paper-line" d="m12 66 12-17 11 14M9 30h19M9 34h11M1 91h70M50 1v21h21"/><path class="paper-shine" d="m7 2 63 41v15L1 13V2Z"/>' },
    'cover-green': { box: '0 0 76 96', paths: '<path class="paper-back" d="M4 5h47l21 21v66H4Z"/><path class="paper-base" d="M1 1h49l21 21v69H1Z"/><path class="paper-fold" d="M50 1v21h21"/><path class="paper-front" d="M10 67h10v13H10Zm18-15h10v28H28Zm18-17h10v45H46Z"/><path class="paper-line" d="M9 30h22M9 34h13M9 83h52M50 1v21h21M13 58l18-15 19-12"/><path class="paper-sun" d="m50 27 3 4-3 4-3-4Z"/><path class="paper-shine" d="m7 2 63 41v15L1 13V2Z"/>' },
    'cover-purple': { box: '0 0 76 96', paths: '<path class="paper-back" d="M4 5h47l21 21v66H4Z"/><path class="paper-base" d="M1 1h49l21 21v69H1Z"/><path class="paper-fold" d="M50 1v21h21"/><path class="paper-line" d="M11 31h24M11 35h15m-15 15 3 3 5-7m5 4h32M11 65l3 3 5-7m5 4h23M50 1v21h21"/><path class="paper-front" d="m47 74 4-5 4 5-4 5Z"/><path class="paper-shine" d="m7 2 63 41v15L1 13V2Z"/>' },
  };
  const library = document.getElementById('icon-library');
  library.innerHTML = '<defs>' + Object.entries(paths).map(([name, path]) => `<symbol id="i-${name}" viewBox="0 0 24 24">${path}</symbol>`).join('') + Object.entries(art).map(([name, item]) => `<symbol id="a-${name}" viewBox="${item.box}">${item.paths}</symbol>`).join('') + '</defs>';
  window.ProtoIcons = {
    names: Object.keys(paths),
    render(name, className = '') { return `<svg class="icon ${className}" aria-hidden="true" focusable="false"><use href="#i-${name}"></use></svg>`; },
    art(name, className = '') { return `<svg class="svg-art ${className}" viewBox="${art[name]?.box || '0 0 100 44'}" aria-hidden="true" focusable="false"><use href="#a-${name}"></use></svg>`; },
  };
})();
