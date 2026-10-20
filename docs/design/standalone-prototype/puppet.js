(() => {
  let sequence = 0
  function create(host, options = {}) {
    // 模型来自随设计稿提供的固定脚本，不接受用户提供的 SVG 或可执行内容。
    host.innerHTML = window.LocalPuppetModel.markup
    const svg = host.querySelector('svg')
    // 总览同时展示多个模型，为渐变、裁剪和复用图形分配独立引用，避免串色或缺失。
    const prefix = `local-puppet-${++sequence}-`
    const ids = new Map([...svg.querySelectorAll('[id]')].map(element => [element.id, prefix + element.id]))
    for (const element of svg.querySelectorAll('*')) {
      if (element.id) element.id = ids.get(element.id)
      for (const attribute of [...element.attributes]) {
        let value = attribute.value.replace(/url\(#([^)]+)\)/g, (match, id) => ids.has(id) ? `url(#${ids.get(id)})` : match)
        if (attribute.localName === 'href' && value.startsWith('#') && ids.has(value.slice(1))) value = '#' + ids.get(value.slice(1))
        if (value !== attribute.value) element.setAttributeNS(attribute.namespaceURI, attribute.name, value)
      }
    }
    const part = name => svg.querySelector(`[data-part="${name}"]`)
    const parts = { head: part('head-front'), back: part('head-back'), upper: part('upper-body'), skirt: part('skirt'), hair: part('hair-back'), leftEar: part('ear-left'), rightEar: part('ear-right'), ribbon: part('ribbon'), wave: part('wave'), mouth: part('mouth') }
    const cheeks = [...svg.querySelectorAll('[data-part="cheek"]')]
    const eyes = ['left', 'right'].map(side => ({ side, open: svg.querySelector(`[data-eye-open="${side}"]`), closed: svg.querySelector(`[data-eye-closed="${side}"]`), pupil: svg.querySelector(`[data-pupil="${side}"]`) }))
    if (Object.values(parts).some(element => !element) || eyes.some(eye => !eye.open || !eye.closed || !eye.pupil)) throw new Error('互动立绘缺少必要的分层。')
    const reduced = matchMedia('(prefers-reduced-motion: reduce)')
    const events = new AbortController()
    let enabled = options.motion !== false, visible = options.visible !== false, frame = null, previous = 0, disposed = false
    let targetX = 0, targetY = 0, currentX = 0, currentY = 0, blinkAt = performance.now() + 2600, blinkStart = null, reactionStart = null, reaction = 'head'
    const clamp = value => Math.max(-1, Math.min(1, value))
    const number = value => value.toFixed(3)
    const transform = (element, value) => element?.setAttribute('transform', value)
    const canAnimate = () => !disposed && enabled && visible && !document.hidden && !reduced.matches

    function pose(now, neutral = false) {
      const time = now / 1000
      const breathe = neutral ? 0 : Math.sin(time * 1.55)
      const elapsed = reactionStart === null ? Infinity : (now - reactionStart) / 1000
      const responding = elapsed >= 0 && elapsed < 1.65 && !neutral
      const response = responding ? Math.sin(Math.PI * elapsed / 1.65) : 0
      const tilt = neutral ? 0 : currentX * 2.7 + Math.sin(time * .73) * .35 + response * (reaction === 'head' ? -3.4 : 1.8)
      const lift = neutral ? 0 : breathe * -.9 + currentY * 1.2
      const head = `translate(${number(neutral ? 0 : currentX * 1.4)} ${number(lift)}) rotate(${number(tilt)} 360 350)`
      transform(parts.head, head); transform(parts.back, head)
      transform(parts.upper, `translate(0 ${number(breathe * -.45)}) translate(360 590) scale(1 ${number(1 + breathe * .0024)}) translate(-360 -590)`)
      transform(parts.skirt, `rotate(${number(neutral ? 0 : Math.sin(time * .9) * .18)} 360 590)`)
      transform(parts.hair, `rotate(${number(neutral ? 0 : Math.sin(time * 1.2 + .6) * .38)} 360 210)`)
      transform(parts.leftEar, `rotate(${number(neutral ? 0 : Math.sin(time * 1.05) * .45 + response * 2.2)} 230 155)`)
      transform(parts.rightEar, `rotate(${number(neutral ? 0 : -Math.sin(time * 1.05 + .35) * .4 - response * 1.4)} 435 125)`)
      transform(parts.ribbon, `rotate(${number(neutral ? 0 : Math.sin(time * 1.4 + .8) * .6)} 362 415)`)
      transform(parts.wave, `rotate(${number(responding ? Math.sin(elapsed * 15) * response * (reaction === 'hand' ? 7 : 3.5) : 0)} 486 482)`)
      transform(parts.mouth, `translate(350 319) scale(${number(1 + response * .045)} ${number(1 + response * .22)}) translate(-350 -319)`)
      cheeks.forEach(element => element.setAttribute('opacity', number(.75 + response * .25)))

      if (!neutral && blinkStart === null && now >= blinkAt) blinkStart = now
      let openness = 1
      if (!neutral && blinkStart !== null) {
        const phase = (now - blinkStart) / 170
        openness = phase < 1 ? 1 - Math.sin(Math.PI * phase) : 1
        if (phase >= 1) { blinkStart = null; blinkAt = now + 2800 + Math.random() * 2600 }
      }
      const wink = responding && reaction === 'head' && elapsed > .25 && elapsed < .85
      for (const eye of eyes) {
        const opening = Math.max(.015, wink && eye.side === 'left' ? .015 : openness)
        transform(eye.open, `translate(403 251) scale(1 ${number(opening)}) translate(-403 -251)`)
        eye.closed.setAttribute('opacity', opening < .22 ? '1' : '0')
        transform(eye.pupil, `translate(${number(neutral ? 0 : currentX * (eye.side === 'left' ? -3.4 : 3.4))} ${number(neutral ? 0 : currentY * 2.2)})`)
      }
    }
    function tick(now) {
      frame = null
      if (disposed || !canAnimate()) { pose(now, true); previous = 0; return }
      const dt = previous ? Math.min(80, now - previous) : 16
      const blend = 1 - Math.exp(-dt / 110)
      currentX += (targetX - currentX) * blend; currentY += (targetY - currentY) * blend
      previous = now; pose(now)
      frame = requestAnimationFrame(tick)
    }
    function wake() {
      if (disposed) return
      if (!canAnimate()) {
        if (frame !== null) cancelAnimationFrame(frame)
        frame = null; previous = 0; blinkStart = null; reactionStart = null
        pose(performance.now(), true)
      } else if (frame === null) { blinkAt = performance.now() + 2200; frame = requestAnimationFrame(tick) }
    }
    function pointFrom(event) {
      const matrix = svg.getScreenCTM()
      if (!matrix) return null
      const point = svg.createSVGPoint(); point.x = event.clientX; point.y = event.clientY
      return point.matrixTransform(matrix.inverse())
    }
    host.addEventListener('pointermove', event => {
      if (event.pointerType === 'touch' && event.buttons === 0) return
      const point = pointFrom(event)
      if (point) { targetX = clamp((point.x - 360) / 240); targetY = clamp((point.y - 240) / 330) }
    }, { signal: events.signal })
    host.addEventListener('pointerleave', () => { targetX = 0; targetY = 0 }, { signal: events.signal })
    function react(kind) {
      if (disposed) return
      reaction = kind; reactionStart = performance.now()
      if (canAnimate() && frame === null) frame = requestAnimationFrame(tick)
      host.dispatchEvent(new CustomEvent('peropero-puppet-greet', { bubbles: true, detail: { area: kind } }))
    }
    host.addEventListener('click', event => {
      if (event.detail === 0) { react('head'); return }
      if (!event.target.closest('path, ellipse, use, circle')) return
      const point = pointFrom(event)
      if (point) react(point.y < 365 ? 'head' : point.x > 485 && point.y < 550 ? 'hand' : 'body')
    }, { signal: events.signal })
    host.addEventListener('keydown', event => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); react('head') } }, { signal: events.signal })
    reduced.addEventListener('change', wake)
    document.addEventListener('visibilitychange', wake, { signal: events.signal })
    pose(performance.now(), true); wake()
    return {
      configure(config) {
        if (disposed) return
        if (typeof config.motion === 'boolean') enabled = config.motion
        if (typeof config.visible === 'boolean') visible = config.visible
        if (typeof config.label === 'string') host.setAttribute('aria-label', `轻触${config.label.slice(0, 30)}打招呼，移动指针可跟随视线`)
        wake()
      },
      setGaze(x, y) { if (!disposed && Number.isFinite(x) && Number.isFinite(y)) { targetX = clamp(x); targetY = clamp(y) } },
      greet() { react('head') },
      destroy() { disposed = true; if (frame !== null) cancelAnimationFrame(frame); frame = null; events.abort(); reduced.removeEventListener('change', wake) },
    }
  }
  window.LocalPuppet = { create }
  // 独立素材页也可直接打开；设计工作台在主页面挂载模型，不再使用嵌套页面。
  const standalone = document.getElementById('puppet')
  if (standalone) {
    const model = create(standalone, { motion: new URLSearchParams(location.search).get('motion') !== '0' })
    window.addEventListener('pagehide', () => model.configure({ visible: false }))
    window.addEventListener('pageshow', () => model.configure({ visible: true }))
  }
})()
