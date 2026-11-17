import { onBeforeUnmount, watch } from 'vue'
import { createTextMeasurer, observeTextLayout, readTextStyle } from '../utils/textLayout'

export function useMeasuredTextarea(element, value, maxHeight = 112) {
  const measurer = createTextMeasurer(120000)
  let style = null
  let width = 0
  let observer = null
  let stopEnvironment = null
  let frame = null
  let disposed = false

  function resize() {
    const input = element.value
    if (!input || !style || !(width > 0)) return
    const text = String(value()).replace(/\r\n?/g, '\n')
    const chromeHeight = style.boxSizing === 'border-box' ? style.paddingY + style.borderY : 0
    const limit = Math.min(style.maxHeight, maxHeight)
    let contentHeight = 0
    // 独立缓存每个逻辑行，输入或粘贴时只准备发生变化的文字。
    for (const line of text.split('\n')) {
      const height = measurer.measure(line || ' ', { ...style, whiteSpace: 'pre-wrap' }, width)
      if (height == null) {
        input.style.height = 'auto'
        const nativeHeight = input.scrollHeight - style.paddingY + chromeHeight
        input.style.height = `${Math.max(style.minHeight, Math.min(nativeHeight, limit))}px`
        return
      }
      contentHeight += Math.max(style.lineHeight, height)
      if (contentHeight + chromeHeight >= limit) break
    }
    const nextHeight = `${Math.max(style.minHeight, Math.min(contentHeight + chromeHeight, limit))}px`
    if (input.style.height !== nextHeight) input.style.height = nextHeight
  }

  function schedule() {
    if (frame !== null || disposed) return
    frame = requestAnimationFrame(() => {
      frame = null
      resize()
    })
  }

  function refreshStyle() {
    if (!element.value) return
    style = readTextStyle(element.value)
    width = Math.max(0, element.value.clientWidth - style.paddingX)
    schedule()
  }

  watch(element, input => {
    observer?.disconnect()
    stopEnvironment?.()
    style = null
    if (!input) return
    refreshStyle()
    if (typeof ResizeObserver !== 'undefined') {
      observer = new ResizeObserver(([entry]) => {
        if (entry.contentRect.width !== width) {
          style = readTextStyle(input)
          width = entry.contentRect.width
          schedule()
        }
      })
      observer.observe(input)
    }
    stopEnvironment = observeTextLayout(input, refreshStyle)
  }, { flush: 'post' })
  watch(value, schedule, { flush: 'post' })
  onBeforeUnmount(() => {
    disposed = true
    observer?.disconnect()
    stopEnvironment?.()
    if (frame !== null) cancelAnimationFrame(frame)
  })
  return schedule
}
