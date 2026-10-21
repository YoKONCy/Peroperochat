(() => {
  const root = document.documentElement;
  const icon = (name, cls = '') => `<svg class="icon ${cls}" aria-hidden="true"><use href="#i-${name}"></use></svg>`;
  const roles = {
    pero: { name: 'Pero', avatar: '../remote-client-prototype/assets/pero-avatar.jpg', cg: '../remote-client-prototype/assets/pero-cg.png', mode: 'light' },
    nana: { name: 'Nana', avatar: '../remote-client-prototype/assets/nana-avatar.jpg', cg: '../remote-client-prototype/assets/nana-cg.png', mode: 'dark' },
  };
  const avatar = role => `<img class="avatar" src="${roles[role].avatar}" alt="${roles[role].name}的头像" />`;
  const btn = (name, label, attrs = '') => `<button class="icon-button" aria-label="${label}" ${attrs}>${icon(name)}</button>`;
  const named = role => `<div class="speaker">${avatar(role)}<strong>${roles[role].name}</strong><span class="speaker-sign">${icon('bow')}</span><time>21:08</time>${btn('more', '消息操作', 'data-sheet="message"')}</div>`;
  const user = (text, extra = '') => `<article class="user-message"><div>${text}</div><span>你 <i></i>21:07 ${extra}</span></article>`;
  const answer = (role, text, cls = '') => `<article class="answer ${cls}">${named(role)}<div class="answer-bubble">${text}</div></article>`;
  const thinking = (active = false, expanded = false) => `<details class="thought-sign ${active ? 'thinking' : ''}" ${expanded ? 'open' : ''}><summary><span class="thought-icon">${icon('thought')}</span><strong>${active ? '在想啦…' : '模型思考'}</strong><span>${active ? '接收中' : '第 2 轮 · 3.2s'}</span>${icon('down')}</summary><div><small>模型返回的思考记录</small><p>先确认出发时间，再把路线按步行距离排好，留出休息的时间。</p></div></details>`;
  function tool(name, title, detail, state = 'done', child = '') {
    return `<details class="tool-step ${state} ${child ? 'nit-step' : ''}"><summary><span class="tool-node">${icon(name)}</span><span class="tool-text"><strong>${title}${child ? '<b>NIT</b>' : ''}</strong><small>${detail}</small></span><span class="tool-state">${state === 'done' ? icon('check') : state === 'error' ? '失败' : '<i class="working-dot"></i>'}</span>${icon('down')}</summary><div class="tool-open">${child || `<dl><div><dt>调用</dt><dd>${title}</dd></div><div><dt>结果</dt><dd>${state === 'error' ? '请求超时。此记录保留在时间线中，不会自动重复执行。' : '已返回服务端结果，点击下方可查看调用参数与原始返回。'}</dd></div></dl><button class="quiet-button" data-sheet="tool">${icon('script')}参数与返回</button>`}</div></details>`;
  }
  const nitChildren = `<div class="nit-topline">${icon('route')}内部调用 <b>3 项</b><span>2 完成 · 1 进行中</span></div><div class="nit-children">${tool('search', '搜索地点', '公园、咖啡馆 · 420ms')}${tool('file', '读取文件', '周末偏好.md · 18ms')}${tool('edit', '写入计划', 'weekend.md', 'active')}</div><button class="quiet-button" data-sheet="tool">${icon('script')}脚本与原始返回</button>`;
  const documentCard = () => `<button class="document-result" data-sheet="file"><span class="file-spine">MD</span><div><small>角色生成的文件</small><strong>周末散步计划</strong><span>weekend.md · 4.2 KB</span></div>${icon('download')}<i class="fold"></i></button>`;
  const miniTools = () => `<div class="tool-timeline">${tool('search', '搜索地点', '找到 4 个结果 · 420ms')}${tool('edit', '写入计划', 'weekend.md · 26ms')}</div>`;
  const approval = () => `<section class="approval-note"><span class="note-pin">${icon('shield')}</span><div class="note-eyebrow">需要你点个头</div><h3>在远程工作区运行命令</h3><p>请确认服务端提供的命令和范围。</p><code>python render_plan.py</code><dl><div><dt>执行位置</dt><dd>当前会话工作区</dd></div><div><dt>审批类型</dt><dd>终端操作</dd></div></dl><button class="quiet-button" data-sheet="tool">查看操作详情 ${icon('down')}</button><div class="approval-buttons"><button class="soft-button" data-demo="已演示拒绝操作；未发送到服务端">这次不了</button><button class="primary-button" data-demo="已演示单次批准；未发送到服务端">${icon('check')}允许这一次</button></div><small>只批准本次，不改变会话权限。</small></section>`;
  const queueCard = (expanded = false) => `<details class="queue-stack" ${expanded ? 'open' : ''}><summary><span class="queue-tab">${icon('queue')}待发送 <b>2</b></span><span>会按顺序，慢慢来</span>${icon('down')}</summary><div class="queue-paper"><article><span class="queue-number">01</span><div><p>把交通预算也补一下。</p><small>等待上一条完成</small><div class="queue-actions"><button data-demo="设计演示：立即提交前会说明其服务端行为">${icon('send')}立即提交</button><button data-sheet="edit">${icon('edit')}编辑</button><button class="quiet-danger" data-demo="设计演示：取消此条待发送内容">取消</button></div></div><button class="grip" aria-label="调整队列顺序" data-demo="正式交互支持拖动排序，同时保留向前／向后按钮">${icon('more')}</button></article><article><span class="queue-number">02</span><div><p>晚上想找个安静的地方。</p><small>还有 1 条在前面</small></div>${btn('more', '第二条队列操作', 'data-sheet="queue"')}</article><button class="queue-pause" data-demo="设计演示：切换队列暂停状态">${icon('stop')}暂停队列</button></div></details>`;
  // 示例文案与角色集中存放；设计稿不会读取或修改正式应用数据。
  const scenes = [
    { id: 'daily', role: 'pero', title: '01 / 日常聊天', subtitle: '软气泡 · 轻陪伴 · 不抢正文', thread: '今天，慢慢来', kind: '聊天', content: () => `<div class="day-divider"><span></span>今天 · 晚上 9:07<span></span></div>${user('终于忙完了，脑袋有点空空的。')}${answer('pero', '<p>回来啦，今天辛苦你了。</p><p>先把肩膀放松一点点。<span class="text-accent">现在可以慢慢呼吸啦。</span></p>')}<div class="response-finish"><span>21:08</span>${btn('copy', '复制回复', 'data-demo="设计演示：复制这条回复"')}${btn('sound', '朗读回复', 'data-demo="设计演示：开始朗读，可随时停止"')}</div>${user('陪我聊点轻松的吧。')}${answer('pero', '<p>好呀。那我先出一道小题——</p><p>如果明天完全不用赶时间，<br />你最想去哪里待一会儿？</p>', 'new-answer')}<button class="suggestion" data-fill="想去有阳光的小公园。">${icon('sparkle')}想去有阳光的小公园。${icon('plus')}</button>` },
    { id: 'work', role: 'nana', title: '02 / 思考与执行', subtitle: '云签思考 · 开放轨迹 · NIT 层级', thread: '给周末留一点空白', kind: '工作', content: () => `<div class="day-divider"><span></span>今天 · 晚上 9:07<span></span></div>${user('帮我整理一条轻松的周末路线。')}${named('nana')}<div class="standalone-line">好吧，那就别把行程塞得太满。</div>${thinking()}<div class="timeline-caption">${icon('route')}执行轨迹 <span>点击查看细节</span></div><div class="tool-timeline">${tool('globe', '打开网页', '城市公园开放时间 · 310ms')}${tool('script', '运行脚本', '整理路线与文件', 'active', nitChildren)}</div><div class="live-caption"><span class="live-breath"></span>正在执行 · 你可以继续输入${btn('stop', '停止当前回复', 'data-demo="设计演示：发出停止请求后，等待服务端确认"')}</div>` },
    { id: 'approval', role: 'pero', title: '03 / 等你确认', subtitle: '确认便笺 · 明确范围 · 单次授权', thread: '给周末留一点空白', kind: '工作', content: () => `${user('把路线排成一张好看的清单吧。')}${answer('pero', '<p>排版准备好啦。接下来这一步，需要你确认一下。</p>')}${approval()}<div class="waiting-caption">${icon('clock')}正在等你，任务不会悄悄继续。</div>` },
    { id: 'queue', role: 'pero', title: '04 / 队列与结果', subtitle: '叠纸队列 · 文件折角 · 不打断阅读', thread: '给周末留一点空白', kind: '工作', queue: true, content: () => `${answer('pero', '<p>路线整理好啦。上午在公园走走，下午留给咖啡和发呆。</p>')}${thinking()}${miniTools()}${documentCard()}<button class="new-message-jump" data-jump>${icon('down')}有新消息 · 回到底部</button><div class="history-hint">正在查看前面的内容，<br />新回复不会把你拉走。</div>` },
    { id: 'model', role: 'nana', title: '05 / 模型与思考强度', subtitle: '半屏抽屉 · 即选即存 · 当前值清晰', thread: '今天，慢慢来', kind: '聊天', initialSheet: 'models', content: () => `${user('接下来，我们慢慢聊。')}${answer('nana', '<p>嗯哼。今天想聊什么？</p>')}<div class="empty-companion"><img src="${roles.nana.cg}" alt="Nana 的示例角色立绘" /><span>不用着急，<br />我在这里。</span></div>` },
    { id: 'edit', role: 'nana', title: '06 / 消息编辑', subtitle: '操作分层 · 保存与重生成各自明确', thread: '给周末留一点空白', kind: '工作', initialSheet: 'edit', content: () => `${user('周六去散步，下午四点前回来。')}${answer('nana', '<p>下午四点前回来，记住了。</p>')}${thinking()}${miniTools()}` },
  ];
  function composer(scene) {
    return `<div class="composer-wrap">${scene.queue ? queueCard(true) : ''}<form class="composer"><div class="composer-config"><button class="model-chip" type="button" data-sheet="models">${icon('model')}Gemini 4 Pro${icon('down')}</button><button class="effort-chip" type="button" data-sheet="models">${icon('thought')}思考 · 高</button><span class="token-count">12.4k<span> / 128k</span></span></div><div class="input-line">${btn('plus', '添加图片或文件', 'type="button" data-sheet="attachments"')}<textarea rows="1" aria-label="消息输入" placeholder="和 ${roles[scene.role].name} 说说你的想法…"></textarea><button class="send-button" type="submit" aria-label="发送消息">${icon('send')}</button></div><div class="composer-bottom"><button type="button" data-sheet="mode">${icon(scene.kind === '工作' ? 'script' : 'chat')}<b>${scene.kind}</b>${icon('down')}</button><button type="button" data-sheet="tools">${icon('sliders')}工具</button><button type="button" data-sheet="voice">${icon('voice')}语音</button><span>${icon('shield')}infOS</span></div></form><div class="home-indicator"></div></div>`;
  }
  function device(scene) {
    return `<figure class="screen-figure"><figcaption><b>${scene.title}</b><span>${scene.subtitle}</span></figcaption><div class="phone theme-${roles[scene.role].mode}" data-scene="${scene.id}" data-role="${scene.role}" data-default-theme="${roles[scene.role].mode}"><div class="phone-status"><b>9:41</b><div><svg viewBox="0 0 38 14" aria-hidden="true"><path d="M1 12V9m4 3V6m4 6V3m4 9V0M20 4q7-6 14 0m-12 3q5-4 10 0m-7 3q2-2 4 0"/><circle cx="27" cy="12" r=".7"/></svg><span class="battery"><i></i></span></div></div><header class="chat-header">${btn('back', '打开会话列表', 'data-sheet="threads"')}${avatar(scene.role)}<button class="thread-heading" data-sheet="threads"><strong>${roles[scene.role].name}<span class="name-ribbon">${icon('bow')}</span></strong><small><i class="online-dot"></i>${scene.thread}</small></button>${btn('more', '会话设置与操作', 'data-sheet="conversation"')}</header><section class="chat-content">${scene.content()}</section>${composer(scene)}<div class="phone-sheet-host"></div><div class="local-toast" role="status"></div></div></figure>`;
  }
  document.getElementById('screen-grid').innerHTML = scenes.map(device).join('');
  const componentDefinitions = [
    ['A', '对话气泡', '用户是收口的信封角，角色是松软的玻璃气泡。表情留给文字，头像和缎带负责角色感。', `${user('今天想做点轻松的事。')}${answer('pero', '<p>好呀，把今天的节奏，<br />稍微放慢一点点。</p>')}`],
    ['B', '思考云签', '小云朵轮廓和轨迹分开。收起时只留轮次、耗时与状态，正文按需展开。', `${thinking(true)}${thinking(false, true)}`],
    ['C', '工具调用轨迹', '开放式时间线，不再一张张套同款卡片。功能图形、目标、状态各在自己的位置。', `${miniTools()}<div class="tool-timeline">${tool('globe', '打开网页', '连接超时 · 8.0s', 'error')}</div>`],
    ['D', 'NIT 的内部调用', '父任务与内部调用有缩进、连接线和数量汇总。不把多层执行挤成一段文字。', `<div class="tool-timeline">${tool('script', '运行脚本', '3 个内部调用', 'active', nitChildren)}</div>`],
    ['E', '文件结果', '折角、类型书脊和文件名组成可识别的文件物件。点击预览，下载操作分开。', `${documentCard()}<p class="inline-caption">${icon('check')}已写入当前会话工作区</p>`],
    ['F', '等待确认', '蜂蜜色便笺代表“需要你”，确认范围先于按钮。拒绝和单次允许保持清晰层级。', approval()],
    ['G', '待发送队列', '带编号的叠纸便签，与聊天记录明显区分。暂停、编辑和立即提交不混成同一按钮。', queueCard(true)],
    ['H', '按钮与轻状态', '主操作实色、次操作轻框、辅助操作无框。错误用局部提醒，不覆盖已写好的输入。', `<div class="button-samples"><button class="primary-button" data-demo="轻按反馈">${icon('send')}发送消息</button><button class="soft-button" data-sheet="models">${icon('sliders')}调整设置</button><button class="quiet-button" data-demo="内容保持在原位置">${icon('copy')}复制</button><button class="danger-button" data-demo="正式操作需要明确确认">${icon('rollback')}回滚到此处</button><button class="primary-button" disabled>暂不可用</button></div><div class="gentle-error">${icon('refresh')}连接暂时中断，草稿还在。<button data-demo="设计演示：重新连接">重连</button></div><div class="saved-note">${icon('check')}已保存到当前 infOS</div>`],
  ];
  document.getElementById('component-grid').innerHTML = componentDefinitions.map(([id, title, copy, content]) => `<article class="component-study"><header><span>${id}</span><div><h3>${title}</h3><p>${copy}</p></div></header><div class="component-canvas theme-light">${content}<div class="phone-sheet-host"></div></div></article>`).join('');
  document.querySelector('.component-canvas .nit-step')?.setAttribute('open', '');
  document.querySelectorAll('.component-study')[3].querySelector('.nit-step').open = true;
  const labels = ['对话', '思考', '轨迹', '脚本', '搜索', '网页', '文件', '工作区', '编辑', '队列', '时间', '模型', '调节', '发送', '停止', '录音', '朗读', '图片', '附件', '审批', '重试', '回滚', '复制', '下载', '完成', '添加', '关闭', '返回', '展开', '更多', '轻提示', '缎带'];
  document.getElementById('icon-grid').innerHTML = window.DesignIcons.map((name, index) => `<button data-demo="${labels[index]} · 24 格 SVG / 可随主题配色" aria-label="${labels[index]}">${icon(name)}<span>${labels[index]}</span></button>`).join('');
  const motionCards = [
    ['消息出现', '6px 上浮、180ms 淡入；只给新消息。', 'message', `<div class="motion-answer"><i></i><span>我在听，慢慢说。</span>${icon('sparkle')}</div>`],
    ['抽屉进出', '240ms 贴地升起，背景仅轻微暗下。', 'sheet', `<div class="motion-sheet"><i></i><strong>选择思考强度</strong><div><span>低</span><span>中</span><span class="selected">高 ${icon('check')}</span></div></div>`],
    ['状态交接', '完成后圆点落定为勾，保留原位置。', 'done', `<div class="motion-task">${icon('file')}<span>整理文件</span><i class="state-orb"></i></div><div class="motion-result">${icon('check')}文件已写入</div>`],
    ['页面切换', '160ms 小幅横移，保留顶部与输入锚点。', 'page', `<div class="motion-pages"><div><span>对话</span><p>有想法，就说出来。</p>${icon('chat')}</div><div><span>会话</span><p>接着聊，还是新话题？</p>${icon('queue')}</div></div>`],
  ];
  document.getElementById('motion-grid').innerHTML = motionCards.map(([title, copy, id, html]) => `<article class="motion-card"><div class="motion-stage" data-motion="${id}">${html}</div><div><h3>${title}</h3><p>${copy}</p><button class="soft-button" data-replay="${id}">${icon('refresh')}播放一次</button></div></article>`).join('');
  let toastTimer;
  function toast(text, parent = null) {
    const target = parent?.querySelector('.local-toast') || document.getElementById('toast');
    target.textContent = text; target.classList.add('visible'); clearTimeout(toastTimer); toastTimer = setTimeout(() => target.classList.remove('visible'), 2600);
  }
  function sheetBody(type, parent) {
    const role = parent.dataset.role || 'pero';
    const title = { models: '这次，怎么思考？', message: '这条消息', edit: '编辑这条消息', queue: '待发送的消息', attachments: '加一点内容', tools: '工具与执行', voice: '说给我听', threads: '接着哪段对话？', conversation: '这个会话', file: '周末散步计划', tool: '调用详情', mode: '这次怎么聊？', rewind: '回滚并重新生成？' }[type] || '设计预览';
    let body = '';
    if (type === 'models') body = `<p class="sheet-intro">只调整当前会话。选择后保存，不影响其他会话。</p><h4>当前模型</h4><button class="model-option selected" data-demo="设计演示：当前会话使用 Gemini 4 Pro">${icon('model')}<span><strong>Gemini 4 Pro</strong><small>Google · 视觉输入</small></span><i>${icon('check')}</i></button><button class="model-option" data-demo="设计演示：已选择跟随默认模型">${icon('refresh')}<span><strong>跟随默认模型</strong><small>使用 infOS 的默认配置</small></span><i></i></button><h4>思考强度</h4><div class="effort-grid">${['跟随模型', '低', '中', '高'].map(label => `<button class="${label === '高' ? 'selected' : ''}" data-effort="${label}">${label}${label === '高' ? icon('check') : ''}</button>`).join('')}</div><p class="sheet-hint">${icon('thought')}选项随模型能力变化，不显示无效档位。</p><div class="sheet-save">${icon('check')}当前配置已同步</div>`;
    else if (type === 'edit') body = `<p class="sheet-intro">正文先改好，再决定是否重新生成。</p><label class="edit-label">消息正文<textarea rows="4">周六去散步，下午三点前回来。</textarea></label><div class="edit-choice"><button class="soft-button" data-demo="设计演示：仅保存正文，后续记录保留">${icon('check')}仅保存</button><p>保留后面的对话。</p></div><div class="edit-choice regenerate"><button class="primary-button" data-sheet="rewind">${icon('refresh')}保存并重新生成</button><p>先保存修订，再确认本轮及后续对话的回滚范围。</p></div><p class="sheet-hint">修改未保存时，关闭会先询问。</p>`;
    else if (type === 'rewind') body = `<div class="rewind-count"><b>2<small>轮对话将撤回</small></b><b>1<small>个文件将恢复</small></b></div><div class="rewind-file">${icon('refresh')}weekend.md <span>恢复到本轮之前</span></div><p class="sheet-intro">使用修订后的原输入重新生成。外部操作不保证可撤销，正式执行前会重新核对服务端清单。</p><button class="primary-button full" data-demo="设计演示：确认后由 Rust 执行回滚与重新提交">${icon('refresh')}确认回滚并重新生成</button>`;
    else if (type === 'message') body = `<div class="action-list"><button data-demo="设计演示：复制这条消息">${icon('copy')}<span>复制正文<small>带走这段话</small></span></button><button data-sheet="edit">${icon('edit')}<span>编辑消息<small>仅保存，或保存后重新生成</small></span></button><button class="quiet-danger" data-sheet="rewind">${icon('rollback')}<span>回滚到此处<small>先查看对话与文件的影响范围</small></span></button></div>`;
    else if (type === 'queue') body = `${queueCard(true)}<p class="sheet-hint">排序只影响尚未发送的条目；当前执行不被悄悄替换。</p>`;
    else if (type === 'file') body = `<div class="file-preview"><span>周末散步计划</span><h3>留一点时间，<br />慢慢走。</h3><p>10:00　城市公园<br />12:00　午饭与休息<br />14:00　咖啡馆<br />15:00　回家</p></div><button class="primary-button full" data-demo="设计演示：下载交给系统保存入口">${icon('download')}保存文件</button>`;
    else if (type === 'tool') body = `<div class="call-summary">${icon('script')}运行脚本<span class="status-tag">进行中</span></div><h4>调用参数</h4><pre class="code-preview">const places = await search_places({
  query: "城市公园 咖啡馆"
});
await write_file("weekend.md", plan);</pre><h4>服务端记录</h4><p class="sheet-intro">内部调用 3 项；已完成 2 项，正在写入 1 项。</p><p class="sheet-hint">这里只展示真实返回的状态、参数和结果；不伪造进度或自动生成任务说明。</p>`;
    else if (type === 'attachments') body = `<div class="attachment-choices"><button data-demo="设计演示：系统选择图片">${icon('photo')}图片</button><button data-demo="设计演示：系统选择文件">${icon('attach')}文件</button><button data-demo="设计演示：打开当前会话工作区">${icon('folder')}工作区引用</button></div><p class="sheet-hint">添加后显示在输入框上方，可随时移除。</p>`;
    else if (type === 'voice') body = `<div class="voice-orbit">${icon('voice')}<i></i><i></i></div><p class="voice-title">说完后，先看看文字。</p><p class="sheet-intro">转写确认后加入输入框，不直接发送。</p><button class="primary-button full" data-demo="仅设计演示，不会开启麦克风">${icon('voice')}开始录音</button>`;
    else if (type === 'mode') body = `<div class="action-list"><button data-demo="设计演示：聊天模式">${icon('chat')}<span>聊天<small>把交流放在前面</small></span></button><button data-demo="设计演示：工作模式">${icon('script')}<span>工作<small>需要工具时展示完整轨迹</small></span></button></div>`;
    else if (type === 'threads') body = `<div class="thread-role">${avatar(role)}<strong>${roles[role].name}</strong>${icon('bow')}</div><div class="action-list"><button data-close-sheet>${icon('chat')}<span>给周末留一点空白<small>刚刚 · 当前会话</small></span>${icon('check')}</button><button data-close-sheet>${icon('chat')}<span>今天，慢慢来<small>今天 20:18</small></span></button><button data-demo="设计演示：新建会话">${icon('plus')}<span>开始新话题</span></button></div>`;
    else if (type === 'tools') body = `<div class="sheet-tabs"><button class="selected">工具</button><button data-demo="执行方式：手动、自动、自动审批、完整权限">执行</button><button data-demo="心流、Token 预算与会话导出">上下文</button></div><p class="sheet-intro">当前会话的工具。系统锁定项会明确标记。</p>${[['网页与搜索', true], ['工作区文件', true], ['远程终端', false]].map(([name, enabled]) => `<label class="switch-row"><span>${name}</span><input type="checkbox" ${enabled ? 'checked' : ''} /><i></i></label>`).join('')}<button class="primary-button full" data-demo="设计演示：工具选择已保存">保存工具选择</button>`;
    else body = `<div class="action-list"><button data-sheet="models">${icon('model')}<span>模型与思考强度</span></button><button data-sheet="tools">${icon('sliders')}<span>工具与执行</span></button><button data-demo="设计演示：打开工作台">${icon('folder')}<span>工作区与文件</span></button><button data-sheet="threads">${icon('chat')}<span>会话列表</span></button><button data-demo="设计演示：会话导出">${icon('download')}<span>导出记录</span></button></div>`;
    return `<div class="sheet-dimmer" data-close-sheet></div><section class="phone-sheet" role="dialog" aria-label="${title}"><div class="sheet-grabber"></div><header><div><small>${roles[role].name} · 当前会话</small><h3>${title}</h3></div>${btn('close', '关闭面板', 'data-close-sheet')}</header><div class="sheet-body">${body}</div></section>`;
  }
  function openSheet(parent, type) {
    if (!parent) { toast('请在「对话全景」中体验完整抽屉。'); return; }
    const host = parent.querySelector('.phone-sheet-host');
    host.innerHTML = sheetBody(type, parent); host.dataset.sheet = type; host.classList.add('is-open');
  }
  document.addEventListener('click', event => {
    const target = event.target.closest('button,[data-close-sheet]'); if (!target) return;
    const phone = target.closest('.phone');
    if (target.dataset.view) {
      document.querySelectorAll('.studio-view').forEach(view => view.classList.toggle('active', view.id === target.dataset.view));
      document.querySelectorAll('[data-view]').forEach(tab => tab.classList.toggle('active', tab === target));
      document.querySelector('.studio-nav').scrollIntoView({ block: 'start', behavior: root.dataset.reduced === 'true' ? 'instant' : 'smooth' });
    }
    if (target.dataset.boardTheme) {
      document.querySelectorAll('[data-board-theme]').forEach(tab => tab.classList.toggle('active', tab === target));
      document.querySelectorAll('.phone').forEach(screen => { screen.classList.remove('theme-light', 'theme-dark'); screen.classList.add(`theme-${target.dataset.boardTheme === 'standard' ? screen.dataset.defaultTheme : target.dataset.boardTheme}`); });
    }
    if (target.dataset.kitTheme) {
      document.querySelectorAll('[data-kit-theme]').forEach(tab => tab.classList.toggle('active', tab === target));
      document.querySelectorAll('.component-canvas').forEach(canvas => { canvas.classList.remove('theme-light', 'theme-dark'); canvas.classList.add(`theme-${target.dataset.kitTheme}`); });
    }
    if (target.hasAttribute('data-close-sheet')) { phone?.querySelector('.phone-sheet-host')?.classList.remove('is-open'); }
    else if (target.dataset.sheet) openSheet(phone, target.dataset.sheet);
    if (target.dataset.demo) { if (target.classList.contains('model-option')) { target.parentElement.querySelectorAll('.model-option').forEach(option => option.classList.toggle('selected', option === target)); }; toast(target.dataset.demo, phone); }
    if (target.dataset.fill) { const input = phone.querySelector('.composer textarea'); input.value = target.dataset.fill; input.focus(); }
    if (target.dataset.effort) { target.parentElement.querySelectorAll('button').forEach(button => button.classList.toggle('selected', button === target)); if (phone) phone.querySelector('.effort-chip').innerHTML = `${icon('thought')}思考 · ${target.dataset.effort}`; toast(`设计演示：思考强度已选「${target.dataset.effort}」`, phone); }
    if (target.hasAttribute('data-jump')) { phone.querySelector('.chat-content').scrollTo({ top: 10000, behavior: root.dataset.reduced === 'true' ? 'instant' : 'smooth' }); target.remove(); }
    if (target.dataset.replay) { const stage = document.querySelector(`[data-motion="${target.dataset.replay}"]`); stage.classList.remove('playing'); requestAnimationFrame(() => requestAnimationFrame(() => stage.classList.add('playing'))); }
    if (target.id === 'reduce-motion') { const reduced = root.dataset.reduced !== 'true'; root.dataset.reduced = String(reduced); target.setAttribute('aria-pressed', String(reduced)); target.textContent = reduced ? '减少动态 · 已开启' : '减少动态'; }
  });
  document.addEventListener('submit', event => {
    if (!event.target.classList.contains('composer')) return; event.preventDefault();
    const input = event.target.querySelector('textarea'), phone = event.target.closest('.phone');
    if (!input.value.trim()) { toast('先写一点想说的话吧。', phone); input.focus(); return; }
    const text = input.value.trim(), article = document.createElement('article'); article.className = 'user-message just-added';
    const bubble = document.createElement('div'); bubble.textContent = text; const meta = document.createElement('span'); meta.textContent = '你 · 设计演示'; article.append(bubble, meta);
    const content = phone.querySelector('.chat-content'); content.append(article); input.value = ''; content.scrollTo({ top: content.scrollHeight, behavior: root.dataset.reduced === 'true' ? 'instant' : 'smooth' }); toast('只加入设计演示，未发送到 infOS。', phone);
  });
  document.addEventListener('keydown', event => { if (event.key === 'Escape') document.querySelectorAll('.phone-sheet-host.is-open').forEach(host => host.classList.remove('is-open')); });
  scenes.filter(scene => scene.initialSheet).forEach(scene => openSheet(document.querySelector(`[data-scene="${scene.id}"]`), scene.initialSheet));
})();
