import { computed, ref, watch, onBeforeUnmount } from 'vue'
import { modeTransition } from '../stores/mode'

const MIN_VISIBLE_MS = 2000

// 最短时长属于展示；页面可以在遮罩下恢复，原生运行权仍由协调器决定。
export function useModeTransition(needed, loading) {
  const visible = ref(false), holding = ref(false)
  const waiting = computed(() => holding.value || loading.value)
  let frame = null, timer = null, revision = 0

  function cancel() { if (frame !== null) cancelAnimationFrame(frame); clearTimeout(timer); frame = null; timer = null }
  function settle() { if (!waiting.value) visible.value = needed.value }
  function start() {
    const request = ++revision
    cancel(); visible.value = true; holding.value = true
    // 先让 SVG 实际绘制，再计算两秒；快速完成的原生操作也不会跳过动画。
    frame = requestAnimationFrame(() => {
      frame = requestAnimationFrame(() => {
        frame = null
        timer = setTimeout(() => { if (request !== revision) return; timer = null; holding.value = false; settle() }, MIN_VISIBLE_MS)
      })
    })
  }

  watch(() => modeTransition.serial, () => start())
  watch(needed, value => { if (value && !visible.value) start(); else settle() }, { immediate: true })
  watch(loading, settle)
  onBeforeUnmount(() => { revision++; cancel() })
  return { visible, waiting }
}
