import { ref, computed, onMounted, onBeforeUnmount } from 'vue'

// 只测量可见区域，兼容软键盘覆盖和调整窗口两种 WebView 行为。
export function useRoomViewport() {
  const height = ref(window.visualViewport?.height || window.innerHeight)
  const top = ref(window.visualViewport?.offsetTop || 0)
  const keyboardOpen = ref(false)
  let baseline = window.innerHeight
  let baselineWidth = window.innerWidth
  let frame = null
  function update() {
    frame = null
    const visible = window.visualViewport?.height || window.innerHeight
    if (Math.abs(window.innerWidth - baselineWidth) > 100) { baseline = window.innerHeight; baselineWidth = window.innerWidth }
    const focused = document.activeElement?.matches('input, textarea, [contenteditable="true"]')
    if (!focused) baseline = Math.max(baseline, window.innerHeight, visible)
    height.value = visible
    top.value = window.visualViewport?.offsetTop || 0
    keyboardOpen.value = !!focused && baseline - visible > 100
  }
  function schedule() { if (!frame) frame = requestAnimationFrame(update) }
  onMounted(() => {
    window.addEventListener('resize', schedule)
    window.visualViewport?.addEventListener('resize', schedule)
    window.visualViewport?.addEventListener('scroll', schedule)
    document.addEventListener('focusin', schedule)
    document.addEventListener('focusout', schedule)
    update()
  })
  onBeforeUnmount(() => {
    cancelAnimationFrame(frame)
    window.removeEventListener('resize', schedule)
    window.visualViewport?.removeEventListener('resize', schedule)
    window.visualViewport?.removeEventListener('scroll', schedule)
    document.removeEventListener('focusin', schedule)
    document.removeEventListener('focusout', schedule)
  })
  return { keyboardOpen, viewportStyle: computed(() => ({ '--room-height': `${height.value}px`, '--room-top': `${top.value}px` })) }
}
