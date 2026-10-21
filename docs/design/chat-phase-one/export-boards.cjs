// 静态画板以 SVG 原语排版导出，不执行 HTML、网页脚本或应用代码。
const fs = require('node:fs');
const path = require('node:path');
const sharp = require(process.env.PPC_DESIGN_SHARP || 'C:/Users/Administrator/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
const here = __dirname;
const source = fs.readFileSync(path.join(here, 'icons.js'), 'utf8');
const icons = Object.fromEntries([...source.matchAll(/^    (\w+): '(.+)',$/gm)].map(match => [match[1], match[2]]));
const colors = {
  light: { ink:'#344b61', muted:'#899bab', dim:'#a5b1be', base:'#eff5f8', surface:'#fdfefe', soft:'#e8f0f6', line:'#dbe6ee', strong:'#bdd3e2', accent:'#5796c1', secondary:'#a8c7df', user:'#dfedf7', thought:'#edf0f8', thinking:'#8b8fab', green:'#558f83', amber:'#b68d52', amberbg:'#fcf7ed', amberline:'#e6d8bc', red:'#bb6b7b' },
  dark: { ink:'#e2dfed', muted:'#9490a8', dim:'#716e85', base:'#211f2d', surface:'#2a2739', soft:'#322c43', line:'#3c364d', strong:'#625470', accent:'#baa8e4', secondary:'#cfb4cf', user:'#463953', thought:'#302c40', thinking:'#b4a2ce', green:'#8fbdad', amber:'#d6b57a', amberbg:'#35302f', amberline:'#65563e', red:'#da9aa7' },
};
const escape = value => String(value).replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;');
const rect = (x,y,w,h,fill,r=0,stroke='none',extra='') => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${r}" fill="${fill}" stroke="${stroke}" ${extra}/>`;
const line = (x,y,x2,y2,color,width=1,extra='') => `<path d="M${x} ${y}H${x2}" fill="none" stroke="${color}" stroke-width="${width}" transform="${y2 === y ? '' : `matrix(1 ${(y2-y)/(x2-x || 1)} 0 1 0 ${-(y2-y)/(x2-x || 1)*x})`}" ${extra}/>`;
const rule = (x,y,w,color) => rect(x,y,w,1,color);
const text = (x,y,value,size=14,color='#344b61',weight=400,extra='') => `<text x="${x}" y="${y}" fill="${color}" font-size="${size}" font-weight="${weight}" ${extra}>${escape(value)}</text>`;
const lines = (x,y,values,size,color,leading=28) => values.map((value,i) => text(x,y+i*leading,value,size,color)).join('');
const ic = (name,x,y,size,color,secondary=color) => `<svg x="${x}" y="${y}" width="${size}" height="${size}" viewBox="0 0 24 24" color="${color}" fill="none" stroke="currentColor" stroke-width="1.65" stroke-linecap="round" stroke-linejoin="round" style="--secondary:${secondary}">${icons[name] || icons.chat}</svg>`;
const avatar = (role,x,y,size=32) => `<svg x="${x}" y="${y}" width="${size}" height="${size}" viewBox="0 0 100 100"><g clip-path="url(#avatar-clip)"><use href="#avatar-${role}"/></g>${rect(1,1,98,98,'none',29,'#c5cadd')}</svg>`;
function baseDefs() {
  const images = ['pero','nana'].map(role => `<image id="avatar-${role}" width="100" height="100" preserveAspectRatio="xMidYMid slice" href="data:image/jpeg;base64,${fs.readFileSync(path.join(here,'../remote-client-prototype/assets',`${role}-avatar.jpg`)).toString('base64')}"/>`).join('');
  return `<defs><linearGradient id="board-gradient" x2="1" y2="1"><stop stop-color="#f1f6fa"/><stop offset="1" stop-color="#f0edf5"/></linearGradient><filter id="phone-shadow" x="-15%" y="-10%" width="130%" height="130%"><feDropShadow dx="0" dy="9" stdDeviation="10" flood-color="#566987" flood-opacity=".09"/></filter><clipPath id="avatar-clip">${rect(0,0,100,100,'white',30)}</clipPath>${images}</defs><style>.wash{fill:currentColor;fill-opacity:.09;stroke:none}.detail{stroke:var(--secondary);stroke-width:1.1;opacity:.75}.jewel{fill:var(--secondary);fill-opacity:.6;stroke-width:.8}text{font-family:'Microsoft YaHei UI','Microsoft YaHei','Segoe UI',sans-serif}svg{overflow:visible}</style>`;
}
function svg(width,height,body) { return `<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">${baseDefs()}${body}</svg>`; }
function speaker(c,role,y,name=role==='pero'?'Pero':'Nana') {
  return avatar(role,20,y,29)+text(58,y+20,name,13,c.accent,600)+ic('bow',name==='Pero'?94:99,y+6,17,c.secondary)+text(283,y+18,'21:08',10,c.dim)+ic('more',333,y+7,17,c.muted);
}
function user(c,y,rows,w=259) {
  const h=rows.length*27+19, x=354-w;
  return `<path d="M${x+17} ${y}H337Q354 ${y} 354 ${y+17}V${y+h-9}l4 3-4 1v5H${x+17}Q${x} ${y+h} ${x} ${y+h-17}V${y+17}Q${x} ${y} ${x+17} ${y}Z" fill="${c.user}" stroke="${c.line}"/>`+lines(x+15,y+29,rows,15,c.ink,27)+text(320,y+h+16,'你 · 21:07',10,c.dim);
}
function answer(c,role,y,rows) {
  const top=y+41,h=rows.length*28+20;
  return speaker(c,role,y)+`<path d="M25 ${top}h7l5-6 6 6h286q23 0 23 23v${h-46}q0 23-23 23H43q-23 0-23-23v-${h-41}q0-5 5-5Z" fill="${c.surface}" stroke="${c.line}"/>`+lines(37,top+30,rows,16,c.ink,28);
}
function thought(c,y,active=false,w=334) {
  return rect(20,y,w,43,c.thought,18,c.line)+`<circle cx="36" cy="${y+43}" r="3.5" fill="${c.thought}" stroke="${c.line}"/><circle cx="28" cy="${y+49}" r="1.5" fill="${c.thinking}" opacity=".3"/>`+ic('thought',33,y+11,20,c.thinking)+text(61,y+27,active?'在想啦…':'模型思考',12,c.thinking,550)+text(w-102,y+27,active?'接收中':'第 2 轮 · 3.2s',10,c.thinking)+ic('down',w+1,y+16,12,c.thinking);
}
function tool(c,y,name,title,detail,active=false,child=false) {
  const x=child?75:24,size=child?26:32;
  return rect(x,y,size,size,c.base,child?8:11,c.line)+ic(name,x+5,y+5,size-10,active?c.accent:c.green)+text(x+size+11,y+14,title,child?11:13,c.ink,550)+text(x+size+11,y+32,detail,child?10:11,c.muted)+(active?`<circle cx="323" cy="${y+14}" r="3" fill="${c.accent}"/>`:ic('check',316,y+7,13,c.green))+ic('down',341,y+9,11,c.muted)+rule(x+size+12,y+47,354-(x+size+12),c.line);
}
function fileCard(c,y,w=334) {
  return rect(20,y,w,89,c.surface,8,c.line)+rect(21,y+1,29,87,c.soft,3)+text(31,y+52,'MD',10,c.accent,500)+text(64,y+23,'角色生成的文件',10,c.muted)+text(64,y+47,'周末散步计划',15,c.ink,550)+text(64,y+67,'weekend.md · 4.2 KB',10,c.dim)+ic('download',w-8,y+36,19,c.accent)+`<path d="m${w+5} ${y} 15 15h-15Z" fill="${c.soft}"/>`;
}
function button(c,x,y,w,label,name='check',primary=true) {
  const fg=primary?(c===colors.dark?'#272334':c.surface):c.ink;
  return rect(x,y,w,44,primary?c.accent:c.surface,12,primary?c.accent:c.line)+ic(name,x+(w-label.length*12-25)/2,y+14,16,fg)+text(x+(w-label.length*12-25)/2+23,y+28,label,12,fg,550);
}
function approval(c,y) {
  return rect(21,y,333,295,c.amberbg,8,c.amberline)+rect(24,y,327,3,c.amberline,1)+rect(37,y-15,32,32,c.amberbg,11,c.amberline)+ic('shield',43,y-9,20,c.amber)+text(82,y+14,'需要你点个头',10,c.amber)+text(38,y+49,'在远程工作区运行命令',16,c.ink,600)+text(38,y+75,'请确认服务端提供的命令和范围。',12,c.muted)+rect(38,y+93,299,37,c.soft,8,c.amberline)+text(49,y+117,'python render_plan.py',11,c.amber)+text(38,y+155,'执行位置',11,c.muted)+text(233,y+155,'当前会话工作区',11,c.ink)+text(38,y+180,'审批类型',11,c.muted)+text(291,y+180,'终端操作',11,c.ink)+rule(38,y+197,299,c.amberline)+button(c,38,y+212,130,'这次不了','close',false)+rect(177,y+212,160,44,c.amber,12)+ic('check',198,y+226,16,c.amberbg)+text(224,y+240,'允许这一次',12,c.amberbg,550)+text(101,y+277,'只批准本次，不改变会话权限。',10,c.muted);
}
function queue(c,y,w=334) {
  return rect(24,y+4,w-8,188,c.soft,10,c.strong)+rect(20,y,w,185,c.surface,12,c.strong)+rule(21,y+35,w-2,c.strong)+ic('queue',31,y+10,16,c.accent)+text(53,y+24,'待发送',11,c.accent)+rect(95,y+11,17,15,c.soft,4)+text(101,y+22,'2',9,c.accent)+text(202,y+24,'会按顺序，慢慢来',10,c.muted)+ic('down',w+2,y+12,12,c.muted)+text(33,y+61,'01',10,c.accent)+text(58,y+61,'把交通预算也补一下。',12,c.ink)+text(58,y+81,'等待上一条完成',10,c.muted)+ic('send',58,y+92,12,c.accent)+text(75,y+102,'立即提交',10,c.accent)+ic('edit',142,y+92,12,c.accent)+text(160,y+102,'编辑',10,c.accent)+text(211,y+102,'取消',10,c.red)+rule(33,y+114,w-25,c.line)+text(33,y+138,'02',10,c.accent)+text(58,y+138,'晚上想找个安静的地方。',12,c.ink)+text(58,y+158,'还有 1 条在前面',10,c.muted)+ic('stop',269,y+165,11,c.muted)+text(284,y+174,'暂停队列',9,c.muted);
}
function composer(c,y=662) {
  return rect(0,y-4,374,817-y+4,c.base)+rect(12,y,350,128,c.surface,20,c.line)+rect(30,y,30,1,c.secondary)+ic('model',25,y+12,15,c.accent)+text(45,y+24,'Gemini 4 Pro',10,c.accent)+ic('down',115,y+17,9,c.muted)+ic('thought',136,y+13,14,c.muted)+text(155,y+24,'思考 · 高',10,c.muted)+text(276,y+24,'12.4k / 128k',9,c.green)+rule(25,y+36,324,c.line)+ic('plus',24,y+53,20,c.accent)+text(53,y+72,'和伙伴说说你的想法…',13,c.dim)+rect(310,y+46,37,37,c.accent,12)+ic('send',319,y+55,20,c===colors.dark?'#272334':c.surface)+ic('script',25,y+97,13,c.accent)+text(44,y+107,'工作',10,c.accent)+ic('down',69,y+100,9,c.muted)+ic('sliders',95,y+97,13,c.muted)+text(114,y+107,'工具',10,c.muted)+ic('voice',158,y+97,13,c.muted)+text(177,y+107,'语音',10,c.muted)+ic('shield',300,y+99,11,c.dim)+text(316,y+108,'infOS',9,c.dim)+rect(140,801,94,4,c.ink,3, 'none','opacity=".3"');
}
function header(c,role,thread) {
  const name=role==='pero'?'Pero':'Nana';
  return text(25,25,'9:41',12,c.ink,600)+`<path d="M290 24v-3m4 3v-6m4 6v-9m9 3q6-5 12 0m-10 3q4-3 8 0" fill="none" stroke="${c.ink}" stroke-width="1.5" stroke-linecap="round"/>`+rect(331,15,19,10,'none',3,c.ink)+rect(333,17,12,6,c.ink,1)+ic('back',13,62,22,c.muted)+avatar(role,50,51,40)+text(103,70,name,18,c.ink,600)+ic('bow',155,54,21,c.secondary)+`<circle cx="105" cy="85" r="2" fill="${c.green}"/>`+text(113,89,thread,11,c.muted)+rect(325,52,33,33,c.surface,11,c.line)+ic('more',333,60,17,c.muted)+rule(0,108,374,c.line)+rect(50,108,42,2,c.secondary);
}
function sheet(c,type) {
  const top=type==='models'?319:299;
  let out=rect(0,35,374,782,'#101025',0,'none','opacity=".35"')+`<path d="M0 ${top+26}Q0 ${top} 26 ${top}h322q26 0 26 26v${817-top-26}H0Z" fill="${c.surface}" stroke="${c.strong}"/>`+rect(170,top+11,34,3,c.dim,3)+text(22,top+46,'Nana · 当前会话',10,c.muted)+text(22,top+78,type==='models'?'这次，怎么思考？':'编辑这条消息',20,c.ink,550)+rect(321,top+40,30,30,c.soft,10)+ic('close',329,top+48,14,c.muted);
  if(type==='models') {
    out+=lines(22,top+110,['只调整当前会话。选择后保存，','不影响其他会话。'],12,c.muted,21)+text(22,top+166,'当前模型',11,c.muted)+rect(22,top+180,330,70,c.soft,13,c.accent)+ic('model',36,top+200,25,c.accent)+text(74,top+206,'Gemini 4 Pro',13,c.ink,550)+text(74,top+228,'Google · 视觉输入',10,c.muted)+ic('check',322,top+208,15,c.accent)+rect(22,top+260,330,64,c.base,13,c.line)+ic('refresh',36,top+280,22,c.accent)+text(74,top+285,'跟随默认模型',13,c.ink,550)+text(74,top+305,'使用 infOS 的默认配置',10,c.muted)+text(22,top+353,'思考强度',11,c.muted);
    ['跟随模型','低','中','高'].forEach((level,i)=>{out+=rect(22+i*85,top+367,76,44,i===3?c.soft:c.base,11,i===3?c.accent:c.line)+text(22+i*85+38,top+394,level,11,i===3?c.accent:c.muted,400,'text-anchor="middle"');});
    out+=ic('thought',23,top+427,14,c.muted)+text(45,top+438,'选项随模型能力变化，不显示无效档位。',10,c.muted)+rule(22,top+457,330,c.line)+ic('check',111,top+474,13,c.green)+text(132,top+484,'当前配置已同步',10,c.green);
  } else {
    out+=text(22,top+112,'正文先改好，再决定是否重新生成。',12,c.muted)+text(22,top+147,'消息正文',11,c.muted)+rect(22,top+160,330,120,c.base,15,c.strong)+lines(38,top+189,['周六去散步，','下午三点前回来。'],15,c.ink,28)+button(c,22,top+303,330,'仅保存','check',false)+text(27,top+370,'保留后面的对话。',11,c.muted)+rule(22,top+391,330,c.line)+button(c,22,top+413,330,'保存并重新生成','refresh')+lines(27,top+480,['先保存修订，再确认本轮及后续对话','的回滚范围。'],11,c.muted,20);
  }
  return out;
}
function phone(scene,index,x,y) {
  const dark=[1,4,5].includes(index), c=colors[dark?'dark':'light'],role=dark?'nana':'pero';
  const clip=`phone-${index}`;
  let content='';
  if(index===0) content=text(137,140,'今天 · 晚上 9:07',10,c.dim)+user(c,163,['终于忙完了，脑袋有点空空的。'],294)+answer(c,role,260,['回来啦，今天辛苦你了。','先把肩膀放松一点点。','现在可以慢慢呼吸啦。'])+ic('copy',298,411,16,c.muted)+ic('sound',332,411,16,c.muted)+user(c,449,['陪我聊点轻松的吧。'],212)+answer(c,role,543,['如果明天完全不用赶时间，','你最想去哪里待一会儿？']);
  if(index===1) content=text(137,140,'今天 · 晚上 9:07',10,c.dim)+user(c,162,['帮我整理一条轻松的周末路线。'],295)+speaker(c,role,257)+text(26,314,'好吧，那就别把行程塞得太满。',15,c.ink)+thought(c,334)+ic('route',22,398,14,c.muted)+text(42,410,'执行轨迹',11,c.muted)+text(273,410,'点击查看细节',10,c.dim)+`<path d="M40 448v160" stroke="${c.strong}"/>`+tool(c,433,'globe','打开网页','城市公园开放时间 · 310ms')+tool(c,494,'script','运行脚本','整理路线与文件',true)+text(78,558,'内部调用 3 项',10,c.accent)+text(249,558,'2 完成 · 1 进行中',9,c.muted)+`<path d="M67 572v93" stroke="${c.strong}" stroke-dasharray="3 3"/>`+tool(c,574,'search','搜索地点','公园、咖啡馆 · 420ms',false,true)+tool(c,627,'edit','写入计划','weekend.md',true,true);
  if(index===2) content=user(c,130,['把路线排成一张好看的清单吧。'],295)+answer(c,role,216,['排版准备好啦。接下来这一步，','需要你确认一下。'])+approval(c,357);
  if(index===3) content=answer(c,role,134,['路线整理好啦。上午在公园走走，','下午留给咖啡和发呆。'])+thought(c,263)+tool(c,333,'search','搜索地点','找到 4 个结果 · 420ms')+tool(c,395,'edit','写入计划','weekend.md · 26ms')+fileCard(c,465)+rect(91,579,193,34,c.surface,17,c.strong)+ic('down',104,589,14,c.accent)+text(126,602,'有新消息 · 回到底部',11,c.accent);
  if(index===4) content=user(c,147,['接下来，我们慢慢聊。'],235)+answer(c,role,251,['嗯哼。今天想聊什么？']);
  if(index===5) content=user(c,147,['周六去散步，下午四点前回来。'],295)+answer(c,role,251,['下午四点前回来，记住了。']);
  const composerY=index===3?665:665;
  if(index===3) content=answer(c,role,132,['路线整理好啦。上午在公园走走，','下午留给咖啡和发呆。'])+thought(c,261)+fileCard(c,329)+queue(c,458);
  const labelColor=dark?'#83728f':'#7499b4';
  return `<g transform="translate(${x} ${y})">${text(0,-35,scene[0],16,'#4a6178',600)}${text(0,-12,scene[1],11,'#95a2b2')}<g filter="url(#phone-shadow)">${rect(-5,-5,384,827,dark?'#302d3d':'#fdfefe',35,dark?'#534d65':'#d1dce6')}</g><defs><clipPath id="${clip}">${rect(0,0,374,817,'white',30)}</clipPath><clipPath id="body-${clip}">${rect(0,111,374,554,'white')}</clipPath></defs><g clip-path="url(#${clip})">${rect(0,0,374,817,c.base,30)}${header(c,role,index===0||index===4?'今天，慢慢来':'给周末留一点空白')}<g clip-path="url(#body-${clip})">${content}</g>${composer(c,composerY)}${index===4?sheet(c,'models'):index===5?sheet(c,'edit'):''}</g>${text(187,850,dark?'月夜 · 标准暗色':'晴空 · 标准浅色',10,labelColor,400,'text-anchor="middle"')}</g>`;
}
const scenes=[['01 / 日常聊天','软气泡 · 轻陪伴 · 正文优先'],['02 / 思考与执行','云签思考 · 开放轨迹 · NIT 层级'],['03 / 等你确认','确认便笺 · 明确范围 · 单次授权'],['04 / 队列与文件','叠纸队列 · 类型书脊 · 清晰状态'],['05 / 模型与思考强度','半屏抽屉 · 即选即存 · 当前值清晰'],['06 / 消息编辑','保存与重新生成，各自说明影响']];
let overview=rect(0,0,1440,2075,'url(#board-gradient)')+text(65,51,'PeroperoChat',23,'#435d74',600)+text(1370,48,'DESIGN STUDY 01 / 仅示例',10,'#9aa9b9',400,'text-anchor="end"')+rule(65,76,1310,'#dce3ec')+text(65,123,'把对话，变成有温度的小小现场。',32,'#455e77',500)+text(65,158,'晴空的轻快，月夜的俏皮。六个对话瞬间，同一套有辨识度的组件语言。',13,'#8c9cae');
scenes.forEach((scene,index)=>{overview+=phone(scene,index,86+(index%3)*450,243+Math.floor(index/3)*901);});
overview+=rule(65,2033,1310,'#dce3ec')+text(65,2059,'独立 SVG 静态排版 / HTML 中可展开思考、轨迹、队列与设置',10,'#8d9eaf')+text(1375,2059,'仅设计演示 · 未实装',10,'#8d9eaf',400,'text-anchor="end"');
function component(c,x,y,w,h,index,title,subtitle,body) {
  return `<g transform="translate(${x} ${y})">${rect(0,0,w,h,c.base,17,'#dce3ec')}${text(21,31,index,11,'#9eafbf')}${text(47,32,title,17,c.ink,550)}${text(21,57,subtitle,11,c.muted)}${rule(21,76,w-42,c.line)}${body}</g>`;
}
const l=colors.light,d=colors.dark;
let components=rect(0,0,1440,1510,'url(#board-gradient)')+text(55,52,'八种组件，一眼就能认出来。',30,'#455e77',500)+text(55,85,'对话、思考、执行、结果、队列、确认——各有自己的轮廓与节奏。',13,'#8c9cae');
components+=component(l,55,120,427,340,'A','柔软气泡','身份轻轻点出，阅读自然展开。',`<g transform="translate(18 0)">${user(l,102,['今天想做点轻松的事。'],230)}${answer(l,'pero',197,['好呀，把今天的节奏，','稍微放慢一点点。'])}</g>`);
components+=component(d,506,120,427,340,'B','思考云签','小云朵收住过程，不冒充角色台词。',`<g transform="translate(18 0)">${thought(d,109,true)}${thought(d,185)}${text(39,263,'模型返回的思考记录',10,d.muted)}${lines(39,290,['先确认出发时间，再把路线按步行','距离排好，留出休息的时间。'],13,d.ink,23)}</g>`);
components+=component(l,957,120,427,340,'C','开放工具轨迹','功能图形、操作对象、状态各在其位。',`<g transform="translate(18 0)"><path d="M40 134v133" stroke="${l.strong}"/>${tool(l,111,'search','搜索地点','找到 4 个结果 · 420ms')}${tool(l,177,'edit','写入计划','weekend.md · 26ms')}${tool(l,243,'globe','打开网页','请求超时 · 8.0s',true)}</g>`);
components+=component(d,55,484,427,340,'D','NIT 内部层级','父任务与子调用，用连接与缩进区分。',`<g transform="translate(18 0)">${tool(d,105,'script','运行脚本','3 个内部调用',true)}${text(77,176,'内部调用 3 项',10,d.accent)}<path d="M66 191v128" stroke="${d.strong}" stroke-dasharray="3 3"/>${tool(d,190,'search','搜索地点','公园与咖啡馆 · 420ms',false,true)}${tool(d,237,'file','读取文件','周末偏好.md · 18ms',false,true)}${tool(d,284,'edit','写入计划','weekend.md',true,true)}</g>`);
components+=component(l,506,484,427,340,'E','文件与轻状态','结果是一件文件小物，不是另一条消息。',`<g transform="translate(18 0)">${fileCard(l,111)}${ic('check',85,231,16,l.green)}${text(109,245,'已写入当前会话工作区',12,l.green)}${rect(20,270,334,43,l.soft,11,l.line)}${ic('refresh',34,283,16,l.red)}${text(61,297,'连接暂时中断，草稿还在。',12,l.red)}</g>`);
components+=component(l,957,484,427,340,'F','叠纸待发送队列','编号、顺序和操作，不挤成一行文字。',`<g transform="translate(18 0)">${queue(l,109)}</g>`);
components+=component(l,55,848,427,422,'G','需要你确认','蜂蜜金便笺，把影响范围说在前面。',`<g transform="translate(18 0)">${approval(l,112)}</g>`);
components+=component(d,506,848,427,422,'H','按钮有层级','实色主操作、轻框次操作、无框辅助。',`${button(d,30,112,170,'发送消息','send')}${button(d,218,112,170,'调整设置','sliders',false)}${ic('copy',47,193,18,d.muted)}${text(76,207,'复制正文',13,d.muted)}${ic('rollback',235,193,18,d.red)}${text(263,207,'回滚到此处',13,d.red)}${rule(30,242,358,d.line)}${ic('check',31,269,15,d.green)}${text(56,282,'已保存到当前 infOS',12,d.green)}${rect(30,309,358,66,d.soft,12,d.line)}${ic('thought',45,331,20,d.accent)}${text(79,335,'轻一点，有回应。',14,d.ink)}${text(79,356,'按压 110ms / 抽屉 240ms / 减少动态',10,d.muted)}`);
components+=component(l,957,848,427,422,'I','定制图标 · 同一家族','双色切面、圆润端点、少量小菱形。',Object.keys(icons).map((name,i)=>ic(name,30+i%6*64,108+Math.floor(i/6)*48,25,l.accent,l.secondary)).join(''));
components+=text(55,1320,'轻量级萌动，来自每次恰到好处的回应。',23,'#6886a1',500)+text(55,1355,'消息轻上浮 6px；抽屉贴地出现；状态就地交接；阅读历史不抢滚动。',13,'#8c9cae')+text(55,1393,'静态图仅展示轮廓与层级，交互动效见同目录 HTML。正式实现沿用角色包与主题变量。',12,'#8c9cae')+rule(55,1448,1329,'#dce3ec')+text(55,1478,'PEROPEROCHAT / COMPONENT LANGUAGE 01',10,'#9aa9b9');
async function output(name,width,height,body) {
  const vector=svg(width,height,body);
  fs.writeFileSync(path.join(here,`${name}.svg`),vector);
  await sharp(Buffer.from(vector)).png().toFile(path.join(here,`${name}.png`));
  process.stdout.write(`已导出设计画板：${name}.svg / ${name}.png\n`);
}
module.exports={colors,rect,rule,text,lines,ic,avatar,svg,speaker,user,answer,thought,tool,button,header,composer,output};
if(require.main===module)(async()=>{
  await output('conversation-overview',1440,2075,overview);
  await output('component-language',1440,1510,components);
})().catch(error=>{process.stderr.write(`设计画板导出失败：${error.message}\n`);process.exitCode=1;});
