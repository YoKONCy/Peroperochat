(() => {
  const data = window.LocalDesignData
  const { escape, action, button, device } = window.LocalDesignComponents
  const icon = window.LocalIcons.render
  const designs = document.getElementById('designs'), dialog = document.getElementById('preview-dialog')
  const initial = () => ({ view: 'gallery', page: 'room', character: { ...data.character }, palette: 'sky', glass: 74, size: 57, motion: !matchMedia('(prefers-reduced-motion: reduce)').matches, outfit: 'maid', drafts: { room: '', chat: '' }, attachments: {}, reminder: { enabled: true, time: '20:30', content: '记得喝水' }, messages: data.messages.map(message => ({ ...message })) })
  let state = initial(), toastTimer = null, attachmentTarget = 'room', returnFocus = null, messageSequence = 0
  function toast(text) {
    const element = document.getElementById('toast')
    clearTimeout(toastTimer); element.textContent = text; element.classList.add('is-visible')
    toastTimer = setTimeout(() => element.classList.remove('is-visible'), 3800)
  }
  function render() {
    document.documentElement.style.setProperty('--accent', data.palettes[state.palette].accent)
    document.documentElement.style.setProperty('--wash', data.palettes[state.palette].wash)
    document.querySelectorAll('[data-studio]').forEach(element => { if (['gallery', 'interactive'].includes(element.dataset.studio)) element.setAttribute('aria-pressed', String(element.dataset.studio === state.view)) })
    document.getElementById('palette-select').value = state.palette
    if (state.view === 'gallery') {
      designs.innerHTML = `<div class="gallery">${data.pages.map((page, index) => `<section class="design-page"><h2 class="page-label"><span>0${index + 1}</span>${page.title}<small>独立模式</small></h2>${device(page.id, state)}</section>`).join('')}</div>`
    } else {
      designs.innerHTML = `<div class="interactive-layout"><nav class="interactive-nav" aria-label="演示页面"><p>一个房间，几段日常</p>${data.pages.map((page, index) => `<button type="button" aria-pressed="${state.page === page.id}" ${action('goto', page.id)}><span>0${index + 1}</span><span>${page.title}</span></button>`).join('')}<small>点击房间里的功能入口。<br />所有演示状态仅保留在本次预览中。</small></nav>${device(state.page, state)}<aside class="material-notes"><h3>轻一点，近一点。</h3><p>房间、立绘与面板独立分层。磨砂玻璃保留背景颜色，文字区域使用更稳定的底色。</p><p>角色内容与配色分开配置；当前示例沿用 ${escape(state.character.name)} 与浅色主题。</p><button type="button" ${action('appearance')}>调整玻璃与角色比例</button><button type="button" ${action('models')}>查看立绘说明</button><p>网页稿使用原画静态 SVG；角色内容与主题独立配置。</p><p>正式应用继续沿用现有 Pio 通用模型，自制动态立绘试作已停止。</p></aside></div>`
    }
  }
  function goto(page) {
    if (!data.pages.some(item => item.id === page)) return
    state.page = page; state.view = 'interactive'; render()
  }
  function openDialog(title, body) {
    if (!dialog.open) returnFocus = document.activeElement
    dialog.innerHTML = `<header class="dialog-heading"><h2 id="dialog-title">${escape(title)}</h2>${button('close', '关闭弹层', 'close')}</header><div class="dialog-body">${body}</div>`
    if (!dialog.open) dialog.showModal()
  }
  const caption = text => `<p class="dialog-caption">${text}</p>`
  function appearance() {
    openDialog('显示偏好', `${caption('只调整当前网页设计预览。角色、房间与主题独立配置。')}<label>点缀配色<select id="dialog-palette">${Object.entries(data.palettes).map(([id, palette]) => `<option value="${id}" ${state.palette === id ? 'selected' : ''}>${palette.name}</option>`).join('')}</select></label><label>玻璃底色不透明度 <output id="glass-value">${state.glass}%</output><input type="range" id="glass-range" min="60" max="94" value="${state.glass}" /></label><label>房间角色占屏高度 <output id="size-value">${state.size}%</output><input type="range" id="size-range" min="44" max="64" value="${state.size}" /></label>${caption('立绘脚底贴近输入区。衣柜与聊天使用各自的构图比例。')}<button class="dialog-primary" type="button" ${action('close')}>完成</button>`)
  }
  function profile() {
    openDialog('角色与提示词', `<form data-dialog-form="profile">${caption('以下仅修改演示角色的显示内容，不保存为角色包，也不修改正式人设。')}<label>角色名字<input name="name" maxlength="20" required value="${escape(state.character.name)}" /></label><label>房间问候<textarea name="greeting" maxlength="100" required>${escape(state.character.greeting)}</textarea></label><label>演示回复<textarea name="acknowledgement" maxlength="200" required>${escape(state.character.acknowledgement)}</textarea></label><button class="dialog-primary" type="submit">应用到预览</button></form>`)
  }
  function reminders() {
    openDialog('日常提醒', `<form data-dialog-form="reminder">${caption('提醒组件的交互示例，不会设置真实闹钟或发送系统通知。')}<label class="reminder-edit"><input name="enabled" type="checkbox" ${state.reminder.enabled ? 'checked' : ''} />在房间显示这条提醒</label><label>时间<input name="time" type="time" value="${escape(state.reminder.time)}" required /></label><label>提醒内容<input name="content" maxlength="40" value="${escape(state.reminder.content)}" required /></label><button class="dialog-primary" type="submit">更新预览</button></form>`)
  }
  function journals() {
    openDialog(`${state.character.name} 的日记`, `${caption('以下为设计演示日记，不读取应用记忆。')}${data.journals.map(item => `<article class="journal-entry"><time>${escape(item.date)}</time><h3>${escape(item.title)}</h3><p>${escape(item.text)}</p></article>`).join('')}`)
  }
  function menu() {
    openDialog('房间里的日常', `<button class="dialog-choice" type="button" ${action('goto', 'chat')}>${icon('chat')}聊天记录</button><button class="dialog-choice" type="button" ${action('goto', 'wardrobe')}>${icon('hanger')}打开衣柜</button><button class="dialog-choice" type="button" ${action('journals')}>${icon('journal')}日记</button><button class="dialog-choice" type="button" ${action('reminders')}>${icon('bell')}日常提醒</button><button class="dialog-choice" type="button" ${action('goto', 'settings')}>${icon('settings')}设置</button>${caption('当前是独立模式的网页设计稿，全部交互仅用于预览。')}`)
  }
  function model() {
    openDialog('模型连接', `<form data-dialog-form="model">${caption('模型设置页的字段与层级示意。请勿输入真实密钥，本页不发起任何 API 请求。')}<label>API 协议<select><option>OpenAI Chat Completions</option><option>OpenAI Responses</option><option>Anthropic Messages</option><option>Google Gemini</option></select></label><label>API 地址<input type="url" readonly value="https://api.example.com/v1" /></label><label>模型名称<input readonly value="示例聊天模型" /></label><label>思考强度<select><option>跟随模型默认</option><option>低</option><option>中</option><option>高</option></select></label><label>温度<input type="range" min="0" max="20" value="8" /></label><button class="dialog-primary" type="submit">完成查看</button></form>`)
  }
  const actions = {
    goto: value => { if (dialog.open) dialog.close(); goto(value) },
    close: () => dialog.close(), menu, profile, appearance, reminders, journals, model,
    motion: () => { state.motion = !state.motion; render() },
    outfit: value => { if (!data.outfits.some(item => item.id === value)) return; state.outfit = value; render(); toast('服装选中态已更新；试作立绘不随之换装') },
    models: () => openDialog('角色展示', `${caption('当前网页设计稿使用原画静态 SVG。')}<p>自制动态立绘试作已停止。正式应用继续沿用现有 Pio 通用模型。</p>${caption('本设计稿不加载 Pio，服装网格只演示缩略图和选中态。')}<button type="button" class="dialog-primary" ${action('close')}>知道了</button>`),
    backup: () => openDialog('数据与备份', `${caption('这里预留正式应用的数据管理与导出入口。设计稿没有连接本地数据库，不提供实际备份或清空操作。')}<p>演示消息、服装选中态、提醒和显示参数仅保存在当前页面内存中。关闭或刷新网页后恢复初始值。</p><button type="button" class="dialog-secondary" ${action('reset')}>恢复当前演示状态</button>`),
    receipt: () => openDialog('提醒记录', `${caption('这是一条预置演示工具记录，不是本次聊天真实执行的结果。')}<p>${icon('bell')} ${escape(state.reminder.time)} · ${escape(state.reminder.content)}</p><button type="button" class="dialog-secondary" ${action('reminders')}>查看提醒组件</button>`),
    attach: value => { attachmentTarget = value === 'chat' ? 'chat' : 'room'; document.getElementById('attachment-input').click() },
    'remove-attachment': value => { delete state.attachments[value]; render() },
    reset: () => { if (dialog.open) dialog.close(); state = initial(); render(); toast('演示已恢复初始状态') },
  }
  document.addEventListener('click', event => {
    const trigger = event.target.closest('[data-action], [data-studio]')
    if (!trigger || trigger.disabled) return
    if (trigger.dataset.studio) {
      if (trigger.dataset.studio === 'reset') actions.reset()
      else { state.view = trigger.dataset.studio; render() }
    } else actions[trigger.dataset.action]?.(trigger.dataset.value)
  })
  function updateMaterial() {
    document.querySelectorAll('.device').forEach(element => {
      element.style.setProperty('--glass-opacity', state.glass / 100)
      element.style.setProperty('--figure-size', `${state.size}%`)
    })
  }
  document.addEventListener('input', event => {
    const element = event.target
    if (element.dataset.draft) state.drafts[element.dataset.draft] = element.value
    if (element.id === 'glass-range') { state.glass = Number(element.value); document.getElementById('glass-value').textContent = `${state.glass}%`; updateMaterial() }
    if (element.id === 'size-range') { state.size = Number(element.value); document.getElementById('size-value').textContent = `${state.size}%`; updateMaterial() }
  })
  document.addEventListener('change', event => {
    if (['palette-select', 'dialog-palette'].includes(event.target.id) && Object.hasOwn(data.palettes, event.target.value)) { state.palette = event.target.value; render() }
  })
  document.addEventListener('keydown', event => {
    if (event.target.matches('[data-draft]') && event.key === 'Enter' && !event.shiftKey && !event.isComposing && event.keyCode !== 229) { event.preventDefault(); event.target.closest('form').requestSubmit() }
  })
  document.addEventListener('submit', event => {
    const form = event.target
    if (form.matches('[data-composer]')) {
      event.preventDefault()
      const page = form.dataset.composer, text = state.drafts[page].trim(), attachment = state.attachments[page]
      if (!text && !attachment) return
      if (state.messages.length >= 60) { toast('演示消息已达上限，可恢复初始状态后继续预览'); return }
      const time = new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', hour12: false })
      const id = ++messageSequence
      state.messages.push({ id: `demo-${id}-user`, role: 'user', text: text + (attachment ? `${text ? '\n' : ''}[附件示意：${attachment}]` : ''), time }, { id: `demo-${id}-assistant`, role: 'assistant', text: state.character.acknowledgement, time })
      state.drafts[page] = ''; delete state.attachments[page]; goto('chat')
      designs.querySelector('.transcript')?.scrollTo({ top: 100000, behavior: 'instant' })
      document.getElementById('message-chat')?.focus({ preventScroll: true }); toast('已显示预设演示回复，未调用模型')
      return
    }
    if (!form.matches('[data-dialog-form]')) return
    event.preventDefault()
    const values = new FormData(form)
    if (form.dataset.dialogForm === 'profile') {
      const name = String(values.get('name') || '').trim(), greeting = String(values.get('greeting') || '').trim(), acknowledgement = String(values.get('acknowledgement') || '').trim()
      if (!name || !greeting || !acknowledgement) { toast('请填写名字、问候与演示回复'); return }
      Object.assign(state.character, { name, greeting, acknowledgement })
    } else if (form.dataset.dialogForm === 'reminder') {
      const content = String(values.get('content') || '').trim()
      if (!content) { toast('请填写提醒内容'); return }
      state.reminder = { enabled: values.get('enabled') === 'on', time: String(values.get('time')), content }
    } else if (form.dataset.dialogForm === 'model') { dialog.close(); toast('模型设置仅为设计示例，未保存或连接服务'); return }
    dialog.close(); render(); toast('已更新当前设计预览')
  })
  document.getElementById('attachment-input').addEventListener('change', event => {
    const file = event.target.files?.[0]
    if (file) { if (!['image/png', 'image/jpeg', 'image/webp'].includes(file.type)) toast('请选择 PNG、JPEG 或 WebP 图片'); else { state.attachments[attachmentTarget] = file.name; render(); toast('仅记录文件名，不读取或上传图片') } }
    event.target.value = ''
  })
  dialog.addEventListener('close', () => { if (returnFocus?.isConnected) returnFocus.focus({ preventScroll: true }); returnFocus = null })
  dialog.addEventListener('click', event => { if (event.target !== dialog) return; const box = dialog.getBoundingClientRect(); if (event.clientX < box.left || event.clientX > box.right || event.clientY < box.top || event.clientY > box.bottom) dialog.close() })
  window.addEventListener('pagehide', () => clearTimeout(toastTimer))
  // 普通脚本与本地相对资源允许直接打开文件；不启动服务，也不导入应用配置。
  render()
})()
