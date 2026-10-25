import { observeRemoteResult } from './remoteObservation.js'

// 同一连接的相同读取只发一次；写入使旧结果失效时，仅重试读取，不重放操作。
export function createRemoteReadCoordinator({ query, assertActive, pause = ms => new Promise(resolve => setTimeout(resolve, ms)) }) {
  const pending = new Map()
  let generation = 0
  return {
    reset() {
      generation++
      for (const entry of pending.values()) entry.controller.abort()
      pending.clear()
    },
    async read(input, { cachedOnly = false, token } = {}) {
      assertActive(token)
      const key = JSON.stringify([token, Object.fromEntries(Object.entries(input).sort(([a], [b]) => a.localeCompare(b))), cachedOnly])
      if (pending.has(key)) return pending.get(key).request
      const controller = new AbortController(), version = generation
      const request = (async () => {
        for (let attempt = 0; ; attempt++) {
          assertActive(token)
          if (version !== generation) throw Object.assign(new Error('正在同步最新内容。'), { code: 'REMOTE_STALE_READ', superseded: true })
          try {
            const result = await observeRemoteResult(query(input, { cachedOnly, token }), { signal: controller.signal })
            assertActive(token)
            if (version !== generation) throw Object.assign(new Error('正在同步最新内容。'), { code: 'REMOTE_STALE_READ', superseded: true })
            return result
          } catch (error) {
            assertActive(token)
            if (version !== generation || cachedOnly || error.code !== 'REMOTE_STALE_READ' || attempt >= 3) throw error
            await pause([50, 150, 400][attempt])
          }
        }
      })()
      pending.set(key, { request, controller })
      const clear = () => { if (pending.get(key)?.request === request) pending.delete(key) }
      request.then(clear, clear)
      return request
    },
  }
}
