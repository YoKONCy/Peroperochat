// 只结束前端等待，不取消原生请求，更不重发写操作；后台回来时按实际经过时间核对超时。
export function observeRemoteResult(operation, { signal, timeout = 75000, error = () => Object.assign(new Error('连接暂时没有回应，正在重新同步。'), { code: 'REMOTE_NETWORK' }) } = {}) {
  return new Promise((resolve, reject) => {
    const deadline = Date.now() + timeout
    let settled = false, timer
    const finish = (callback, value) => {
      if (settled) return
      settled = true; clearTimeout(timer)
      signal?.removeEventListener('abort', superseded)
      globalThis.document?.removeEventListener('visibilitychange', check)
      globalThis.window?.removeEventListener('pageshow', check)
      callback(value)
    }
    const superseded = () => finish(reject, Object.assign(new Error('正在同步最新内容。'), { code: 'REMOTE_STALE_READ', superseded: true }))
    const check = () => {
      if (settled) return
      if (Date.now() >= deadline) finish(reject, error())
      else { clearTimeout(timer); timer = setTimeout(check, deadline - Date.now()) }
    }
    signal?.addEventListener('abort', superseded, { once: true })
    globalThis.document?.addEventListener('visibilitychange', check)
    globalThis.window?.addEventListener('pageshow', check)
    timer = setTimeout(check, timeout)
    Promise.resolve(operation).then(value => finish(resolve, value), failure => finish(reject, failure))
    if (signal?.aborted) superseded()
  })
}
