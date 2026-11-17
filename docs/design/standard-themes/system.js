const ThemeStudy = (function system(){
  const icon = studyIcon
  const esc = text => String(text ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]))
  const assets = '../../../public/'
  const avatar = name => `<img class="avatar" src="${assets}avatars/${name === 'Nana' ? 'nana' : 'pero'}.jpg" alt="">`
  const pict = name => `<span class="pict" aria-hidden="true">${journalArt.objects[name] || icon(name)}</span>`
  const button = (label, action, cls='secondary') => `<button type="button" class="${cls}" data-do="${esc(action)}">${label}</button>`
  const ib = (name,label,action) => `<button type="button" class="ib" data-do="${esc(action)}" aria-label="${esc(label)}" title="${esc(label)}">${icon(name)}</button>`
  const row = (lead, title, sub, action, end='') => `<button class="row" data-do="${esc(action)}">${lead}<span class="row-copy"><b>${esc(title)}</b>${sub ? `<small>${esc(sub)}</small>` : ''}</span>${end ? `<span class="row-end">${esc(end)}</span>` : ''}${icon('next')}</button>`
  const group = (label, rows, after='') => `<section class="group"><header><h3>${esc(label)}</h3>${after}</header><div class="lines">${rows}</div></section>`
  const stack = names => `<span class="stack">${names.map(avatar).join('')}</span>`
  const head = (title, action='', back=false, sub='') => `<header class="screen-head ${back ? 'nested' : ''}">${back ? ib('back','返回','back') : ''}<div><h2>${esc(title)}</h2>${sub ? `<small>${esc(sub)}</small>` : ''}</div>${action}</header>`
  const tabs = (values, active, prefix) => `<div class="tabs">${values.map(value=>`<button aria-pressed="${value===active}" data-do="${esc(prefix + value)}">${esc(value)}</button>`).join('')}</div>`
  const empty = (title, extra='') => `<div class="empty">${icon('empty')}<b>${esc(title)}</b>${extra}</div>`
  const field = (label,name,value='',long=false,hint='') => `<label class="field">${esc(label)}${long ? `<textarea name="${name}" rows="5" placeholder="${esc(hint)}" required maxlength="1600">${esc(value)}</textarea>` : `<input name="${name}" value="${esc(value)}" placeholder="${esc(hint)}" required maxlength="180">`}</label>`
  const state = {
    theme:'light', sheet:'', filter:'全部角色', draftFilter:'全部角色', taskTab:'当前', libraryTab:'文件',
    scenario:'正常', query:'', model:'Gemini 3 Pro', effort:'标准', taskId:'walk', resourceId:'map',
    richTab:'图文手记', richWrap:false, richSource:false,
    role:'Nana', localRole:'Nana', thread:'main', room:'客厅', pairStep:0, reduce:false,
    threads:[{id:'rain',name:'下雨时的备选路线'}], msgs:{}, drafts:{}, openTraces:{},
    skills:{'散步路线':true,'文档整理':false}, plugins:{'日历':true,'天气':false}, threadSkills:{'散步路线':true,'文档整理':false},
    mind:[{name:'安排出门时',value:'先问问我今天想走多久，再推荐合适的路线。'}],
    tasks:[
      {id:'walk',name:'整理河畔散步路线',agent:'Nana',time:'今天 15:30',tab:'当前',status:'待确认',result:'map',body:'路线草稿已写好，等你决定是否保存。'},
      {id:'books',name:'把读书记录按月份归档',agent:'Pero',time:'今天 14:10',tab:'当前',status:'进行中'},
      {id:'week',name:'汇总本周的日记',agent:'Pero',time:'每周五',tab:'定期',status:'已启用'},
      {id:'lamp',name:'带上相机电池',agent:'Nana',time:'周六 15:00',tab:'提醒',status:'已安排'},
      {id:'list',name:'整理野餐清单',agent:'Nana',time:'昨天',tab:'历史',status:'已完成',result:'kit'}],
    resources:[
      {id:'map',name:'河畔散步路线.md',type:'文件',agent:'Nana',topic:'周末去河边走走',updated:'今天 15:42',info:'Markdown · 2.4 KB',body:'从东岸步道开始，穿过柳桥，沿河走到旧书店。全程约 2.6 公里。'},
      {id:'kit',name:'周末出门清单.txt',type:'文件',agent:'Pero',topic:'出门前的准备',updated:'昨天',info:'纯文本 · 680 B',body:'折叠伞、水杯、耳机和相机备用电池。'},
      {id:'quiet',name:'你偏爱安静的步道',type:'记忆',agent:'Nana',topic:'周末去河边走走',updated:'昨天',info:'个人偏好',body:'你喜欢人少、有树荫的路线。下次安排散步时，优先留出在河边停一停的时间。'},
      {id:'journal',name:'沿河散步手记',type:'文档',agent:'Pero',topic:'旧书店的窗边',updated:'昨天',info:'散步手记',body:'柳桥旁的柳树已经黄了。桥下有一条很少有人走的小路，适合傍晚慢慢走。'}]
  }
  const pages = [
    ['home','小屋','home','小屋与会话','both'],['chats','会话列表','chat','小屋与会话','both'],['rooms','群聊房间','people','小屋与会话','remote'],
    ['chat','单聊','chat','小屋与会话','both'],['group','群聊','people','小屋与会话','both'],['thread','会话设置','tuning','小屋与会话','remote'],
    ['inbox','通知','bell','小屋与会话','remote'],['tasks','任务列表','tasks','任务与资料','remote'],['task','任务详情','tasks','任务与资料','remote'],
    ['newtask','新建任务','plus','任务与资料','remote'],['library','资料列表','library','任务与资料','remote'],['resource','资料详情','file','任务与资料','remote'],
    ['settings','管理','settings','管理与连接','both'],['themes','主题','theme','管理与连接','both'],['roles','角色资料','profile','管理与连接','both'],
    ['models','模型设置','model','管理与连接','both'],['mind','回复前的叮嘱','mind','管理与连接','remote'],['skills','技能','skill','管理与连接','remote'],
    ['plugins','插件','plugin','管理与连接','remote'],['connections','连接与模式','connection','管理与连接','both'],['pair','配对引导','screen','管理与连接','both'],
    ['rich','富文本消息','chat','设计规范','both'],['system','颜色与组件','tuning','设计规范','both']
  ]
  return {icon,esc,assets,avatar,button,ib,pict,row,group,stack,head,tabs,empty,field,state,pages}
})()
