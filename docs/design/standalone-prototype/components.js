(() => {
  const icon = window.LocalIcons.render
  const data = window.LocalDesignData
  const escape = (value = '') => String(value).replace(/[&<>"']/g, char => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[char]))
  const action = (name, value = '') => `data-action="${escape(name)}" data-value="${escape(value)}"`
  const button = (name, label, actionName, value = '', extra = '') => `<button type="button" class="icon-button ${extra}" aria-label="${escape(label)}" ${action(actionName, value)}>${icon(name)}</button>`
  const avatar = character => `<img class="avatar" src="${escape(character.avatar)}" alt="${escape(character.name)}的头像" width="34" height="34" />`
  const status = () => `<div class="status-bar" aria-hidden="true"><span>19:12</span><span>${icon('signal')}${icon('wifi')}${icon('battery')}</span></div>`
  const scene = character => `<div class="scene" aria-hidden="true"><img src="${escape(character.room)}" alt="" /></div>`
  const figure = state => `<img class="character" src="${escape(state.character.sprite)}" alt="${escape(state.character.name)}的原画 SVG 立绘" draggable="false" />`
  function header(state, page) {
    const character = state.character
    return `<header class="app-header">${page === 'chat' ? button('back', '回到房间', 'goto', 'room') : ''}<button type="button" class="identity glass" ${action('profile')}>${avatar(character)}<span><strong>${escape(character.name)}</strong><small>独立模式</small></span></button>${button(page === 'chat' ? 'down' : 'menu', page === 'chat' ? '收起对话' : '打开房间菜单', page === 'chat' ? 'goto' : 'menu', page === 'chat' ? 'room' : '', 'glass')}</header>`
  }
  function composer(state, page) {
    return `<form class="composer glass" data-composer="${page}" aria-label="给${escape(state.character.name)}发消息">${state.attachments[page] ? `<span class="attachment-note">${escape(state.attachments[page])}<button type="button" aria-label="移除所选附件" ${action('remove-attachment', page)}>${icon('close')}</button></span>` : ''}${button('image', '选择图片附件', 'attach', page)}<textarea id="message-${page}" name="message" rows="1" maxlength="2000" aria-label="消息内容" placeholder="和 ${escape(state.character.name)} 说说话…" data-draft="${page}">${escape(state.drafts[page] || '')}</textarea><button type="submit" class="send-button" aria-label="发送演示消息">${icon('send')}</button></form>`
  }
  function reminder(state) {
    return state.reminder.enabled ? `<button type="button" class="reminder-chip glass" ${action('reminders')}>${icon('bell')}<span>${escape(state.reminder.time)} ${escape(state.reminder.content)}</span>${icon('next')}</button>` : ''
  }
  function room(state) {
    const character = state.character
    return `${scene(character)}${status()}${figure(state)}${header(state, 'room')}<div class="room-balloon" role="status"><span class="name-tag">${escape(character.name)}</span><span class="room-line">${escape(character.greeting)}</span></div><nav class="room-rail" aria-label="房间功能"><button type="button" class="glass" ${action('goto', 'chat')}>${icon('chat')}<span>聊天</span></button><button type="button" class="glass" ${action('goto', 'wardrobe')}>${icon('hanger')}<span>换装</span></button><button type="button" class="glass" ${action('journals')}>${icon('journal')}<span>日记</span></button></nav>${reminder(state)}${composer(state, 'room')}<div class="home-indicator"></div>`
  }
  function message(item, state) {
    return `<article class="message ${item.role === 'user' ? 'user' : ''}">${item.role !== 'user' ? avatar(state.character) : ''}<div class="message-content">${item.role !== 'user' ? `<span class="message-name">${escape(state.character.name)}</span>` : ''}<div class="message-bubble">${escape(item.text)}</div><time>${escape(item.time)}</time></div></article>`
  }
  function chat(state) {
    return `${scene(state.character)}${status()}${header(state, 'chat')}<div class="chat-peek"><img src="${escape(state.character.sprite)}" alt="" /></div><section class="chat-sheet glass" aria-label="聊天记录"><div class="sheet-notch"></div><div class="transcript" role="log" aria-label="演示消息"><p class="chat-date">今天 · 19:42</p>${state.messages.map((item, index) => `${message(item, state)}${index === 2 && state.reminder.enabled ? `<div class="tool-receipt"><button type="button" ${action('receipt')}>${icon('bell')}已设置提醒 · ${escape(state.reminder.time)} ${icon('next')}</button></div>` : ''}`).join('')}</div></section>${composer(state, 'chat')}<div class="home-indicator"></div>`
  }
  function outfit(item, selected) {
    // 仅展示本目录内的固定服装贴图裁切，不加载模型或执行 Live2D。
    return `<button type="button" class="outfit" aria-pressed="${selected}" ${action('outfit', item.id)}><span class="outfit-art"><svg viewBox="115 0 280 390" role="img" aria-label="${escape(item.name)}的服装细节"><image href="assets/outfits/${escape(item.texture)}" width="1024" height="1024" transform="translate(0 610) rotate(-90)" /></svg></span>${selected ? `<span class="outfit-check">${icon('check')}</span>` : ''}<span>${escape(item.name)}</span></button>`
  }
  function wardrobe(state) {
    return `${scene(state.character)}${status()}${figure(state)}${header(state, 'wardrobe')}<section class="wardrobe-sheet glass" aria-label="衣柜"><div class="sheet-notch"></div><header class="sheet-header"><div><h2>${escape(state.character.name)} 的衣柜</h2><p>挑一套今天喜欢的</p></div>${button('close', '关闭衣柜', 'goto', 'room')}</header><div class="outfit-grid">${data.outfits.map(item => outfit(item, state.outfit === item.id)).join('')}</div><footer class="wardrobe-footer"><button type="button" class="secondary-button" ${action('models')}>${icon('hanger')}角色展示说明</button></footer></section><div class="home-indicator"></div>`
  }
  const row = (symbol, title, description, actionName, trailing = '') => `<button type="button" class="setting-row" ${action(actionName)}><span class="setting-symbol">${icon(symbol)}</span><span class="setting-text"><strong>${title}</strong><small>${description}</small></span>${trailing}${icon('next')}</button>`
  function settings(state) {
    return `${scene(state.character)}${status()}<section class="settings-scroll" aria-label="设置"><header class="settings-heading">${button('back', '回到房间', 'goto', 'room')}<h2>设置</h2><span></span></header><div class="settings-group glass"><button type="button" class="settings-profile" ${action('profile')}>${avatar(state.character)}<span><strong>${escape(state.character.name)}</strong><small>独立模式</small><small>一直在你身边的陪伴</small></span>${icon('next')}</button>${row('model', '模型连接', '服务商、模型与生成参数', 'model')}${row('prompt', '角色与提示词', `设定 ${escape(state.character.name)} 的性格与回复风格`, 'profile')}</div><div class="settings-group glass">${row('palette', '界面主题', escape(data.palettes[state.palette].name), 'appearance', '<span class="color-dot"></span>')}<div class="setting-row setting-row-label"><span class="setting-symbol">${icon('motion')}</span><span class="setting-text"><strong id="motion-label">动态效果</strong><small>界面的轻量过渡与反馈</small></span><button type="button" role="switch" class="switch" aria-checked="${state.motion}" aria-labelledby="motion-label" ${action('motion')}></button></div></div><div class="settings-group glass">${row('bell', '日常提醒', '喝水、休息与自定义提醒', 'reminders')}${row('journal', '日记', `与 ${escape(state.character.name)} 的点滴记录`, 'journals')}</div><div class="settings-group glass">${row('database', '数据与备份', '本地数据管理与导出', 'backup')}</div><p class="setting-version">PEROPEROCHAT · DESIGN PREVIEW</p></section><div class="home-indicator"></div>`
  }
  function device(page, state) {
    const palette = data.palettes[state.palette]
    const view = { room, chat, wardrobe, settings }[page] || room
    return `<div class="device" data-reduce-motion="${!state.motion}" style="--accent:${palette.accent};--wash:${palette.wash};--glass-opacity:${state.glass / 100};--figure-size:${state.size}%"><div class="app app-${page}" data-animate="${state.motion}">${view(state)}</div></div>`
  }
  window.LocalDesignComponents = { escape, action, button, avatar, row, device }
})()
