/* 聊天信息流的独立设计演示；思考摘录、工具记录和回复均为示例。 */
(() => {
  const P = window.PPC, S = P.state, { icon:i, avatar:av, btn, esc } = P;
  const folds = new Map(), animations = new Map();
  const motionPreference = matchMedia('(prefers-reduced-motion: reduce)');
  let demoTimer, demoVersion = 0, demoStage = 'done', context;
  const reduced = () => S.reduced || motionPreference.matches;
  const role = () => S.chatRole === 'nana' ? 'nana' : 'pero';
  const speaker = () => `<div class="speaker">${av(role())}<strong>${role() === 'nana' ? 'Nana' : 'Pero'}</strong>${i('bow')}<time>09:33</time>${btn('more','消息操作','message-menu')}</div>`;

  // 每条折叠记录都有自己的标识，切屏后保留阅读状态。
  function fold(key, cls, heading, body, open = false) {
    const expanded = folds.has(key) ? folds.get(key) : open;
    return `<details class="flow-fold ${cls}" data-flow-fold="${key}" data-expanded="${expanded}" ${expanded ? 'open' : ''}>
      <summary aria-expanded="${expanded}">${heading}<span class="flow-chevron">${i('down')}</span></summary>
      <div class="flow-panel"><div class="flow-panel-inner">${body}</div></div>
    </details>`;
  }

  const pulse = () => '<span class="flow-pulse" aria-hidden="true"><b></b><b></b><b></b></span>';
  function thought(key, running = false, elapsed = '6.2 秒') {
    const effort = key === 'stream' ? esc(S.effort) : '高';
    return fold(`${key}-thought`, `flow-thought ${running ? 'is-running' : ''}`,
      `<span class="flow-symbol">${i('thought')}</span><strong>${running ? '正在想' : '想好了'}</strong>${running ? pulse() : ''}<span class="flow-meta">${running ? '已等待' : `${effort} ·`} <time>${elapsed}</time></span>`,
      `<div class="flow-reasoning"><p>先看看公园几点开放，路上大约要走多久。</p><p>下午不用排得太满，给休息和看日落留一点时间。</p></div><div class="flow-footnote">${i(running ? 'clock' : 'check')}${running ? '思考还在继续' : '思考结束'}</div>`);
  }

  function toolStep(key, name, symbol, state, duration, body) {
    const label = state === 'running' ? '进行中' : state === 'failed' ? '未完成' : state === 'pending' ? '待开始' : '完成';
    return `<li class="flow-step" data-state="${state}">${fold(key, 'flow-tool',
      `<span class="flow-step-icon">${i(symbol)}</span><strong>${name}</strong><span class="flow-step-meta">${duration || label}</span><span class="flow-step-state" aria-label="${label}">${state === 'running' ? '<span class="flow-spinner"></span>' : i(state === 'failed' ? 'offline' : state === 'pending' ? 'clock' : 'check')}</span>`, body)}</li>`;
  }

  function execution(key, stage = 'done') {
    const running = stage === 'tools', failed = stage === 'failed';
    const headline = failed ? '路线已查好，文档未保存' : running ? '正在整理路线' : '查了路线，存好了文档';
    const searchBody = `<dl class="flow-tool-data"><div><dt>查找</dt><dd>滨江公园的开放时间与步行路线</dd></div><div><dt>找到</dt><dd>公园全天开放，沿河步行约 25 分钟。</dd></div></dl>
      ${fold(`${key}-substeps`, 'flow-substeps', `<strong>查看子步骤</strong><span class="flow-meta">2 项</span>`, '<ol><li><span>核对开放时间</span><span>0.9 秒</span></li><li><span>整理步行路线</span><span>1.4 秒</span></li></ol>')}`;
    const fileBody = failed
      ? `<p class="flow-tool-note">连接中断了，已查到的路线还在。</p><button class="flow-retry" data-action="flow-retry">${i('refresh')}再试一次</button>`
      : `<dl class="flow-tool-data"><div><dt>${running ? '正在写入' : '已保存'}</dt><dd>周末路线.md</dd></div><div><dt>内容</dt><dd>沿途地点、步行时间与休息安排。</dd></div></dl>${running ? '<p class="flow-tool-note">写好后会出现在回复里。</p>' : '<button class="flow-result-link" data-go="file-detail">打开文档 ' + i('next') + '</button>'}`;
    return fold(`${key}-tools`, `flow-execution ${running ? 'is-running' : ''} ${failed ? 'has-error' : ''}`,
      `<span class="flow-symbol">${running ? '<span class="flow-spinner" aria-hidden="true"></span>' : i(failed ? 'offline' : 'route')}</span><strong>${headline}</strong><span class="flow-meta">${running ? '1 / 2' : failed ? '1 项未完成' : '2 项'}</span>`,
      `<ol class="flow-steps">${toolStep(`${key}-search`,'查找开放时间','search','done','2.3 秒',searchBody)}${toolStep(`${key}-file`,'整理路线文档','file',failed ? 'failed' : running ? 'running' : 'done',running ? '' : failed ? '' : '0.8 秒',fileBody)}</ol>${failed ? '<div class="flow-error-action"><span>路线还在，可以继续保存。</span><button data-action="flow-retry">重试 ' + i('refresh') + '</button></div>' : ''}`, failed);
  }

  const process = (key, stage = 'done') => `<div class="flow-process">${thought(key, stage === 'thinking', stage === 'thinking' ? '3 秒' : '6.2 秒')}${stage === 'thinking' ? '' : execution(key, stage)}</div>`;
  const file = () => `<button class="flow-file" data-go="file-detail"><span class="flow-file-cover">${i('file')}<small>MD</small></span><span><strong>周末路线.md</strong><small>路线与停留地点 · 2 KB</small></span><span class="flow-file-open">${i('next')}</span></button>`;
  const reply = (arrival = false) => `<article class="reply-bubble rich flow-rich ${arrival ? 'flow-arrival' : ''}">
    <p>那就给周末留一点空白吧。</p><p class="expression">（把地图轻轻推到你面前）</p>
    <h3><span class="flow-heading-icon">${i('sun')}</span>一个慢悠悠的下午</h3>
    <ul class="flow-itinerary"><li><time>15:00</time><span>去河边散步，<strong>不用赶路。</strong></span></li><li><time>16:30</time><span>找家小店，喝点喜欢的。</span></li></ul>
    <blockquote><p>最后一站，留给日落。</p><small>走到喜欢的地方，就多待一会儿。</small></blockquote>
    <p>开放时间也核对过啦。<button class="flow-inline-link" data-action="flow-source">公园信息 ${i('next')}</button></p>${file()}
  </article>`;
  const messageActions = () => `<div class="message-actions"><small>09:34</small>${btn('copy','复制回复','copy')}${btn('audio','朗读回复','speak')}</div>`;

  P.screens.chat = () => `<div class="chat-flow"><div class="date-divider">今天 · 09:32</div><article class="user-message"><div>周末想出门走走，别太赶。</div><small>你 · 09:32</small></article>${speaker()}${process('chat')}${reply()}${messageActions()}${S.sent.map(text => `<article class="user-message"><div>${esc(text)}</div><small>你 · 刚刚</small></article>`).join('')}<div id="stream-reply"></div></div>`;
  P.flowWaiting = elapsed => `<div class="chat-flow">${thought('stream', true, `${elapsed} 秒`)}</div>`;
  P.flowStreamStart = (elapsed = 2) => `<div class="chat-flow">${speaker()}${thought('stream', false, `${elapsed} 秒`)}<article class="reply-bubble flow-rich flow-arrival is-streaming"><p class="flow-stream-text"></p></article></div>`;

  function preview() {
    return `${process('preview', demoStage)}${demoStage === 'done' ? reply(true) : `<div class="flow-live-caption" role="status">${i(demoStage === 'failed' ? 'offline' : 'spark')}<span>${demoStage === 'thinking' ? '先把思路理一理…' : demoStage === 'tools' ? '查到的内容，正在整理。' : '这一步没完成，可以重试。'}</span></div>`}`;
  }

  const richSpecimen = () => `<article class="reply-bubble rich flow-rich flow-specimen">
    <span class="flow-eyebrow">写给这个周末</span><h2>把时间，留给喜欢的事。</h2><p>沿着河边慢慢走，<strong>不用赶时间</strong>。<em>累了就坐一会儿</em>，第一段路大约 <code>25 分钟</code>。</p>
    <h3><span class="flow-heading-icon">${i('route')}</span>出门前的小清单</h3><ul class="flow-checklist"><li>${i('check')}带一瓶水</li><li>${i('check')}穿舒服的鞋</li><li class="unchecked">${i('clock')}出门前再看一眼天气</li></ul>
    <blockquote><p>不用把一天填满，也可以很开心。</p><small>慢一点，反而会遇见更多。</small></blockquote>
    <div class="flow-table-scroll" tabindex="0" role="region" aria-label="散步安排，可横向滚动"><table><caption>沿途的两次停留</caption><thead><tr><th scope="col">时间</th><th scope="col">去哪里</th><th scope="col">留多久</th></tr></thead><tbody><tr><td>15:00</td><td>河边步道</td><td>40 分钟</td></tr><tr><td>16:30</td><td>街角小店</td><td>随你喜欢</td></tr></tbody></table></div>
    <figure class="flow-code"><figcaption><span>${i('code')}JSON</span><button data-action="flow-copy-code" aria-label="复制代码">${i('copy')}复制</button></figcaption><pre tabindex="0" aria-label="路线配置代码"><code>{
  <span class="code-key">"出发时间"</span>: <span class="code-string">"15:00"</span>,
  <span class="code-key">"步行分钟"</span>: <span class="code-number">25</span>
}</code></pre></figure>
    <p class="expression">（把这份小计划折好，放进你的口袋）</p>${file()}
  </article>`;

  P.pages['chat-flow'] = {id:'chat-flow',title:'聊天信息流',tab:'threads',parent:'chat',icon:'chat',heading:'正文慢慢读，过程轻轻收',note:'思考与工具默认收成短行，输入、结果和子步骤按需展开。试试状态切换、折叠、流式回复，以及深浅色和自定义背景。'};
  P.groups.push(['聊天信息流专题',['chat-flow']]);
  P.screens['chat-flow'] = () => `<div class="chat-flow flow-study"><div class="flow-study-heading"><span>${i('thought')}${i('route')}</span><h2>留更多位置，给想说的话。</h2><p>思考轻一点，过程收一点。</p></div>
    <div class="flow-demo-controls" aria-label="信息流演示状态">${[['thinking','思考中'],['tools','用工具'],['done','已完成'],['failed','未完成']].map(([state,label]) => `<button data-action="flow-stage" data-stage="${state}" aria-pressed="${demoStage === state}">${label}</button>`).join('')}</div>
    <div class="flow-demo-toolbar"><small>示例状态 · 可展开查看</small><button data-action="flow-replay">${i('play')}播放过程</button></div><div data-flow-preview>${preview()}</div>
    <div class="flow-section-heading"><span>${i('file')}</span><div><h3>正文，保留自己的节奏。</h3><p>标题、引用、表格和代码各有层次。</p></div></div>${richSpecimen()}<button class="secondary-btn full" data-go="chat">${i('chat')}回到聊天里看看</button></div>`;

  function stopDemo() { clearTimeout(demoTimer); demoVersion++; }
  function updateDemo() {
    const root = document.querySelector('#app [data-flow-preview]');
    if (!root) return;
    finishAnimations(root);
    root.innerHTML = preview();
    document.querySelectorAll('#app [data-stage]').forEach(button => button.setAttribute('aria-pressed', String(button.dataset.stage === demoStage)));
  }
  function replay() {
    stopDemo();
    if (S.page !== 'chat-flow') context.navigate('chat-flow');
    demoStage = 'thinking'; updateDemo();
    const version = demoVersion;
    demoTimer = setTimeout(() => {
      if (version !== demoVersion) return;
      demoStage = 'tools'; updateDemo();
      demoTimer = setTimeout(() => {
        if (version !== demoVersion) return;
        demoStage = 'done'; updateDemo();
      }, 2000);
    }, 1700);
  }

  function retry() {
    stopDemo();
    if (S.page !== 'chat-flow') context.navigate('chat-flow');
    demoStage = 'tools'; updateDemo();
    const version = demoVersion;
    demoTimer = setTimeout(() => {
      if (version !== demoVersion) return;
      demoStage = 'done'; updateDemo();
    }, 1500);
  }

  // 用内容实际高度过渡；连续点击以当前高度接续，减少动态时直接完成。
  function finishAnimations(root = document) {
    for (const [details, record] of animations) {
      if (root.contains(details)) record.finish();
    }
  }
  document.addEventListener('click', event => {
    const summary = event.target.closest('summary');
    const details = summary?.parentElement;
    if (!details?.matches('#app details[data-flow-fold]')) return;
    event.preventDefault();
    const panel = details.querySelector(':scope > .flow-panel');
    const expanded = details.dataset.expanded !== 'true';
    const height = details.open ? panel.getBoundingClientRect().height : 0;
    animations.get(details)?.animation.cancel();
    animations.delete(details);
    folds.set(details.dataset.flowFold, expanded);
    details.dataset.expanded = String(expanded);
    summary.setAttribute('aria-expanded', String(expanded));
    details.open = true;
    panel.inert = !expanded;
    const finish = () => { details.open = expanded; panel.inert = !expanded; animations.delete(details); };
    if (reduced() || typeof panel.animate !== 'function') { finish(); return; }
    const animation = panel.animate([{height:`${height}px`,opacity:expanded ? .45 : 1},{height:expanded ? `${panel.scrollHeight}px` : '0px',opacity:expanded ? 1 : 0}],{duration:expanded ? 230 : 180,easing:'cubic-bezier(.2,.75,.25,1)'});
    const settle = () => { animation.cancel(); finish(); };
    animations.set(details,{animation,finish:settle});
    animation.onfinish = settle;
  });
  P.flowBeforeRender = () => { stopDemo(); finishAnimations(); };
  P.finishFlowAnimations = finishAnimations;
  P.flowMotionChanged = () => { if (reduced()) finishAnimations(); };
  motionPreference.addEventListener('change', P.flowMotionChanged);
  window.addEventListener('pagehide', () => { stopDemo(); finishAnimations(); });
  document.addEventListener('visibilitychange', () => { if (document.hidden) { stopDemo(); finishAnimations(); } });
  P.flowActions = api => {
    context = api;
    return {
      'flow-stage': el => { stopDemo(); demoStage = el.dataset.stage; updateDemo(); },
      'flow-replay': replay,
      'flow-retry': retry,
      'flow-source': () => api.showSheet('公园信息', `<div class="flow-source-card">${i('globe')}<div><strong>滨江公园 · 开放信息</strong><p>示例来源：全天开放，沿河步道约 25 分钟。</p><small>设计稿演示来源预览，不访问外部网站。</small></div></div>`, '<button class="primary-btn" data-action="close-sheet">知道了</button>', 'globe'),
      'flow-copy-code': async el => {
        const code = el.closest('.flow-code')?.querySelector('code')?.textContent;
        if (!code) return;
        try { await navigator.clipboard.writeText(code); api.toast('代码已复制'); }
        catch { api.toast('暂时无法复制，可以选中文字后复制',{tone:'amber'}); }
      }
    };
  };
})();
