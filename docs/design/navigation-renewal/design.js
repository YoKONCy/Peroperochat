/* 导航、轻面板与主题均为本地演示，不请求网络或修改正式应用数据。 */
(() => {
  const P=window.PPC,S=P.state,{icon:i,avatar:av,btn,go,badge,tag,field,choice,toggle,notice,brand,esc}=P;
  const $=s=>document.querySelector(s),app=$('#app'),overlay=$('#overlay-root');
  let toastTimer,streamTimer,modeTimer,overviewVisible=false,overlayReturn=null,pendingNavigation=null,currentSheet='',longPressHandled=false,navHistory=[],scrollPositions={},lastTabs={home:'home',threads:'threads',tasks:'tasks',files:'files',manage:'manage'};
  const editable=['memory-edit','prompt-edit','model-detail','profile'];
  const flowSets=[['home','connections','threads','chat'],['threads','groups','base','group-chat','group-settings'],['tasks','task-detail','chat'],['files','file-detail','memories','memory-detail','memory-edit'],['manage','models','model-detail','assignments','voice','tools'],['manage','roles','role-detail','companion','prompts','prompt-edit'],['manage','appearance','profile','usage','runtime'],['threads','chat','chat-flow']];
  function applyTheme(){
    const {palette,packs,mix}=window.Chromatic,values=palette(S.pack,S.dark),accent=values.accent;
    if(S.dark){Object.assign(values,{page:mix(accent,'#111521',.055),paper:mix(accent,'#202432',.06),raised:mix(accent,'#2b3040',.08),ink:'#ecedf5',muted:'#a8b0c4',line:mix(accent,'#343b4c',.13),shadow:'#070b14',on:'#182033'});}
    else{Object.assign(values,{page:mix(accent,'#f7f8fb',.025),paper:'#ffffff',raised:'#ffffff',ink:'#30394e',muted:'#7a869a',line:mix(accent,'#e6e9ef',.08),shadow:'#cad5e5',on:'#ffffff'});}
    values.primary=accent;values.primarySoft=mix(accent,values.paper,S.dark?.13:.11);values.primaryEdge=mix(accent,values.paper,S.dark?.32:.27);
    for(const n of ['rose','mint','violet','amber','blue']){values[n+'Soft']=S.dark?mix(values[n],values.paper,.13):mix(values[n],values.paper,.10);values[n+'Edge']=mix(values[n],values.paper,S.dark?.29:.26);}
    values.silver=S.dark?'#a6b0c1':'#77879c';values.silverSoft=mix(values.silver,values.paper,.09);values.inset=mix(values.page,values.paper,.75);values.glass=`color-mix(in srgb, ${values.paper} calc(var(--surface-opacity, .92) * 100%), transparent)`;
    Object.entries(values).forEach(([k,v])=>document.documentElement.style.setProperty('--'+k.replace(/[A-Z]/g,m=>'-'+m.toLowerCase()),v));
    document.documentElement.dataset.theme=S.dark?'dark':'light';
    document.documentElement.dataset.reduced=String(S.reduced);
    $('#theme-toggle').innerHTML=i(S.dark?'sun':'moon')+`<span>${S.dark?'浅色':'暗色'}</span>`;
    $('#motion-toggle').setAttribute('aria-pressed',String(S.reduced));
    $('#palette-caption').textContent=packs.find(x=>x[0]===S.pack)[1]+' · '+(S.dark?'暗色':'浅色');
    document.querySelectorAll('[data-pack]').forEach(el=>{el.setAttribute('aria-pressed',String(el.dataset.pack===S.pack));el.classList.toggle('selected',el.dataset.pack===S.pack);});
    try{localStorage.setItem('peropero-design-navigation-theme',JSON.stringify({pack:S.pack,dark:S.dark,reduced:S.reduced}));}catch{}
    P.syncMaterials?.();
    P.flowMotionChanged?.();
  }
  function header(id){
    const p=P.pages[id],chat=id==='chat'||id==='group-chat';
    if(id==='local')return '';
    if(chat)return `<header class="chat-header">${btn('back','返回会话列表','back')}${id==='chat'?av(S.chatRole||'pero'):badge('people','rose')}<div><h1>${id==='chat'?(S.chatRole==='nana'?'Nana':'Pero'):'放课后客厅'}${i(id==='chat'?'bow':'people')}</h1><button class="header-subtitle" data-action="${id==='chat'?'chat-switch':'group-members'}"><i class="dot"></i><span>${id==='chat'?esc(S.title):'星光小屋 · 2 位角色'}</span>${i('down')}</button></div>${btn('more',id==='chat'?'会话操作':'群聊设置',id==='chat'?'chat-menu':'group-menu')}</header>`;
    const eyebrow=p.parent?(P.pages[p.parent]?.title||'infOS'):(id==='home'?'infOS · 我的日常':'infOS');
    const action=id==='home'?btn('bell','查看通知','notifications','filled'):id==='threads'||id==='groups'?btn('plus',id==='groups'?'新建群聊':'新建会话',id==='groups'?'new-group':'new-chat','filled'):id==='files'?btn('more','资料操作','library-menu'):id==='memories'?btn('more','记忆操作','memory-menu'):p.parent?'':btn('search','查找内容','page-search');
    return `<header class="app-header ${p.parent?'has-back':'main-header'}">${p.parent?btn('back','返回'+P.pages[p.parent].title,'back'):''}<div class="header-copy"><small>${eyebrow}</small><h1>${p.title}${!p.parent&&id==='home'?i('bow'):''}</h1></div>${action}</header>`;
  }
  function footer(id){
    if(id==='local')return '';
    if(['chat','group-chat'].includes(id))return `${id==='chat'?P.queueSummary():''}${P.composer(id==='group-chat')}<div class="home-indicator"></div>`;
    if(editable.includes(id))return `<footer class="save-bar"><small id="save-state">${S.dirty?'有未保存的修改':'未修改'}</small><button class="primary-btn" data-action="save-editor">${i('check')}保存修改</button></footer>`;
    if(id==='memory-detail')return `<footer class="detail-actions"><button class="secondary-btn" data-action="archive-memory">${i(S.archived?'refresh':'archive')}${S.archived?'恢复记忆':'归档'}</button><button class="primary-btn" data-go="memory-edit">${i('edit')}编辑记忆</button></footer>`;
    if(id==='file-detail')return `<footer class="detail-actions"><button class="secondary-btn" data-action="share-file">${i('connection')}复制引用</button><button class="primary-btn" data-action="download">${i('download')}下载文件</button></footer>`;
    return P.nav(pTab(id));
  }
  function pTab(id){return P.pages[id].tab;}
  function markup(id){const c=['chat','group-chat'].includes(id);return P.status()+header(id)+(id==='local'?P.screens.local():`<div class="app-scroll ${c?'chat-scroll':''}" data-scroll>${P.screens[id]()}</div>`)+footer(id);}
  function setClass(el,id){el.className='app'+(['chat','group-chat'].includes(id)?' app-chat':'')+(id==='local'?' app-local':'');el.dataset.page=id;el.dataset.background=S.background||'default';}
  function render(id,kind='tab',restore=true){
    P.flowBeforeRender?.();closeSheet();clearInterval(streamTimer);S.page=id;setClass(app,id);app.innerHTML=markup(id);
    $('#route-label').textContent=(P.pages[id].parent?P.pages[P.pages[id].parent].title+' / ':'')+P.pages[id].title;
    document.querySelectorAll('#page-index [data-go]').forEach(x=>{x.classList.toggle('selected',x.dataset.go===id);if(x.dataset.go===id)x.setAttribute('aria-current','page');else x.removeAttribute('aria-current');});
    const scroll=app.querySelector('[data-scroll]');if(scroll){scroll.classList.add('enter-'+kind);requestAnimationFrame(()=>{if(restore)scroll.scrollTop=scrollPositions[id]||0;});}
    renderNotes();
    P.syncMaterials?.();
    if(S.offline&&id!=='connections'&&id!=='runtime'&&id!=='local'){const status=app.querySelector('.header-subtitle')||app.querySelector('.header-copy>small');if(status){const control=document.createElement('button');control.className='connection-status-inline';control.dataset.go='connections';control.innerHTML=i('offline')+'连接已断开'+i('next');status.replaceWith(control);}}
    if(S.chatRole==='nana'&&id==='chat'){app.querySelectorAll('.speaker .avatar').forEach(image=>{image.src=P.assets+'avatars/nana.jpg';image.alt='Nana';});app.querySelectorAll('.speaker strong').forEach(label=>label.textContent='Nana');const input=app.querySelector('[data-draft]');if(input)input.placeholder='和 Nana 接着聊…';}
  }
  function remember(){const scroll=app.querySelector('[data-scroll]');if(scroll)scrollPositions[S.page]=scroll.scrollTop;}
  function navigate(id,kind='forward',force=false){
    if(!P.pages[id])return;
    if(!force&&P.beforeGuidanceNavigate?.(()=>navigate(id,kind,true))===false)return;
    if(S.dirty&&editable.includes(S.page)&&!force){pendingNavigation=()=>navigate(id,kind,true);showSheet('修改还没有保存',`<p>保存后再离开，或放弃这次修改。</p>`,`<button class="secondary-btn" data-action="discard-edits">放弃修改</button><button class="primary-btn" data-action="save-leave">保存并离开</button>`,'edit');return;}
    remember();hideToast();if(S.page!==id){navHistory.push(S.page);history.pushState({page:id},'',`#${id}`);}S.dirty=false;
    if(['home','threads','groups','tasks','files','memories','knowledge','manage'].includes(id))lastTabs[pTab(id)]=id;
    if(overviewVisible)toggleOverview(false);render(id,kind);
  }
  function back(){navigate(navHistory.pop()||P.pages[S.page].parent||'home','back');navHistory.pop();}
  function renderNotes(){
    const p=P.pages[S.page],flow=flowSets.find(f=>f.includes(S.page))||['manage',S.page];
    $('#notes').innerHTML=`<div class="note-counter">${String(Object.keys(P.pages).indexOf(S.page)+1).padStart(2,'0')} / ${Object.keys(P.pages).length} PAGES</div><h2>${p.heading}</h2><p>${p.note}</p><div class="note-rule"></div><div class="note-label">试着沿这条路径走走</div><div class="flow-list">${flow.map((id,j)=>`${j?'<span class="flow-arrow">↓</span>':''}<button data-go="${id}" class="${id===S.page?'selected':''}">${i(P.pages[id].icon)}${id==='groups'?'群聊列表':id==='threads'?'单聊列表':id==='files'?'文件列表':id==='memories'?'记忆列表':P.pages[id].title}</button>`).join('')}</div><div class="note-rule"></div><div class="note-label">体验其他状态</div><div class="note-states"><button data-action="demo-toast">轻气泡反馈</button><button data-action="demo-offline">${S.offline?'恢复在线':'断网后回来'}</button>${S.page==='chat'?'<button data-action="demo-queue">排队与等待</button>':''}<button data-go="memory-edit">未保存时返回</button></div><div class="note-foot">所有页面共用组件和颜色令牌。<br>点击顶部「页面总览」横向查看。<br>此稿只演示交互，不发送真实请求。</div>`;
    const additions=S.page==='connections'?'<button data-action="demo-first-connect">首次连接</button><button data-action="demo-connection-failed">连接失败</button><button data-action="demo-reauthorize">需要重新配对</button><button data-action="demo-certificate-changed">证书变化</button>':S.page==='chat'?'<button data-action="demo-queue-paused">队列暂停</button><button data-action="demo-queue-failed">发送未完成</button>':'';
    $('#notes .note-states').insertAdjacentHTML('beforeend',additions+'<button data-go="chat-flow">聊天信息流与动效</button><button data-go="feedback">提示与链接专题</button><button data-go="connection-help">连接指引</button><button data-go="materials">背景与玻璃</button><button data-go="material-states">背景搭配对照</button>');
  }
  function toggleOverview(next=!overviewVisible){if(next)P.flowBeforeRender?.();overviewVisible=next;$('#stage').hidden=next;$('#notes').hidden=next;$('#overview').hidden=!next;$('#view-toggle').innerHTML=i(next?'play':'grid')+`<span>${next?'返回体验':'页面总览'}</span>`;if(next)renderOverview();}
  function renderOverview(){
    $('#overview').innerHTML=`<header class="overview-hero"><div><span class="overline">PEROPEROCHAT / NAVIGATION & EVERYDAY</span><h2>把每一步，放在好找的地方。</h2><p>五个主入口 · 逐层深入 · 浅色与暗色共用八套配色<br>页面均可进入体验，返回后保留阅读位置。</p></div><span>${Object.keys(P.pages).length}<small>个页面</small></span></header>`+P.groups.map(([label,ids],j)=>`<section class="overview-section"><h2><span>${String(j+1).padStart(2,'0')}</span>${label}</h2><div class="gallery-grid">${ids.map(id=>`<article class="gallery-card"><div class="gallery-caption"><strong>${id==='groups'?'群聊':id==='threads'?'单聊':id==='files'?'文件':id==='memories'?'记忆':id==='knowledge'?'知识':P.pages[id].title}</strong><span>${String(Object.keys(P.pages).indexOf(id)+1).padStart(2,'0')}</span></div><div class="mini-frame"><div class="mini-device" inert><div class="app ${['chat','group-chat'].includes(id)?'app-chat':''} ${id==='local'?'app-local':''}">${markup(id)}</div></div></div><button class="gallery-open" data-go="${id}">进入体验 ${i('next')}</button><p class="gallery-note">${P.pages[id].note}</p></article>`).join('')}</div></section>`).join('');
    P.syncMaterials?.();
  }
  let toastRemaining=0,toastStarted=0,toastPaused=false;
  function hideToast(){clearTimeout(toastTimer);$('#toast').classList.remove('visible');toastRemaining=0;toastPaused=false;}
  function pauseToast(){if(!toastRemaining||toastPaused)return;clearTimeout(toastTimer);toastRemaining=Math.max(0,toastRemaining-(Date.now()-toastStarted));toastPaused=true;}
  function resumeToast(){if(!toastRemaining||!toastPaused||$('#toast').contains(document.activeElement)||$('#toast').matches(':hover'))return;toastPaused=false;toastStarted=Date.now();toastTimer=setTimeout(hideToast,toastRemaining);}
  function toast(text,options={}){clearTimeout(toastTimer);const el=$('#toast'),tone=options.tone||'mint';el.dataset.tone=tone;el.innerHTML=`<span class="toast-symbol ${options.busy?'is-busy':''}">${i(options.icon||(tone==='rose'?'offline':tone==='blue'?'connection':'checkcircle'))}</span><span class="toast-copy"><strong>${esc(text)}</strong>${options.detail?`<small>${esc(options.detail)}</small>`:''}</span>${options.action?`<button class="toast-action" data-action="${options.action}">${esc(options.label||'查看')}</button>`:''}<button class="toast-close" data-action="dismiss-toast" aria-label="关闭提示">${i('close')}</button>`;el.classList.add('visible');toastRemaining=options.duration??(options.action?6000:3200);toastPaused=false;toastStarted=Date.now();toastTimer=setTimeout(hideToast,toastRemaining);}
  $('#toast').addEventListener('pointerenter',pauseToast);$('#toast').addEventListener('pointerleave',resumeToast);$('#toast').addEventListener('focusin',pauseToast);$('#toast').addEventListener('focusout',()=>requestAnimationFrame(resumeToast));
  function showSheet(title,body,foot='',symbol='bow'){
    overlayReturn=document.activeElement;overlay.innerHTML=`<div class="overlay"><div class="overlay-scrim" data-action="close-sheet"></div><section class="sheet" role="dialog" aria-modal="true" aria-labelledby="sheet-title"><div class="sheet-grip"></div><header class="sheet-header">${i(symbol)}<h2 id="sheet-title">${title}</h2>${btn('close','关闭面板','close-sheet')}</header><div class="sheet-body">${body}</div>${foot?`<footer class="sheet-footer">${foot}</footer>`:''}</section></div>`;
    app.inert=true;requestAnimationFrame(()=>overlay.querySelector('.sheet-header button')?.focus({preventScroll:true}));
  }
  function closeSheet(){overlay.innerHTML='';app.inert=false;currentSheet='';if(overlayReturn?.isConnected)overlayReturn.focus({preventScroll:true});overlayReturn=null;}
  function menu(title,items,symbol='more'){showSheet(title,`<div class="sheet-menu">${items.map(([n,t,s,action,cls])=>`<button data-action="${action}" class="${cls||''}">${i(n)}<span><strong>${t}</strong>${s?`<small>${s}</small>`:''}</span>${cls==='danger'?'':i('next')}</button>`).join('')}</div>`,'',symbol);}
  function picker(title,options,action,symbol='tuning'){showSheet(title,`<div class="option-list">${options.map(([value,label,sub,n,selected])=>`<button data-pick="${action}" data-value="${esc(value)}" class="${selected?'selected':''}">${badge(n||'checkcircle','primary')}<span><strong>${label}</strong>${sub?`<small>${sub}</small>`:''}</span>${selected?i('check'):''}</button>`).join('')}</div>`,'',symbol);}
  function confirm(title,text,action,button='确认',danger=false){showSheet(title,`<p>${text}</p>`,`<button class="secondary-btn" data-action="close-sheet">取消</button><button class="primary-btn ${danger?'danger-solid':''}" data-action="${action}">${button}</button>`,danger?'trash':'checkcircle');}
  function markDirty(){if(!editable.includes(S.page)&&S.page!=='role-detail')return;S.dirty=true;const el=$('#save-state');if(el)el.textContent='有未保存的修改';}
  function saveEditor(leave=false){
    const memory=app.querySelector('[data-field="memoryText"]'),prompt=app.querySelector('[data-field="promptText"]');if(memory)S.memoryText=memory.value;if(prompt)S.promptText=prompt.value;
    S.dirty=false;closeSheet();if(leave&&pendingNavigation){const next=pendingNavigation;pendingNavigation=null;next();}else if(S.page==='memory-edit')navigate('memory-detail','back',true);else if(S.page==='prompt-edit')navigate('prompts','back',true);else render(S.page,'tab');toast('修改已保存');
  }
  function localFile(kind){
    const input=document.createElement('input');input.type='file';input.accept=['background','avatar','image'].includes(kind)?'image/*':'';input.style.display='none';document.body.append(input);input.addEventListener('change',()=>{const f=input.files?.[0];if(f){closeSheet();if(kind==='background'){P.acceptMaterialBackground?.(f,toast);}else if(kind==='avatar'){const url=URL.createObjectURL(f);app.querySelectorAll('.user-avatar').forEach(x=>{x.style.backgroundImage=`url("${url}")`;x.style.backgroundSize='cover';x.style.color='transparent';});toast('头像预览已更新');}else{toast('已选择：'+f.name);if(S.page==='chat'){S.draft='[附件：'+f.name+'] '+S.draft;render('chat','tab');}}}input.remove();},{once:true});input.addEventListener('cancel',()=>input.remove(),{once:true});input.click();
  }
  function send(form){
    const textarea=form.querySelector('textarea'),text=textarea.value.trim();if(!text){textarea.focus();return;}
    if(S.offline){toast('还没连接，内容已留在输入框',{tone:'amber',icon:'offline',action:'show-connection',label:'去连接'});return;}
    if(S.page==='chat'&&S.queue){P.enqueue(text);S.draft='';remember();render('chat','tab');toast('已排好，稍后轮到这条',{action:'queue',label:'查看'});return;}
    const group=form.dataset.form==='group-chat';(group?S.groupSent:S.sent).push(text);if(group)S.groupDraft='';else S.draft='';remember();render(group?'group-chat':'chat','tab');
    const scroll=app.querySelector('[data-scroll]');scroll.scrollTop=scroll.scrollHeight;
    if(!group){
      let elapsed=0;const target=$('#stream-reply');target.innerHTML=P.flowWaiting(0);scroll.scrollTop=scroll.scrollHeight;
      streamTimer=setInterval(()=>{
        elapsed++;const line=target.querySelector('time');if(line)line.textContent=`${elapsed} 秒`;
        if(elapsed>=2){
          clearInterval(streamTimer);P.finishFlowAnimations?.(target);
          const reply='好呀，我记住啦。接下来的时间，就按你喜欢的节奏来。';let n=0;
          const follow=scroll.scrollHeight-scroll.scrollTop-scroll.clientHeight<80;
          target.innerHTML=P.flowStreamStart(elapsed);if(follow)scroll.scrollTop=scroll.scrollHeight;
          streamTimer=setInterval(()=>{
            const follow=scroll.scrollHeight-scroll.scrollTop-scroll.clientHeight<80;
            const p=target.querySelector('.flow-stream-text'),fragment=document.createElement('span');
            fragment.className='flow-fragment';fragment.textContent=reply.slice(n,n+2);n+=2;if(p)p.append(fragment);
            if(follow)scroll.scrollTop=scroll.scrollHeight;
            if(n>=reply.length){clearInterval(streamTimer);target.querySelector('.is-streaming')?.classList.remove('is-streaming');}
          },45);
        }
      },1000);
    }
  }
  const actions={
    back, 'close-sheet':closeSheet, notifications:()=>navigate('notifications'),
    'page-search':()=>{const input=app.querySelector('input[type=search]');if(input)input.focus();else navigate('manage');},
    'demo-toast':()=>toast('已保存，一会儿见。'),
    'demo-offline':()=>{S.offline=!S.offline;remember();render(S.page,'tab');toast(S.offline?'暂时离线，聊天记录还在':'已重新连接，内容正在更新');},
    'demo-queue':()=>{S.queue=!S.queue;remember();render('chat','tab');},
    'thread-menu':()=>menu('这段会话',[['edit','重命名','','rename-thread'],['trash','删除会话','','delete-thread','danger']]),
    'chat-menu':()=>menu('当前会话',[['edit','重命名','','rename-thread'],['folder','会话文件','','go-files'],['trash','删除会话','','delete-thread','danger']]),
    'chat-switch':()=>picker('切换会话',[[S.title,S.title,'Pero · 今天','chat',true],['晚一点也没关系','晚一点也没关系','Nana · 昨天','chat',false]],'thread'),
    'rename-thread':()=>showSheet('重命名会话',field('会话名称',S.title,{key:'rename'}),'<button class="primary-btn" data-action="confirm-rename">保存名称</button>','edit'),
    'confirm-rename':()=>{const input=overlay.querySelector('[data-field=rename]');if(!input?.value.trim()){input?.focus();return;}S.title=input.value.trim();closeSheet();remember();render(S.page,'tab');toast('会话名称已更新');},
    'delete-thread':()=>confirm('删除这段会话？','聊天记录会一起删除。这个操作不能撤销。','confirm-delete-thread','删除会话',true),
    'confirm-delete-thread':()=>{closeSheet();navigate('threads','back');toast('演示：会话已删除');},
    'new-chat':()=>picker('这次想和谁聊聊',[['pero','Pero','活泼的日常搭子','chat',false],['nana','Nana','嘴硬心软，偶尔撒娇','moon',false]],'new-role','people'),
    'message-menu':()=>menu('这条消息',[['copy','复制正文','','copy'],['edit','编辑消息','可保存，或保存后重新生成','edit-message'],['refresh','回滚到这里','撤回这一轮及后续对话','rollback']]),
    'copy':()=>{closeSheet();toast('已复制');},
    'edit-message':()=>showSheet('编辑消息',field('消息正文','周末想出门走走，别太赶。',{area:true,rows:5}),'<button class="secondary-btn" data-action="message-save">仅保存</button><button class="primary-btn" data-action="message-reset">保存并重新生成</button>','edit'),
    'message-save':()=>{closeSheet();toast('消息已保存');},
    'message-reset':()=>{closeSheet();toast('已保存，正在重新生成');},
    'rollback':()=>confirm('回到这一轮之前？','这一轮和后续对话将被撤回。相关文件变更会先列出来供你确认。','rollback-review','查看变更'),
    'rollback-review':()=>showSheet('这次回滚的范围',notice('撤回 1 轮对话，移除 1 个生成文件。','amber','refresh')+'<div class="file-row">'+badge('file','blue')+'<span><strong>周末路线.md</strong><small>本轮创建的文件</small></span></div>','<button class="secondary-btn" data-action="close-sheet">取消</button><button class="primary-btn" data-action="rollback-done">确认回滚</button>','refresh'),
    'rollback-done':()=>{closeSheet();toast('演示：已回到这一轮之前');},
    'go-files':()=>navigate('files'),
    'model-picker':()=>picker('选择聊天模型',[['Gemini 3 Pro','Gemini 3 Pro','默认 · Google','model',S.model==='Gemini 3 Pro'],['DeepSeek','DeepSeek','DeepSeek','model',S.model==='DeepSeek']], 'model','model'),
    'effort':()=>picker('思考强度',[['不指定','不额外指定','不发送思考参数','moon',S.effort==='不指定'],['低','低','简单问题，轻快回应','thought',S.effort==='低'],['中','中','兼顾速度与思考','thought',S.effort==='中'],['高','高','多一点推敲','thought',S.effort==='高'],['极高','极高','留出更多思考空间','spark',S.effort==='极高']],'effort','thought'),
    'chat-mode':()=>picker('聊天方式',[['极简','极简','直接聊，少一点展开','chat',false],['聊天','聊天','自然对话与日常陪伴','heart',true],['工作','工作','完成任务、使用更多工具','tool',false]],'chat-mode'),
    'chat-tools':()=>menu('工具与回复',[['tool','工具权限','选择这次聊天可使用的工具','go-tools'],['image','图片识别方式','自动识别','vision-mode'],['edit','回答预填充','设置回复开头','prefill']],'tool'),
    'go-tools':()=>navigate('tools'),
    'vision-mode':()=>picker('图片识别',[['自动','自动','按模型能力选择','image',true],['视觉模型','使用视觉模型','识别后提供给聊天模型','model',false]],'simple'),
    'attach':()=>localFile('file'), 'upload':()=>localFile('file'), 'import':()=>localFile('file'),
    'voice-input':()=>showSheet('语音输入',`<div class="sheet-illustration">${i('mic')}</div><p class="sheet-explain">点击开始后录音，结束后先检查文字。</p>`,'<button class="primary-btn" data-action="voice-demo">体验识别过程</button>','mic'),
    'voice-demo':()=>showSheet('确认识别内容',field('识别到的文字','周末我们一起去河边走走吧。',{area:true,rows:3,key:'recognized'}),'<button class="primary-btn" data-action="voice-insert">放入输入框</button>','mic'),
    'voice-insert':()=>{S.draft=overlay.querySelector('[data-field=recognized]').value;closeSheet();navigate('chat');},
    'queue':()=>showSheet('排队中的消息',`<div class="queued-message"><span>${tag('等待上一条完成','amber')}</span><p>顺便找一家沿途的咖啡店。</p><small>你可以继续输入，消息会依次发送。</small></div>`,'<button class="secondary-btn" data-action="queue-cancel">取消排队</button><button class="primary-btn" data-action="queue-now">立即提交</button>','queue'),
    'queue-cancel':()=>{S.queue=false;closeSheet();remember();render('chat','tab');toast('已取消排队');},
    'queue-now':()=>{S.queue=false;closeSheet();remember();render('chat','tab');toast('消息已提交');},
    'group-menu':()=>menu('放课后客厅',[['people','成员与接话','','group-members'],['settings','房间设置','','go-group-settings'],['home','前往星光小屋','','go-base']],'people'),
    'group-members':()=>showSheet('房间里的大家',`<div class="option-list"><button data-pick="mention" data-value="Pero">${av('pero')}<span><strong>Pero</strong><small>这次想问她</small></span>${i('chat')}</button><button data-pick="mention" data-value="Nana">${av('nana')}<span><strong>Nana</strong><small>这次想问她</small></span>${i('chat')}</button></div>`,'<button class="secondary-btn" data-action="go-group-settings">管理成员</button>','people'),
    'go-group-settings':()=>navigate('group-settings'),'go-base':()=>navigate('base'),
    'mention':()=>picker('这次想问谁',[['大家','大家','让大家一起聊','people',true],['自动选择','自动选择','由房间决定谁来接话','spark',false],['Pero','Pero','','chat',false],['Nana','Nana','','moon',false]],'mention','people'),
    'new-group':()=>showSheet('创建群聊',field('房间名称','周末小客厅',{key:'room-name'})+choice('所属据点','星光小屋','go-base','home')+`<div class="invite-options"><label class="custom-check"><input type="checkbox" checked>${av('pero')}Pero</label><label class="custom-check"><input type="checkbox" checked>${av('nana')}Nana</label></div>`,'<button class="primary-btn" data-action="create-group">创建并进入</button>','people'),
    'create-group':()=>{closeSheet();navigate('group-chat');toast('群聊已创建');},
    'invite':()=>picker('邀请角色',[['Pero','Pero','已在当前房间','people',true],['Nana','Nana','已在当前房间','people',true]],'simple','people'),
    'rename-group':()=>showSheet('房间名称',field('名称','放课后客厅'),'<button class="primary-btn" data-action="generic-save">保存</button>','edit'),
    'base-edit':()=>showSheet('编辑据点',field('据点名称','星光小屋')+field('介绍','聊天、发呆，还有一起待着。',{area:true,rows:3}),'<button class="primary-btn" data-action="generic-save">保存据点</button>','home'),
    'environment':()=>picker('房间氛围',[['白天 · 晴朗','白天 · 晴朗','窗边有阳光','sun',true],['黄昏 · 微风','黄昏 · 微风','慢慢等日落','sun',false],['夜晚 · 安静','夜晚 · 安静','留一盏小灯','moon',false]],'simple','sun'),
    'delete-room':()=>confirm('删除这个房间？','房间和其中的聊天记录将一起删除。','room-deleted','删除房间',true),
    'room-deleted':()=>{closeSheet();navigate('groups');toast('演示：房间已删除');},
    'group-progress':()=>showSheet('这一轮的接话',`<div class="member-row">${av('pero')}<span>Pero</span>${tag('已完成','mint')}</div><div class="member-row">${av('nana')}<span>Nana</span>${tag('已完成','mint')}</div>`,'','people'),
    'approve-task':()=>{S.taskApproved=true;remember();render('task-detail','tab');toast('已允许，文档已保存');},
    'deny-task':()=>confirm('这次先不保存？','任务会停在这里，你仍然可以查看已经整理的内容。','task-denied','暂不允许'),
    'task-denied':()=>{closeSheet();toast('本次操作已取消');},
    'file-location':()=>picker('浏览位置',[['当前会话','当前会话','给周末留一点空白','chat',true],['工作区','工作区','当前 infOS 的文件','folder',false]],'simple','folder'),
    'file-sort':()=>picker('文件排序',[['最近更新','最近更新','','clock',true],['名称','名称 A—Z','','file',false],['大小','文件大小','','folder',false]],'simple'),
    'folder-weekend':()=>showSheet('周末计划',P.screens['file-detail'](),'<button class="primary-btn" data-go="file-detail">打开完整预览</button>','folder'),
    'folder-notes':()=>navigate('knowledge'),
    'image-preview':()=>showSheet('喜欢的房间.png',`<img class="image-preview" src="${P.assets}companion/sunlit-room.png" alt="房间大图"><p class="preview-caption">图片原地展开，关闭后回到文件列表。</p>`,'<button class="secondary-btn" data-action="download">下载图片</button>','image'),
    'download':()=>{const blob=new Blob(['给周末留一点空白\n\n15:00 去河边散步\n16:30 找家小店喝下午茶\n17:30 在长椅上等日落\n\nPeroperoChat 交互设计稿示例文件'],{type:'text/plain;charset=utf-8'});const a=document.createElement('a');a.href=URL.createObjectURL(blob);a.download='周末路线-设计示例.txt';a.click();setTimeout(()=>URL.revokeObjectURL(a.href),1000);toast('示例文件已下载');},
    'share-file':()=>toast('已复制文件引用'),
    'library-menu':()=>menu('资料操作',[['plus','上传文件','','upload'],['file','导入知识','','import']],'folder'),
    'memory-menu':()=>menu('记忆操作',[['archive','查看已归档记忆','','go-archived'],['tuning','按角色筛选','','memory-filter']],'memory'),
    'go-archived':()=>{S.memoryFilter='archived';navigate('memories');},
    'memory-filter':()=>picker('查看谁的记忆',[['全部角色','全部角色','','people',true],['Pero','Pero','','person',false],['Nana','Nana','','person',false]],'simple','people'),
    'archive-memory':()=>confirm(S.archived?'恢复这条记忆？':'归档这条记忆？',S.archived?'恢复后，可以再次用于日常回忆。':'归档后暂不用于日常回忆，之后可以恢复。','confirm-archive',S.archived?'恢复':'归档'),
    'confirm-archive':()=>{S.archived=!S.archived;closeSheet();render('memory-detail','tab');toast(S.archived?'已归档，可在「已归档」中找到':'记忆已恢复');},
    'delete-memory':()=>confirm('删除这条记忆？','删除后无法恢复，原始聊天不会被删除。','confirm-delete-memory','删除记忆',true),
    'confirm-delete-memory':()=>{closeSheet();navigate('memories','back');toast('演示：记忆已删除');},
    'memory-versions':()=>showSheet('记忆的历史版本',`<div class="timeline"><div class="done">${i('check')}<span><strong>当前版本</strong><small>今天 09:30 · 已更新</small></span></div><div>${i('clock')}<span><strong>最初记下的内容</strong><small>昨天 · 喜欢安静的散步</small></span></div></div>`,'','memory'),
    'save-editor':()=>saveEditor(), 'save-leave':()=>saveEditor(true),
    'discard-edits':()=>{S.dirty=false;closeSheet();const next=pendingNavigation;pendingNavigation=null;next?.();},
    'generic-save':()=>{closeSheet();toast('已保存');},
    'save-role':()=>{S.dirty=false;toast('角色资料已保存');},
    'role-avatar':()=>localFile('avatar'),'role-cover':()=>localFile('image'),'user-avatar':()=>localFile('avatar'),
    'add-role':()=>showSheet('添加角色',field('角色名称','')+field('一句话介绍','',{area:true,rows:3}),'<button class="primary-btn" data-action="role-created">创建角色</button>','people'),
    'role-created':()=>{closeSheet();navigate('role-detail');toast('演示：新角色已创建');},
    'role-style':()=>picker('角色的外观',[['跟随应用主题','跟随应用主题','使用当前配色与组件样式','palette',true],['角色样式包','角色样式包','使用角色包提供的样式','image',false]],'simple','palette'),
    'fetch-models':()=>picker('可添加的模型',[['gemini-3-pro','Gemini 3 Pro','gemini-3-pro','model',true],['gemini-3-flash','Gemini 3 Flash','gemini-3-flash','model',false],['gemini-2.5-pro','Gemini 2.5 Pro','gemini-2.5-pro','model',false]],'catalog','model'),
    'add-model':()=>showSheet('添加模型',field('显示名称','')+choice('服务商','选择服务商','provider','connection')+field('API 密钥','',{type:'password'}),'<button class="primary-btn" data-action="fetch-models">获取模型列表</button>','model'),
    'test-model':()=>toast('演示连接成功，可以使用'),
    'provider':()=>picker('选择服务商',[['Google','Google','Gemini API','model',true],['OpenAI','OpenAI','OpenAI API','model',false],['自定义','自定义连接','填写兼容的 API 地址','connection',false]],'simple','connection'),
    'temperature':()=>showSheet('回复的变化程度','<p>数值较低时更稳定，较高时更多变化。</p><div class="range-label"><span>稳定</span><output id="temperature-value">0.7</output><span>多变化</span></div><input type="range" min="0" max="2" step="0.1" value="0.7" data-range="temperature-value" aria-label="温度">','<button class="primary-btn" data-action="generic-save">确定</button>','tuning'),
    'output-length':()=>picker('输出长度',[['默认','跟随模型','','model',true],['4096','4,096 Token','','file',false],['8192','8,192 Token','','file',false]],'simple'),
    'tts-provider':()=>picker('朗读服务',[['跟随 infOS','跟随 infOS','使用已配置的语音服务','audio',true],['单独选择','单独选择','查看服务端可用服务','audio',false]],'simple','audio'),
    'voice-picker':()=>picker('选择声音',[['自然','温柔 · 自然','适合日常聊天','audio',true],['明亮','明亮 · 轻快','活泼一些的语气','audio',false],['沉静','沉静 · 舒缓','慢慢说，不着急','audio',false]],'voice','audio'),
    'speak':()=>{if('speechSynthesis' in window){speechSynthesis.cancel();const utterance=new SpeechSynthesisUtterance('今天也辛苦啦，慢慢来就好。');utterance.lang='zh-CN';utterance.rate=.9;speechSynthesis.speak(utterance);toast('正在用浏览器声音试听');}else toast('此浏览器无法试听语音');},
    'language':()=>picker('界面语言',[['zh-CN','简体中文','','globe',true],['zh-TW','繁體中文','','globe',false],['en-US','English','','globe',false]],'language','globe'),
    'quiet-time':()=>showSheet('安静时间',`<div class="two-fields">${field('开始','22:30',{type:'time'})}${field('结束','08:00',{type:'time'})}</div><p>在这段时间内，不主动发起联系。</p>`,'<button class="primary-btn" data-action="generic-save">保存时间</button>','moon'),
    'add-prompt':()=>navigate('prompt-edit'),
    'prompt-scope':()=>picker('提示词作用范围',[['日常聊天','日常聊天','','chat',true],['工作','工作与工具调用','','tool',false],['角色','指定角色','','person',false]],'simple','file'),
    'prefill':()=>showSheet('回答预填充','<p>让下一次回复从你写的内容开始。</p>'+field('回复开头','',{area:true,rows:4,key:'prefill'})+toggle('持续使用','关闭时，仅对下一次回复生效。',false,'prefill-persistent'),'<button class="secondary-btn" data-action="close-sheet">取消</button><button class="primary-btn" data-action="generic-save">保存</button>','edit'),
    'background-upload':()=>localFile('background'),
    'connection-details':()=>showSheet('家里的 infOS',`<div class="stat-line"><span>地址</span><strong>192.168.1.8:9122</strong></div><div class="stat-line"><span>连接方式</span><strong>HTTPS</strong></div><div class="stat-line"><span>证书</span><strong>已核对</strong></div><div class="sheet-menu"><button data-action="certificate">${i('shield')}<span><strong>核对证书</strong></span>${i('next')}</button><button data-action="paste-link">${i('connection')}<span><strong>重新授权</strong></span>${i('next')}</button><button data-action="remove-connection" class="danger">${i('trash')}<span><strong>移除连接</strong></span></button></div>`,'','connection'),
    'certificate':()=>showSheet('核对这台设备',`<p>请与电脑端显示的证书指纹比对，确认连接的是你的 infOS。</p><div class="certificate-fingerprint">7A:91:CE:2F:89:BB:06:D1<br>A6:22:38:BE:71:4D:95:0A<br>3C:28:67:84:A1:2B:FF:9D<br>B0:44:55:01:E8:9C:2A:63</div><label class="custom-check"><input type="checkbox" id="trust-check">我已与电脑端核对</label>`,'<button class="secondary-btn" data-action="close-sheet">取消</button><button class="primary-btn" id="trust-button" data-action="trust-cert" disabled>信任这台设备</button>','shield'),
    'trust-cert':()=>{closeSheet();toast('已保存这台设备的信任');},
    'scan':()=>showSheet('扫描电脑上的二维码',`<div class="scan-art">${i('qr')}</div><p class="sheet-explain">在 infOS 中打开「连接设备」，将二维码放进框内。</p><p class="prototype-scan-note">设计稿演示扫码后的确认流程，不调用摄像头。</p>`,'<button class="primary-btn" data-action="scan-result">模拟识别成功</button>','qr'),
    'scan-result':()=>showSheet('找到你的 infOS',`<div class="sheet-illustration">${i('connection')}</div><p class="sheet-explain"><strong>家里的 infOS</strong><br>https://192.168.1.8:9122</p>`,'<button class="secondary-btn" data-action="close-sheet">取消</button><button class="primary-btn" data-action="certificate">确认连接</button>','connection'),
    'paste-link':()=>showSheet('粘贴配对链接',field('来自电脑的配对链接','',{area:true,rows:3,placeholder:'在这里粘贴',key:'pairing-link'}),'<button class="primary-btn" data-action="scan-result">继续</button>','connection'),
    'manual-connect':()=>showSheet('手动添加连接',field('服务地址','http://192.168.1.8:9122',{key:'endpoint'})+field('配对码','',{key:'pairing-code'})+`<label class="custom-check"><input type="checkbox" id="http-consent">这是我信任的网络，允许使用 HTTP</label>`,'<button class="primary-btn" id="manual-connect-button" data-action="connect-manual" disabled>连接</button>','globe'),
    'connect-manual':()=>{closeSheet();toast('演示：已通过 HTTP 连接');},
    'reconnect':()=>{S.offline=false;remember();render(S.page,'tab');toast('已连接你的 infOS');},
    'remove-connection':()=>confirm('移除这个连接？','之后可以重新配对。服务端的聊天与资料不会被删除。','connection-removed','移除连接',true),
    'connection-removed':()=>{closeSheet();toast('演示：连接已移除');},
    'background-service':()=>showSheet('后台接收',toggle('应用在后台时接收消息','离开当前页面后，继续接收进展通知。',true,'background')+'<p>接收效果受手机的后台运行权限影响。</p>','<button class="primary-btn" data-action="generic-save">完成</button>','bell'),
    'read-all':()=>{app.querySelectorAll('.notification-card.unread').forEach(e=>e.classList.remove('unread'));toast('全部标为已读');},
    'notice-unread':()=>{app.querySelectorAll('.notification-card:not(.unread)').forEach(e=>e.hidden=true);toast('只显示未读通知');},
    'local-role':()=>picker('今天和谁待着',[['pero','Pero','晴空房间','sun',true],['nana','Nana','月夜房间','moon',false]],'local-role','people'),
    'local-menu':()=>menu('独立空间',[['people','家庭群聊','','local-group'],['star','21 点','','local-game'],['palette','外观与显示','','go-appearance'],['settings','本地设置','','local-settings']],'bow'),
    'local-chat':()=>showSheet('和 Pero 聊聊',`<div class="speaker">${av('pero')}<strong>Pero</strong></div><div class="reply-bubble"><p>今天有什么想分享的小事吗？</p></div>${field('想说的话','',{area:true,rows:3,key:'local-message'})}`,'<button class="primary-btn" data-action="local-send">发送</button>','chat'),
    'local-send':()=>{closeSheet();toast('设计稿：Pero 收到啦');},
    'local-diary':()=>showSheet('今天的小日记',`<article class="memory-paper"><h2>阳光刚刚好</h2><p>一起聊了周末想做的事情。没有特别复杂的计划，能慢慢待着也很好。</p><small>保存在独立空间</small></article>`,'','memory'),
    'local-display':()=>picker('角色显示',[['live2d','Live2D','使用通用 Pio 模型','person',false],['standing','静态立绘','使用角色包中的立绘','image',true]],'simple','image'),
    'local-group':()=>showSheet('家庭群聊',`<div class="speaker">${av('pero')}<strong>Pero</strong></div><div class="group-bubble">今天想一起聊些什么？</div><div class="speaker">${av('nana')}<strong>Nana</strong></div><div class="group-bubble nana">我也在听。</div>`,'<button class="primary-btn" data-action="local-chat">和大家说说话</button>','people'),
    'local-game':()=>showSheet('来一局 21 点',`<div class="playing-cards"><span>7<small>方块</small></span><span>9<small>红桃</small></span></div><p class="sheet-explain">你的点数：16</p>`,'<button class="secondary-btn" data-action="game-hold">停牌</button><button class="primary-btn" data-action="game-card">再来一张</button>','star'),
    'game-hold':()=>toast('设计稿：已停牌'),'game-card':()=>toast('设计稿：抽到 5，刚好 21 点'),
    'go-appearance':()=>navigate('appearance'),
    'local-settings':()=>showSheet('独立空间设置',choice('聊天模型','本机配置','model-picker','model')+choice('外观','跟随应用','go-appearance','palette')+toggle('互动台词','触摸角色时显示轻轻浮起的文字。',true),'','settings')
  };
  document.addEventListener('click',e=>{
    const el=e.target.closest('button,[data-action]');if(!el||el.disabled)return;
    if(longPressHandled&&el.closest('.thread-card')){e.preventDefault();longPressHandled=false;return;}
    if(el.id==='theme-toggle'){S.dark=!S.dark;applyTheme();if(S.page==='appearance')render('appearance');if(overviewVisible)renderOverview();return;}
    if(el.id==='motion-toggle'){S.reduced=!S.reduced;applyTheme();return;}
    if(el.id==='view-toggle'){toggleOverview();return;}
    if(el.dataset.pack){S.pack=el.dataset.pack;applyTheme();if(S.page==='appearance')render('appearance');if(overviewVisible)renderOverview();return;}
    if(el.dataset.go){if(el.dataset.chatRole)S.chatRole=el.dataset.chatRole;navigate(el.dataset.go);return;}
    if(el.dataset.tab){navigate(lastTabs[el.dataset.tab]||el.dataset.tab,'tab');return;}
    if(el.dataset.action){actions[el.dataset.action]?.(el);return;}
    if(el.dataset.role){S.selectedRole=el.dataset.role;remember();render('threads','tab');return;}
    if(el.dataset.taskFilter){S.taskFilter=el.dataset.taskFilter;render('tasks','tab',false);return;}
    if(el.dataset.memoryFilter){S.memoryFilter=el.dataset.memoryFilter;render('memories','tab',false);return;}
    if(el.dataset.pick){const k=el.dataset.pick,v=el.dataset.value;closeSheet();if(k==='model'){S.model=v;remember();render(S.page,'tab');toast('已切换模型');}else if(k==='effort'){S.effort=v;remember();render(S.page,'tab');}else if(k==='thread'){S.title=v;navigate('chat');}else if(k==='new-role'){S.chatRole=v;S.title='新的对话';S.sent=[];navigate('chat');toast('新会话已准备好');}else if(k==='mention'){if(S.page!=='group-chat')navigate('group-chat');$('#mention-label').textContent=v;}else if(k==='catalog'){toast('已选择模型');navigate('model-detail');}else if(k==='voice'){toast('声音已选择，可点击试听');}else if(k==='language'){toast('设计稿展示中文；实装沿用 i18n 文案');}else if(k==='local-role'){const image=app.querySelector('.local-character');image.src=P.assets+'companion/'+(v==='nana'?'nana-standing.png':'pero-standing.svg');app.querySelector('.local-top strong').textContent=v==='nana'?'Nana':'Pero';app.querySelector('.local-top .avatar').src=P.assets+'avatars/'+v+'.jpg';}else toast('已选择：'+v);return;}
    if(el.dataset.effort){S.effort=el.dataset.effort;el.parentElement.querySelectorAll('button').forEach(b=>b.classList.toggle('selected',b===el));markDirty();return;}
    if(el.dataset.anchor){const scroll=app.querySelector('[data-scroll]'),target=app.querySelector('#'+el.dataset.anchor);scroll.scrollTo({top:target.offsetTop-60,behavior:S.reduced?'instant':'smooth'});el.parentElement.querySelectorAll('button').forEach(b=>b.classList.toggle('selected',b===el));return;}
    if(el.dataset.companion){el.parentElement.querySelectorAll('button').forEach(b=>b.classList.toggle('selected',b===el));toast('陪伴节奏已调整');return;}
    if(el.dataset.promptCategory){el.parentElement.querySelectorAll('button').forEach(b=>b.classList.toggle('selected',b===el));app.querySelectorAll('[data-category]').forEach(b=>b.hidden=el.dataset.promptCategory!=='全部'&&el.dataset.promptCategory!==b.dataset.category);return;}
    if(el.dataset.insert){const area=app.querySelector('[data-field=promptText]');area.value+=el.dataset.insert;area.focus();markDirty();return;}
    if(el.dataset.theme){S.dark=el.dataset.theme==='system'?matchMedia('(prefers-color-scheme: dark)').matches:el.dataset.theme==='dark';applyTheme();remember();render('appearance','tab');return;}
    if(el.dataset.background){P.selectMaterialBackground?.(el.dataset.background);el.parentElement.querySelectorAll('button').forEach(b=>b.classList.toggle('selected',b===el));return;}
    if(el.dataset.period){el.parentElement.querySelectorAll('button').forEach(b=>b.classList.toggle('selected',b===el));$('#usage-count').innerHTML=(el.dataset.period==='今天'?'24':el.dataset.period==='近 7 天'?'168':'624')+' <span>条</span>';return;}
    if(el.dataset.mode){if(el.dataset.mode==='remote'){toast('当前正在使用 infOS 客户端');return;}confirm('回到独立空间？','这里的会话会保留，下次连接可以继续。','switch-local','前往独立空间');}
  });
  actions['switch-local']=()=>{closeSheet();const layer=document.createElement('div');layer.className='mode-transition';layer.innerHTML=`<div class="mode-orbit">${i('bow')}</div><h2>回到熟悉的小房间</h2><p>小提示：喜欢的立绘和 Live2D 可以随时切换。</p>`;$('#device').append(layer);clearTimeout(modeTimer);modeTimer=setTimeout(()=>{layer.remove();navigate('local');},2200);};
  document.addEventListener('input',e=>{
    const el=e.target;if(P.materialInput?.(el))return;if(el.dataset.search){const text=el.value.trim().toLowerCase();app.querySelectorAll('[data-searchable]').forEach(x=>x.hidden=!x.dataset.searchable.toLowerCase().includes(text));return;}
    if(el.dataset.draft){S[el.dataset.draft==='group'?'groupDraft':'draft']=el.value;el.style.height='auto';el.style.height=Math.min(el.scrollHeight,104)+'px';return;}
    if(el.dataset.range){const out=document.getElementById(el.dataset.range);if(out)out.textContent=el.value+(el.dataset.suffix||'');if(el.id==='surface-range')document.documentElement.style.setProperty('--surface-opacity',String(Number(el.value)/100));markDirty();return;}
    if(el.dataset.field&&!el.closest('.sheet'))markDirty();
    if(el.dataset.field==='endpoint'||el.dataset.field==='pairing-code')P.updateConnectionForm?.(overlay);
  });
  document.addEventListener('change',e=>{const el=e.target;if(P.materialChange?.(el))return;if(el.id==='trust-check')$('#trust-button').disabled=!el.checked;if(el.id==='http-consent')P.updateConnectionForm?.(overlay);if(el.dataset.toggle==='motion'){S.reduced=!el.checked;applyTheme();}else if(el.dataset.toggle){if(editable.includes(S.page))markDirty();else toast('设置已更新');}if(el.dataset.tool){S.tools[el.dataset.tool]=el.checked;el.closest('.tool-family')?.classList.toggle('disabled',!el.checked);toast(el.checked?'工具已启用':'工具已停用');}});
  document.addEventListener('submit',e=>{if(e.target.matches('[data-form]')){e.preventDefault();send(e.target);}});
  document.addEventListener('keydown',e=>{if(e.key==='Escape'&&overlay.children.length){e.preventDefault();actions['close-sheet']();}if(e.key==='Tab'&&overlay.children.length){const focusables=[...overlay.querySelectorAll('button:not(:disabled),input,textarea,summary,[tabindex="0"]')],first=focusables[0],last=focusables.at(-1);if(e.shiftKey&&document.activeElement===first){e.preventDefault();last?.focus();}else if(!e.shiftKey&&document.activeElement===last){e.preventDefault();first?.focus();}}});
  let pressTimer;app.addEventListener('pointerdown',e=>{longPressHandled=false;if(e.target.closest('.thread-card'))pressTimer=setTimeout(()=>{longPressHandled=true;actions['thread-menu']();},550);});for(const event of ['pointerup','pointercancel','pointermove'])app.addEventListener(event,()=>clearTimeout(pressTimer));
  // 浏览器后退只恢复已有历史条目，避免再次压入同一路由。
  function restoreHistory(){const id=location.hash.slice(1);if(!P.pages[id]||id===S.page)return;if(S.dirty&&editable.includes(S.page)){history.pushState({page:S.page},'',`#${S.page}`);navigate(id,'back');return;}remember();S.dirty=false;navHistory.pop();if(overviewVisible)toggleOverview(false);render(id,'back');}
  window.addEventListener('popstate',restoreHistory);
  window.addEventListener('hashchange',restoreHistory);
  try{const saved=JSON.parse(localStorage.getItem('peropero-design-navigation-theme'));if(saved&&window.Chromatic.packs.some(x=>x[0]===saved.pack))Object.assign(S,{pack:saved.pack,dark:!!saved.dark,reduced:!!saved.reduced});}catch{}
  $('#palette-picker').innerHTML=window.Chromatic.packs.map(([id,label,color])=>`<button data-pack="${id}" style="--swatch:${color}" aria-label="${label}" title="${label}"></button>`).join('');
  $('#page-index').innerHTML=P.groups.map(([label,ids])=>`<section class="index-group"><h2>${label}</h2>${ids.map(id=>`<button data-go="${id}">${i(P.pages[id].icon)}${id==='threads'?'单聊列表':id==='groups'?'群聊列表':id==='files'?'文件列表':id==='memories'?'记忆列表':id==='knowledge'?'知识库':P.pages[id].title}<small>${String(Object.keys(P.pages).indexOf(id)+1).padStart(2,'0')}</small></button>`).join('')}</section>`).join('');
  document.querySelectorAll('[data-symbol]').forEach(el=>el.innerHTML=i(el.dataset.symbol));
  function fitPreview(){document.documentElement.style.setProperty('--preview-scale',String(Math.min(1,Math.max(.68,(innerHeight-190)/860))));}
  window.addEventListener('resize',fitPreview);fitPreview();
  Object.assign(actions,P.guidanceActions({toast,hideToast,showSheet,closeSheet,confirm,menu,navigate,remember,render,overlay,app}));
  Object.assign(actions,P.materialActions({toast,navigate,remember,render,showSheet}));
  Object.assign(actions,P.flowActions({toast,navigate,showSheet}));
  applyTheme();const initial=P.pages[location.hash.slice(1)]?location.hash.slice(1):'home';history.replaceState({page:initial},'',`#${initial}`);render(initial);
})();
