/* 背景与材质设计专题；只保存此设计稿的参数，不读取正式应用配置。 */
(() => {
  const P=window.PPC,S=P.state,{icon:i,avatar:av,badge,tag,section:sec,esc}=P,A=P.assets;
  const storageKey='peropero-design-materials-v1';
  const defaults={background:'none',profile:'soft',surface:86,veil:38,blur:2,glass:16,saturation:94,x:50,y:50,auto:true,reduced:false};
  S.materials={...defaults};
  const choices=[
    {id:'none',title:'主题底色',description:'干净的主题渐层',ambient:'plain'},
    {id:'day',title:'晴日房间',description:'明亮、柔和',file:'companion/sunlit-room.png',ambient:'bright'},
    {id:'twilight',title:'落日余温',description:'色彩丰富',file:'companion/pero-twilight-room.png',ambient:'busy'},
    {id:'night',title:'月色小屋',description:'安静的暗背景',file:'companion/moonlit-room.png',ambient:'dark'},
    {id:'illustration',title:'插画背景',description:'检视复杂画面的可读性',file:'remote/pero-cg.png',ambient:'busy'}
  ];
  const profiles=[['soft','轻玻璃','柔软，有分寸的透'],['airy','通透','多留一点背景'],['clear','清晰','把注意力留给内容']];
  const supportsGlass=typeof CSS!=='undefined'&&(CSS.supports('backdrop-filter','blur(1px)')||CSS.supports('-webkit-backdrop-filter','blur(1px)'));
  let customUrl='',customName='',customAmbient='busy',decodeVersion=0;
  try{const saved=JSON.parse(localStorage.getItem(storageKey));if(saved&&typeof saved==='object'){
    const next={...defaults,...saved};
    next.background=choices.some(x=>x.id===next.background)?next.background:'none';
    next.profile=profiles.some(x=>x[0]===next.profile)?next.profile:'soft';
    for(const [key,min,max] of [['surface',65,100],['veil',10,80],['blur',0,12],['glass',0,24],['saturation',40,110],['x',0,100],['y',0,100]]){const value=Number(next[key]);next[key]=Math.min(max,Math.max(min,Number.isFinite(value)?value:defaults[key]));}
    next.auto=typeof saved.auto==='boolean'?saved.auto:true;next.reduced=!!saved.reduced;S.materials=next;
  }}catch{}
  const save=()=>{try{localStorage.setItem(storageKey,JSON.stringify({...S.materials,background:S.materials.background==='custom'?'none':S.materials.background}));}catch{}};
  const selected=()=>S.materials.background==='custom'?{id:'custom',title:'我的背景',description:customName,ambient:customAmbient,url:customUrl}:choices.find(x=>x.id===S.materials.background)||choices[0];
  const urlFor=item=>item.url||(item.file?A+item.file:'');
  const colorMix=(color,percent)=>`color-mix(in srgb, ${color} ${Math.round(percent)}%, transparent)`;
  const effective=(item=selected(),config=S.materials)=>{
    const present=item.id!=='none',solid=config.reduced||!present;
    const extra=S.dark?3:0;
    const floors={card:config.profile==='clear'?93:config.profile==='airy'?72:82,body:config.profile==='clear'?98:config.profile==='airy'?87:91,chrome:config.profile==='clear'?98:config.profile==='airy'?83:89,float:96};
    const opacity=solid?100:config.surface;
    const autoVeil=!config.auto?config.veil:S.dark?(item.ambient==='bright'?60:item.ambient==='busy'?56:24):(item.ambient==='dark'?53:item.ambient==='busy'?45:31);
    return {present,solid,card:Math.min(100,opacity),body:solid?100:Math.min(100,Math.max(config.surface+5,floors.body)+extra),chrome:solid?100:Math.min(100,Math.max(config.surface+2,floors.chrome)+extra),floating:solid?100:Math.min(100,Math.max(config.surface+9,floors.float)),veil:autoVeil,glass:solid||!supportsGlass?0:config.glass};
  };
  function setVariables(target,item=selected(),config=S.materials){
    const actual=effective(item,config),url=urlFor(item),v={
      '--wallpaper-image':url?`url(${JSON.stringify(url)})`:'none',
      '--wallpaper-veil':colorMix('var(--page)',actual.veil),
      '--wallpaper-blur':`${config.blur}px`,
      '--wallpaper-saturation':`${config.saturation}%`,
      '--wallpaper-position':`${config.x}% ${config.y}%`,
      '--material-card':colorMix('var(--paper)',actual.card),
      '--material-body':colorMix('var(--paper)',actual.body),
      '--material-chrome':colorMix('var(--paper)',actual.chrome),
      '--material-float':colorMix('var(--raised)',actual.floating),
      '--material-inset':colorMix('var(--inset)',actual.solid?100:Math.max(92,actual.card)),
      '--material-line':colorMix('var(--line)',actual.solid?100:86),
      '--material-highlight':colorMix('var(--paper)',S.dark?10:75),
      '--material-user':actual.solid?'var(--primary-soft)':colorMix('var(--primary-soft)',Math.max(actual.body,91)),
      '--material-tint-opacity':`${actual.solid?100:Math.max(86,actual.card)}%`,
      '--material-blur':`${actual.glass}px`,
      '--material-noise-floor':`${actual.card}%`
    };
    Object.entries(v).forEach(([k,value])=>target.style.setProperty(k,value));
    target.dataset.wallpaper=actual.present?'on':'off';
    target.dataset.transparency=config.reduced?'reduced':config.profile;
    target.dataset.glass=actual.glass?'on':'off';
    return actual;
  }
  P.syncMaterials=()=>{
    document.querySelectorAll('.device,.mini-device').forEach(target=>setVariables(target));
    // 每个对照样例单独设定材质，避免被全局预览覆盖。
    document.querySelectorAll('[data-material-scene]').forEach(target=>{
      const live=target.closest('.material-live-sample'),scene=live?selected():choices.find(x=>x.id===target.dataset.materialScene)||selected();
      setVariables(target,scene,live?S.materials:{...S.materials,profile:target.dataset.materialProfile||S.materials.profile,reduced:target.dataset.materialSolid==='true',auto:true});
    });
    const item=selected(),actual=effective();
    document.querySelectorAll('[data-material-value]').forEach(el=>{
      const key=el.dataset.materialValue;el.textContent=key==='effective-veil'?`${actual.veil}%`:key==='effective-surface'?`${actual.card}%`:key==='background-name'?item.title:key==='glass-description'?actual.glass?`${actual.glass} px`:'关闭':key==='state'?S.materials.reduced?'减少透明':S.materials.profile==='clear'?'清晰':S.materials.profile==='airy'?'通透':'轻玻璃':String(S.materials[key])+(el.dataset.unit||'');
    });
    document.querySelectorAll('[data-material-background]').forEach(el=>{const active=el.dataset.materialBackground===S.materials.background;el.classList.toggle('selected',active);el.setAttribute('aria-pressed',String(active));});
    document.querySelectorAll('[data-material-profile]').forEach(el=>{if(el.matches('button')){const active=el.dataset.materialProfile===S.materials.profile;el.classList.toggle('selected',active);el.setAttribute('aria-pressed',String(active));}});
    document.querySelectorAll('[data-material-range="veil"]').forEach(el=>{el.disabled=S.materials.auto;el.closest('.material-range')?.classList.toggle('is-inherited',S.materials.auto);});
    document.querySelectorAll('[data-material-range="glass"]').forEach(el=>{el.disabled=S.materials.reduced;});
    document.querySelectorAll('[data-material-focus]').forEach(el=>el.classList.toggle('selected',Number(el.dataset.materialFocus)===S.materials.y));
    document.querySelectorAll('[data-material-toggle]').forEach(el=>{el.checked=!!S.materials[el.dataset.materialToggle];});
    document.querySelectorAll('.sample-caption').forEach(el=>el.textContent=item.id==='none'?'先选一张背景，再看看通透的效果。':'调整后，其他页面也会一起更新。');
    document.querySelectorAll('[data-material-reduced-note]').forEach(el=>el.hidden=!S.materials.reduced);
    save();
  };
  const range=(key,label,min,max,unit='%',hint='')=>`<label class="material-range"><span><strong>${label}</strong><output data-material-value="${key}" data-unit="${unit}">${S.materials[key]}${unit}</output></span><input type="range" min="${min}" max="${max}" step="1" value="${S.materials[key]}" data-material-range="${key}" aria-label="${label}">${hint?`<small>${hint}</small>`:''}</label>`;
  const toggle=(key,title,description)=>`<label class="toggle-row"><span><strong>${title}</strong><small>${description}</small></span><input type="checkbox" data-material-toggle="${key}" ${S.materials[key]?'checked':''}><span class="switch" aria-hidden="true"></span></label>`;
  const swatches=()=>`<div class="wallpaper-picker">${choices.map(item=>`<button data-action="material-background" data-material-background="${item.id}" class="${S.materials.background===item.id?'selected':''}"><span class="wallpaper-swatch ${item.id==='none'?'no-image':''}">${item.file?`<img src="${urlFor(item)}" alt="${item.title}" loading="lazy">`:i('palette')}<i>${i('check')}</i></span><strong>${item.title}</strong></button>`).join('')}${customUrl?`<button data-action="material-background" data-material-background="custom" class="${S.materials.background==='custom'?'selected':''}"><span class="wallpaper-swatch"><img src="${customUrl}" alt="自定义背景"><i>${i('check')}</i></span><strong>我的背景</strong></button>`:''}<button data-action="background-upload"><span class="wallpaper-swatch upload">${i('plus')}</span><strong>上传图片</strong></button></div>`;
  P.materialPreview=(scene='day',profile='soft',solid=false)=>`
    <div class="material-sample" data-material-scene="${scene}" data-material-profile="${profile}" ${solid?'data-material-solid="true"':''}>
      <div class="sample-viewport">
        <div class="sample-wallpaper" aria-hidden="true"></div>
        <div class="sample-top">${av('pero')}<span><strong>Pero ${i('bow')}</strong><small>给周末留一点空白</small></span>${i('more')}</div>
        <div class="sample-conversation">
          <div class="sample-thought">${i('thought')}想好了<span>高 · 6.2 秒</span></div>
          <div class="sample-bubble"><p>把喜欢的风景留在身后，<br><strong>把想说的话留给我。</strong></p><small>（轻轻把地图展开）</small></div>
          <div class="sample-user">那就慢慢走吧。</div>
        </div>
        <div class="sample-compose"><div>${P.brand()}<span>Gemini 3 Pro</span>${i('thought')}<small>高</small></div><span>${i('plus')}和 Pero 接着聊…<b>${i('send')}</b></span></div>
      </div>
    </div>`;
  const checks=()=>`<div class="material-check-links"><button data-go="chat">${i('chat')}看看聊天页 ${i('next')}</button><button data-go="manage">${i('settings')}看看管理页 ${i('next')}</button><button data-action="material-test-sheet">${i('grid')}看看弹层 ${i('next')}</button><button data-action="material-test-toast">${i('bell')}看看提示 ${i('next')}</button></div>`;
  P.screens.materials=()=>`<div class="materials-summary"><span>${i('image')}<b data-material-value="background-name">${selected().title}</b></span><span data-material-value="state">轻玻璃</span></div><div class="material-live-sample">${P.materialPreview(S.materials.background==='none'?'none':S.materials.background,S.materials.profile,S.materials.reduced)}<span class="sample-caption">调整后，其他页面也会一起更新。</span></div>${sec('背景图片')}<p class="section-hint">选一张喜欢的，也可以上传自己的图片。</p>${swatches()}${sec('界面的质感')}<div class="material-presets">${profiles.map(([id,title,desc])=>`<button data-action="material-profile" data-material-profile="${id}" class="${S.materials.profile===id?'selected':''}"><span class="material-preset-art ${id}"><i></i><b></b></span><strong>${title}</strong><small>${desc}</small></button>`).join('')}</div><div class="material-reduced-note" data-material-reduced-note ${S.materials.reduced?'':'hidden'}><span>已开启减少透明，内容会使用更实的底色。</span><button data-action="material-enable-transparency">恢复通透</button></div><div class="material-controls">${range('surface','面板实度',65,100,'%','数值越高，内容越清楚。')}<div class="material-effective">聊天正文和编辑区会保留更清楚的底色。</div>${toggle('auto','自动照顾文字清晰度','根据深浅色和背景明暗，调整底层遮罩。')}${range('veil','背景遮罩',10,80,'%','自动开启时，会随背景调整。')}${range('glass','毛玻璃强度',0,24,' px','主要作用于导航、输入栏和弹层。')}</div><details class="material-detail-controls"><summary>${i('tuning')}画面与构图 ${i('down')}</summary>${range('blur','背景柔化',0,12,' px')}${range('saturation','背景饱和度',40,110,'%')}<div class="material-focus"><span>画面位置</span><div>${[['25','靠上'],['50','居中'],['75','靠下']].map(([v,t])=>`<button data-action="material-focus" data-material-focus="${v}" class="${S.materials.y===Number(v)?'selected':''}">${t}</button>`).join('')}</div></div>${range('x','水平位置',0,100,'%')}${range('y','垂直位置',0,100,'%')}</details><div class="list-card material-accessibility">${toggle('reduced','减少透明效果','使用更实的内容底色，同时关闭毛玻璃。')}</div>${sec('换个页面看看')}${checks()}<button class="text-btn material-reset" data-action="material-reset">${i('refresh')}恢复默认材质</button>`;
  P.screens['material-states']=()=>`<section class="material-states-intro"><span>${i('image')}${i('spark')}</span><h2>看见背景，也看清内容。</h2><p>切换顶部的深浅色与配色，<br>对照明亮、复杂、夜色三种画面。</p></section>${[
    ['day','明亮房间','背景轻轻透出，正文留在清楚的底色上。','soft',false],
    ['illustration','复杂插画','增加遮罩，保留轮廓与颜色，不让细节穿过文字。','soft',false],
    ['night','夜色房间','跟随深浅色调整遮罩，浮层更清楚。','airy',false],
    ['twilight','减少透明','关闭毛玻璃，依靠实底、细边与阴影分层。','clear',true]
  ].map(([scene,title,desc,profile,solid])=>`<section class="material-comparison"><header><h3>${title}</h3>${tag(solid?'更清晰':profile==='airy'?'通透':'轻玻璃',solid?'mint':'violet')}</header>${P.materialPreview(scene,profile,solid)}<p>${desc}</p><button class="text-btn" data-action="material-try-scene" data-scene="${scene}" data-profile="${profile}" data-solid="${solid}">用这组效果看看 ${i('next')}</button></section>`).join('')}${checks()}`;
  // 外观页保留常用选择，详细材质参数放到独立页。
  P.screens.appearance=()=>`<section class="appearance-preview"><div class="preview-message">${av('pero')}<div><strong>Pero</strong><p>今天想用什么颜色？${i('spark')}</p></div></div><div class="preview-reply">就这个，很喜欢。</div></section>${sec('显示模式')}<div class="display-modes"><button data-theme="light" class="${!S.dark?'selected':''}">${i('sun')}浅色</button><button data-theme="dark" class="${S.dark?'selected':''}">${i('moon')}暗色</button><button data-theme="system">${i('settings')}跟随系统</button></div>${sec('配色包')}<div class="app-palettes">${window.Chromatic.packs.map(([id,title,color,dark])=>`<button data-pack="${id}" class="${S.pack===id?'selected':''}"><span style="--swatch:${S.dark?dark:color}">${S.pack===id?i('check'):''}</span><small>${title}</small></button>`).join('')}</div>${sec('背景与质感')}<button class="material-destination" data-go="materials"><span class="material-destination-cover">${urlFor(selected())?`<img src="${urlFor(selected())}" alt="当前背景">`:i('image')}</span><span><strong>背景、透明度与毛玻璃</strong><small><b data-material-value="background-name">${selected().title}</b> · <b data-material-value="state">轻玻璃</b></small></span>${i('next')}</button><button class="material-reference-link" data-go="material-states">${i('grid')}查看不同画面的搭配 ${i('next')}</button>${sec('交互反馈')}<div class="list-card">${P.toggle('轻量交互动效','按下、展开和切屏时轻轻回应。',!S.reduced,'motion')}</div>`;
  Object.assign(P.pages,{
    materials:{id:'materials',title:'背景与玻璃',tab:'manage',parent:'appearance',icon:'image',heading:'把背景和界面，一起调舒服',note:'分开设置背景柔化、遮罩、面板实度与浮层毛玻璃。正文、编辑区和确认操作保留可读的底色。'},
    'material-states':{id:'material-states',title:'背景搭配对照',tab:'manage',parent:'materials',icon:'grid',heading:'同一套组件，不同画面也清楚',note:'对照明亮图片、复杂插画、夜景和减少透明模式。所有颜色跟随现有八套主题，未使用角色固定配色。'}
  });
  P.groups.push(['背景与材质专题',['materials','material-states']]);
  P.materialInput=el=>{
    const key=el.dataset.materialRange;if(!key)return false;
    if(['surface','veil','blur','glass','saturation','x','y'].includes(key)){S.materials[key]=Number(el.value);P.syncMaterials();}
    return true;
  };
  P.materialChange=el=>{const key=el.dataset.materialToggle;if(!['auto','reduced'].includes(key))return false;S.materials[key]=el.checked;P.syncMaterials();return true;};
  P.selectMaterialBackground=id=>{S.materials.background=id==='default'?'none':id==='room'?'day':id;P.syncMaterials();};
  P.acceptMaterialBackground=async(file,notify)=>{
    if(!file.type.startsWith('image/')){notify?.('选择一张图片，就可以预览背景。');return;}
    const version=++decodeVersion,nextUrl=URL.createObjectURL(file),image=new Image();
    try{await new Promise((resolve,reject)=>{image.onload=resolve;image.onerror=reject;image.src=nextUrl;});}catch{URL.revokeObjectURL(nextUrl);notify?.('这张图片没能打开，换一张试试。');return;}
    if(version!==decodeVersion){URL.revokeObjectURL(nextUrl);return;}
    customAmbient='busy';
    try{
      // 缩小采样只用于判断明暗与复杂度，不改写原图。
      const canvas=document.createElement('canvas');canvas.width=24;canvas.height=24;const context=canvas.getContext('2d',{willReadFrequently:true});context.drawImage(image,0,0,24,24);const data=context.getImageData(0,0,24,24).data,values=[];
      for(let n=0;n<data.length;n+=4)values.push((data[n]*.2126+data[n+1]*.7152+data[n+2]*.0722)/255);
      const mean=values.reduce((a,b)=>a+b,0)/values.length,variance=values.reduce((a,b)=>a+(b-mean)**2,0)/values.length;
      customAmbient=variance>.047?'busy':mean>.58?'bright':mean<.32?'dark':'busy';
    }catch{}
    if(customUrl)URL.revokeObjectURL(customUrl);customUrl=nextUrl;customName=file.name;S.materials.background='custom';S.materials.x=50;S.materials.y=50;P.syncMaterials();notify?.('背景已换好，其他页面也会一起更新。');
  };
  P.materialActions=api=>{
    const {toast,navigate,remember,render,showSheet}=api;
    const refresh=()=>{remember();render(S.page,'tab');P.syncMaterials();};
    const setProfile=id=>{S.materials.profile=id;S.materials.surface=id==='airy'?74:id==='clear'?95:86;S.materials.glass=id==='airy'?20:id==='clear'?10:16;};
    return {
      'show-materials':()=>navigate('materials'),
      'material-background':el=>{S.materials.background=el.dataset.materialBackground;refresh();},
      'material-profile':el=>{setProfile(el.dataset.materialProfile);P.syncMaterials();document.querySelectorAll('[data-material-range]').forEach(input=>input.value=S.materials[input.dataset.materialRange]);},
      'material-enable-transparency':()=>{S.materials.reduced=false;P.syncMaterials();},
      'material-focus':el=>{S.materials.y=Number(el.dataset.materialFocus);P.syncMaterials();document.querySelectorAll('[data-material-range="y"]').forEach(input=>input.value=S.materials.y);},
      'material-reset':()=>{Object.assign(S.materials,defaults);refresh();toast('已恢复默认背景与材质');},
      'material-try-scene':el=>{S.materials.background=el.dataset.scene;setProfile(el.dataset.profile);S.materials.reduced=el.dataset.solid==='true';S.materials.auto=true;P.syncMaterials();navigate('chat');toast('已应用，看看文字和背景的搭配');},
      'material-test-toast':()=>toast('背景换好了，文字也清清楚楚',{icon:'image',tone:'violet',action:'show-materials',label:'调整'}),
      'material-test-sheet':()=>showSheet('留一点通透，也留一点清楚',`<div class="material-layer-demo"><span>${badge('image','blue')}背景在最下面</span><span>${badge('file','violet')}卡片轻轻透出画面</span><span>${badge('shield','mint')}浮层保留更清楚的底色</span></div><p>导航、输入栏和弹层使用毛玻璃；长正文与编辑区更实一些。</p><div class="list-card">${choice('当前背景',selected().title,'show-materials','image')}${choice('毛玻璃',effective().glass?'开启':'关闭','show-materials','palette')}</div>`,'<button class="secondary-btn" data-action="close-sheet">关闭</button><button class="primary-btn" data-action="show-materials">调整效果</button>','palette'),
      'background-upload':()=>{
        const input=document.createElement('input');input.type='file';input.accept='image/*';input.hidden=true;document.body.append(input);
        input.addEventListener('change',async()=>{const file=input.files?.[0];input.remove();if(!file)return;await P.acceptMaterialBackground(file,toast);if(["materials","appearance"].includes(S.page))refresh();else P.syncMaterials();},{once:true});
        input.addEventListener('cancel',()=>input.remove(),{once:true});input.click();
      }
    };
  };
})();
