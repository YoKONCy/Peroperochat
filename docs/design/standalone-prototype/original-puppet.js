(() => {
  const clamp = value => Math.max(-1, Math.min(1, value))
  const number = value => value.toFixed(6)
  const reduced = matchMedia('(prefers-reduced-motion: reduce)')
  const decodedTextures = new Map()
  const smoothstep = value => { const t = Math.max(0, Math.min(1, value)); return t * t * t * (t * (t * 6 - 15) + 10) }

  function texture(source) {
    const url = new URL(source, document.baseURI).href
    if (!decodedTextures.has(url)) {
      const loading = new Promise((resolve, reject) => {
        const image = new Image()
        image.decoding = 'async'
        image.onload = async () => {
          try { if (image.decode) await image.decode(); resolve(image) }
          catch (error) { reject(error) }
        }
        image.onerror = () => reject(new Error('原画分层纹理加载失败。'))
        image.src = url
      }).catch(error => { decodedTextures.delete(url); throw error })
      decodedTextures.set(url, loading)
    }
    return decodedTextures.get(url)
  }

  async function create(host, options) {
    const definition = window.OriginalPuppetModel
    const names = ['body', 'face', 'lid', 'eye', 'pupil', 'closed', 'mouth', 'hand']
    if (!names.every(name => definition?.layers?.[name])) throw new Error('原画动态立绘缺少必要纹理。')
    // 透明图层从原 SVG 离线渲染，播放时不再解析或反复绘制上万条路径。
    const images = await Promise.all(names.map(name => texture(definition.layers[name].src)))
    await texture(definition.irisMask)
    if (!options.isMounted()) return null
    const loaded = Object.fromEntries(names.map((name, index) => [name, images[index]]))
    const events = new AbortController(), lastStyles = new Map()
    const stage = document.createElement('div')
    stage.className = 'original-rig'
    stage.setAttribute('aria-hidden', 'true')
    const origin = [0, 0]
    const parts = {}
    function group(parent, bounds, parentOrigin = origin) {
      const element = document.createElement('div')
      element.className = 'rig-part'
      Object.assign(element.style, { left: `${bounds[0] - parentOrigin[0]}px`, top: `${bounds[1] - parentOrigin[1]}px`, width: `${bounds[2]}px`, height: `${bounds[3]}px` })
      parent.append(element)
      return element
    }
    function picture(name, parent, parentOrigin = origin) {
      const element = loaded[name].cloneNode(false), bounds = definition.layers[name].bounds
      element.className = 'rig-texture'
      element.alt = ''; element.draggable = false
      Object.assign(element.style, { left: `${bounds[0] - parentOrigin[0]}px`, top: `${bounds[1] - parentOrigin[1]}px`, width: `${bounds[2]}px`, height: `${bounds[3]}px` })
      parent.append(element)
      return element
    }
    parts.figure = group(stage, [0, 0, 1024, 1536])
    parts.figure.style.transformOrigin = '512px 1518px'
    picture('body', parts.figure)
    const faceBounds = definition.layers.face.bounds, headOrigin = faceBounds.slice(0, 2)
    parts.head = group(parts.figure, faceBounds)
    parts.head.style.transformOrigin = `${554 - headOrigin[0]}px ${338 - headOrigin[1]}px`
    picture('face', parts.head, headOrigin)
    parts.lid = picture('lid', parts.head, headOrigin)
    const eyeBounds = definition.layers.eye.bounds, eyeOrigin = eyeBounds.slice(0, 2)
    parts.eye = group(parts.head, eyeBounds, headOrigin)
    parts.eye.style.transformOrigin = `${577 - eyeOrigin[0]}px ${232 - eyeOrigin[1]}px`
    picture('eye', parts.eye, eyeOrigin)
    const pupilBounds = definition.layers.pupil.bounds
    const irisWindow = group(parts.eye, pupilBounds, eyeOrigin)
    const irisMask = `url("${new URL(definition.irisMask, document.baseURI).href}")`
    Object.assign(irisWindow.style, { maskImage: irisMask, webkitMaskImage: irisMask, maskSize: '100% 100%', webkitMaskSize: '100% 100%', maskRepeat: 'no-repeat', webkitMaskRepeat: 'no-repeat' })
    parts.pupil = picture('pupil', irisWindow, pupilBounds.slice(0, 2))
    parts.closed = picture('closed', parts.head, headOrigin)
    parts.mouth = picture('mouth', parts.head, headOrigin)
    const mouthBounds = definition.layers.mouth.bounds
    parts.mouth.style.transformOrigin = `${547 - mouthBounds[0]}px ${297 - mouthBounds[1]}px`
    parts.hand = picture('hand', parts.figure)
    const handBounds = definition.layers.hand.bounds
    parts.hand.style.transformOrigin = `${719 - handBounds[0]}px ${398 - handBounds[1]}px`
    // 让实际挂载的图层也完成解码，再替换已有静态原画，避免载入过程闪白。
    await Promise.all([...stage.querySelectorAll('img')].map(image => image.decode ? image.decode() : Promise.resolve()))
    if (!options.isMounted()) return null

    let frame = null, lastTime = 0, clock = 0, disposed = false, visible = false, motion = false, paused = true, promoted = false
    let viewport = null, pointer = null, blinkStart = null, blinkAt = 2.4, reactionStart = null, reaction = 'head'
    const gazeX = { value: 0, velocity: 0 }, gazeY = { value: 0, velocity: 0 }
    let targetX = 0, targetY = 0
    const set = (name, property, value) => {
      const key = `${name}:${property}`
      if (lastStyles.get(key) === value) return
      lastStyles.set(key, value); parts[name].style[property] = value
    }
    const canAnimate = () => !disposed && visible && motion && !paused && !document.hidden && !reduced.matches
    function promote(active) {
      if (promoted === active) return
      promoted = active
      for (const name of ['figure', 'head', 'hand', 'mouth', 'eye', 'pupil']) parts[name].style.willChange = active ? 'transform' : 'auto'
      for (const name of ['lid', 'closed']) parts[name].style.willChange = active ? 'opacity' : 'auto'
    }
    function spring(axis, target, dt) {
      // 临界阻尼的解析更新：跟随速度与屏幕刷新率无关，快速移动后平滑收敛。
      const frequency = 15, offset = axis.value - target, decay = Math.exp(-frequency * dt)
      const term = (axis.velocity + frequency * offset) * dt
      axis.value = target + (offset + term) * decay
      axis.velocity = (axis.velocity - frequency * term) * decay
    }
    function pose(neutral = false) {
      const breath = neutral ? 0 : Math.sin(clock * 1.35)
      const elapsed = reactionStart === null ? Infinity : clock - reactionStart
      const responding = !neutral && elapsed >= 0 && elapsed < 1.8
      const response = responding ? Math.sin(Math.PI * elapsed / 1.8) ** 2 : 0
      const x = neutral ? 0 : gazeX.value, y = neutral ? 0 : gazeY.value
      const tilt = neutral ? 0 : x * .72 + Math.sin(clock * .6) * .16 - response * (reaction === 'head' ? .9 : .3)
      // 缩放保留六位小数，避免原先三位精度把呼吸量化成少数几个档位。
      set('figure', 'transform', `scaleY(${number(1 + breath * .0012)})`)
      set('head', 'transform', `translate3d(${number(x * .55)}px, ${number(neutral ? 0 : breath * -.45 + y * .5)}px, 0) rotate(${number(tilt)}deg)`)
      set('hand', 'transform', `rotate(${number(responding ? Math.sin(elapsed * 10.5) * response * (reaction === 'hand' ? 1.3 : .6) : 0)}deg)`)
      set('mouth', 'transform', `scale(${number(1 + response * .018)}, ${number(1 + response * .055)})`)
      if (!neutral && blinkStart === null && clock >= blinkAt) blinkStart = clock
      let openness = 1
      if (!neutral && blinkStart !== null) {
        const progress = (clock - blinkStart) / .24
        if (progress < .35) openness = 1 - .985 * smoothstep(progress / .35)
        else if (progress < .45) openness = .015
        else if (progress < 1) openness = .015 + .985 * smoothstep((progress - .45) / .55)
        else { blinkStart = null; blinkAt = clock + 3.3 + Math.random() * 2.4 }
      }
      const closed = 1 - smoothstep((openness - .04) / .24)
      set('eye', 'transform', `scaleY(${number(openness)})`)
      set('lid', 'opacity', number(1 - smoothstep((openness - .94) / .06)))
      set('closed', 'opacity', number(closed))
      set('eye', 'opacity', number(1 - closed))
      set('pupil', 'transform', `translate3d(${number(x * 1.65)}px, ${number(y * .85)}px, 0)`)
    }
    function point(clientX, clientY) {
      if (!viewport?.scale) return null
      return { x: (clientX - viewport.left - viewport.x) / viewport.scale, y: (clientY - viewport.top - viewport.y) / viewport.scale }
    }
    function tick(now) {
      frame = null
      if (!canAnimate()) { lastTime = 0; return }
      const dt = lastTime ? Math.min(.05, Math.max(0, (now - lastTime) / 1000)) : 1 / 60
      lastTime = now; clock += dt
      if (pointer) {
        const position = point(pointer.x, pointer.y)
        if (position) { targetX = clamp((position.x - 554) / 220); targetY = clamp((position.y - 232) / 360) }
        pointer = null
      }
      spring(gazeX, targetX, dt); spring(gazeY, targetY, dt); pose()
      frame = requestAnimationFrame(tick)
    }
    function wake() {
      if (disposed) return
      const active = canAnimate()
      promote(active)
      if (!active) {
        if (frame !== null) cancelAnimationFrame(frame)
        frame = null; lastTime = 0
        // 离屏和弹层暂停保持姿态；关闭动效才恢复静止，避免暂停恢复时跳变。
        if (!motion || reduced.matches) {
          clock = 0; blinkAt = 2.4; blinkStart = null; reactionStart = null; pointer = null
          gazeX.value = 0; gazeX.velocity = 0; gazeY.value = 0; gazeY.velocity = 0; targetX = 0; targetY = 0
          pose(true)
        }
      } else if (frame === null) frame = requestAnimationFrame(tick)
    }
    function refreshLayout() {
      const box = host.getBoundingClientRect()
      const scale = Math.min(box.width / 1024, box.height / 1536)
      const x = (box.width - 1024 * scale) / 2, y = box.height - 1536 * scale
      const fit = `translate3d(${number(x)}px, ${number(y)}px, 0) scale(${number(scale)})`
      if (viewport?.fit !== fit) stage.style.transform = fit
      viewport = { left: box.left, top: box.top, x, y, scale, fit }
      return box
    }
    function react(area) {
      if (disposed) return
      reaction = area; reactionStart = canAnimate() ? clock : null
      options.onReact?.(area)
    }
    host.addEventListener('click', event => {
      if (event.detail === 0) { react('head'); return }
      refreshLayout()
      const position = point(event.clientX, event.clientY)
      if (!position) return
      const head = position.x > 310 && position.x < 760 && position.y < 358
      const hand = position.x > 650 && position.x < 805 && position.y > 260 && position.y < 440
      const body = position.x > 350 && position.x < 700 && position.y > 350 && position.y < 680
      if (hand || head || body) react(hand ? 'hand' : head ? 'head' : 'body')
    }, { signal: events.signal })
    host.addEventListener('keydown', event => {
      if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); react('head') }
    }, { signal: events.signal })
    pose(true); refreshLayout(); host.replaceChildren(stage)
    return {
      host, refreshLayout,
      configure(config) {
        if (typeof config.motion === 'boolean') motion = config.motion
        if (typeof config.paused === 'boolean') paused = config.paused
        if (typeof config.visible === 'boolean') visible = config.visible
        wake()
      },
      gaze(event) { pointer = { x: event.clientX, y: event.clientY } },
      center() { pointer = null; targetX = 0; targetY = 0 },
      greet() { react('head') },
      destroy() { disposed = true; if (frame !== null) cancelAnimationFrame(frame); promote(false); events.abort() },
    }
  }

  function mount(root, options = {}) {
    const models = [], events = new AbortController()
    const hosts = [...root.querySelectorAll('[data-original-puppet]')]
    let motion = options.motion !== false, paused = options.paused === true, disposed = false, layoutFrame = null
    let loading = hosts.length, greetQueued = false
    function sync() {
      if (disposed) return
      for (const model of models) {
        const box = model.refreshLayout()
        const visible = !document.hidden && box.width > 0 && box.height > 0 && box.bottom > 0 && box.right > 0 && box.top < innerHeight && box.left < innerWidth
        model.configure({ motion, paused, visible })
      }
    }
    function scheduleSync() {
      if (disposed || layoutFrame !== null) return
      layoutFrame = requestAnimationFrame(() => { layoutFrame = null; sync() })
    }
    const observer = typeof IntersectionObserver === 'function' ? new IntersectionObserver(scheduleSync) : null
    const resize = typeof ResizeObserver === 'function' ? new ResizeObserver(scheduleSync) : null
    for (const host of hosts) {
      observer?.observe(host); resize?.observe(host)
      create(host, { onReact: options.onReact, isMounted: () => !disposed && host.isConnected }).then(model => {
        if (!model) return
        if (disposed) { model.destroy(); return }
        models.push(model); sync()
        if (greetQueued) { greetQueued = false; model.greet() }
      }).catch(error => {
        if (!disposed) { host.dataset.staticFallback = 'true'; console.warn('原画动态立绘加载失败，保留静态原画。', error) }
      }).finally(() => { loading-- })
    }
    root.addEventListener('pointermove', event => {
      if (event.pointerType === 'touch' || !motion || paused) return
      const app = event.target.closest('.app')
      for (const model of models) if (app?.contains(model.host)) model.gaze(event)
    }, { passive: true, signal: events.signal })
    root.addEventListener('pointerout', event => {
      for (const model of models) {
        const app = model.host.closest('.app')
        if (app && (!(event.relatedTarget instanceof Node) || !app.contains(event.relatedTarget))) model.center()
      }
    }, { signal: events.signal })
    document.addEventListener('visibilitychange', sync, { signal: events.signal })
    window.addEventListener('scroll', scheduleSync, { passive: true, capture: true, signal: events.signal })
    window.addEventListener('resize', scheduleSync, { passive: true, signal: events.signal })
    reduced.addEventListener('change', sync)
    return {
      configure(config) {
        if (disposed) return
        if (typeof config.motion === 'boolean') motion = config.motion
        if (typeof config.paused === 'boolean') paused = config.paused
        sync()
      },
      greet() {
        if (disposed) return false
        if (!models.length) { greetQueued = loading > 0; return greetQueued }
        sync(); models[0].greet(); return true
      },
      destroy() {
        disposed = true; observer?.disconnect(); resize?.disconnect(); events.abort()
        if (layoutFrame !== null) cancelAnimationFrame(layoutFrame)
        reduced.removeEventListener('change', sync); models.forEach(model => model.destroy())
      },
    }
  }
  window.OriginalPuppet = { mount }
})()
