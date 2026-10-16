import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { queryReminders } from '../services/reminders'

export function useReminders(agentId, keyword = ref('')) {
  const entries = ref([]), total = ref(0), now = ref(''), loading = ref(false), error = ref('')
  let version = 0, disposed = false, timer = null
  const unlisteners = []
  async function reload() {
    const id = agentId.value, current = ++version
    if (!id || disposed) return
    loading.value = true
    try {
      const result = await queryReminders(id, { keyword: keyword.value, limit: 50 })
      if (disposed || current !== version) return
      entries.value = result.entries; total.value = result.total; now.value = result.now; error.value = ''
    } catch (failure) { if (!disposed && current === version) error.value = failure.message }
    finally { if (!disposed && current === version) loading.value = false }
  }
  function changed(event) { if ((event.payload || event.detail)?.agentId === agentId.value) reload() }
  watch(agentId, () => { entries.value = []; total.value = 0; error.value = ''; reload() }, { immediate: true, flush: 'sync' })
  watch(keyword, () => { clearTimeout(timer); timer = setTimeout(reload, 250) })
  onMounted(async () => {
    window.addEventListener('ppc:reminders-changed', changed)
    for (const name of ['ppc:reminders-updated', 'ppc:reminder-due']) {
      try { const stop = await listen(name, changed); if (disposed) stop(); else unlisteners.push(stop) }
      catch (failure) { if (!disposed) console.warn('提醒展示监听不可用：', failure) }
    }
  })
  onBeforeUnmount(() => { disposed = true; version++; clearTimeout(timer); unlisteners.forEach(stop => stop()); window.removeEventListener('ppc:reminders-changed', changed) })
  return { entries, total, now, loading, error, reload }
}
