import { onBeforeUnmount, watch } from 'vue'
import { findReadingAnchor } from '../utils/chatScroll'

export function useChatScroll(container, following, active = () => true) {
  let anchor = null
  let frame = null
  let observer = null
  let programmaticTop = null
  let force = false
  let disposed = false
  const observedRows = new Set()

  function observeRows(list) {
    if (!observer) return
    for (const row of observedRows) {
      if (!list.contains(row)) {
        observer.unobserve(row)
        observedRows.delete(row)
      }
    }
    for (const row of list.querySelectorAll('[data-chat-message]')) {
      if (!observedRows.has(row)) {
        observedRows.add(row)
        observer.observe(row)
      }
    }
  }

  function hold() {
    if (!container.value || !active()) return
    anchor = findReadingAnchor(container.value, '[data-chat-message]')
    following.value = false
  }

  function capture() {
    const list = container.value
    if (!list || !active() || list.clientHeight === 0) return
    if (programmaticTop !== null && Math.abs(list.scrollTop - programmaticTop) < 1) return
    programmaticTop = null
    following.value = list.scrollHeight - list.scrollTop - list.clientHeight < 72
    anchor = following.value ? null : findReadingAnchor(list, '[data-chat-message]')
  }

  function refresh(toEnd = false) {
    if (disposed) return
    force ||= toEnd
    if (frame !== null) return
    frame = requestAnimationFrame(() => {
      frame = null
      const list = container.value
      const requested = force
      force = false
      if (!list || !active() || list.clientHeight === 0) return
      observeRows(list)
      // 弹性滚动期间让浏览器完成回弹，避免位置补偿与手势互相拉扯。
      const limit = Math.max(0, list.scrollHeight - list.clientHeight)
      if (!requested && (list.scrollTop < 0 || list.scrollTop > limit)) return
      let nextTop = list.scrollTop
      if (requested || following.value) nextTop = list.scrollHeight
      else if (anchor?.element.isConnected && list.contains(anchor.element)) {
        nextTop += anchor.element.getBoundingClientRect().top - list.getBoundingClientRect().top - anchor.offset
      }
      if (Math.abs(nextTop - list.scrollTop) > .5) list.scrollTop = nextTop
      // 使用浏览器回读的位置，避免滚动取整被误认为用户滚动。
      programmaticTop = list.scrollTop
      if (requested) following.value = true
    })
  }

  watch(container, (list, previous) => {
    observer?.disconnect()
    observedRows.clear()
    previous?.removeEventListener('load', resized, true)
    previous?.removeEventListener('error', resized, true)
    anchor = null
    programmaticTop = null
    if (!list) return
    list.addEventListener('load', resized, true)
    list.addEventListener('error', resized, true)
    if (typeof ResizeObserver !== 'undefined') {
      observer = new ResizeObserver(resized)
      observer.observe(list)
    }
    refresh()
  }, { flush: 'post' })
  function resized() { refresh() }
  onBeforeUnmount(() => {
    disposed = true
    observer?.disconnect()
    container.value?.removeEventListener('load', resized, true)
    container.value?.removeEventListener('error', resized, true)
    if (frame !== null) cancelAnimationFrame(frame)
  })
  return { capture, hold, refresh }
}
