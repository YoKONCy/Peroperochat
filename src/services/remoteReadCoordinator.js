// 同一连接的相同读取只发一次；写入使旧结果失效时，仅重试读取，不重放操作。
export function createRemoteReadCoordinator({ query, assertActive, pause = ms => new Promise(resolve => setTimeout(resolve, ms)) }) {
  const pending = new Map()
  return {
    async read(input, { cachedOnly = false, token } = {}) {
      assertActive(token)
      const key = JSON.stringify([token, Object.fromEntries(Object.entries(input).sort(([a], [b]) => a.localeCompare(b))), cachedOnly])
      if (pending.has(key)) return pending.get(key)
      const request = (async () => {
        for (let attempt = 0; ; attempt++) {
          assertActive(token)
          try {
            const result = await query(input, { cachedOnly, token })
            assertActive(token)
            return result
          } catch (error) {
            assertActive(token)
            if (cachedOnly || error.code !== 'REMOTE_STALE_READ' || attempt >= 3) throw error
            await pause([50, 150, 400][attempt])
          }
        }
      })()
      pending.set(key, request)
      const clear = () => { if (pending.get(key) === request) pending.delete(key) }
      request.then(clear, clear)
      return request
    },
  }
}
