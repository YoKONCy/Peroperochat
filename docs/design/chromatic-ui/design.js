(() => {
  const { packs, palette, roles } = window.Chromatic;
  const assets = '../../../public/';
  const extra = {
    people: '<path class="wash" d="M3 21v-3a7 7 0 0 1 14 0v3Z"/><circle cx="10" cy="6" r="4"/><path d="M3 21v-3a7 7 0 0 1 14 0v3M18 3a4 4 0 0 1 0 8m3 10v-3a7 7 0 0 0-3-6"/>',
    folder: '<path class="wash" d="M2 6h7l3 3h10v11H2Z"/><path d="M2 6h7l3 3h10v11H2ZM2 6V3h8l3 3h7v3M6 13h6"/>',
    tasks: '<rect class="wash" x="4" y="4" width="16" height="18" rx="3"/><rect x="4" y="4" width="16" height="18" rx="3"/><path d="M9 4V2h6v4H9Zm-2 8 2 2 3-4m2 3h3M8 18h9"/>',
    settings: '<path class="wash" d="m9 2 6 0 1 3 3 1 3 6-3 6-3 1-1 3H9l-1-3-3-1-3-6 3-6 3-1Z"/><path d="m9 2 6 0 1 3 3 1 3 6-3 6-3 1-1 3H9l-1-3-3-1-3-6 3-6 3-1Z"/><circle cx="12" cy="12" r="4"/>',
    home: '<path class="wash" d="m3 10 9-8 9 8v12H3Z"/><path d="m1 12 11-10 11 10M4 9v13h16V9M9 22v-7h6v7"/><path d="m10 8 2-2 2 2-2 2Z"/>',
  };
  const icon = name => extra[name] ? `<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.65" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${extra[name]}</svg>` : window.designIcon(name);
  const pages = [ ['conversations', '会话', '真实头像 / 轻重分明'], ['groups', '群聊', '独立空间 / 角色同场'], ['tasks', '任务', '进展与待确认分层'], ['library', '资料', '按内容类型识别'], ['management', '管理', '有色彩，也有秩序'], ['model', '模型调参', '分区卡片 / 明确操作'] ];
  let pack = 'sky', dark = false, filter = 'all';
  const state = { selectedAgent: 'pero', reasoning: '高', completed: false, taskFilter: '全部', title: '周末，去海边吧', saved: true };
  function variables(element) { for (const [key, value] of Object.entries(palette(pack, dark))) element.style.setProperty('--' + key, value); }
  const avatar = (name, cls = '') => `<img class="avatar ${cls}" src="${assets}avatars/${name}.jpg" alt="${name === 'nana' ? 'Nana' : 'Pero'}" draggable="false" />`;
  const badge = (name, tone) => `<span class="badge tone ${tone}">${icon(name)}</span>`;
  const tag = (label, tone, name = '') => `<span class="tag tone ${tone}">${name ? icon(name) : ''}${label}</span>`;
  const arrow = () => icon('next');
  const action = (name, label, data, cls = 'icon-btn') => `<button type="button" class="${cls}" aria-label="${label}" data-action="${data}">${icon(name)}${cls.includes('main-btn') ? label : ''}</button>`;
  function status() { return `<div class="status"><b>9:41</b><span>${icon('signal')}${icon('wifi')}${icon('battery')}</span></div>`; }
  function header(name, subtitle, type = 'new') { return `<header class="page-header"><div class="head-copy"><small>${subtitle}</small><h2>${name}</h2></div>${icon('bow').replace('class="icon', 'class="icon header-bow')}${action(type === 'new' ? 'plus' : type === 'appearance' ? 'star' : type, type === 'new' ? '新建' : type === 'appearance' ? '显示与主题' : '更多操作', type)}</header>`; }
  function footer(current) { return `<nav class="phone-footer" aria-label="应用导航">${[['home', '首页', 'management'], ['chat', '会话', 'conversations'], ['tasks', '任务', 'tasks'], ['folder', '资料', 'library'], ['settings', '管理', 'management']].map(([symbol, label, id], index) => `<button data-action="navigate" data-page="${id}" class="${id === current && index !== 0 ? 'selected' : ''}"><span>${icon(symbol)}</span>${label}</button>`).join('')}</nav>`; }
  function section(title, right = '', actionName = 'more') { return `<div class="section-label"><strong>${title}</strong>${right ? `<button data-action="${actionName}">${right}${arrow()}</button>` : ''}</div>`; }
  function segments(groups = false) { return `<div class="segments"><button class="${groups ? '' : 'selected'}" data-action="navigate" data-page="conversations">${icon('chat')}单聊</button><button class="${groups ? 'selected rose' : ''}" data-action="navigate" data-page="groups">${icon('people')}群聊</button></div>`; }
  function conversations() {
    const who = state.selectedAgent, name = who === 'pero' ? 'Pero' : 'Nana';
    return `${segments()}<div class="character-rail">${['pero', 'nana'].map(id => `<button data-action="agent" data-id="${id}" class="${who === id ? 'selected' : ''}">${avatar(id)}<span>${id === 'pero' ? 'Pero' : 'Nana'}</span></button>`).join('')}<button data-action="add-role"><span class="add-avatar">${icon('plus')}</span><span>添加角色</span></button></div>${section('接着上次的话题', '全部会话')}<article class="paper recent-card"><div class="thread-head">${avatar(who)}<div><strong>${state.title}</strong><small>${name} · 今天 09:38</small></div>${action('more', '管理会话', 'thread-more')}</div><p class="thread-preview">“海风、日落，还有你。不用赶路的那种。”</p><div class="thread-foot">${tag('上次聊到这里', 'rose', 'chat')}<button class="soft-btn tone action-solid" data-action="open-chat">接着聊${arrow()}</button></div></article>${section('最近聊过')}<article class="paper list-thread">${avatar('nana')}<div><strong>今天的灵感收集</strong><small>昨天 · 角色陪伴</small></div>${tag('工作', 'violet')}</article><article class="paper list-thread">${avatar('pero')}<div><strong>晚饭吃什么好呢</strong><small>周二 · 日常聊天</small></div>${action('more', '管理会话', 'thread-more')}</article><div class="tiny-note">${icon('shield')}对话保存在你连接的 infOS</div>`;
  }
  function groups() {
    return `${segments(true)}<section class="group-hero"><small>OUR LITTLE SPACE</small><h3>热闹一点，也很好。</h3><p>把大家叫到一起，<br />从一个小话题开始。</p><div class="avatar-stack">${avatar('pero')}${avatar('nana')}<span class="self-avatar">你</span></div>${icon('people')}</section>${section('我的群聊', '新建', 'new-group')}<article class="paper group-entry">${badge('people', 'rose')}<div><h4>我们的周末小分队</h4><p>Pero、Nana 和你</p>${tag('3 位成员', 'rose')}</div><button data-action="group" aria-label="进入群聊">${arrow()}</button></article><article class="paper group-entry">${badge('star', 'amber')}<div><h4>灵感碰头会</h4><p>一起把脑洞变成计划</p>${tag('最近活跃', 'mint')}</div><button data-action="group" aria-label="进入灵感碰头会">${arrow()}</button></article>${section('上次聊到')}<article class="paper chat-group-preview"><div class="mini-person">${avatar('nana')}<strong>Nana</strong><time>09:32</time></div><div class="mini-speech">那就说好了！<br />这次谁都不许临时赖床哦。</div><button class="soft-btn tone rose action-solid" data-action="group">回到周末小分队${arrow()}</button></article>`;
  }
  function tasks() {
    const all = state.taskFilter === '全部', done = state.taskFilter === '已完成';
    return `<section class="task-overview"><div class="task-ring"><span>3<small>正在推进</small></span></div><div><h3>小事，一件件做好。</h3><p>今天已完成 2 件</p></div></section><div class="task-filters">${['全部', '进行中', '待确认', '已完成'].map(label => `<button data-action="task-filter" data-value="${label}" class="${state.taskFilter === label ? 'selected' : ''}">${label}</button>`).join('')}</div>${all || state.taskFilter === '进行中' ? `<article class="paper task-card"><div class="task-line">${badge('file', 'mint')}<strong>整理海边行程</strong>${tag('进行中', 'mint')}</div><p>已找到路线，正在整理交通与住宿。</p><div class="progress"><i></i></div><div class="task-card-footer"><span>${avatar('pero')}Pero · 2 / 3 步</span><button class="soft-btn tone mint action-solid" data-action="task">查看进展${arrow()}</button></div></article>` : ''}${all || state.taskFilter === '待确认' ? `<article class="paper task-card needs-attention"><div class="task-line">${badge('shield', 'amber')}<strong>有一步，需要你点头</strong></div><p>允许保存整理好的行程文件吗？</p><div class="task-card-footer">${tag('等待确认', 'amber')}<button class="soft-btn tone amber" data-action="approval">查看并确认${arrow()}</button></div></article>` : ''}${all || done ? `${section('已经做好啦')}<article class="paper task-complete">${icon('check')}<span>整理本周的读书笔记</span><small>08:46</small></article><article class="paper task-complete" style="margin-top:10px">${icon('check')}<span>查好周末的天气</span><small>08:20</small></article>` : ''}`;
  }
  function library() {
    return `<label class="library-search">${icon('search')}<input placeholder="找文件、记忆和文档…" aria-label="搜索资料" data-search="true" /></label><div class="category-grid">${[['folder', '文件', '12 个文件', 'blue'], ['thought', '记忆', '一起积累', 'violet'], ['file', '文档', '随时翻一翻', 'amber']].map(([symbol, label, desc, tone]) => `<button class="tone ${tone}" data-action="library" data-value="${label}">${icon(symbol)}<strong>${label}</strong><small>${desc}</small></button>`).join('')}</div>${section('最近打开', '管理资料', 'manage-library')}<article class="paper file-feature"><div class="file-preview"><div class="document-page"><strong>海边的小旅行</strong><p>不赶路，给日落留一点时间。</p><div class="document-lines"><i></i><i></i><i></i><i></i></div></div></div><div class="file-info">${badge('file', 'blue')}<span><strong>海边小旅行.md</strong><small>Markdown · 今天 09:36</small></span>${action('next', '展开文件', 'file')}</div></article><article class="paper folder-row">${badge('folder', 'amber')}<span><strong>旅行与日常</strong><small>6 个文件</small></span><button data-action="folder" aria-label="打开旅行与日常">${arrow()}</button></article><article class="paper folder-row">${badge('image', 'rose')}<span><strong>给你的照片.png</strong><small>图片 · 昨天</small></span><button data-action="image" aria-label="展开图片">${arrow()}</button></article><section class="memory-note"><span>${icon('thought')}记忆里的一小页</span><p>你喜欢不赶时间的旅行，<br />也喜欢沿着海边慢慢走。</p></section>`;
  }
  function management() {
    return `<section class="paper management-hero">${avatar('nana')}<div><strong>把陪伴调成喜欢的样子</strong><p>角色、模型，还有日常习惯。</p>${tag('已连接 infOS', 'mint', 'connection')}</div></section>${section('常用设置')}<div class="settings-grid">${[['model', '模型与调参', '2 个模型 · 随时调整', 'blue', 'model'], ['people', '角色管理', '头像、人设与互动', 'rose', 'roles'], ['sun', '主动陪伴', '什么时候来找你', 'amber', 'companionship'], ['thought', '记忆与资料', '一起记住的小事', 'violet', 'memory']].map(([symbol, title, sub, tone, target]) => `<button class="settings-tile tone ${tone}" data-action="${target}">${badge(symbol, tone)}<strong>${title}</strong><small>${sub}</small>${arrow()}</button>`).join('')}</div>${section('应用与连接')}<div class="paper settings-list"><button class="settings-row" data-action="appearance">${badge('star', 'violet')}<span><strong>显示与主题</strong><small>配色包 · 字号 · 轻动效</small></span>${arrow()}</button><button class="settings-row" data-action="connection">${badge('connection', 'blue')}<span><strong>连接管理</strong><small>家里的 infOS · 已连接</small></span>${arrow()}</button></div><button class="usage-strip" data-action="usage"><span>${icon('pulse')}运行概览</span><small>设备用量${arrow()}</small></button>`;
  }
  function model() {
    return `<section class="paper model-banner"><span class="model-brand"><img src="${assets}model-brands/deepseek-color.svg" alt="DeepSeek" /></span><div><small>当前使用的模型</small><h3>DeepSeek V4</h3><p>deepseek-v4-pro</p></div>${tag('使用中', 'mint')}</section><div class="segments model-tabs"><button data-action="model-identity">${icon('model')}模型</button><button class="selected" data-action="model-tuning">${icon('tuning')}调参</button><button data-action="model-connection">${icon('connection')}连接</button></div><section class="thought-options"><header>${icon('thought')}<strong>思考强度</strong><small>慢慢想，或直接回答</small></header><div>${['默认', '低', '中', '高', '极高'].map(label => `<button class="${state.reasoning === label ? 'selected' : ''}" data-action="reasoning" data-value="${label}">${label}</button>`).join('')}</div></section>${section('回复参数')}<div class="parameter-grid">${[['温度', '.8', '更灵活地组织语言', 'rose'], ['Top P', '.9', '用词的选择范围', 'mint'], ['最大输出', '8192', '一次回复的长度上限', 'blue'], ['上下文', '131072', '模型可读取的内容', 'violet']].map(([label, value, help, tone]) => `<label class="parameter tone ${tone}"><span>${label}<button type="button" data-action="reset">默认</button></span><input type="number" step="any" value="${value}" aria-label="${label}" data-parameter="true" /><small>${help}</small></label>`).join('')}</div>${section('输入与输出')}<div class="model-flags">${[['image', '图片理解', 'blue'], ['pulse', '流式回复', 'mint'], ['thought', '显示思考', 'violet'], ['audio', '音频输入', 'rose']].map(([symbol, label, tone], index) => `<label class="flag tone ${tone}">${icon(symbol)}${label}<input type="checkbox" role="switch" aria-label="${label}" ${index < 3 ? 'checked' : ''} /></label>`).join('')}</div>`;
  }
  const contents = { conversations, groups, tasks, library, management, model };
  const subtitles = { conversations: '与你的角色，接着聊', groups: '把大家叫到一起', tasks: '和你一起，把事情做好', library: '一起积累的小世界', management: '陪伴，也有自己的偏好', model: '调成顺手的样子' };
  function render() {
    document.body.classList.toggle('dark', dark);
    document.querySelector('#mode').textContent = dark ? '切换浅色' : '切换暗色';
    document.querySelector('#mode').setAttribute('aria-pressed', dark);
    document.querySelector('#swatches').innerHTML = packs.map(([id, name, light, night]) => `<button style="background:${dark ? night : light}" data-pack="${id}" title="${name}" aria-label="${name}" aria-pressed="${pack === id}"></button>`).join('');
    document.querySelector('#filters').innerHTML = [['all', '全部页面'], ...pages.map(([id, label]) => [id, label])].map(([id, label]) => `<button data-filter="${id}" class="${filter === id ? 'selected' : ''}">${label}</button>`).join('');
    document.querySelector('#screens').innerHTML = pages.map(([id, label, note], index) => `<section class="screen-wrap ${filter !== 'all' && filter !== id ? 'is-hidden' : ''}" data-screen="${id}"><div class="screen-title"><strong>0${index + 1} / ${label}</strong><small>${note}</small></div><article class="phone soft-arrive" data-phone="${id}">${status()}${header(label, subtitles[id], id === 'model' ? 'more' : id === 'management' ? 'appearance' : 'new')}<div class="phone-scroll">${contents[id]()}</div>${id === 'model' ? `<footer class="save-tray"><span>${icon(state.saved ? 'check' : 'edit')}${state.saved ? '设置已保存' : '有修改'}</span><button class="main-btn" data-action="save">${icon('check')}保存模型</button></footer>` : footer(id === 'groups' ? 'conversations' : id)}</article></section>`).join('');
    document.querySelector('#color-roles').innerHTML = roles.map(([tone, title, desc]) => `<div class="role-chip tone ${tone}"><i></i><span><strong>${title}</strong><small>${desc}</small></span></div>`).join('');
    document.querySelectorAll('.phone, #color-roles, #sheet').forEach(variables);
  }
  function sheet(title, description, body = '') {
    document.querySelector('#sheet-content').innerHTML = `<h2>${title}</h2><p>${description}</p>${body}`;
    document.querySelector('#sheet').showModal();
  }
  function go(page) { filter = page; render(); document.querySelector('#filters').scrollIntoView({ behavior: document.body.classList.contains('reduce-motion') ? 'instant' : 'smooth', block: 'start' }); }
  document.querySelector('#brand-icon').innerHTML = icon('bow');
  document.querySelector('#sheet-close').innerHTML = icon('close');
  document.addEventListener('click', event => {
    const target = event.target.closest('button'); if (!target) return;
    if (target.dataset.pack) { pack = target.dataset.pack; render(); return; }
    if (target.dataset.filter) { filter = target.dataset.filter; render(); return; }
    if (target.id === 'mode') { dark = !dark; render(); return; }
    if (target.id === 'motion') { const reduce = document.body.classList.toggle('reduce-motion'); target.setAttribute('aria-pressed', reduce); return; }
    const action = target.dataset.action;
    if (!action) return;
    if (action === 'navigate') { go(target.dataset.page); return; }
    if (action === 'model') { go('model'); return; }
    if (action === 'agent') { state.selectedAgent = target.dataset.id; render(); return; }
    if (action === 'reasoning') { state.reasoning = target.dataset.value; state.saved = false; render(); return; }
    if (action === 'task-filter') { state.taskFilter = target.dataset.value; render(); return; }
    if (action === 'save') { state.saved = true; render(); sheet('保存好了', '这是设计稿中的保存反馈，不会修改你的 infOS。'); return; }
    if (action === 'reset') { target.closest('label').querySelector('input').value = ''; state.saved = false; return; }
    if (action === 'thread-more') { sheet('会话操作', '操作收进短菜单，不挤占列表。', '<label>会话名称<input id="thread-name" value="周末，去海边吧" /></label><button class="main-btn sheet-submit" data-action="rename">保存名称</button>'); return; }
    if (action === 'rename') { state.title = document.querySelector('#thread-name').value.trim().slice(0, 25) || state.title; document.querySelector('#sheet').close(); render(); return; }
    const details = {
      'open-chat': ['接着聊', '聊天仍沿用前一版的紧凑布局；这里重点展示会话入口与信息页配色。'],
      group: ['我们的周末小分队', '群聊拥有和单聊并列的入口，成员、房间和最新对话分别呈现。'],
      task: ['整理海边行程', '已查好天气与路线，正在整理最后一段交通。进展先给摘要，步骤点击再展开。'],
      approval: ['保存行程文件', '待确认事项使用琥珀色独立呈现；确认前展示操作范围与具体内容。'],
      file: ['海边小旅行.md', '文件以缩略小窗进入完整阅读页；图片、音频和视频仍原地展开。'],
      image: ['图片预览', '点击图片进入可放大的独立预览，返回后保留资料页位置。'],
      roles: ['角色管理', '真实头像、人设和互动设置分区；自定义角色使用用户提供的图片与名称。'],
      companionship: ['主动陪伴', '模式使用本地化名称，常用选项用可识别的功能卡片呈现。'],
      memory: ['记忆与资料', '记忆使用柔和紫色识别，与普通文件及待处理事项分开。'],
      appearance: ['显示与主题', '上方八个配色按钮可以实际切换本稿所有页面；功能辅色也会随配色和明暗适配。'],
      usage: ['运行与用量', '这里集中设备用量与运行概览，不再摆满移动端不常用的高级管理功能。'],
      connection: ['连接管理', '连接身份、授权状态和进入动作分行排版，避免长名称挤压按钮。'],
      'add-role': ['添加角色', '头像、名称、人设与立绘均来自角色内容，不绑定 Pero 或 Nana。'],
      'model-identity': ['模型', '配置显示名称、模型 ID、输入能力，并获取服务商模型列表。'],
      'model-connection': ['连接', '选择共享连接或配置独立连接；密钥与连接状态按区域归类。'],
      'manage-library': ['管理资料', '文件、记忆、文档分别进入对应管理页。常用动作带图标和明确按钮底色。'],
      library: ['资料分类', `这里可进入对应分类，保留当前搜索与浏览位置。`],
    };
    const info = details[action] || [action === 'new-group' ? '创建群聊' : action === 'new' ? '新建' : '更多内容', '这是交互设计示意；本稿不发送网络请求，也不修改应用数据。'];
    sheet(...info);
  });
  document.addEventListener('input', event => { if (event.target.dataset.parameter) { state.saved = false; const tray = event.target.closest('.phone').querySelector('.save-tray>span'); tray.innerHTML = `${icon('edit')}有修改`; } });
  document.querySelector('#sheet').addEventListener('click', event => { if (event.target === event.currentTarget) event.currentTarget.close(); });
  render();
})();
