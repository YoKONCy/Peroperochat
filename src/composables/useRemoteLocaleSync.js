import { watch, onMounted, onBeforeUnmount } from 'vue'
import { currentToken, assertToken, sameToken } from '../stores/mode'
import { remoteState } from '../stores/remote'
import { remoteClient } from '../services/remote'
import { setRemoteLocale, remoteLocale } from '../i18n/remote'

export function useRemoteLocaleSync() {
  const token = currentToken()
  const originalLanguage = document.documentElement.lang
  watch(remoteLocale, value => { document.documentElement.lang = value }, { immediate: true })
  let disposed = false, reading = false, disposeEvents = null
  setRemoteLocale('zh-CN')
  async function refresh() {
    if (disposed || reading || remoteState.connection !== 'online' || document.hidden) return
    reading = true
    try { const result = await remoteClient.manage({ operation: 'configs.batch', body: { keys: ['user.language'] } }, { token }); assertToken(token, 'remote'); if (!disposed) setRemoteLocale(result.data?.['user.language']) }
    catch { /* 语言读取失败不阻断对话；重连或切回时再与服务端同步。 */ }
    finally { reading = false }
  }
  watch(() => remoteState.connection, refresh)
  onMounted(async () => {
    document.addEventListener('visibilitychange', refresh)
    await refresh()
    if (disposed) return
    try {
      const dispose = await remoteClient.listen(event => {
        const payload = event.kind === 'gateway' ? event.data?.payload : null
        if (payload?.action === 'state_update' && typeof payload.language === 'string') { try { assertToken(token, 'remote'); if (!disposed) setRemoteLocale(payload.language) } catch { /* 已切换连接。 */ } }
      })
      if (disposed) dispose(); else disposeEvents = dispose
    } catch { /* 事件监听不可用时，切回应用仍会重新读取语言。 */ }
  })
  onBeforeUnmount(() => {
    disposed = true; disposeEvents?.(); document.removeEventListener('visibilitychange', refresh)
    try { const active = currentToken(); assertToken(active, 'remote'); if (!sameToken(active, token)) return } catch { /* 已返回独立模式。 */ }
    setRemoteLocale('zh-CN'); document.documentElement.lang = originalLanguage
  })
}
