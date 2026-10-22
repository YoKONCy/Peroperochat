/* 独立设计稿：不读取应用存储，不调用模型，也不接入服务端。 */
(() => {
  const { packs, palette } = globalThis.Chromatic;
  const state = { pack: 'sky', dark: false, reduced: false, level: 'high' };
  const levels = [
    ['off', '关闭', '直接回应', 'blue', '静态 · 一笔小云'],
    ['auto', '自动', '沿用模型设置', 'primary', '轻呼吸 · 留白小环'],
    ['low', '低', '轻轻想一下', 'mint', '4.8 秒 · 一颗小芽'],
    ['medium', '中', '多想一小步', 'blue', '4 秒 · 双叶小芽'],
    ['high', '高', '仔细想一想', 'violet', '3.8 秒 · 一圈星轨'],
    ['xhigh', '极高', '再多推敲一点', 'rose', '3.2 秒 · 双层星轨'],
    ['max', '最高', '充分展开思考', 'amber', '2.8 秒 · 小小星系'],
  ];
  const icon = name => window.designIcon(name);
  const escape = value => String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
  const avatar = id => `<img class="avatar" src="../../../public/avatars/${id}.jpg" alt="${id === 'nana' ? 'Nana' : 'Pero'}" />`;
  function art(id) {
    const cloud = '<path class="accent-fill" d="M7 23a6 6 0 0 1-1-12A9 9 0 0 1 23 10a7 7 0 0 1 1 13Z"/><path d="M7 23a6 6 0 0 1-1-12A9 9 0 0 1 23 10a7 7 0 0 1 1 13Z"/>';
    const star = '<path class="core" d="m16 9 2 5 5 2-5 2-2 5-2-5-5-2 5-2Z"/>';
    const spark = '<g class="spark"><path d="m26 3 1.2 3.8L31 8l-3.8 1.2L26 13l-1.2-3.8L21 8l3.8-1.2Z"/><circle cx="5" cy="26" r="1" fill="currentColor" stroke="none"/></g>';
    let paths = id === 'off' ? cloud + '<path d="M12 16h8"/>' : id === 'auto' ? cloud + '<path class="core" d="M12 17a4 4 0 1 1 7 2m0-4v4h-4"/>' : id === 'low' || id === 'medium' ? `<path class="accent-fill" d="M16 24C5 24 5 13 7 9c6 0 10 7 9 15Z"/><path class="core" d="M16 27V16M16 24C5 24 5 13 7 9c6 0 10 7 9 15Z${id === 'medium' ? 'M16 19C16 10 21 6 27 5c1 8-5 14-11 14Z' : ''}"/><circle cx="25" cy="23" r="1"/>` : `<circle class="accent-fill" cx="16" cy="16" r="13"/>${star}<g class="orbit"><ellipse cx="16" cy="16" rx="14" ry="9" transform="rotate(-35 16 16)" stroke-opacity=".55"/>${id !== 'high' ? '<ellipse cx="16" cy="16" rx="14" ry="9" transform="rotate(35 16 16)" stroke-opacity=".35"/>' : ''}</g>${spark}${id === 'max' ? '<circle cx="7" cy="5" r="1.6" fill="currentColor" stroke="none"/><circle cx="26" cy="27" r="1.4" fill="currentColor" stroke="none"/>' : ''}`;
    return `<svg class="effort-art" viewBox="0 0 32 32" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths}</svg>`;
  }
  function themeVars(c) { return Object.entries(c).map(([key, value]) => `--${key}:${value}`).join(';'); }
  function levelVars(id, c) { const item = levels.find(item => item[0] === id) || levels[1], tone = item[3]; return `--level:${c[tone]};--levelSoft:${c[tone+'Soft']};--levelEdge:${c[tone+'Edge']}`; }
  const head = (id, sub='给今天留一点空白') => `<div class="status">9:41<span>${icon('signal')}${icon('wifi')}${icon('battery')}</span></div><header class="head">${icon('back')}${avatar(id)}<div><strong>${id === 'nana' ? 'Nana' : 'Pero'}${icon('bow')}</strong><small>${sub}</small></div><button class="icon-button" aria-label="会话菜单">${icon('more')}</button></header>`;
  const person = id => `<div class="person">${avatar(id)}<strong>${id === 'nana' ? 'Nana' : 'Pero'}</strong>${icon('bow')}<time>21:08</time>${icon('more')}</div>`;
  const user = text => `<div class="user"><p>${text}</p><small>你 · 21:07</small></div>`;
  function aside(text) { return `<span class="aside-text"><span class="paren">（</span>${text}<span class="paren">）</span></span>`; }
  function composer(c, level='high', name='Pero') { const selected = levels.find(item => item[0] === level); return `<div class="composer"><div class="composer-meta"><span class="model">${icon('star')}Gemini 4 Pro${icon('down')}</span><button class="effort-chip" style="${levelVars(level,c)}" data-choose-level>${art(level)}思考 · ${selected[1]}${icon('down')}</button><small>12.4k / 128k</small></div><div class="composer-input">${icon('plus')}<span>和 ${name} 说说你的想法…</span><button class="send" aria-label="发送">${icon('send')}</button></div><div class="composer-bottom"><span>${icon('chat')}聊天</span><span>${icon('sliders')}工具</span><span>${icon('audio')}语音</span><span>${icon('shield')}infOS</span></div></div><span class="home-bar"></span>`; }
  function thought(c,id,live=false,open=false) { if (id === 'off') return '<div class="sample-note">已关闭额外思考，直接回应。</div>'; const label = levels.find(item=>item[0]===id)[1]; return `<details class="thought ${live && id !== 'off' ? 'is-running' : ''}" data-level="${id}" style="${levelVars(id,c)}" ${open?'open':''}><summary>${art(id)}<strong>模型思考 · ${label}</strong><span>${live?'思考中':'第 2 轮 · 3.2s'}</span>${icon('down')}</summary><p>${live?'先把你的偏好放在前面，再看看路线和时间能不能自然地串起来。':'先确认通勤时间，再把散步和晚饭排得松一点。只保留两个地点，就不用急着赶路。'}</p>${live?'<div class="thinking-state"><i></i><i></i><i></i> 正在想，也可以继续输入</div>':''}</details>`; }
  function first(c) { return `${head('pero')}<div class="messages"><div class="date">今天 · 晚上 9:07</div>${user('今天忙得脑袋空空，陪我坐一会儿吧。')}${person('pero')}<article class="speech"><p>${aside('挪过来一点，把靠垫放在你身边')}<br/>回来啦。今天已经做得很好了。</p><p>先不急着说话，${aside('轻轻晃了晃脚尖')}<br/>就这样陪你待一会儿，好不好？</p><p>想听故事的话，<strong>我也准备好啦。</strong></p></article><div class="message-foot">21:08${icon('copy')}${icon('volume')}</div>${user('那就讲个轻松的小故事。')}<div class="sample-note">括号像旁白，台词仍是视觉主角。<br/>不缩成难以阅读的小字，不给每句话加厚底框。</div></div>${composer(c)}`; }
  function second(c) { return `${head('pero','聊天里的富文本，也留住小表情')}<div class="messages">${person('pero')}<article class="speech"><p>${aside('认真翻开小本子')}<br/><strong>明天的小计划</strong>，就先留这两件事。</p><h3>01 / 去看一小会儿海</h3><p>坐车到海边，慢慢走到灯塔。${aside('如果下雨，就在窗边听雨，也很好')}</p><h3>02 / 留一点甜甜的时间</h3><p>傍晚回家做松饼，${aside('多加一点<em>你喜欢的蜂蜜</em>')}，然后一起看<a href="#rules">上次没看完的电影</a>。</p><p><code>fn(x)</code> 和 <code>calc(100% - 8px)</code> 仍是代码。</p></article><div class="stream-controls"><button id="replay">${icon('refresh')}重播流式</button><small id="stream-status">闭合后，旁白轻轻显现</small></div><div class="speech stream-text" id="stream-demo">先把肩膀放松一点。${aside('把热茶推到你手边')} 余下的事情，晚一点再说吧。</div></div>${composer(c,'medium')}`; }
  function selector(c) { return `<section class="selector"><i class="sheet-handle"></i><div class="selector-title"><div><h3>这次，要想多深？</h3><small>思考强度</small></div><button class="icon-button" data-close-sheet aria-label="收起">${icon('close')}</button></div>${levels.map(([id,label,description])=>`<button class="row ${state.level===id?'selected':''}" data-level="${id}" style="${levelVars(id,c)}" aria-pressed="${state.level===id}">${art(id)}<div><strong>${label}</strong><small>${description}</small></div>${state.level===id?icon('check'):''}</button>`).join('')}<p>只显示当前模型支持的选项；自动会沿用模型设置。</p></section>`; }
  function third(c) { return `${head('pero')}<div class="messages">${user('帮我安排一条轻松的周末路线。')}${person('pero')}${thought(c,state.level,true,true)}<div class="speech">我来把路线排得松一点。${aside('先把你的休息时间圈出来')}</div></div>${composer(c,state.level)}${selector(c)}`; }
  function fourth(c) { return `${head('nana','把周末留一点给自己')}<div class="messages">${user('不要排得太满，去两个地方就好。')}${person('nana')}<div class="speech">知道了。${aside('把多余的安排划掉')}<br/>这次就让行程有一点空白。</div>${thought(c,'high',false,false)}<section class="work-trace"><div class="work-step"><span class="icon-button">${icon('search')}</span><div><strong>查找开放时间</strong><small>城市公园 · 310ms</small></div>${icon('check')}</div><div class="work-step"><span class="icon-button">${icon('file')}</span><div><strong>整理行程文件</strong><small>周末散步.md</small></div>${icon('check')}</div></section>${thought(c,'xhigh',true,true)}<div class="running-foot">${icon('star')}正在继续整理<button class="icon-button" aria-label="停止">${icon('stop')}</button></div></div>${composer(c,'xhigh','Nana')}`; }
  let streamTimer = null;
  function render() {
    clearTimeout(streamTimer);
    const c=palette(state.pack,state.dark), dark=palette(state.pack,true);
    document.body.style=themeVars(c);
    document.body.classList.toggle('reduced',state.reduced);
    document.querySelector('#mode').textContent=state.dark?'切换浅色':'切换暗色';
    document.querySelector('#mode').setAttribute('aria-pressed',String(state.dark));
    document.querySelector('#motion').setAttribute('aria-pressed',String(state.reduced));
    document.querySelector('#motion').textContent=state.reduced?'恢复轻动态':'减少动态';
    document.querySelector('#palettes').innerHTML=packs.map(([id,label,light])=>`<button style="--swatch:${light}" data-pack="${id}" title="${label}" aria-label="${label}" aria-pressed="${state.pack===id}"></button>`).join('');
    const pages=[['轻声旁白','01',first(c),'完整的动作与语气；柔和底线，正文保持清晰。',c],['长文与流式','02',second(c),'内联强调与链接照常工作；代码和公式不受影响。',c],['选择思考强度','03',third(c),'点选档位，图形、选中态、输入栏标识同步变化。',c],['思考过程 · 暗色','04',fourth(dark),'完成静止，执行轻动；只给小图标动效。',dark]];
    document.querySelector('#screens').innerHTML=pages.map(([title,index,body,caption,color])=>`<section><h2 class="screen-label"><span class="screen-index">${index}</span>${title}<small>${index==='04'?'DARK':'LIGHT / THEME'}</small></h2><div class="phone ${index==='03'?'dimmed':''}" style="${themeVars(color)}">${body}</div><p class="phone-caption">${caption}</p></section>`).join('');
    document.querySelector('#level-gallery').innerHTML=levels.map(([id,label,description,tone,motion])=>`<button class="level-sample ${id==='off'?'':'is-running'}" data-level="${id}" style="${levelVars(id,c)}">${art(id)}<strong>${label}</strong><p>${description}</p><small>${motion}</small></button>`).join('');
  }
  document.addEventListener('click',event=>{
    const target=event.target.closest('button');if(!target)return;
    if(target.id==='mode'){state.dark=!state.dark;render();}
    else if(target.id==='motion'){state.reduced=!state.reduced;render();}
    else if(target.dataset.pack){state.pack=target.dataset.pack;render();}
    else if(target.dataset.level){state.level=target.dataset.level;render();}
    else if(target.hasAttribute('data-close-sheet')){target.closest('.phone').querySelector('.selector').hidden=true;target.closest('.phone').classList.remove('dimmed');}
    else if(target.hasAttribute('data-choose-level')){const phone=target.closest('.phone');let sheet=phone.querySelector('.selector');if(!sheet){phone.insertAdjacentHTML('beforeend',selector(palette(state.pack,phone.closest('section').querySelector('.screen-index').textContent==='04'||state.dark)));sheet=phone.querySelector('.selector');}sheet.hidden=false;phone.classList.add('dimmed');}
    else if(target.id==='replay')replay();
  });
  function replay(){
    clearTimeout(streamTimer);
    const text='先把肩膀放松一点。（把热茶推到你手边）余下的事情，晚一点再说吧。',host=document.querySelector('#stream-demo');let position=0,decorated=false,tail=document.createTextNode('');
    host.replaceChildren(tail);
    function tick(){
      const char=text[position++];tail.appendData(char);
      if(char==='）'&&!decorated){const fragment=text.slice(0,position),open=fragment.indexOf('（');host.innerHTML=escape(fragment.slice(0,open))+aside(escape(fragment.slice(open+1,-1)));tail=document.createTextNode('');host.append(tail);decorated=true;}
      document.querySelector('#stream-status').textContent=position===text.length?'已完成 · 无整段重播':decorated?'括号闭合 · 旁白轻显现':'正在输出 · 不急着换样式';
      if(position<text.length)streamTimer=setTimeout(tick,document.hidden?300:70);
    }
    tick();
  }
  document.addEventListener('visibilitychange',()=>document.body.classList.toggle('paused',document.hidden));
  render();
})();
