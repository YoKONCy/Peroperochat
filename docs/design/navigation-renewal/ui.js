/* 独立设计稿的组件与示例数据；所有状态仅存在于此预览中。 */
(() => {
  const P = window.PPC = {};
  P.assets = '../../../public/';
  P.state = { page: 'home', pack: 'sky', dark: false, reduced: false, selectedRole: 'all', taskFilter: 'all', memoryFilter: 'active', model: 'Gemini 3 Pro', effort: '高', dirty: false, archived: false, taskApproved: false, sent: [], groupSent: [], title: '给周末留一点空白', offline: false, tools: { search: true, files: true, script: false }, draft: '', groupDraft: '', memoryText: '你喜欢不赶时间的散步。人少的河边、安静的小店，还有回家路上的日落，都是你愿意多停留一会儿的地方。', promptText: '用自然、简短的语言回应。\n记住对方说过的小事，保持角色自己的语气。\n需要执行任务时，清楚说明结果；没有完成的事情不要说已经完成。' };
  P.esc = s => String(s ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
  const paths = {
    home: '<path class="wash" d="M3 11 12 3l9 8v10H3Z"/><path d="m2 11 10-9 10 9M5 10v11h14V10M10 21v-7h4v7"/><path class="detail" d="m10 8 2-2 2 2-2 2Z"/>',
    up: '<path d="m6 15 6-6 6 6"/>',
    people: '<circle cx="9" cy="7" r="3.5"/><path class="wash" d="M2 21v-2a7 7 0 0 1 14 0v2Z"/><path d="M2 21v-2a7 7 0 0 1 14 0v2M17 4a4 4 0 0 1 0 8m2 3a6 6 0 0 1 3 6"/>',
    person: '<circle cx="12" cy="7" r="4"/><path class="wash" d="M4 21v-2a8 8 0 0 1 16 0v2Z"/><path d="M4 21v-2a8 8 0 0 1 16 0v2"/>',
    folder: '<path class="wash" d="M3 7h6l3 3h9v10H3Z"/><path d="M3 7h6l3 3h9v10H3Zm0 0V4h7l3 3h7v3M7 15h6"/>',
    tasks: '<rect class="wash" x="5" y="4" width="15" height="18" rx="3"/><rect x="5" y="4" width="15" height="18" rx="3"/><path d="M9 4V2h7v4H9Zm-1 9 2 2 3-5m2 3h2M9 19h7"/>',
    settings: '<path class="wash" d="m9 3 6 0 2 3 3 2 1 5-3 4-2 4H9l-2-3-4-2-1-5 3-4Z"/><path d="m9 3 6 0 2 3 3 2 1 5-3 4-2 4H9l-2-3-4-2-1-5 3-4Z"/><circle cx="11.5" cy="12" r="3.5"/>',
    spark: '<path class="wash" d="m12 2 2.7 7.3L22 12l-7.3 2.7L12 22l-2.7-7.3L2 12l7.3-2.7Z"/><path d="m12 2 2.7 7.3L22 12l-7.3 2.7L12 22l-2.7-7.3L2 12l7.3-2.7Z"/>',
    heart: '<path class="wash" d="M12 21 3 12a5.5 5.5 0 0 1 9-7 5.5 5.5 0 0 1 9 7Z"/><path d="M12 21 3 12a5.5 5.5 0 0 1 9-7 5.5 5.5 0 0 1 9 7Z"/>',
    memory: '<rect class="wash" x="5" y="3" width="15" height="18" rx="3"/><rect x="5" y="3" width="15" height="18" rx="3"/><path d="M8 3v18m3-7h5m-5 3h3m0-12 2 3-2 3-2-3Z"/>',
    archive: '<rect class="wash" x="4" y="8" width="16" height="13" rx="2"/><rect x="4" y="8" width="16" height="13" rx="2"/><rect x="2" y="3" width="20" height="5" rx="1.5"/><path d="M9 12h6"/>',
    clock: '<circle class="wash" cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="9"/><path d="M12 6v6l4 2"/>',
    bell: '<path class="wash" d="M6 9a6 6 0 0 1 12 0v6l2 3H4l2-3Z"/><path d="M6 9a6 6 0 0 1 12 0v6l2 3H4l2-3ZM10 22h4M12 1v2"/>',
    grid: '<rect x="3" y="3" width="7" height="7" rx="2"/><rect x="14" y="3" width="7" height="7" rx="2"/><rect x="3" y="14" width="7" height="7" rx="2"/><rect x="14" y="14" width="7" height="7" rx="2"/>',
    palette: '<path class="wash" d="M12 2a10 10 0 1 0 0 20h2a3 3 0 0 0 0-6 2 2 0 0 1 0-4h4a4 4 0 0 0 4-4c0-4-5-6-10-6Z"/><path d="M12 2a10 10 0 1 0 0 20h2a3 3 0 0 0 0-6 2 2 0 0 1 0-4h4a4 4 0 0 0 4-4c0-4-5-6-10-6Z"/><circle cx="6" cy="11" r="1"/><circle cx="9" cy="6" r="1"/><circle cx="15" cy="6" r="1"/>',
    qr: '<path d="M3 8V3h5m8 0h5v5M3 16v5h5m8 0h5v-5"/><path d="M7 7h3v3H7Zm7 0h3v3h-3ZM7 14h3v3H7Zm7 0h3m-3 0v3h3v-2"/>',
    globe: '<circle class="wash" cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="9"/><ellipse cx="12" cy="12" rx="4" ry="9"/><path d="M3 12h18M5 7h14M5 17h14"/>',
    chart: '<path class="wash" d="M4 20V9h4v11m2 0V4h4v16m2 0v-8h4v8"/><path d="M4 20V9h4v11m2 0V4h4v16m2 0v-8h4v8M2 21h20"/>',
    logout: '<path d="M9 3H4v18h5m5-14 5 5-5 5m-5-5h12"/>',
    download: '<path d="M12 3v12m-5-5 5 5 5-5M4 17v4h16v-4"/>',
    play: '<path class="wash" d="m8 4 12 8-12 8Z"/><path d="m8 4 12 8-12 8Z"/>',
    pause: '<rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/>',
    code: '<path d="m7 6-5 6 5 6m10-12 5 6-5 6M14 3l-4 18"/>',
    checkcircle: '<circle class="wash" cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="9"/><path d="m7 12 3 3 7-7"/>',
    route: '<circle cx="5" cy="5" r="2"/><circle cx="19" cy="19" r="2"/><path d="M7 5h9a4 4 0 0 1 0 8H8a3 3 0 0 0 0 6h9"/>',
    offline: '<path d="m3 3 18 18M2 8a16 16 0 0 1 3-2m4-2a16 16 0 0 1 13 4M6 12a10 10 0 0 1 3-2m5 0a10 10 0 0 1 4 2m-8 4a4 4 0 0 1 4 0"/><circle cx="12" cy="20" r="1"/>'
  };
  P.icon = n => paths[n] ? `<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.65" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths[n]}</svg>` : window.designIcon(n);
  P.avatar = (id = 'pero', cls = '') => `<img class="avatar ${cls}" src="${P.assets}avatars/${id}.jpg" alt="${id === 'pero' ? 'Pero' : 'Nana'}" draggable="false">`;
  P.btn = (name, label, action, cls = '') => `<button type="button" class="icon-btn ${cls}" data-action="${action}" aria-label="${label}" title="${label}">${P.icon(name)}</button>`;
  P.go = (id, text, symbol = '', cls = 'text-btn') => `<button class="${cls}" data-go="${id}">${symbol ? P.icon(symbol) : ''}${text}</button>`;
  P.badge = (name, tone = 'primary', cls = '') => `<span class="badge ${cls}" data-tone="${tone}">${P.icon(name)}</span>`;
  P.tag = (text, tone = 'primary', dot = false) => `<span class="tag" data-tone="${tone}">${dot ? '<i class="dot"></i>' : ''}${text}</span>`;
  P.section = (title, link = '', to = '') => `<div class="section-title"><h3>${title}</h3>${link ? `<button class="text-btn" ${to.startsWith('!') ? `data-action="${to.slice(1)}"` : `data-go="${to}"`}>${link}${P.icon('next')}</button>` : ''}</div>`;
  P.row = (symbol, title, sub, dest, tone = 'primary', tail = '') => `<button class="list-row" data-go="${dest}">${P.badge(symbol, tone)}<span class="row-copy"><strong>${title}</strong>${sub ? `<small>${sub}</small>` : ''}</span>${tail ? `<span class="row-value">${tail}</span>` : ''}${P.icon('next')}</button>`;
  P.search = (placeholder, scope) => `<label class="search-box">${P.icon('search')}<input type="search" placeholder="${placeholder}" aria-label="${placeholder}" data-search="${scope}"></label>`;
  P.field = (label, value, options = {}) => `<label class="field"><span>${label}${options.hint ? `<small>${options.hint}</small>` : ''}</span>${options.area ? `<textarea rows="${options.rows || 5}" data-field="${options.key || label}" ${options.placeholder ? `placeholder="${options.placeholder}"` : ''}>${P.esc(value)}</textarea>` : `<input type="${options.type || 'text'}" value="${P.esc(value)}" data-field="${options.key || label}">`}</label>`;
  P.toggle = (title, sub, checked = true, key = '') => `<label class="toggle-row"><span><strong>${title}</strong>${sub ? `<small>${sub}</small>` : ''}</span><input type="checkbox" ${checked ? 'checked' : ''} data-toggle="${key || title}"><span class="switch" aria-hidden="true"></span></label>`;
  P.choice = (label, value, action, name = '') => `<button class="choice-row" data-action="${action}"><span>${name ? P.icon(name) : ''}${label}</span><strong>${value}${P.icon('next')}</strong></button>`;
  P.notice = (text, tone = 'mint', name = 'checkcircle') => `<div class="inline-note" data-tone="${tone}">${P.icon(name)}<span>${text}</span></div>`;
  P.brand = (id = 'gemini') => `<img class="brand-logo" src="${P.assets}model-brands/${id === 'openai' ? 'openai' : id + '-color'}.svg" alt="${id}">`;
  P.tabs = (items, active) => `<nav class="page-tabs" aria-label="页面分区">${items.map(([id,label,n]) => `<button data-go="${id}" class="${active === id ? 'selected' : ''}" aria-current="${active === id ? 'page' : 'false'}">${n ? P.icon(n) : ''}${label}</button>`).join('')}</nav>`;
  P.groups = [
    ['从这里开始', ['home','notifications','connections','local']],
    ['会话与据点', ['threads','chat','groups','group-chat','base','group-settings']],
    ['任务与进展', ['tasks','task-detail']],
    ['资料与记忆', ['files','file-detail','memories','memory-detail','memory-edit','knowledge']],
    ['管理与偏好', ['manage','roles','role-detail','models','model-detail','assignments','voice','tools','companion','prompts','prompt-edit','profile','appearance','usage','runtime']]
  ];
  const defs = [
    ['home','首页','home',null,'home','日常入口','继续上次会话、查看待处理事项、进入连接设置。'],
    ['notifications','通知','home','home','bell','只呈现需要留意的事','操作直接带往相关对象，不重复堆放系统说明。'],
    ['connections','连接与模式','manage','home','connection','连接有独立位置','日常连接、添加设备、证书确认分开；保留 HTTP。'],
    ['local','独立空间','home','connections','heart','两种模式，共用设计语言','房间承担陪伴感，独立数据不与远程混合。'],
    ['threads','会话','threads',null,'chat','从人到话题','头像筛选、摘要和时间帮助识别会话；长按也可打开操作。'],
    ['chat','给周末留一点空白','threads','threads','chat','聊天独占内容空间','移除常驻底部导航；返回列表原位置，详细工具收进二级入口。'],
    ['groups','会话','threads',null,'people','群聊与单聊并列','房间有成员、最近消息和所属据点，不再藏在管理中。'],
    ['group-chat','放课后客厅','threads','groups','people','群聊有自己的节奏','保留发言者身份与接话状态；选成员是轻面板。'],
    ['base','星光小屋','threads','groups','home','据点承接空间和成员','从群聊列表进入据点，房间、成员、环境各有位置。'],
    ['group-settings','房间设置','threads','group-chat','settings','群聊的设置属于群聊','成员、房间资料与环境集中于此。'],
    ['tasks','任务','tasks',null,'tasks','先处理，再看过程','待确认、进行中、已完成分层；无虚构百分比。'],
    ['task-detail','周末散步路线','tasks','tasks','route','每个状态都有下一步','查看原会话、执行步骤与结果；审批说明具体范围。'],
    ['files','资料','files',null,'folder','资料按内容形态组织','文件、记忆、知识独立分区；上传入口明显。'],
    ['file-detail','周末路线.md','files','files','file','先读内容，再做操作','轻量预览、下载与来源会话共用固定工具条。'],
    ['memories','资料','files',null,'memory','可读的记忆列表','有效、归档、事实清楚分开；编辑与来源不重叠。'],
    ['memory-detail','记忆详情','files','memories','memory','正文先行，元信息次之','编辑、归档、来源有独立位置，不挤在文本旁。'],
    ['memory-edit','编辑记忆','files','memory-detail','edit','长编辑使用完整页面','单列居中，保存固定；退出有未保存保护。'],
    ['knowledge','资料','files',null,'file','可浏览的知识目录','分类、导入和文档预览形成完整路径。'],
    ['manage','管理','manage',null,'settings','按用途寻找设置','四组主分类；只有运行概览和用量统计作为高级入口。'],
    ['roles','角色与陪伴','manage','manage','people','先选角色，再改资料','头像保持完整，个性化内容和通用设置分开。'],
    ['role-detail','Pero 的资料','manage','roles','person','角色内容不硬编码','头像、介绍、人设和风格包可替换；此处均为演示内容。'],
    ['models','模型与语音','manage','manage','model','常用配置有清晰入口','模型列表、用途分配、语音与工具分别进入。'],
    ['model-detail','Gemini 3 Pro','manage','models','model','模型编辑分组','连接和调参不混排；获取模型列表有明确按钮。'],
    ['assignments','模型分工','manage','models','route','用用途解释配置','主要聊天、视觉、后台整理分组，保留继承关系。'],
    ['voice','语音','manage','models','audio','试听后再选择','识别、朗读分组，朗读声音能就地试听。'],
    ['tools','工具权限','manage','models','tool','权限一眼可辨','系统、启用、停用分组；父工具下的小工具贴在一起。'],
    ['companion','主动陪伴','manage','roles','heart','按陪伴节奏选择','模式使用中文名称，详细时间设置渐进展开。'],
    ['prompts','提示词','manage','roles','file','分类与编辑分屏','先按用途找条目，打开独立编辑页，无需向下寻找编辑栏。'],
    ['prompt-edit','回复风格','manage','prompts','edit','专注编辑','明确作用范围，固定保存；退出保护未保存修改。'],
    ['profile','个人资料','manage','manage','person','用户偏好集中','头像、称呼、回答偏好与预填充分组。'],
    ['appearance','外观与交互','manage','manage','palette','样式可组合','八套配色各有深浅色；背景、透明度与动态可以调整。'],
    ['usage','用量统计','manage','manage','chart','概览先于数字明细','按时间查看用量分布，清楚区分消息与 Token。'],
    ['runtime','运行概览','manage','manage','pulse','状态对应行动','服务状态、当前任务、连接恢复；技术信息默认收起。']
  ];
  P.pages = Object.fromEntries(defs.map(([id,title,tab,parent,icon,heading,note]) => [id,{id,title,tab,parent,icon,heading,note}]));
  P.nav = active => `<nav class="bottom-nav" aria-label="主要页面">${[['home','首页','home'],['threads','会话','chat'],['tasks','任务','tasks'],['files','资料','folder'],['manage','管理','settings']].map(([id,label,n]) => `<button data-tab="${id}" class="${active === id ? 'selected' : ''}" ${active===id?'aria-current="page"':''}><span>${P.icon(n)}${id === 'tasks' && !P.state.taskApproved ? '<i class="nav-dot"></i>' : ''}</span><small>${label}</small></button>`).join('')}</nav>`;
  P.status = () => `<div class="status-bar"><strong>9:41</strong><span>${P.icon('signal')}${P.icon('wifi')}${P.icon('battery')}</span></div>`;
  P.empty = (title, text, action = '', label = '') => `<div class="empty-state"><span class="empty-art">${P.icon('spark')}<i></i></span><h3>${title}</h3><p>${text}</p>${action ? `<button class="primary-btn" data-action="${action}">${label}</button>` : ''}</div>`;
})();
