(function prototype(){
  const D=ThemeStudy, {state:s,pages,icon,esc,button,ib}=D
  const select=id=>document.getElementById(id)
  const primary=select('primary'),comparison=select('comparison')
  const positions=new Map(),formValues=new Map()
  const roots={remote:[['home','小屋','home'],['chats','会话','chat'],['tasks','任务','tasks'],['library','资料','library'],['settings','管理','settings']],
    local:[['home','小屋','home'],['chats','会话','chat'],['settings','设置','settings']]}
  const parent={chats:'home',rooms:'chats',chat:'chats',group:'rooms',thread:'chat',inbox:'home',task:'tasks',newtask:'tasks',resource:'library',themes:'settings',roles:'settings',models:'settings',mind:'settings',skills:'settings',plugins:'settings',connections:'settings',pair:'connections',rich:'chat',settings:'home',tasks:'home',library:'home',system:'home'}
  const hints={rich:'切换图文、路线与代码，查看同一条消息的不同内容。',home:'插画集中在小屋，常用入口紧接在房间下方。',chats:'主会话按伙伴聚合，侧会话按话题排列。',rooms:'先选房间，再查看成员与消息。',chat:'发送、展开思考，或编辑刚发出的消息。',group:'点右上角查看房间成员。',tasks:'切换任务分类，或按负责伙伴筛选。',library:'文件、记忆和文档各自保留独立入口。',themes:'每个主题包含完整的背景、表面、文字与状态色。',pair:'配对使用样例设备信息。',system:'同一组组件在两个主题中使用对应的语义颜色。'}
  let mode='remote',id='home',arg='',route='',serial=0,firstFrame=true,sheetTrigger=''
  function valid(key,space){return pages.some(([k,,,,allowed])=>k===key&&(allowed==='both'||allowed===space))}
  function parse(){
    const [space,screen,param]=location.hash.slice(1).split('/')
    const m=space==='local'?'local':'remote'
    let value=''
    try{value=decodeURIComponent(param||'')}catch{/* 地址片段无效时使用页面默认内容。 */}
    return [m,valid(screen,m)?screen:'home',value]
  }
  function path(m,p,value=''){return '#'+m+'/'+p+(value?'/'+encodeURIComponent(value):'')}
  function key(){return ThemeScreens.keyOf(mode,id)}
  function scrollPort(el=primary){return el.querySelector('.scroll')}
  function keep(){const el=scrollPort();if(el&&route)positions.set(route,el.scrollTop)}
  function syncScroll(){
    const live=scrollPort(),copy=scrollPort(comparison)
    if(live&&copy)copy.scrollTop=live.scrollTop
  }
  function directory(){
    const available=pages.filter(p=>valid(p[0],mode))
    const groups=[...new Set(available.map(p=>p[3]))]
    select('directory').innerHTML=groups.map(g=>`<section class="rail-group"><h2>${g}</h2>${available.filter(p=>p[3]===g).map(([k,title,glyph])=>`<button data-jump="${k}" ${k===id?'aria-current="page"':''}>${icon(glyph)}${title}</button>`).join('')}</section>`).join('')
    select('page-choice').innerHTML=groups.map(g=>`<optgroup label="${g}">${available.filter(p=>p[3]===g).map(([k,title])=>`<option value="${k}">${title}</option>`).join('')}</optgroup>`).join('')
    select('page-choice').value=id
    const meta=available.find(p=>p[0]===id)
    select('page-name').textContent=meta[1]
    select('section-name').textContent=meta[3]
    select('page-note').textContent=mode==='local'&&id==='chats'?'选择伙伴或进入家庭群聊。':hints[id]||'使用页内返回，回到刚才的阅读位置。'
    select('space').value=mode
    select('theme-choice').value=s.theme
    const states=['正常']
    if(mode==='remote'&&['home','chat','group','connections'].includes(id))states.push('离线')
    if(['inbox','library','mind'].includes(id))states.push('空列表')
    const scenario=select('scenario')
    scenario.innerHTML=states.map(n=>`<option>${n}</option>`).join('')
    if(!states.includes(s.scenario))s.scenario='正常'
    scenario.value=s.scenario
    scenario.parentElement.hidden=states.length===1
  }
  function label(theme){return theme==='light'?'标准浅色':'标准暗色'}
  function phone(el,theme,motion){
    el.dataset.theme=theme;el.dataset.page=id;el.dataset.reduce=String(s.reduce)
    const v=ThemeScreens.page(id,mode,theme)
    const active=id==='rooms'?'chats':id
    const nav=v.back===false?`<nav class="mobile-nav" aria-label="主导航">${roots[mode].map(([dest,title,glyph])=>`<button data-do="go:${dest}" ${dest===active?'aria-current="page"':''}><span>${icon(glyph)}</span>${title}</button>`).join('')}</nav>`:''
    const banner=mode==='remote'&&s.scenario==='离线'?`<div class="notice">${icon('connection')}连接已中断${button('重新连接','reconnect','')}</div>`:''
    const sheet=s.sheet?ThemeScreens.sheet(mode,id):null
    el.innerHTML=`<div class="status" aria-hidden="true"><time>15:42</time>${icon('signal')}${icon('wifi')}${icon('battery')}</div>
      <section class="screen ${motion}" ${sheet?'inert':''}>${v.header}<div class="scroll ${v.chat?'chat-scroll':''}" tabindex="0">${banner}${v.body}</div>${v.dock||''}${nav}</section>
      <div class="safe" aria-hidden="true"></div>${sheet?`<div class="sheet-layer"><section class="sheet ${motion==='sheet-open'?'sheet-open':''}" role="dialog" aria-modal="true" aria-label="${esc(sheet[0])}"><header><h3>${esc(sheet[0])}</h3>${ib('close','关闭','dismiss')}</header>${sheet[1]}</section></div>`:''}`
    for(const form of el.querySelectorAll('form:not(.composer)')){
      const data=formValues.get(formKey(form))
      if(data)for(const input of form.elements)if(input.name&&input.name in data)input.value=data[input.name]
    }
  }
  function formKey(form){return [route,s.sheet,s.fieldIndex,form.dataset.form].join('|')}
  function draw(motion='',where=null){
    const old=scrollPort()?.scrollTop||0
    const focus=primary.contains(document.activeElement)?document.activeElement:null
    const focusAction=focus?.dataset.do
    const other=s.theme==='light'?'dark':'light'
    directory()
    select('primary-name').textContent=label(s.theme)
    select('compare-name').textContent=label(other)
    phone(primary,s.theme,motion);phone(comparison,other,motion)
    const list=scrollPort()
    list.scrollTop=where==='end'?list.scrollHeight:(typeof where==='number'?where:old)
    list.addEventListener('scroll',syncScroll,{passive:true})
    syncScroll()
    if(s.sheet&&motion==='sheet-open')primary.querySelector('.sheet .ib')?.focus({preventScroll:true})
    else if(!motion&&focusAction)primary.querySelector(`[data-do="${CSS.escape(focusAction)}"]`)?.focus({preventScroll:true})
  }
  function applyRoute(motion){
    const next=parse()
    mode=next[0];id=next[1];arg=next[2]
    route=path(mode,id,arg)
    if(id==='task'&&s.tasks.some(t=>t.id===arg))s.taskId=arg
    if(id==='resource'&&s.resources.some(r=>r.id===arg))s.resourceId=arg
    if(['group','chat'].includes(id)){
      if(id==='group')s.room=mode==='local'?'家庭群聊':(['客厅','卧室','书房'].includes(arg)?arg:'客厅')
      else{
        const t=s.threads.find(t=>t.id===arg)
        s.role=t?(t.agent||'Nana'):['Nana','Pero'].includes(arg)?arg:mode==='local'?s.localRole:'Nana'
        s.thread=t?t.id:'main'
      }
    }
    if(id==='rich')s.readingRole=['Nana','Pero'].includes(arg)?arg:'Pero'
    if(id==='home'&&mode==='local')s.role=s.localRole
    if(id==='pair'&&motion!=='return')s.pairStep=0
    s.sheet=''
    draw(firstFrame?'arrival':motion,positions.get(route)??(['chat','group'].includes(id)?'end':0))
    firstFrame=false
  }
  function go(dest,value='',space=mode,reset=false){
    if(!valid(dest,space))return
    keep()
    const hash=path(space,dest,value)
    if(hash===location.hash){s.sheet='';draw();return}
    const depth=reset?0:(history.state?.depth||0)+1
    history.pushState({depth},'',hash)
    applyRoute('enter')
  }
  function back(){
    if(s.sheet){dismiss();return}
    if(history.state?.depth>0){history.back();return}
    keep()
    let dest=parent[id]||'home'
    if(!valid(dest,mode))dest='chats'
    history.replaceState({depth:0},'',path(mode,dest));applyRoute('return')
  }
  function dismiss(){
    s.sheet='';draw()
    if(sheetTrigger)primary.querySelector(`[data-do="${CSS.escape(sheetTrigger)}"]`)?.focus({preventScroll:true})
  }
  function openSheet(name,action){
    sheetTrigger=action
    s.sheet=name
    if(name==='filter')s.draftFilter=s.filter
    const formName={editmsg:'editmsg',mind:'mind',newthread:'thread'}[name]
    if(formName)formValues.delete(formKey({dataset:{form:formName}}))
    draw('sheet-open')
  }
  function deliver(text,edit=false){
    const chatKey=key()
    const turns=s.msgs[chatKey]||=[]
    if(turns.some(t=>t.pending))return
    const turn=edit?turns.at(-1):{}
    if(!turn)return
    if(!edit)turns.push(turn)
    const turnIndex=turns.length-1
    Object.assign(turn,{text,reply:'',pending:true})
    const typing=document.activeElement?.hasAttribute('data-draft')
    if(!edit)s.drafts[chatKey]=''
    s.sheet='';draw(edit?'':'just-sent','end')
    if(typing)primary.querySelector('[data-draft]')?.focus({preventScroll:true})
    if(!edit)for(const el of [primary,comparison]){
      el.querySelector(`[data-turn="${turnIndex}"]`)?.previousElementSibling?.classList.add('message-new')
    }
    const replies=edit?['那我们把路线缩短一点。','从东岸走到柳桥，再决定要不要继续。']:['好呀，我们按你的节奏走。','到了柳桥，可以坐下来看看河面。']
    const answer=turns.length%2?replies.join(''):replies[0]
    const timer=setInterval(()=>{
      turn.reply=answer.slice(0,turn.reply.length+4)
      turn.pending=turn.reply.length<answer.length
      if(!turn.pending)clearInterval(timer)
      if(key()!==chatKey||!['chat','group'].includes(id))return
      // 仅更新当前回复，让用户继续输入或阅读之前的消息。
      for(const el of [primary,comparison]){
        const message=el.querySelector(`[data-turn="${turnIndex}"]`)
        if(!message)continue
        const port=scrollPort(el)
        const follow=port.scrollHeight-port.clientHeight-port.scrollTop<45
        message.querySelector('.prose').textContent=turn.reply
        if(!turn.pending){
          message.querySelector('.reply-dots')?.remove()
          if(!el.querySelector('.chat-actions')&&s.scenario!=='离线')port.insertAdjacentHTML('beforeend',`<div class="chat-actions">${button('编辑并重新生成','sheet:editmsg','link')}</div>`)
          const send=el.querySelector('button[aria-label="发送消息"]')
          if(send)send.disabled=s.scenario==='离线'
        }
        if(follow)port.scrollTop=port.scrollHeight
      }
    },65)
  }
  function submit(event){
    event.preventDefault()
    const form=event.target,kind=form.dataset.form
    const values=Object.fromEntries(new FormData(form))
    for(const field of form.querySelectorAll('[required]')){
      if(!field.value.trim()){field.setCustomValidity('请填写内容');field.reportValidity();return}
    }
    if(kind==='chat'||kind==='editmsg'){
      const text=String(kind==='chat'?values.message:values.text).trim()
      if(text&&s.scenario!=='离线')deliver(text,kind==='editmsg')
    }else if(kind==='newtask'){
      const task={id:'draft'+(++serial),name:values.title.trim(),body:values.body.trim(),agent:values.agent,tab:'当前',time:'刚刚',status:'进行中'}
      s.tasks.unshift(task);s.taskTab='当前';s.filter='全部角色'
      formValues.delete(formKey(form));go('task',task.id)
    }else if(kind==='thread'){
      const t={id:'side'+(++serial),name:values.title.trim(),agent:s.role}
      s.threads.push(t);go('chat',t.id)
    }else if(kind==='mind'){
      const f={name:values.name.trim(),value:values.value.trim()}
      if(s.fieldIndex<0)s.mind.push(f);else s.mind[s.fieldIndex]=f
      s.scenario='正常'
      dismiss()
    }
  }
  function action(raw){
    const cut=raw.indexOf(':')
    const verb=cut<0?raw:raw.slice(0,cut),value=cut<0?'':raw.slice(cut+1)
    switch(verb){
      case 'go':return go(value)
      case 'back':return back()
      case 'dismiss':return dismiss()
      case 'sheet':return openSheet(value,raw)
      case 'theme':s.theme=value;break
      case 'mode':s.scenario='正常';return go('home','',value,true)
      case 'channel':return go(value==='群聊'?'rooms':'chats')
      case 'main':return go('chat',value)
      case 'side':return go('chat',value)
      case 'room':return go('group',value)
      case 'partner':s.role=value;s.localRole=value;s.sheet='';return draw('partner-change')
      case 'portrait':s.profile=value;return openSheet('portrait',raw)
      case 'extension':s.extension=value;return openSheet('extension',raw)
      case 'extensiontoggle':{const items=id==='skills'?s.skills:s.plugins;items[s.extension]=!items[s.extension];break}
      case 'threadskill':s.threadSkills[value]=!s.threadSkills[value];break
      case 'tasktab':s.taskTab=value;break
      case 'filterclear':s.filter='全部角色';break
      case 'draftfilter':s.draftFilter=value;break
      case 'applyfilter':s.filter=s.draftFilter;return dismiss()
      case 'librarytab':s.libraryTab=value;return draw('',0)
      case 'rich':s.richTab='图文手记';return go('rich',value)
      case 'richtab':s.richTab=value;return draw('',0)
      case 'richwrap':s.richWrap=!s.richWrap;break
      case 'richsource':s.richSource=!s.richSource;break
      case 'resource':case 'task':return go(verb,value)
      case 'approve':{const task=s.tasks.find(t=>t.id===s.taskId);if(task){task.status='已完成';task.tab='历史'}return draw('approved')}
      case 'model':s.model=value;return dismiss()
      case 'effort':s.effort=value;return dismiss()
      case 'trace':{const traceKey=key()+value;s.openTraces[traceKey]=!s.openTraces[traceKey];break}
      case 'mind':s.fieldIndex=value==='new'?-1:Number(value);return openSheet('mind',raw)
      case 'reduce':s.reduce=!s.reduce;break
      case 'reconnect':s.scenario='正常';break
      case 'pair':
        if(value==='start'){s.pairStep=0;return go('pair')}
        if(value==='prev')s.pairStep=Math.max(0,s.pairStep-1)
        else if(s.pairStep<2)s.pairStep++
        else {s.scenario='正常';return go('home','','remote',true)}
        break
      default:return
    }
    draw()
  }
  primary.addEventListener('click',event=>{
    const target=event.target.closest('[data-do]')
    if(target){
      if(target.tagName==='A'){
        if(event.metaKey||event.ctrlKey||event.shiftKey||event.altKey)return
        event.preventDefault()
      }
      action(target.dataset.do)
    }
    else if(event.target.classList.contains('sheet-layer'))dismiss()
  })
  primary.addEventListener('submit',submit)
  function mirror(event){
    const el=event.target, form=el.form
    if(!form||!el.name)return
    if(el.setCustomValidity)el.setCustomValidity('')
    if(el.hasAttribute('data-draft'))s.drafts[key()]=el.value
    else formValues.set(formKey(form),Object.fromEntries(new FormData(form)))
    const other=comparison.querySelector(`form[data-form="${form.dataset.form}"] [name="${el.name}"]`)
    if(other)other.value=el.value
  }
  primary.addEventListener('input',mirror)
  primary.addEventListener('change',mirror)
  document.addEventListener('keydown',e=>{
    if(e.key==='Escape'&&s.sheet){e.preventDefault();dismiss()}
    if(e.key==='Tab'&&s.sheet){
      const items=[...primary.querySelectorAll('.sheet :is(button,input,textarea,select):not(:disabled)')]
      if(items.length&&e.shiftKey&&document.activeElement===items[0]){e.preventDefault();items.at(-1).focus()}
      if(items.length&&!e.shiftKey&&document.activeElement===items.at(-1)){e.preventDefault();items[0].focus()}
    }
  })
  select('directory').addEventListener('click',e=>{const target=e.target.closest('[data-jump]');if(target)go(target.dataset.jump)})
  select('page-choice').addEventListener('change',e=>go(e.target.value))
  select('theme-choice').addEventListener('change',e=>action('theme:'+e.target.value))
  select('space').addEventListener('change',e=>action('mode:'+e.target.value))
  select('scenario').addEventListener('change',e=>{s.scenario=e.target.value;draw()})
  window.addEventListener('popstate',()=>{keep();applyRoute('return')})
  select('brand-icon').innerHTML=icon('bow')
  if(!location.hash)history.replaceState({depth:0},'',path('remote','settings'))
  applyRoute('')
})()
