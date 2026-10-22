// 离线绘制静态设计板，不启动浏览器、Vite、Tauri 或应用验收程序。
const fs = require('node:fs');
const path = require('node:path');
const sharp = require('C:/Users/Administrator/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
require('./palette.js');
const { palette } = globalThis.Chromatic;
const here = __dirname, assets = path.resolve(here, '../../../public');
const source = fs.readFileSync(path.join(here, '../anime-ui-directions/icons.js'), 'utf8');
const icons = Object.fromEntries([...source.matchAll(/^    (\w+): '(.+)',$/gm)].map(match => [match[1], match[2]]));
Object.assign(icons, {
  people: '<circle cx="10" cy="6" r="4"/><path d="M3 21v-3a7 7 0 0 1 14 0v3M18 3a4 4 0 0 1 0 8m3 10v-3a7 7 0 0 0-3-6"/>',
  folder: '<path class="wash" d="M2 6h7l3 3h10v11H2Z"/><path d="M2 6h7l3 3h10v11H2ZM2 6V3h8l3 3h7v3M6 13h6"/>',
  tasks: '<rect class="wash" x="4" y="4" width="16" height="18" rx="3"/><rect x="4" y="4" width="16" height="18" rx="3"/><path d="M9 4V2h6v4H9Zm-2 8 2 2 3-4m2 3h3M8 18h9"/>',
  settings: '<path class="wash" d="m9 2 6 0 1 3 3 1 3 6-3 6-3 1-1 3H9l-1-3-3-1-3-6 3-6 3-1Z"/><path d="m9 2 6 0 1 3 3 1 3 6-3 6-3 1-1 3H9l-1-3-3-1-3-6 3-6 3-1Z"/><circle cx="12" cy="12" r="4"/>',
  home: '<path class="wash" d="m3 10 9-8 9 8v12H3Z"/><path d="m1 12 11-10 11 10M4 9v13h16V9M9 22v-7h6v7"/>',
});
const esc = value => String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('"', '&quot;');
const r = (x,y,w,h,fill,rx=0,stroke='none',sw=1) => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${rx}" fill="${fill}" stroke="${stroke}" stroke-width="${sw}"/>`;
const t = (x,y,value,size,color,weight=400,anchor='start') => `<text x="${x}" y="${y}" fill="${color}" font-size="${size}" font-weight="${weight}" text-anchor="${anchor}">${esc(value)}</text>`;
const ic = (name,x,y,size,color) => `<svg x="${x}" y="${y}" width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="${color}" stroke-width="1.65" stroke-linecap="round" stroke-linejoin="round" color="${color}">${icons[name] || icons.star}</svg>`;
const line = (x,y,w,color) => `<path d="M${x} ${y}h${w}" stroke="${color}"/>`;
const circ = (cx,cy,rad,fill,stroke='none',sw=1) => `<circle cx="${cx}" cy="${cy}" r="${rad}" fill="${fill}" stroke="${stroke}" stroke-width="${sw}"/>`;
function card(c,x,y,w,h,tone='',radius=20) { return r(x,y+4,w,h,c.shadow,radius)+r(x,y,w,h,c.paper,radius,tone?c[tone+'Edge']:c.line); }
function av(c,id,x,y,size=44) { return r(x-1,y-1,size+2,size+2,c.paper,size*.3,c.line)+`<svg x="${x}" y="${y}" width="${size}" height="${size}" viewBox="0 0 100 100"><g clip-path="url(#av)"><use href="#${id}"/></g></svg>`; }
function badge(c,name,tone,x,y,size=37) { return `<g transform="rotate(-4 ${x+size/2} ${y+size/2})">${r(x,y+2,size,size,c[tone+'Edge'],12)}${r(x,y,size,size,c[tone+'Soft'],12,c[tone+'Edge'])}${ic(name,x+8,y+8,size-16,c[tone])}</g>`; }
function tag(c,label,tone,x,y,w) { return r(x,y,w,22,c[tone+'Soft'],7,c[tone+'Edge'])+t(x+w/2,y+15,label,9,c[tone],550,'middle'); }
function btn(c,label,tone,x,y,w,solid=false) { return (solid?r(x,y+3,w,35,c[tone+'Edge'],12):'')+r(x,y,w,35,solid?c[tone]:c[tone+'Soft'],12,c[tone+'Edge'])+t(x+w/2,y+23,label,11,solid?c.on:c[tone],650,'middle'); }
function iconButton(c,name,x,y,size=35) { return r(x,y+3,size,size,c.shadow,12)+r(x,y,size,size,c.paper,12,c.line)+ic(name,x+9,y+9,size-18,c.primary); }
function label(c,name,y,right='') { return t(19,y,name,13,c.ink,650)+(right?t(369,y,right,10,c.muted,400,'end'):''); }
function header(c,title,sub,control='plus') {
  return t(24,24,'9:41',11,c.ink,650)+ic('signal',306,12,13,c.ink)+ic('wifi',324,12,13,c.ink)+ic('battery',342,11,22,c.ink)+t(21,53,sub,9,c.muted)+t(20,83,title,24,c.ink,750)+ic('bow',300,56,26,c.rose)+iconButton(c,control,337,48);
}
function nav(c,selected) {
  let out=r(0,795,390,70,c.paper,22)+line(0,795,390,c.line);
  [['home','首页'],['chat','会话'],['tasks','任务'],['folder','资料'],['settings','管理']].forEach(([symbol,name],i)=>{
    const x=14+i*75,active=i===selected;
    if(active)out+=r(x+10,803,39,29,c.primarySoft,11)+r(x+15,830,29,2,c.primaryEdge,2);
    out+=ic(symbol,x+19,807,20,active?c.primary:c.muted)+t(x+29,848,name,9,active?c.primary:c.muted,active?650:400,'middle');
  });
  return out+r(145,857,100,4,c.muted,3);
}
function segments(c,groups=false,y=109) {
  return r(17,y+2,356,45,c.shadow,14)+r(17,y,356,45,c.paper,14,c.line)+r(groups?198:22,y+5,170,35,groups?c.roseSoft:c.primarySoft,10)+ic('chat',76,y+13,18,groups?c.muted:c.primary)+t(104,y+27,'单聊',12,groups?c.muted:c.primary,groups?400:650)+ic('people',251,y+13,18,groups?c.rose:c.muted)+t(279,y+27,'群聊',12,groups?c.rose:c.muted,groups?650:400);
}
function conversations(c) {
  let out=header(c,'会话','与你的角色，接着聊')+segments(c);
  out+=r(28,176,62,62,c.paper,21,c.accent,2)+av(c,'pero',31,179,56)+av(c,'nana',114,179,56)+r(197,179,56,56,c.paper,20,c.line)+ic('plus',214,196,22,c.muted)+t(59,257,'Pero',11,c.primary,650,'middle')+t(142,257,'Nana',11,c.muted,400,'middle')+t(225,257,'添加角色',11,c.muted,400,'middle');
  out+=label(c,'接着上次的话题',291,'全部会话')+card(c,17,306,356,173)+r(17,327,5,35,c.rose,3)+av(c,'pero',33,322,45)+t(91,342,'周末，去海边吧',14,c.ink,650)+t(91,361,'Pero · 今天 09:38',10,c.muted)+ic('more',337,332,18,c.muted);
  out+=r(32,381,326,39,c.roseSoft,11)+t(44,405,'“海风、日落，还有你。不用赶路的那种。”',11,c.muted)+tag(c,'上次聊到这里','rose',33,438,92)+btn(c,'接着聊  ›','primary',267,432,91,true);
  out+=label(c,'最近聊过',513)+card(c,17,528,356,79)+av(c,'nana',32,545,43)+t(88,563,'今天的灵感收集',12,c.ink,650)+t(88,583,'昨天 · 角色陪伴',10,c.muted)+tag(c,'工作','violet',318,553,38);
  out+=card(c,17,622,356,79)+av(c,'pero',32,639,43)+t(88,657,'晚饭吃什么好呢',12,c.ink,650)+t(88,677,'周二 · 日常聊天',10,c.muted)+ic('more',337,652,18,c.muted);
  out+=ic('shield',104,730,13,c.violet)+t(124,741,'对话保存在你连接的 infOS',10,c.muted)+nav(c,1);return out;
}
function groups(c) {
  let out=header(c,'群聊','把大家叫到一起')+segments(c,true)+r(17,177,356,152,c.shadow,23)+r(17,173,356,152,c.roseSoft,23,c.roseEdge);
  out+=`<g opacity=".2" transform="rotate(10 319 260)">${ic('people',268,212,105,c.rose)}</g>`+t(35,199,'OUR LITTLE SPACE',9,c.rose,650)+t(35,234,'热闹一点，也很好。',23,c.ink,750)+t(35,258,'把大家叫到一起，从一个小话题开始。',11,c.rose)+av(c,'pero',35,279,29)+av(c,'nana',61,279,29)+circ(101,294,14,c.paper)+t(101,298,'你',9,c.rose,550,'middle');
  out+=label(c,'我的群聊',360,'新建 +');
  for(const [i,[name,desc,symbol,tone,tagLabel]]of [['我们的周末小分队','Pero、Nana 和你','people','rose','3 位成员'],['灵感碰头会','一起把脑洞变成计划','star','amber','最近活跃']].entries()) {
    const y=375+i*110;out+=card(c,17,y,356,96)+badge(c,symbol,tone,32,y+24)+t(82,y+27,name,13,c.ink,650)+t(82,y+47,desc,11,c.muted)+tag(c,tagLabel,i?'mint':'rose',82,y+58,77)+ic('next',342,y+37,15,c.muted);
  }
  out+=label(c,'上次聊到',617)+card(c,17,631,356,140)+av(c,'nana',32,645,28)+t(70,664,'Nana',11,c.violet,650)+t(354,664,'09:32',9,c.muted,400,'end')+r(32,683,326,40,c.violetSoft,12)+t(45,708,'那就说好了！这次谁都不许临时赖床哦。',11,c.ink)+btn(c,'回到周末小分队  ›','rose',32,733,326,true)+nav(c,1);return out;
}
function tasks(c) {
  let out=header(c,'任务','和你一起，把事情做好')+card(c,17,109,356,104,'mint',22)+r(18,110,354,101,c.mintSoft,22);
  out+=circ(70,161,31,c.paper,c.mintEdge,5)+`<path d="M70 130a31 31 0 1 1-30 39" fill="none" stroke="${c.mint}" stroke-width="5" stroke-linecap="round"/>`+t(70,162,'3',22,c.mint,750,'middle')+t(70,177,'正在推进',8,c.muted,400,'middle')+t(121,151,'小事，一件件做好。',17,c.ink,700)+t(121,177,'今天已完成 2 件',11,c.muted);
  out+=r(17,234,59,32,c.mintSoft,10)+t(46,255,'全部',11,c.mint,650,'middle')+t(108,255,'进行中',11,c.muted)+t(183,255,'待确认',11,c.muted)+t(260,255,'已完成',11,c.muted);
  out+=card(c,17,281,356,157)+badge(c,'file','mint',32,296)+t(81,320,'整理海边行程',13,c.ink,650)+tag(c,'进行中','mint',306,303,51)+t(32,354,'已找到路线，正在整理交通与住宿。',11,c.muted)+r(32,372,326,5,c.mintSoft,3)+r(32,372,213,5,c.mint,3)+av(c,'pero',33,398,23)+t(63,414,'Pero · 2 / 3 步',10,c.muted)+btn(c,'查看进展  ›','mint',267,391,91,true);
  out+=card(c,17,457,356,132,'amber')+r(18,458,354,129,c.amberSoft,19)+badge(c,'shield','amber',32,472,28)+t(70,492,'有一步，需要你点头',13,c.ink,650)+t(32,522,'允许保存整理好的行程文件吗？',11,c.muted)+tag(c,'等待确认','amber',33,545,65)+btn(c,'查看并确认  ›','amber',249,539,108);
  out+=label(c,'已经做好啦',625);
  for(const [i,[name,time]]of [['整理本周的读书笔记','08:46'],['查好周末的天气','08:20']].entries()) {const y=640+i*65;out+=card(c,17,y,356,51)+ic('check',32,y+16,19,c.mint)+t(63,y+31,name,11,c.muted)+t(357,y+31,time,10,c.muted,400,'end');}
  return out+nav(c,2);
}
function library(c) {
  let out=header(c,'资料','一起积累的小世界')+r(17,109,356,42,c.paper,13,c.line)+ic('search',30,122,17,c.muted)+t(57,136,'找文件、记忆和文档…',12,c.muted);
  [['folder','文件','12 个文件','blue'],['thought','记忆','一起积累','violet'],['file','文档','随时翻一翻','amber']].forEach(([symbol,name,desc,tone],i)=>{const x=17+i*122;out+=r(x,168,112,95,c[tone+'Soft'],16,c[tone+'Edge'])+ic(symbol,x+12,181,25,c[tone])+t(x+12,230,name,12,c[tone],650)+t(x+12,249,desc,9,c[tone]);});
  out+=label(c,'最近打开',299,'管理资料')+card(c,17,316,356,181)+r(18,317,354,119,c.blueSoft,19);
  out+=`<g transform="rotate(-3 195 383)">${r(48,333,296,126,c.paper,7,c.blueEdge)}${t(65,358,'海边的小旅行',14,c.ink,700)}${t(65,380,'不赶路，给日落留一点时间。',10,c.muted)}${r(65,393,234,3,c.line,2)}${r(65,403,227,3,c.line,2)}${r(65,413,180,3,c.line,2)}</g>`+r(17,428,356,69,c.paper,19,c.line)+badge(c,'file','blue',32,444,34)+t(78,461,'海边小旅行.md',12,c.ink,650)+t(78,480,'Markdown · 今天 09:36',10,c.muted)+ic('next',339,455,17,c.primary)+`<g transform="rotate(6 335 315)">${r(318,311,32,8,c.amberSoft,2,c.amberEdge)}</g>`;
  [['folder','旅行与日常','6 个文件','amber'],['image','给你的照片.png','图片 · 昨天','rose']].forEach(([symbol,name,desc,tone],i)=>{const y=516+i*78;out+=card(c,17,y,356,65)+badge(c,symbol,tone,32,y+14,35)+t(81,y+28,name,12,c.ink,650)+t(81,y+46,desc,10,c.muted)+ic('next',341,y+24,14,c.muted);});
  out+=r(17,687,356,83,c.violetSoft,17,c.violetEdge)+ic('thought',32,702,18,c.violet)+t(57,716,'记忆里的一小页',11,c.violet,650)+t(32,742,'你喜欢不赶时间的旅行，',12,c.ink)+t(32,760,'也喜欢沿着海边慢慢走。',12,c.ink)+nav(c,3);return out;
}
function management(c) {
  let out=header(c,'管理','陪伴，也有自己的偏好','star')+card(c,17,109,356,110,'rose')+r(18,110,354,107,c.roseSoft,19)+av(c,'nana',32,131,61)+t(106,139,'把陪伴调成喜欢的样子',16,c.ink,700)+t(106,161,'角色、模型，还有日常习惯。',11,c.muted)+tag(c,'已连接 infOS','mint',106,178,93)+label(c,'常用设置',253);
  [['model','模型与调参','2 个模型 · 随时调整','blue'],['people','角色管理','头像、人设与互动','rose'],['sun','主动陪伴','什么时候来找你','amber'],['thought','记忆与资料','一起记住的小事','violet']].forEach(([symbol,name,desc,tone],i)=>{const x=17+(i%2)*184,y=268+Math.floor(i/2)*137;out+=card(c,x,y,172,123,tone,17);if(i===0||i===3)out+=r(x+1,y+1,170,120,c[tone+'Soft'],16);out+=badge(c,symbol,tone,x+13,y+13,33)+ic('next',x+144,y+20,12,c[tone])+t(x+13,y+79,name,13,c.ink,650)+t(x+13,y+102,desc,10,c.muted);});
  out+=label(c,'应用与连接',572)+card(c,17,588,356,131);
  [['star','显示与主题','配色包 · 字号 · 轻动效','violet'],['connection','连接管理','家里的 infOS · 已连接','blue']].forEach(([symbol,name,desc,tone],i)=>{const y=599+i*62;out+=badge(c,symbol,tone,32,y+5,32)+t(77,y+20,name,12,c.ink,650)+t(77,y+39,desc,10,c.muted)+ic('next',342,y+15,14,c.muted);if(!i)out+=line(32,y+53,326,c.line);});
  out+=r(17,738,356,35,c.mintSoft,13,c.mintEdge)+ic('pulse',30,748,15,c.mint)+t(54,761,'运行概览',10,c.mint,550)+t(352,761,'设备用量  ›',10,c.mint,400,'end')+nav(c,4);return out;
}
function model(c) {
  let out=header(c,'模型调参','调成顺手的样子','more')+card(c,17,109,356,85,'blue')+r(18,110,354,82,c.blueSoft,19)+r(32,127,46,46,c.paper,15,c.blueEdge)+`<use href="#deepseek" x="41" y="136" width="28" height="28"/>`+t(91,131,'当前使用的模型',9,c.muted)+t(91,155,'DeepSeek V4',16,c.ink,700)+t(91,176,'deepseek-v4-pro',10,c.muted)+tag(c,'使用中','mint',306,137,51);
  out+=r(17,211,356,43,c.paper,14,c.line)+r(138,216,113,33,c.primarySoft,10)+ic('model',52,224,16,c.muted)+t(76,237,'模型',11,c.muted)+ic('tuning',169,224,16,c.primary)+t(193,237,'调参',11,c.primary,650)+ic('connection',286,224,16,c.muted)+t(310,237,'连接',11,c.muted);
  out+=r(17,275,356,96,c.shadow,18)+r(17,271,356,96,c.violetSoft,18,c.violetEdge)+ic('thought',31,284,22,c.violet)+t(62,301,'思考强度',12,c.violet,650)+t(358,299,'慢慢想，或直接回答',9,c.violet,400,'end');
  ['默认','低','中','高','极高'].forEach((value,i)=>{const x=31+i*67;out+=r(x,321,59,32,i===3?c.violet:c.paper,10)+t(x+29.5,342,value,10,i===3?c.on:c.muted,i===3?650:400,'middle');});
  out+=label(c,'回复参数',401);
  [['温度','0.8','更灵活地组织语言','rose'],['Top P','0.9','用词的选择范围','mint'],['最大输出','8192','一次回复的长度上限','blue'],['上下文','131072','模型可读取的内容','violet']].forEach(([name,value,help,tone],i)=>{const x=17+(i%2)*184,y=416+Math.floor(i/2)*111;out+=card(c,x,y,172,98,'',16)+t(x+12,y+23,name,10,c.muted)+r(x+126,y+10,33,21,c[tone+'Soft'],6)+t(x+142,y+24,'默认',9,c[tone],400,'middle')+t(x+12,y+61,value,value.length>5?24:27,c.ink,750)+t(x+12,y+84,help,9,c.muted);});
  out+=label(c,'输入与输出',657);
  [['image','图片理解','blue'],['pulse','流式回复','mint'],['thought','显示思考','violet'],['audio','音频输入','rose']].forEach(([symbol,name,tone],i)=>{const x=17+(i%2)*184,y=671+Math.floor(i/2)*51;out+=r(x,y,172,42,c[tone+'Soft'],13,c[tone+'Edge'])+ic(symbol,x+10,y+13,16,c[tone])+t(x+35,y+27,name,11,c[tone],550)+r(x+135,y+13,26,17,i===3?c.line:c[tone],9)+circ(x+(i===3?143:152),y+21.5,5.5,c.paper);});
  out+=r(0,792,390,73,c.paper,21)+line(0,792,390,c.line)+ic('check',19,819,14,c.muted)+t(40,830,'设置已保存',10,c.muted)+btn(c,'✓  保存模型','primary',239,809,134,true)+r(145,857,100,4,c.muted,3);return out;
}
function defs(c) {
  const images=['pero','nana'].map(id=>`<image id="${id}" width="100" height="100" href="data:image/jpeg;base64,${fs.readFileSync(path.join(assets,`avatars/${id}.jpg`)).toString('base64')}"/>`).join('');
  const brand=fs.readFileSync(path.join(assets,'model-brands/deepseek-color.svg'),'utf8');
  return `<defs>${images}<image id="deepseek" width="28" height="28" href="data:image/svg+xml;base64,${Buffer.from(brand).toString('base64')}"/><clipPath id="av">${r(0,0,100,100,'white',28)}</clipPath><clipPath id="phone">${r(0,0,390,865,'white',31)}</clipPath></defs><style>text{font-family:'Microsoft YaHei UI','Microsoft YaHei','Segoe UI',sans-serif}.wash{fill:currentColor;fill-opacity:.2;stroke:none}.detail{stroke:${c.rose};opacity:.8}</style>`;
}
async function board(key,pack,dark,items) {
  const c=palette(pack,dark),bg=dark?'#191722':'#eeedf4',studioInk=dark?'#e3ddeb':'#38354c',muted=dark?'#ab9ab8':'#8b8199';
  let out=r(0,0,1280,1000,bg)+defs(c)+ic('bow',34,22,32,c.accent)+t(79,46,'PeroperoChat',21,studioInk,750)+t(275,45,'轻彩层次 · 设计稿 03',15,studioInk,650)+t(1240,42,dark?'月夜紫 / 暗色适配':'晴空蓝 / 多辅色层次',11,muted,400,'end')+t(36,77,'真实头像  /  安静底色  /  功能辅色  /  清楚的前后关系',11,muted);
  for(const [i,[name,note,renderer]]of items.entries()) {
    const x=40+i*415;
    out+=t(x,116,`0${i+1} / ${name}`,13,studioInk,650)+t(x+359,116,note,10,muted,400,'end');
    out+=`<g transform="translate(${x} 137) scale(.92)">${r(-4,-4,398,873,c.paper,35,c.line)}<g clip-path="url(#phone)">${r(0,0,390,865,c.page,31)}${renderer(c)}</g></g>`;
  }
  out+=t(40,978,'静态设计板 · HTML 交互稿另附，支持八套配色包与明暗切换',10,muted)+t(1240,978,'示例内容 / 尚未接入应用',10,muted,400,'end');
  const svg=`<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="1280" height="1000" viewBox="0 0 1280 1000">${out}</svg>`;
  fs.writeFileSync(path.join(here,key+'.svg'),svg);
  await sharp(Buffer.from(svg),{density:144}).png().toFile(path.join(here,key+'.png'));
  process.stdout.write(`已绘制 ${key}.png\n`);
}
(async()=>{
  await board('01-social-and-tasks','sky',false,[['会话','真实角色 / 会话卡片',conversations],['群聊','角色同场 / 独立空间',groups],['任务','薄荷推进 / 琥珀待确认',tasks]]);
  await board('02-library-and-settings','sky',false,[['资料','内容分色 / 文件小窗',library],['管理','功能分区 / 操作层次',management],['模型调参','思考与参数有自己的材质',model]]);
  await board('03-dark-adaptation','night',true,[['会话','暗色也保留真实头像',conversations],['管理','明暗同时保持辅色识别',management],['模型调参','低亮背景 / 清晰信息层',model]]);
})().catch(error=>{process.stderr.write(`设计板绘制失败：${error.message}\n`);process.exitCode=1;});
