// 静态对照图使用 SVG 原语排版，不打开浏览器、不执行网页或应用代码。
const fs = require('node:fs');
const path = require('node:path');
const sharp = require('C:/Users/Administrator/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
const here = __dirname, publicRoot = path.resolve(here, '../../../public');
const iconsSource = fs.readFileSync(path.join(here, 'icons.js'), 'utf8');
const icons = Object.fromEntries([...iconsSource.matchAll(/^    (\w+): '(.+)',$/gm)].map(match => [match[1], match[2]]));
const esc = value => String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('"', '&quot;');
const rect = (x,y,w,h,fill,r=0,stroke='none',sw=1) => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${r}" fill="${fill}" stroke="${stroke}" stroke-width="${sw}"/>`;
const text = (x,y,value,size=12,color='#253c56',weight=400,extra='') => `<text x="${x}" y="${y}" font-size="${size}" font-weight="${weight}" fill="${color}" ${extra}>${esc(value)}</text>`;
const line = (x1,y1,x2,y2,color,width=1,dash='') => `<path d="M${x1} ${y1}H${x2}" transform="translate(0 ${y2-y1})" fill="none" stroke="${color}" stroke-width="${width}" ${dash ? `stroke-dasharray="${dash}"` : ''}/>`;
const ic = (name,x,y,size,color,secondary=color) => `<svg x="${x}" y="${y}" width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="${color}" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" style="color:${color};--secondary:${secondary}">${icons[name] || icons.star}</svg>`;
function palette(theme,style) {
  if (theme==='dark') return style==='a' ? {page:'#202133',surface:'#2b2c43',soft:'#373759',ink:'#e6e5fa',muted:'#9897b8',line:'#494862',accent:'#a6a0ff',deep:'#6c65c3',second:'#efb5da',thought:'#3c3759',violet:'#c2a1f3',green:'#87cdbd',user:'#6d68b3',userInk:'#faf5ff',warm:'#3b3048',shadow:'#121120'} : {page:'#292435',surface:'#363045',soft:'#463b60',ink:'#f1e8fb',muted:'#b2a1c5',line:'#5b4c6b',accent:'#c5a2f2',deep:'#8764b8',second:'#f0a5c5',thought:'#4d3d62',violet:'#d1aff5',green:'#87cdbd',user:'#61445e',userInk:'#ffe5f2',warm:'#4b3549',shadow:'#1b1427'};
  return style==='a' ? {page:'#eaf5ff',surface:'#fff',soft:'#dfefff',ink:'#203c59',muted:'#708eaa',line:'#bcd6f1',accent:'#237fe9',deep:'#1462c9',second:'#ffb573',thought:'#e9e7ff',violet:'#8d79d1',green:'#2f9b92',user:'#237fe9',userInk:'#fff',warm:'#fff2df',shadow:'#d3e5f8'} : {page:'#f4f1fb',surface:'#fff',soft:'#ece6ff',ink:'#423c59',muted:'#93849f',line:'#d9cdeb',accent:'#9974d5',deep:'#7755b3',second:'#f49fbd',thought:'#eae2ff',violet:'#9c74d0',green:'#4eaa9a',user:'#f8dfed',userInk:'#774966',warm:'#fff0e9',shadow:'#e7def1'};
}
function avatar(theme,x,y,size,c,round=false) {
  return rect(x-1,y-1,size+2,size+2,c.surface,round?size/2+1:11,c.line)+`<svg x="${x}" y="${y}" width="${size}" height="${size}" viewBox="0 0 100 100"><g clip-path="url(#${round?'round':'avatar'}-clip)"><use href="#avatar-${theme}"/></g></svg>`;
}
function status(c) { return rect(0,0,360,34,c.surface)+text(20,24,'9:41',12,c.ink,650)+ic('signal',278,11,14,c.ink)+ic('wifi',297,11,14,c.ink)+ic('battery',316,10,23,c.ink); }
function appHeader(c,theme,style,config=false) {
  const name=theme==='light'?'Pero':'Nana';
  let out = status(c)+rect(0,34,360,75,c.surface,style==='b'?20:0)+ic('back',14,59,21,c.muted);
  if(config) out+=text(50,65,'模型与回复',17,c.ink,650)+text(50,85,'调成顺手的样子',10,c.muted)+avatar(theme,306,49,37,c,true);
  else out+=avatar(theme,47,49,44,c,style==='b')+text(103,68,name,20,c.ink,650)+ic('bow',157,49,25,style==='b'?c.second:c.accent)+`<circle cx="106" cy="86" r="2.7" fill="${c.green}"/>`+text(115,90,'海边的小计划',10,c.muted)+ic('down',192,79,12,c.muted)+rect(311,54,33,33,style==='b'?c.soft:c.surface,style==='b'?17:9,style==='b'?'none':c.line)+ic('more',317,61,20,c.accent);
  if(style==='a')out+=line(66,109,111,109,c.accent,3);
  else out+=`<path d="m283 106 24-3" stroke="${c.second}" stroke-width="3" stroke-linecap="round"/>`;
  return out;
}
function chat(c,theme,style) {
  const b=style==='b',name=theme==='light'?'Pero':'Nana';
  let out=appHeader(c,theme,style)+text(180,137,'今天 · 09:41',10,c.muted,400,'text-anchor="middle"')+line(95,134,119,134,c.line)+line(241,134,265,134,c.line);
  out+=rect(79,157,264,75,c.shadow,b?21:16)+rect(79,153,264,75,c.user,b?21:16,b?c.line:'none')+text(94,180,'周末想去海边，帮我安排一个',14,c.userInk)+text(94,205,'轻松点的行程吧。',14,c.userInk)+text(335,247,'你 · 09:41',9,c.muted,400,'text-anchor="end"');
  out+=avatar(theme,18,267,25,c,b)+text(50,285,name,12,c.accent,650)+ic('star',89,271,13,c.second)+text(342,284,'09:41',9,c.muted,400,'text-anchor="end"');
  out+=rect(17,309,326,79,c.shadow,b?21:18)+rect(17,304,326,79,c.surface,b?21:18,c.line);
  if(!b)out+=`<path d="m27 304 5-6 6 6" fill="${c.surface}" stroke="${c.line}"/>`;
  else out+=`<circle cx="332" cy="384" r="5" fill="${c.second}" stroke="${c.page}" stroke-width="2"/>`;
  out+=text(34,334,'当然！给你留一点',15,c.ink)+text(34,362,'看海发呆',15,c.accent,650)+text(94,362,'的时间～',15,c.ink);
  out+=rect(b?24:17,402,b?315:326,40,c.thought,b?19:10,c.line)+ic('thought',30,411,22,b?c.violet:c.accent)+text(61,428,'想一想',12,b?c.violet:c.deep,650)+text(299,427,'2.1 s',10,c.muted,400,'text-anchor="end"')+ic('down',314,416,12,c.muted);
  if(b)out+=`<circle cx="31" cy="447" r="3" fill="${c.thought}" stroke="${c.line}"/>`;
  out+=ic('pulse',23,467,15,c.accent)+text(44,479,'这就安排',11,c.muted,550)+text(331,479,'2 步已完成',9,c.muted,400,'text-anchor="end"');
  out+=`<path d="M44 512V566" fill="none" stroke="${c.line}" stroke-width="2" ${b?'stroke-dasharray="2 4"':''}/>`;
  const rows=[['search','查找海边路线','车程、天气与日落时间',489,'1.2 s',theme==='dark'?'#8ed2ed':'#2698bc'],['file','整理行程','留白多一点，玩得轻松些',538,'0.8 s',c.violet]];
  for(const [symbol,title,description,y,time,color]of rows) {
    if(b)out+=rect(66,y+1,276,44,c.surface,12);
    out+=rect(28,y+4,32,34,c.soft,b?11:8)+ic(symbol,34,y+10,20,color)+text(71,y+20,title,12,c.ink,600)+text(71,y+37,description,9,c.muted)+ic('check',315,y+8,15,c.green)+text(332,y+36,time,8,c.muted,400,'text-anchor="end"');
    if(!b&&y===489)out+=line(70,536,331,536,c.line,1,'3 3');
  }
  out+=rect(17,607,326,76,c.shadow,b?17:12)+rect(17,603,326,76,c.surface,b?17:12,c.line)+rect(28,615,43,51,b?c.warm:c.soft,9,c.line)+ic('file',37,620,26,b?c.second:c.accent)+text(49,660,'MD',8,b?c.violet:c.accent,750,'text-anchor="middle"')+text(83,626,'海边小旅行.md',12,c.ink,650)+text(83,646,'两天一夜 · 行程清单',10,c.muted)+text(83,663,'Markdown · 3.2 KB',8,c.muted)+rect(310,628,23,23,c.soft,b?12:6)+ic('next',315,633,13,c.accent);
  if(b)out+=`<path d="m286 601 30 3" stroke="${c.second}" stroke-width="7" opacity=".6"/>`;
  out+=text(20,702,'计划收好啦，海风和日落都有份。',11,c.ink)+ic('star',224,691,12,c.second);
  // 输入托盘覆盖阅读区底部，示意独立的玻璃层。
  out+=rect(11,731,338,120,c.shadow,22)+rect(11,726,338,120,c.surface,b?23:19,c.line)+line(27,726,57,726,b?c.second:c.accent,b?5:3);
  out+=ic('model',26,741,14,c.accent)+text(45,752,'DeepSeek V4',10,c.accent)+ic('down',109,743,9,c.accent)+ic('thought',130,741,14,c.accent)+text(149,752,'思考 · 高',10,c.accent)+ic('down',195,743,9,c.accent)+text(333,751,'12.4k / 128k',8,c.green,400,'text-anchor="end"')+line(25,763,335,763,c.line);
  out+=ic('plus',26,781,23,c.accent)+text(60,799,`和 ${name} 接着聊…`,13,c.muted)+rect(298,778,37,39,c.deep,b?14:10)+rect(298,775,37,39,c.accent,b?14:10)+ic('send',305,782,23,theme==='dark'?'#2b2440':'#fff');
  out+=ic('chat',26,825,12,c.accent)+text(43,835,'聊天',9,c.accent)+ic('down',67,827,8,c.accent)+ic('tool',90,825,12,c.muted)+text(108,835,'工具',9,c.muted)+ic('mic',144,825,12,c.muted)+text(161,835,'语音',9,c.muted)+ic('shield',299,826,11,c.muted)+text(313,835,'infOS',8,c.muted);
  return out;
}
function config(c,theme,style) {
  const b=style==='b';let out=appHeader(c,theme,style,true);
  out+=rect(16,130,328,90,c.shadow,b?19:14)+rect(16,126,328,90,b?c.warm:c.soft,b?19:14,c.line);
  out+=`<clipPath id="hero-${theme}-${style}">${rect(16,126,328,90,'white',b?19:14)}</clipPath><g opacity=".22" clip-path="url(#hero-${theme}-${style})"><circle cx="316" cy="176" r="40" fill="none" stroke="${b?c.second:c.accent}" stroke-width="10"/><circle cx="316" cy="176" r="25" fill="none" stroke="${b?c.second:c.accent}" stroke-width="1.2"/></g>`;
  out+=rect(29,146,46,46,c.surface,b?24:12,c.line);
  out+=`<use href="#deepseek" x="37" y="154" width="30" height="30"/>`+text(88,149,'当前聊天模型',9,c.muted)+rect(157,139,36,16,c.soft,4)+text(175,151,'使用中',8,c.green,550,'text-anchor="middle"')+text(87,176,'DeepSeek V4',18,c.ink,650)+text(88,197,'deepseek-v4-pro',10,c.muted)+ic('bow',298,133,33,b?c.second:c.accent);
  out+=rect(16,234,328,44,c.surface,b?14:10,c.line);
  for(const [i,name,symbol]of[[0,'模型','model'],[1,'调参','tuning'],[2,'连接','connection']]) {
    const x=21+i*107;if(i===1)out+=rect(x,239,104,34,b?c.soft:c.accent,b?10:7,b?c.line:c.deep);
    out+=ic(symbol,x+25,248,15,i===1?(b?c.accent:theme==='dark'?'#292039':'white'):c.muted)+text(x+46,260,name,11,i===1?(b?c.deep:theme==='dark'?'#292039':'white'):c.muted,i===1?650:450);
  }
  out+=rect(16,295,328,88,c.thought,b?18:11,c.line)+ic('thought',29,307,20,b?c.violet:c.accent)+text(57,324,'思考强度',12,b?c.violet:c.deep,650)+text(331,323,'跟着你的节奏',9,c.muted,400,'text-anchor="end"');
  for(const [i,label]of['默认','低','中','高','极高'].entries()) {
    const x=28+i*62;out+=rect(x,339,57,30,i===3?(b?c.violet:c.accent):c.surface,b?12:6,i===3?c.deep:c.line)+text(x+28.5,359,label,11,i===3?(theme==='dark'?'#292039':'#fff'):c.muted,i===3?650:400,'text-anchor="middle"');
  }
  out+=ic('tuning',19,403,17,c.accent)+text(44,417,'回复参数',12,c.ink,650)+text(342,416,'留空使用默认',9,c.muted,400,'text-anchor="end"');
  const values=[['温度','0.8','','较低更稳，较高更灵活','sun'],['Top P','0.9','','用词的选择范围','tuning'],['最大输出','8192','Token','一次回复的长度上限','file'],['上下文','131072','Token','模型可读取的内容','model']];
  for(const [i,[label,value,unit,help,symbol]]of values.entries()) {
    const x=16+(i%2)*169,y=429+Math.floor(i/2)*108;out+=rect(x,y+3,159,98,c.shadow,b?15:10)+rect(x,y,159,98,c.surface,b?15:10,c.line)+ic(symbol,x+10,y+11,14,b?c.violet:c.accent)+text(x+29,y+22,label,10,c.muted)+rect(x+120,y+9,28,16,c.soft,b?7:4)+text(x+134,y+20,'重置',8,c.accent,400,'text-anchor="middle"')+text(x+11,y+60,value,26,c.ink,650)+text(x+147,y+58,unit,8,c.muted,400,'text-anchor="end"')+text(x+11,y+82,help,8,c.muted);
  }
  out+=ic('star',19,657,17,c.accent)+text(44,671,'输入与输出',12,c.ink,650)+text(342,670,'按模型能力开启',9,c.muted,400,'text-anchor="end"');
  for(const [i,[label,symbol,checked]]of[['图片理解','image',true],['流式回复','pulse',true],['显示思考','thought',true],['音频输入','audio',false]].entries()) {
    const x=16+(i%2)*169,y=683+Math.floor(i/2)*62;out+=rect(x,y+2,159,53,b?c.shadow:c.page,b?13:9)+rect(x,y,159,53,checked?c.soft:c.surface,b?13:9,c.line)+ic(symbol,x+9,y+18,17,checked?c.accent:c.muted)+text(x+34,y+32,label,11,c.ink,550)+rect(x+121,y+20,26,16,checked?c.accent:c.line,b?8:9)+`<circle cx="${x+(checked?140:128)}" cy="${y+28}" r="5" fill="${checked?'white':c.surface}"/>`;
  }
  out+=rect(0,815,360,64,c.surface,b?18:0)+line(0,815,360,815,c.line)+`<circle cx="22" cy="847" r="3" fill="${c.second}"/>`+text(32,851,'有修改',10,c.muted)+rect(199,832,145,40,c.deep,b?15:10)+rect(199,828,145,40,c.accent,b?15:10)+ic('check',220,840,17,theme==='dark'?'#2b243d':'white')+text(245,854,'保存模型',13,theme==='dark'?'#2b243d':'white',600)+ic('star',335,819,17,c.second);
  return out;
}
function phone(x,y,theme,style,screen) {
  const c=palette(theme,style),scale=.9;
  const content=rect(0,0,360,900,c.page,31)+`<g clip-path="url(#phone-clip)">${screen==='chat'?chat(c,theme,style):config(c,theme,style)}</g>`+rect(131,885,98,4,c.ink,3);
  return `<g transform="translate(${x} ${y}) scale(${scale})">${rect(-4,-4,368,908,theme==='dark'?'#504a61':'#fff',36,theme==='dark'?'#686077':'#ccd9e9',1)}${content}</g>`;
}
function defs(theme) {
  const assets=['light','dark'].map(key=>`<image id="avatar-${key}" width="100" height="100" href="data:image/jpeg;base64,${fs.readFileSync(path.join(publicRoot,'avatars',key==='light'?'pero.jpg':'nana.jpg')).toString('base64')}"/>`).join('');
  const deepseek=fs.readFileSync(path.join(publicRoot,'model-brands/deepseek-color.svg'),'utf8');
  return `<defs>${assets}<image id="deepseek" width="30" height="30" href="data:image/svg+xml;base64,${Buffer.from(deepseek).toString('base64')}"/><clipPath id="phone-clip">${rect(0,0,360,900,'#fff',31)}</clipPath><clipPath id="avatar-clip">${rect(0,0,100,100,'#fff',23)}</clipPath><clipPath id="round-clip"><circle cx="50" cy="50" r="50"/></clipPath></defs><style>text{font-family:'Microsoft YaHei UI','Microsoft YaHei','Segoe UI',sans-serif}.wash{fill:currentColor;fill-opacity:.17;stroke:none}.detail{stroke-width:1.2}svg{overflow:visible}</style>`;
}
async function exportBoard(theme) {
  const dark=theme==='dark',bg=dark?'#181b2b':'#eaf0f8',ink=dark?'#e1e6f4':'#27394f',muted=dark?'#818ba8':'#8192a8';
  let content=rect(0,0,1510,1050,bg)+defs(theme)+ic('bow',35,23,34,dark?'#c3b8ff':'#2789e9')+text(82,48,'PeroperoChat',21,ink,700)+text(270,47,'视觉方向对照',15,ink,600)+text(1459,44,`${theme==='light'?'Pero · 浅色':'Nana · 暗色'} / 静态设计稿`,11,muted,400,'text-anchor="end"')+line(34,69,1476,69,dark?'#343b52':'#d6e1ef');
  for(const [style,x,title,subtitle]of[['a',39,'清透活泼','清亮色块 · 轻玻璃 · 有棱角的小贴片'],['b',788,'柔软可爱','柔润轮廓 · 糖果色 · 轻轻按下的触感']]) {
    content+=rect(x,91,37,37,style==='a'?(dark?'#8f8be5':'#2f8ce5'):(dark?'#bb97c8':'#a084ce'),12)+text(x+18.5,118,style.toUpperCase(),20,'white',750,'text-anchor="middle"')+text(x+51,107,title,18,ink,650)+text(x+51,126,subtitle,10,muted)+text(x+4,157,'01 / 会话',10,muted)+text(x+369,157,'02 / 模型配置',10,muted);
    content+=phone(x+3,174,theme,style,'chat')+phone(x+369,174,theme,style,'config');
  }
  content+=text(39,1020,'HTML 交互稿提供：思考展开、工具详情、文件预览、模型分区、开关与轻提示。',10,muted)+text(1460,1020,'示例数据 / 未接入服务端',10,muted,400,'text-anchor="end"');
  const source=`<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="1510" height="1050" viewBox="0 0 1510 1050">${content}</svg>`;
  const filename=path.join(here,`comparison-${theme}`);
  fs.writeFileSync(`${filename}.svg`,source);
  await sharp(Buffer.from(source),{density:144}).png().toFile(`${filename}.png`);
  process.stdout.write(`已导出 ${path.basename(filename)}.svg 与 .png\n`);
}
exportBoard('light').then(()=>exportBoard('dark')).catch(error=>{process.stderr.write(`设计稿导出失败：${error.message}\n`);process.exitCode=1});
