(() => {
  const D = window.ProtoData;
  const C = window.ProtoComponents;
  const { icon, escape, action } = C;
  const root = document.getElementById('designs');
  const dialog = document.getElementById('preview-dialog');
  const paletteSelect = document.getElementById('palette-select');
  const contexts = new Map();
  const pages = { home: '角色首页', chat: '会话', tasks: '执行详情', files: '资料', appearance: '显示偏好' };
  const storageKey = 'peropero-design-preview-v1';
  let toastTimer;
  let state = { view: 'gallery', palette: 'standard' };
  let preview = { id: 'preview', role: 'pero', mode: 'light', palette: 'sky', page: 'chat', art: true, fileTab: 'files', toolOpen: true, chatMode: 'chat', messages: [] };
  const roleMessages = { pero: [], nana: [] };

  // 演示偏好使用独立命名空间；不读取应用凭据、会话或其他运行数据。
  try {
    const saved = JSON.parse(localStorage.getItem(storageKey) || 'null');
    if (saved && typeof saved === 'object') {
      if (['gallery', 'interactive', 'components'].includes(saved.view)) state.view = saved.view;
      if (saved.palette === 'standard' || D.palettes.some((p) => p.id === saved.palette)) state.palette = saved.palette;
      if (Object.hasOwn(D.roles, saved.role)) preview.role = saved.role;
      if (['light', 'dark'].includes(saved.mode)) preview.mode = saved.mode;
      if (Object.hasOwn(pages, saved.page)) preview.page = saved.page;
      if (typeof saved.art === 'boolean') preview.art = saved.art;
      if (typeof saved.skin === 'boolean') preview.skin = saved.skin;
    }
  } catch {
    // 本地存储不可用或内容损坏时，直接使用完整的默认设计。
  }
  function persist() {
    try { localStorage.setItem(storageKey, JSON.stringify({ ...state, role: preview.role, mode: preview.mode, page: preview.page, art: preview.art, skin: !!preview.skin })); }
    catch { /* 隐私模式下继续保留当前内存状态。 */ }
  }
  function paletteFor(mode) { return state.palette === 'standard' ? (mode === 'light' ? 'sky' : 'night') : state.palette; }
  function toast(message) {
    const element = document.getElementById('studio-toast');
    element.textContent = message;
    element.classList.add('is-visible');
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => element.classList.remove('is-visible'), 3400);
  }
  function register(ctx) { contexts.set(ctx.id, ctx); return C.phone(ctx, pages[ctx.page]); }
  function board(role, mode, index) {
    const isLight = mode === 'light';
    return `<section class="theme-section ${isLight ? 'board-light' : 'board-dark'}"><div class="board-heading"><span class="board-emblem">${C.ornament(mode)}</span><div><small>0${index} / STANDARD ${isLight ? 'LIGHT' : 'DARK'}</small><h2>${isLight ? '晴空伴行' : '月夜星轨'}</h2><p>${isLight ? '清透的蓝白与折纸缎带，保留轻快的陪伴感。' : '柔和的紫与玫瑰色，安静地连接每一个日常。'}</p></div><div class="board-notes"><p>${D.roles[role].name} · 标准${isLight ? '浅色' : '暗色'}主题<br />点击导航、气泡内的文件或角色 CG 体验</p></div></div><div class="phone-grid">${['home', 'chat', 'tasks', 'files'].map((page) => register({ id: `${role}-${page}`, role, mode, page, art: true, palette: paletteFor(mode), fileTab: 'files', toolOpen: true, messages: [], chatMode: 'chat' })).join('')}</div></section>`;
  }
  function roleControls() {
    return `<div class="role-switcher" role="group" aria-label="预览角色">${Object.values(D.roles).map((role) => `<button class="role-option ${preview.role === role.id ? 'is-active' : ''}" ${action('preview-role', role.id)}><img src="${role.avatar}" alt="" />${role.name}</button>`).join('')}</div><div class="desktop-theme-switch" role="group" aria-label="预览主题"><button class="${preview.mode === 'light' ? 'is-active' : ''}" ${action('preview-mode', 'light')}>${icon('sun')}浅色</button><button class="${preview.mode === 'dark' ? 'is-active' : ''}" ${action('preview-mode', 'dark')}>${icon('moon')}暗色</button></div>`;
  }
  function interactive() {
    preview.palette = paletteFor(preview.mode);
    preview.messages = roleMessages[preview.role];
    return `<section class="interactive-layout"><aside class="interactive-notes"><span class="eyebrow">TAKE A LITTLE LOOK AROUND</span><h2>一点熟悉，<br />一点新的可能。</h2><p>角色与明暗主题分别选择。<br />从上次会话继续，也能随时回到角色空间。</p>${roleControls()}<p class="preview-note">当前为本地交互原型。消息、确认与文件均使用示例数据，未连接 infOS 服务端。</p></aside>${register(preview)}<aside class="interactive-notes"><span class="eyebrow">EXPLORE THE SPACE</span><div class="preview-links">${Object.entries(pages).map(([page, label]) => `<button class="${preview.page === page ? 'is-active' : ''}" ${action('preview-page', page)}>${icon(({ home: 'home', chat: 'chat', tasks: 'tasks', files: 'files', appearance: 'palette' })[page])}${label}${icon('next')}</button>`).join('')}<button ${action('art', preview.role)}>${icon('image')}独立角色 CG${icon('next')}</button></div><div class="preview-note">会话中的大图默认收起，角色头像保留。首页可以展示 CG，也可以切换成完整的头像布局。</div></aside></section>`;
  }
  function sample(mode, content) {
    return `<div class="device component-sample" data-mode="${mode}" style="${C.theme({ mode, palette: paletteFor(mode) })}">${content}</div>`;
  }
  function components() {
    const item = (title, description, renderer) => `<article class="component-tile"><h3>${title}</h3>${['light', 'dark'].map((mode) => sample(mode, renderer({ id: `sample-${mode}`, role: mode === 'light' ? 'pero' : 'nana', mode, palette: paletteFor(mode), page: 'chat' }))).join('')}<p>${description}</p></article>`;
    return `<section class="component-section"><span class="eyebrow">ONE SYSTEM. MANY PERSONALITIES.</span><h2>同一套组件，两套标准主题。</h2><p>保留定稿的缎带、星轨与折角语言。角色内容独立配置，配色与样式逐层继承。</p><div class="component-grid">${item('01 / 角色与对话', '头像、名称和实际回复来自角色数据。', (ctx) => C.roleMessage(ctx, D.roles[ctx.role].reply))}${item('02 / 工具与执行', '执行状态与操作统一，主题负责节点、描边与材质。', (ctx) => C.toolSummary(ctx))}${item('03 / 文件卡片', '文件名、类型和操作保持可读；装饰不会占据内容区。', () => C.fileCard(D.files[0]))}${item('04 / 确认与异常', '关键操作始终具备明确文案和独立按钮。', () => `<div class="permission-card"><div class="permission-title">${icon('shield')}<h4>需要你的确认</h4></div><p>是否保存这份行程？</p><div class="permission-actions"><button class="secondary-button" ${action('sample-toast')}>暂不保存</button><button class="primary-button" ${action('sample-toast')}>确认保存</button></div></div>`)}${item('05 / 输入与导航', '相同布局与操作习惯，随主题呈现不同的细节。', (ctx) => `<div class="composer"><div class="composer-input">${icon('attach')}<span style="font-size:10px;color:var(--muted);flex:1">和角色说点什么…</span><button class="send-button" aria-label="发送示例" ${action('sample-toast')}>${icon('send')}</button></div></div>${C.nav(ctx)}`)}${item('06 / 空状态与恢复', '状态提示由应用提供，角色包可补充陪伴文案。', (ctx) => `<div class="empty-state">${icon('folder')}<strong>这里还没有资料</strong><p>${escape(D.roles[ctx.role].empty)}</p></div><div class="permission-title">${icon('alert')}<h4>连接中断</h4><button class="quiet-button" ${action('sample-toast')}>重试 ${icon('refresh')}</button></div>`)}</div></section><section class="component-section"><span class="eyebrow">A COLOR FOR EVERY CHARACTER</span><h2>八组配色，各有自己的心情。</h2><p>每组同时具备浅色和暗色变量。选择后，上方的实际组件与预览同步切换。</p><div class="palette-showcase">${D.palettes.map((p) => `<button ${action('global-palette', p.id)} aria-label="应用${p.name}配色"><span class="palette-duo">${['light', 'dark'].map((mode) => `<span class="palette-mini" style="--mini-bg:${p[mode][1]};--mini-color:${p[mode][0]}"><i></i><i></i><i></i></span>`).join('')}</span><span class="palette-name"><strong>${p.name}</strong><small>${p.mood}</small></span></button>`).join('')}</div><div class="inheritance-strip"><strong>标准浅色 / 标准暗色</strong>${icon('next')}<span>可选角色样式包</span>${icon('next')}<span>用户显示偏好</span></div></section><section class="component-section"><span class="eyebrow">THE CHARACTERS, ON THEIR OWN</span><h2>独立 CG，随时相见。</h2><div class="cg-collection">${Object.values(D.roles).map((role) => `<button ${action('art', role.id)}><img src="${role.cg}" alt="${role.name}的独立CG缩略图" loading="lazy" /><span><strong>${role.name}</strong><small>独立插画素材 · 点击查看原图</small></span>${icon('expand')}</button>`).join('')}</div></section>`;
  }
  function iconCatalog() {
    const entries = [['home', '首页'], ['chat', '会话'], ['tasks', '任务'], ['files', '资料'], ['memory', '记忆'], ['folder', '文件夹'], ['sun', '天气'], ['moon', '夜间'], ['route', '路线'], ['globe', '网络'], ['shield', '确认'], ['palette', '配色'], ['send', '发送'], ['attach', '附件'], ['image', '图片'], ['expand', '展开'], ['search', '搜索'], ['settings', '设置'], ['open', '打开'], ['download', '下载'], ['refresh', '重试'], ['alert', '异常'], ['bow', '缎带'], ['orbit', '星轨']];
    return `<section class="component-section"><span class="eyebrow">DRAWN FOR THIS LITTLE WORLD</span><h2>SVG 图标与装饰细节</h2><p>统一的双色填充、折边与切面。功能图标以小尺寸辨识为先，缎带和星轨使用独立画布。</p><div class="icon-catalog">${entries.map(([name, label]) => `<figure class="icon-study"><div>${icon(name)}${icon(name)}</div><figcaption>${label}</figcaption></figure>`).join('')}</div><div class="motif-study" aria-label="浅色缎带与暗色星轨装饰对照">${C.ornament('light')}${C.ornament('dark')}</div></section>`;
  }
  function render() {
    contexts.clear();
    document.querySelectorAll('.view-tabs button').forEach((element) => {
      const selected = element.dataset.value === state.view;
      element.classList.toggle('is-active', selected);
      element.setAttribute('aria-pressed', String(selected));
    });
    paletteSelect.value = state.palette;
    root.innerHTML = state.view === 'gallery' ? board('pero', 'light', 1) + board('nana', 'dark', 2) : state.view === 'interactive' ? interactive() : components() + iconCatalog();
    persist();
  }
  function refresh(ctx) {
    const figure = root.querySelector(`[data-figure="${ctx.id}"]`);
    if (!figure) return;
    const view = figure.querySelector('.app-view');
    const feed = figure.querySelector('.chat-feed');
    const oldScroll = view?.scrollTop || 0;
    const feedScroll = feed?.scrollTop || 0;
    figure.outerHTML = C.phone(ctx, pages[ctx.page]);
    const next = root.querySelector(`[data-figure="${ctx.id}"]`);
    if (next?.querySelector('.app-view')) next.querySelector('.app-view').scrollTop = oldScroll;
    if (next?.querySelector('.chat-feed')) next.querySelector('.chat-feed').scrollTop = feedScroll;
    if (ctx.id === 'preview') {
      preview = ctx;
      root.querySelectorAll('.preview-links [data-action="preview-page"]').forEach((element) => element.classList.toggle('is-active', element.dataset.value === ctx.page));
      root.querySelectorAll('[data-action="preview-mode"]').forEach((element) => element.classList.toggle('is-active', element.dataset.value === ctx.mode));
    }
    persist();
  }
  function navigate(ctx, page, recordHistory = true) {
    if (!Object.hasOwn(pages, page)) return;
    if (recordHistory && ctx.page !== page) {
      ctx.history ||= [];
      ctx.history.push(ctx.page);
      if (ctx.history.length > 20) ctx.history.shift();
    }
    ctx.page = page;
    refresh(ctx);
    const view = root.querySelector(`[data-phone="${ctx.id}"] .app-view`);
    if (view) view.scrollTop = 0;
  }
  function openDialog(title, subtitle, content, className = '') {
    if (dialog.open) dialog.close();
    dialog.className = `preview-dialog ${className}`;
    dialog.innerHTML = `<header class="dialog-header"><div><h2 id="dialog-title">${escape(title)}</h2><small>${escape(subtitle)}</small></div><button class="dialog-close" aria-label="关闭" ${action('dialog-close')}>${icon('close')}</button></header>${content}`;
    dialog.showModal();
  }
  function showArt(roleId) {
    const role = D.roles[roleId] || D.roles[preview.role];
    if (!role.cg) { toast('这个角色暂未提供 CG，可以继续使用头像布局'); return; }
    openDialog(role.name, '独立角色 CG · 点击关闭返回原页面', `<img src="${role.cg}" alt="${escape(role.name)}的完整独立CG" /><div class="art-caption"><span>CHARACTER CG / ${role.name.toUpperCase()}</span><a href="${role.cg}" download="${role.id}-cg.png">保存原图 ${icon('download')}</a></div>`, 'art-dialog');
  }
  function showFile(id) {
    const file = D.files.find((entry) => entry.id === id) || D.files[0];
    openDialog(file.name, '示例内容预览 · 当前未连接远程文件服务', `<article class="document-preview"><span class="eyebrow">A LITTLE PLAN, A LOVELY DAY</span><h3>${escape(file.title)}</h3><p>${escape(file.subtitle)}</p>${id === 'budget' ? '<div class="document-route"><div><strong>公共交通 · 30 元</strong><small>优先选择地铁与步行</small></div><div><strong>午餐与下午茶 · 120 元</strong><small>给好吃的留一点余地</small></div><div><strong>其他 · 50 元</strong><small>留出灵活预算</small></div></div>' : '<div class="document-route"><div><strong>10:00 · 滨河散步</strong><small>慢慢出发，沿河看看风景。</small></div><div><strong>12:00 · 找一家喜欢的小店</strong><small>午餐不赶时间，下午留给咖啡和闲逛。</small></div><div><strong>16:30 · 带着好心情回家</strong><small>喜欢的照片和发现，之后再一起整理。</small></div></div>'}<p>路线、预算及文案均为设计演示数据。</p></article><div class="dialog-actions"><button ${action('dialog-close')}>返回</button><button class="dialog-primary" ${action('download-sample', file.id)}>${icon('download')} 下载示例文本</button></div>`);
  }
  function downloadSample(id) {
    const file = D.files.find((entry) => entry.id === id) || D.files[0];
    const contents = {
      trip: '10:00 滨河散步\n12:00 午餐与自由活动\n16:30 返程',
      budget: '公共交通：30 元\n午餐与下午茶：120 元\n其他：50 元\n合计：200 元',
      list: '出门前：手机、钥匙、饮水\n确认公共交通路线\n带着好心情出发',
    };
    const text = `${file.title}\n${file.subtitle}\n\n本文件为 PeroperoChat UI 设计原型的示例文本，不是远程服务生成的真实文件。\n\n${contents[file.id]}\n`;
    const url = URL.createObjectURL(new Blob([text], { type: 'text/plain;charset=utf-8' }));
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `${file.name.replace(/\.[^.]+$/, '')}-示例.txt`;
    anchor.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  document.addEventListener('click', (event) => {
    const target = event.target.closest('[data-action]');
    if (!target) return;
    const name = target.dataset.action;
    const value = target.dataset.value || '';
    const ctx = contexts.get(target.closest('[data-phone]')?.dataset.phone);
    if (name === 'view') { state.view = value; render(); return; }
    if (name === 'reset') {
      state = { view: 'gallery', palette: 'standard' };
      roleMessages.pero = []; roleMessages.nana = [];
      preview = { id: 'preview', role: 'pero', mode: 'light', palette: 'sky', page: 'chat', art: true, fileTab: 'files', toolOpen: true, chatMode: 'chat', messages: [] };
      render(); toast('已恢复标准浅色与暗色设计'); return;
    }
    if (name === 'global-palette') { state.palette = value; render(); toast(`已应用${D.palettes.find((p) => p.id === value)?.name || '标准'}配色`); return; }
    if (name === 'preview-role') { preview.role = value; preview.messages = roleMessages[value]; render(); return; }
    if (name === 'preview-mode') { preview.mode = value; render(); return; }
    if (name === 'preview-page') { navigate(preview, value); return; }
    if (name === 'dialog-close') { dialog.close(); return; }
    if (name === 'art') { showArt(value || ctx?.role); return; }
    if (name === 'file') { showFile(value); return; }
    if (name === 'download-sample') { downloadSample(value); return; }
    if (name === 'sample-toast') { toast('这是组件演示，未执行远程操作'); return; }
    if (name === 'connection') { openDialog('家庭 infOS', '远程连接状态 · 设计示例', `<p class="dialog-copy">这里展示客户端连接信息、连接中断提示与重新连接入口。当前页面使用本地示例数据，未发起远程连接。</p><div class="dialog-actions"><button ${action('dialog-close')}>知道了</button></div>`); return; }
    if (name === 'menu') {
      openDialog('会话与显示', '设计预览', `<div class="dialog-menu"><button ${action('dialog-art', ctx?.role || preview.role)}>${icon('image')}查看角色 CG</button><button ${action('dialog-appearance', ctx?.id || 'preview')}>${icon('palette')}显示偏好</button></div>`); return;
    }
    if (name === 'dialog-art') { showArt(value); return; }
    if (name === 'dialog-appearance') { dialog.close(); const targetContext = contexts.get(value); if (targetContext) navigate(targetContext, 'appearance'); return; }
    if (name === 'attach') { openDialog('添加附件', '本地交互演示', `<p class="dialog-copy">选择一个文件体验附件入口。此原型只显示文件名，不读取内容，也不上传。</p><input type="file" id="demo-attachment" aria-label="选择演示附件" /><div class="dialog-actions"><button ${action('dialog-close')}>关闭</button></div>`); return; }
    if (!ctx) { if (name === 'page') toast('请在交互预览中体验页面导航'); return; }
    if (name === 'page') { navigate(ctx, value); return; }
    if (name === 'back') { navigate(ctx, ctx.history?.pop() || 'home', false); return; }
    if (name === 'mode') { ctx.mode = value; ctx.palette = paletteFor(value); refresh(ctx); return; }
    if (name === 'palette') { ctx.palette = value === 'standard' ? (ctx.mode === 'light' ? 'sky' : 'night') : value; if (ctx.id === 'preview') { state.palette = value; paletteSelect.value = value; } refresh(ctx); return; }
    if (name === 'toggle-art') { ctx.art = !ctx.art; refresh(ctx); return; }
    if (name === 'skin') { ctx.skin = !ctx.skin; refresh(ctx); toast(ctx.skin ? '已启用预置角色样式，返回会话查看气泡变化' : '已恢复标准组件样式'); return; }
    if (name === 'decision') { ctx.decision = value === 'reset' ? undefined : value; refresh(ctx); if (value !== 'reset') toast('已切换确认状态；本次演示未写入远程资料'); return; }
    if (name === 'file-tab') { ctx.fileTab = value; refresh(ctx); return; }
    if (name === 'search') { ctx.search = !ctx.search; refresh(ctx); root.querySelector(`[data-phone="${ctx.id}"] [data-file-search]`)?.focus(); return; }
    if (name === 'chat-mode') { ctx.chatMode = value; refresh(ctx); toast(value === 'work' ? '已切换工作输入样式（交互演示）' : '已切换聊天输入样式（交互演示）'); return; }
    if (name === 'suggestion') { const input = root.querySelector(`[data-phone="${ctx.id}"] textarea`); if (input) { input.value = value; input.focus(); } }
  });
  document.addEventListener('submit', (event) => {
    if (!event.target.matches('[data-composer]')) return;
    event.preventDefault();
    const ctx = contexts.get(event.target.closest('[data-phone]')?.dataset.phone);
    const input = event.target.elements.message;
    const text = input.value.trim();
    if (!ctx || !text) return;
    ctx.messages ||= [];
    ctx.messages.push({ role: 'user', text }, { role: 'assistant', text: D.roles[ctx.role].sampleReply });
    refresh(ctx);
    const feed = root.querySelector(`[data-phone="${ctx.id}"] .chat-feed`);
    if (feed) feed.scrollTop = feed.scrollHeight;
    root.querySelector(`[data-phone="${ctx.id}"] textarea`)?.focus();
    toast('消息仅用于本地交互演示，未请求模型');
  });
  document.addEventListener('keydown', (event) => {
    if (!event.target.matches('[data-composer] textarea')) return;
    if (event.key === 'Enter' && !event.shiftKey && !event.isComposing && event.keyCode !== 229) {
      event.preventDefault();
      event.target.form.requestSubmit();
    }
  });
  document.addEventListener('input', (event) => {
    if (!event.target.matches('[data-file-search]')) return;
    const ctx = contexts.get(event.target.closest('[data-phone]')?.dataset.phone);
    if (!ctx) return;
    ctx.query = event.target.value;
    refresh(ctx);
    const input = root.querySelector(`[data-phone="${ctx.id}"] [data-file-search]`);
    input?.focus();
  });
  document.addEventListener('change', (event) => {
    if (event.target === paletteSelect) { state.palette = paletteSelect.value; render(); }
    if (event.target.id === 'demo-attachment') { const file = event.target.files?.[0]; if (file) toast(`已选择 ${file.name}（未读取或上传）`); }
  });
  dialog.addEventListener('click', (event) => {
    if (event.target !== dialog) return;
    const bounds = dialog.getBoundingClientRect();
    if (event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom) dialog.close();
  });
  paletteSelect.insertAdjacentHTML('beforeend', D.palettes.map((p) => `<option value="${p.id}">${p.name} · ${p.mood}</option>`).join(''));
  render();
})();
