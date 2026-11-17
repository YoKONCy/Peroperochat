const ThemeScreens = (function screens(D){
  const {icon,esc,assets,avatar,button,ib,pict,row,group,stack,head,tabs,empty,field,state:s}=D
  const view=(title,body,options={})=>({header:head(title,options.action||'',options.back!==false,options.sub||''),body,...options})
  const kv=(key,value)=>`<div class="kv"><span>${esc(key)}</span><b>${esc(value)}</b></div>`
  const pill=(label,tone='')=>`<span class="pill ${tone}">${esc(label)}</span>`
  const opt=(label,action,selected)=>`<button class="option" data-do="${esc(action)}" aria-pressed="${selected}"><span>${esc(label)}</span>${selected?icon('check'):''}</button>`
  const toggle=(label,action,enabled)=>`<button class="option" data-do="${esc(action)}" aria-pressed="${enabled}"><span>${esc(label)}</span><span class="switch"><i></i></span></button>`
  const form=(name,content,submit)=>`<form data-form="${name}">${content}<button class="primary wide" type="submit">${submit}</button></form>`
  const keyOf=(mode,page)=>[mode,page,page==='group'?s.room:s.role,page==='group'?'':s.thread].join('/')
  const threadTitle=(mode)=>mode==='local'?`${s.role} 的对话`:s.thread==='main'?'主会话':(s.threads.find(t=>t.id===s.thread)?.name||'侧会话')
  const actor=()=>s.role
  const offline=(mode)=>mode==='remote'&&s.scenario==='离线'
  function home(mode,theme){
    const local=mode==='local',name=local?s.localRole:s.role,nana=name==='Nana'
    const room=nana?(theme==='light'?'nana-day-room.png':'moonlit-room.png'):(theme==='light'?'sunlit-room.png':'pero-night-room.png')
    const hero=`<figure class="room-print"><div class="hero ${local?'local-room':''}"><img class="hero-scene" src="${assets}companion/${room}" alt=""><img class="hero-person" src="${assets}companion/${nana?'nana-standing.png':'pero-standing.svg'}" alt=""></div><figcaption><div><small>我的伙伴</small><h3>${name}</h3></div>${local?button('找她聊聊 '+icon('arrow'),'main:'+name,'room-link'):button('角色资料 '+icon('next'),'portrait:'+name,'room-link')}</figcaption></figure>`
    const resume=`<button class="resume" data-do="main:${name}">${avatar(name)}<span class="row-copy"><small>上次聊到</small><b>周末去河边走走</b></span>${icon('arrow')}</button>`
    const attention=local?group('小屋里的伙伴',row(avatar(nana?'Pero':'Nana'),nana?'Pero':'Nana','切换当前伙伴','partner:'+(nana?'Pero':'Nana'))+row(stack(['Pero','Nana']),'家庭群聊','两位伙伴都在','go:group')):
      group('今天',row(pict('tasks'),s.tasks[0].name,s.tasks[0].status,'task:'+s.tasks[0].id)+row(pict('file'),'河畔散步路线.md','最近打开的文件','resource:map'))
    return {header:`<header class="room-head"><div><small>${local?'独立空间':'家里的 infOS'}</small><h2>${local?'我的小屋':'小屋'}</h2></div>${local?ib('people','切换伙伴','sheet:partners'):ib('bell','通知','go:inbox')}</header>`,body:hero+(local?'':resume)+attention,back:false}
  }
  function chats(mode,rooms,theme){
    const local=mode==='local'
    const nav=local?'':tabs(['单聊','群聊'],rooms?'群聊':'单聊','channel:')
    if(rooms){
      const places=[['客厅','sunlit-room.png','pero-night-room.png',['Nana','Pero']],['卧室','nana-day-room.png','moonlit-room.png',['Nana']],['书房','pero-twilight-room.png','nana-twilight-room.png',[]]]
      const rows=places.map(([name,day,night,members])=>`<button class="row place" data-do="room:${name}"><img class="room-thumb" src="${assets}companion/${theme==='light'?day:night}" alt=""><span class="row-copy"><b>${name}</b><small>${members.length?members.join('、')+' 在场':'暂时无人'}</small></span>${stack(members)}${icon('next')}</button>`).join('')
      return view('会话',nav+group('我的据点',rows),{back:false})
    }
    const main=['Nana','Pero'].map(n=>row(avatar(n),n,n==='Nana'?'周末去河边走走':'出门清单已经整理好了','main:'+n,n==='Nana'?'15:42':'昨天')).join('')
    const other=local?group('群聊',row(stack(['Nana','Pero']),'家庭群聊','Nana、Pero','go:group')):
      group('侧会话',s.threads.map(t=>row(pict('branch'),t.name,`${t.agent||'Nana'} · 独立话题`,'side:'+t.id)).join(''),button('新建','sheet:newthread','link'))
    return view('会话',nav+group(local?'伙伴':'主会话',main)+other,{back:false})
  }
  const trace=(key,label,text,msgKey)=>`<div class="trace"><button data-do="trace:${key}" aria-expanded="${!!s.openTraces[msgKey+key]}">${icon(key==='thought'?'thought':'tool')}${esc(label)}${icon('down')}</button>${s.openTraces[msgKey+key]?`<p>${esc(text)}</p>`:''}</div>`
  function chat(mode,isGroup){
    const local=mode==='local',key=keyOf(mode,isGroup?'group':'chat'),name=isGroup?(s.room==='卧室'?'Nana':'Pero'):actor()
    const emptyRoom=isGroup&&s.room==='书房'&&!local
    const freshSide=!local&&!isGroup&&s.thread!=='main'&&s.thread!=='rain'
    const title=isGroup?(local?'家庭群聊':s.room):actor()
    const toolbar=isGroup?ib('people','房间成员','sheet:members'):(local?ib('profile','角色资料','go:roles'):ib('tuning','会话设置','go:thread'))
    const by=(who,time,pending=false)=>`<div class="byline">${avatar(who)}${who}${pending?'<span class="reply-dots" role="status" aria-label="正在回复"><i></i><i></i><i></i></span>':''}<small>${time}</small></div>`
    const example=emptyRoom||freshSide?'':`<div class="date">今天 · 15:40</div><div class="chat-item mine">周末想去河边走走，有没有轻松一点的路线？</div>
      <article class="chat-item">${by(name,'15:41')}<div class="trace-strip">${trace('thought','思考摘要 · 3 秒','已参考你喜欢树荫、想少走台阶的偏好。',key)}${trace('tools','工具调用 · 2 步','读取散步偏好 → 查看路线文档，两步均已完成。',key)}</div>
      ${journalReading.excerpt(name)}</article>
      ${isGroup&&(local||s.room==='客厅')?`<article class="chat-item">${by('Nana','15:42')}我把旧书店也记下了，可以顺路看看。</article>`:''}`
    const turns=s.msgs[key]||[]
    const more=turns.map((turn,index)=>`<div class="chat-item mine">${esc(turn.text)}</div><article class="chat-item reply-turn" data-turn="${index}">${by(name,'刚刚',turn.pending)}<div class="prose">${esc(turn.reply)}</div></article>`).join('')
    const redo=turns.length&&!turns.at(-1).pending&&!offline(mode)?`<div class="chat-actions">${button('编辑并重新生成','sheet:editmsg','link')}</div>`:''
    const dock=`<form class="composer" data-form="chat"><div class="compose-tools">${button(icon('model')+' '+esc(s.model)+' '+icon('down'),'sheet:model','')}${!local&&!isGroup?button(icon('skill')+' 本次对话的技能','sheet:threadskills',''):''}</div><div class="compose-line"><textarea name="message" data-draft rows="1" maxlength="4000" aria-label="消息" placeholder="${offline(mode)?'草稿保留在手机上':'写点什么…'}">${esc(s.drafts[key]||'')}</textarea><button class="ib" aria-label="发送消息" ${offline(mode)||turns.some(t=>t.pending)?'disabled':''}>${icon('send')}</button></div></form>`
    return view(title,(example||more?'':empty(emptyRoom?'房间里暂时没有伙伴':'可以开始这个新话题了'))+example+more+redo,{action:toolbar,sub:isGroup?'群聊':threadTitle(mode),dock:emptyRoom?'':dock,chat:true})
  }
  function tasks(){
    const items=s.tasks.filter(t=>t.tab===s.taskTab&&(s.filter==='全部角色'||t.agent===s.filter))
    const sections=s.taskTab==='当前'
      ? [['等我确认',items.filter(t=>t.status==='待确认')],['伙伴在做',items.filter(t=>t.status!=='待确认')]]
      : [[{历史:'已完成',定期:'定期安排',提醒:'即将提醒'}[s.taskTab],items]]
    function taskRow(t){
      const review=t.status==='待确认'
      return `<button class="agenda-row ${review?'review-slip':''}" data-do="task:${esc(t.id)}">
        <span class="agenda-text"><b>${esc(t.name)}</b>${review?`<span class="task-excerpt">${esc(t.body)}</span>`:''}<small>${avatar(t.agent)}${esc(t.agent)}<span>·</span>${esc(t.time)}</small></span>
        <span class="agenda-open">${review?'去确认':icon(t.status==='已完成'?'check':'next')}${review?icon('next'):''}</span></button>`
    }
    const filter=s.filter!=='全部角色'?button(esc(s.filter)+' '+icon('close'),'filterclear','filter-label'):''
    const controls=`<div class="desk-actions">${ib('tuning','按伙伴筛选','sheet:filter')}${ib('plus','新建任务','go:newtask')}</div>`
    const lists=sections.filter(([,rows])=>rows.length).map(([label,rows])=>group(label,rows.map(taskRow).join(''),`<span class="index-count">${rows.length}</span>`)).join('')
    return view('任务',tabs(['当前','历史','定期','提醒'],s.taskTab,'tasktab:')+filter+(lists||empty('这里还没有任务',s.filter==='全部角色'?button('新建任务','go:newtask','link'):button('显示所有伙伴','filterclear','link'))),{back:false,action:controls})
  }
  function task(){
    const t=s.tasks.find(t=>t.id===s.taskId)||s.tasks[0]
    const needs=t.status==='待确认',finished=t.status==='已完成'
    const result=s.resources.find(r=>r.id===t.result)
    const desc=finished&&result?'已保存到资料。':t.body||(t.id==='books'?'按月份归集读书笔记，并保留每篇的原文。':'')
    const steps=needs?['读取散步偏好','整理路线草稿','等你确认保存']:finished?['完成整理','保存结果']:t.tab==='提醒'?['设定提醒','到点提醒']:['收到安排',t.status]
    const timeline=`<ol class="task-timeline">${steps.map((step,i)=>`<li ${i===steps.length-1&&!finished?'aria-current="step"':''}>${icon(i===steps.length-1&&!finished?'clock':'check')}<span>${esc(step)}</span></li>`).join('')}</ol>`
    const cover=`<article class="task-docket"><div>${finished?'<span class="completion-mark">'+icon('check')+'</span>':''}${pill(t.status,needs?'warn':finished?'good':'')}</div><h3>${esc(t.name)}</h3>${desc?`<p>${esc(desc)}</p>`:''}<small>${avatar(t.agent)}${esc(t.agent)} · ${esc(t.time)}</small></article>`
    const attachment=result?group(needs?'待保存的草稿':'任务结果',row(pict('file'),result.name,result.info,'resource:'+result.id)):''
    return view('任务详情',cover+attachment+(needs?button('确认保存文档','approve','primary wide'):'')+group('进度',timeline))
  }
  function library(){
    const items=s.scenario==='空列表'?[]:s.resources.filter(r=>r.type===s.libraryTab)
    const section={文件:'会话附件',记忆:'伙伴记下的事',文档:'手记与文档'}[s.libraryTab]
    const rows=items.map(r=>{
      const graphic=r.id==='map'?`<img class="shelf-sketch" src="${journalReading.imagePath}" alt="" width="68" height="52">`:pict(r.type==='文件'?'file':r.type==='文档'?'notebook':'thought')
      return `<button class="shelf-row" data-kind="${r.type}" data-do="resource:${r.id}">${graphic}<span class="shelf-copy"><b>${esc(r.name)}</b><span>${esc(r.body)}</span><small>${esc(r.agent)} · ${esc(r.info)}</small></span>${icon('next')}</button>`
    }).join('')
    const contents=items.length?group(section,rows,`<span class="index-count">${items.length}</span>`):empty('还没有'+s.libraryTab)
    return view('资料',tabs(['文件','记忆','文档'],s.libraryTab,'librarytab:')+contents,{back:false})
  }
  function resource(){
    const r=s.resources.find(r=>r.id===s.resourceId)||s.resources[0]
    const meta=`<div class="reading-author">${avatar(r.agent)}<b>${esc(r.agent)}</b><span>${esc(r.updated)} · ${esc(r.info)}</span></div>`
    let body=`<p>${esc(r.body)}</p>`
    if(r.id==='map')body+=journalReading.illustration()+'<h4>步行顺序</h4><ol><li>从东岸入口沿河向北。</li><li>在柳桥休息，或过桥走到旧书店。</li></ol>'
    if(r.type==='记忆')body=`<blockquote>${esc(r.body)}</blockquote>`
    const article=`<article class="folio" data-kind="${r.type}"><header><small>${r.type}</small><h3>${esc(r.name)}</h3>${meta}</header><div class="prose rich-prose">${body}</div></article>`
    return view(r.type==='文件'?'文件预览':r.type+'详情',article+group('来自会话',row(avatar(r.agent),r.topic,r.agent,'main:'+r.agent)))
  }
  function settings(mode,theme){
    const local=mode==='local'
    const entry=(symbol,title,to,tail='')=>row(symbol?pict(symbol):'',title,'','go:'+to,tail)
    const cameo=`<button class="photo-note" data-do="portrait:Pero" aria-label="查看 Pero 的角色资料"><img src="${journalArt.portrait}" width="64" height="52" alt=""><span aria-hidden="true">Pero</span></button>`
    return view(local?'设置':'管理',group('外观与设备',entry('theme','主题','themes',theme==='light'?'标准浅色':'标准暗色')+entry('','连接与模式','connections',local?'独立空间':'家里的 infOS'))+
      group('角色与模型',entry('profile','角色资料','roles')+entry('','聊天模型','models',s.model)+(local?'':entry('','回复前的叮嘱','mind')))+
      (local?'':group('扩展',entry('skill','技能','skills')+entry('','插件','plugins'))),{back:false,action:cameo})
  }
  function themes(theme){
    const cards=['light','dark'].map(key=>`<button class="theme-card" aria-pressed="${key===theme}" data-do="theme:${key}"><span class="theme-sample ${key}"><i></i><i></i><i></i></span><b>标准${key==='light'?'浅':'暗'}色</b>${key===theme?icon('check'):''}</button>`).join('')
    return view('主题',`<div class="theme-list">${cards}</div>`+group('动效',toggle('减少动态效果','reduce',s.reduce)))
  }
  function connections(mode){
    return view('连接与模式',group('当前空间',row(pict(mode==='local'?'phone':'screen'),mode==='local'?'本机独立空间':'家里的 infOS',mode==='local'?'使用手机上保存的角色和对话':offline(mode)?'连接已中断':'电脑在线 · 已配对','sheet:spaceinfo'))+
      group('切换空间',mode==='remote'?row(pict('phone'),'进入独立空间','打开本机的角色和对话','mode:local'):row(pict('screen'),'进入家里的 infOS','打开电脑上的会话、任务和资料','mode:remote'))+
      button('配对另一台电脑','pair:start','secondary wide'))
  }
  function pair(){
    const idx=s.pairStep
    const titles=['准备电脑','取得配对信息','确认设备']
    const descriptions=['在电脑端 infOS 的「局域网与设备」里生成配对信息。手机和电脑可以先连到同一个 Wi-Fi。','在手机上打开电脑生成的配对链接。这里可以使用样例体验下一步。','进入后会显示电脑上的内容。本机对话仍保存在独立空间。']
    return view('连接 infOS',`<div class="detail-hero">${pict(idx===1?'connection':'screen')}<small class="meta">第 ${idx+1} 步，共 3 步</small><h3>${titles[idx]}</h3><p>${descriptions[idx]}</p></div>`+
      (idx===2?group('即将连接',kv('电脑','家里的 infOS')+kv('设备','这台手机')):'')+button(['电脑准备好了','使用示例配对链接','连接并进入 infOS'][idx],'pair:next','primary wide')+(idx?button('上一步','pair:prev','link'):''))
  }
  function extensions(kind){
    const skill=kind==='skills',items=skill?s.skills:s.plugins
    return view(skill?'技能':'插件',group(skill?'已安装的技能':'已安装的插件',Object.entries(items).map(([label,on])=>row(pict(skill?'skill':'plugin'),label,'','extension:'+label,on?'已启用':'已停用')).join('')))
  }
  function systemPage(theme){
    const colors=theme==='light'?['#fbf7ef','#fffdf7','#302923','#706456','#344f91','#89501c']:['#222430','#2b2d39','#f2e9d9','#c4b9ad','#aec1ff','#eabe82']
    return view('颜色与组件',`<div class="swatches">${['页面','表面','正文','辅助文字','主操作','待处理'].map((name,i)=>`<span><i style="background:${colors[i]}"></i>${name}<code>${colors[i]}</code></span>`).join('')}</div>`+
      group('文字层级',kv('页面标题','30 px · 宋体')+kv('列表正文','14 px · 中等')+kv('辅助信息','11–12 px'))+
      group('状态',`<div class="kv">${pill('进行中')}${pill('已完成','good')}${pill('待确认','warn')}</div>`)+
      group('动效',kv('进入详情 / 返回','170 / 160 ms')+kv('底部抽屉','190 ms')+toggle('减少动态效果','reduce',s.reduce)))
  }
  function page(id,mode,theme){
    switch(id){
      case 'home':return home(mode,theme)
      case 'chats':case 'rooms':return chats(mode,id==='rooms',theme)
      case 'chat':case 'group':return chat(mode,id==='group')
      case 'tasks':return tasks()
      case 'task':return task()
      case 'newtask':return view('新建任务',form('newtask',field('任务名称','title')+field('要完成什么','body','',true)+`<label class="field">负责伙伴<select name="agent"><option>Nana</option><option>Pero</option></select></label>`,'创建任务'))
      case 'library':return library()
      case 'resource':return resource()
      case 'settings':return settings(mode,theme)
      case 'themes':return themes(theme)
      case 'connections':return connections(mode)
      case 'pair':return pair()
      case 'skills':case 'plugins':return extensions(id)
      case 'rich':return journalReading.page(mode,s.readingRole)
      case 'system':return systemPage(theme)
      case 'inbox':return view('通知',s.scenario==='空列表'?empty('暂时没有新通知'):group('今天',row(pict('tasks'),'路线草稿已就绪','Nana · 15:42','task:walk')+row(pict('chat'),'客厅里有新消息','Pero · 14:20','room:客厅')))
      case 'thread':return view('会话设置',group('会话',kv('类型',threadTitle(mode))+kv('伙伴',actor()))+group('回答与工具',row(pict('model'),'聊天模型','','sheet:model',s.model)+row(pict('skill'),'本次对话的技能','','sheet:threadskills'))+button('新建侧会话','sheet:newthread','secondary wide'))
      case 'roles':return view('角色资料',group('我的伙伴',['Nana','Pero'].map(n=>row(avatar(n),n,n==='Nana'?'喜欢散步和旧书店':'喜欢读书和做笔记','portrait:'+n)).join('')))
      case 'models':return view('聊天模型',group('回答设置',row(pict('model'),'使用模型','','sheet:model',s.model)+row(pict('thought'),'思考强度','','sheet:effort',s.effort)))
      case 'mind':{
        const add=ib('plus','写一条叮嘱','mind:new')
        const rows=s.mind.map((v,i)=>row('',v.name,v.value,'mind:'+i)).join('')
        const list=rows&&s.scenario!=='空列表'?group('我的叮嘱',rows):empty('还没给伙伴留过叮嘱',button('写第一条','mind:new','link'))
        return view('回复前的叮嘱','<p class="mind-purpose">告诉伙伴，回答你之前要先考虑什么。</p>'+list,{action:add})
      }

    }
  }
  function sheet(mode,page){
    const kind=s.sheet,chatKey=keyOf(mode,page)
    switch(kind){
      case 'filter':return ['筛选任务',['全部角色','Nana','Pero'].map(n=>opt(n,'draftfilter:'+n,s.draftFilter===n)).join('')+button('应用筛选','applyfilter','primary wide')]
      case 'model':return ['选择模型',['Gemini 3 Pro','本地模型'].map(m=>opt(m,'model:'+m,m===s.model)).join('')]
      case 'effort':return ['思考强度',['快速','标准','深入'].map(n=>opt(n,'effort:'+n,n===s.effort)).join('')]
      case 'members':return ['房间成员',(mode==='remote'&&s.room==='书房'?empty('暂时无人'):(mode==='remote'&&s.room==='卧室'?['Nana']:['Nana','Pero']).map(n=>row(avatar(n),n,'','portrait:'+n)).join(''))]
      case 'partners':return ['切换伙伴',['Nana','Pero'].map(n=>row(avatar(n),n,'','partner:'+n)).join('')]
      case 'portrait':{
        const nana=s.profile==='Nana'
        const picture=nana?assets+'avatars/nana.jpg':journalArt.portrait
        return [s.profile,`<div class="profile-note"><img src="${picture}" width="100" height="100" alt=""><p>${nana?'把想去的地方一点点记下，周末再一起走。':'读完一本书，总想找个人聊一聊。'}</p></div>`+button('进入对话','main:'+s.profile,'primary wide')]
      }
      case 'newthread':return ['新建侧会话',form('thread',field('话题名称','title'),'创建并进入')]
      case 'editmsg':return ['编辑消息',form('editmsg',field('消息内容','text',s.msgs[chatKey]?.at(-1)?.text||'',true),'保存并重新生成')]
      case 'threadskills':return ['本次对话的技能',Object.entries(s.threadSkills).filter(([n])=>s.skills[n]).map(([n,on])=>toggle(n,'threadskill:'+n,on)).join('')||empty('还没有启用的技能')]
      case 'extension':{const table=page==='skills'?s.skills:s.plugins;return [s.extension,kv('来源','家里的 infOS')+kv('版本','1.0.0')+toggle('启用','extensiontoggle',table[s.extension])]}
      case 'mind':{
        const item=s.mind[s.fieldIndex]
        return [item?'编辑叮嘱':'写一条叮嘱',form('mind',field('什么时候用','name',item?.name,false,'比如安排出门时')+field('想让伙伴先考虑什么','value',item?.value,true,'先问问我想走多久，再帮我选路线。'),'保存叮嘱')]
      }
      case 'bridge':return ['柳桥与旧书店',`<img class="sketch-detail" src="${journalReading.imagePath}" width="800" height="534" alt="树荫下的柳桥，桥的右边是旧书店">`]
      case 'spaceinfo':return ['当前空间',kv('数据位置',mode==='local'?'这台手机':'家里的电脑')+kv('运行模式',mode==='local'?'独立空间':'infOS 客户端')]
      default:return ['', '']
    }
  }
  return {page,sheet,keyOf}
})(ThemeStudy)
