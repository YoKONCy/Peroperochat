const messages = {
  REMOTE_NETWORK: '暂时连接不上 infOS，请检查网络。',
  REMOTE_HTTP: 'infOS 暂时没有响应，请稍后再试。',
  REMOTE_TOO_LARGE: '这部分内容暂时无法载入，请稍后再试。',
  REMOTE_AUTH_REQUIRED: '登录已过期，请重新连接。',
  CREDENTIAL_UNAVAILABLE: '请重新登录此连接。',
  PROFILE_NOT_FOUND: '此连接已被移除，请重新添加。',
  INVALID_PROFILE: '连接信息不完整，请重新添加。',
  STORAGE_ERROR: '本机保存失败，请稍后重试。',
  REMOTE_RANGE_UNSUPPORTED: '暂时无法预览这个文件，可以保存原件查看。',
  REMOTE_IDENTITY_CHANGED: '这个地址的 infOS 已更换，请重新添加连接。',
  REMOTE_INCOMPATIBLE: '当前 infOS 版本暂不支持，请更新后重试。',
  REMOTE_PROTOCOL: '暂时无法读取 infOS 的内容，请确认双方已更新。',
  REMOTE_FORBIDDEN: '当前账号没有这项操作的权限。',
  REMOTE_NOT_FOUND: '这项内容已不存在，或当前 infOS 版本暂不支持。',
  REMOTE_CONFLICT: '内容刚刚有变化，请再试一次。',
  REMOTE_INVALID_INPUT: '没能完成这项操作，请检查填写的内容。',
  REMOTE_RATE_LIMIT: '操作有些频繁，稍等一下再试吧。',
  REMOTE_PENDING_LIMIT: '还有一些操作结果未确认，请先到通知中查看。',
  REQUEST_UNRESOLVED: '这项操作还在确认中，可以到通知中查看。',
  REQUEST_ALREADY_SENT: '这项操作已经发送，请到通知或对应会话中查看结果。',
  REQUEST_ID_REUSED: '这项操作已经处理，请关闭后重新打开。',
  REQUEST_STATE_CHANGED: '这项操作已经处理，刷新即可查看。',
  REMOTE_INPUT_SEAT_RETRY: '连接刚刚恢复，请再试一次。',
}
const silentCodes = new Set(['REMOTE_STALE_READ', 'REMOTE_SELECTION_CHANGED', 'MODE_INACTIVE', 'MODE_CHANGED', 'STALE_MODE', 'CANCELLED'])
const technicalCopy = /投影|运行代次|运行身份|结果信封|输入席位|JSON|IPC|HTTP \d|TypeError|ReferenceError|SyntaxError|fetch failed|Failed to fetch|NetworkError|读取期间远程状态|请重新读取/i

// 只转换应用错误，不处理角色正文、工具结果或用户写入的内容。
export function remoteFeedbackMessage(value) {
  const code = value?.code
  if (silentCodes.has(code)) return ''
  if (value?.outcome === 'unknown') return '暂时没能确认结果，请到通知中查看。'
  if (messages[code]) return messages[code]
  const message = typeof value === 'string' ? value : value?.message || ''
  if (/读取期间远程状态|远程角色选择已经改变/.test(message)) return ''
  if (technicalCopy.test(message) || /^[A-Z_]{4,}$/.test(message)) return '暂时没能完成，请稍后再试。'
  return message
}

export function presentRemoteError(error) {
  error.message = remoteFeedbackMessage(error)
  return error
}

export function createRemoteFeedbackLifecycle({ show, dismissed = () => {}, duration = () => 5500, now = Date.now, schedule = setTimeout, cancel = clearTimeout }) {
  let timer = null, lastText = '', lastShownAt = 0, visible = false, disposed = false
  function hold() { if (timer !== null) cancel(timer); timer = null }
  function dismiss() { hold(); if (!visible || disposed) return; visible = false; show(false); dismissed() }
  function resume() { hold(); if (visible && !disposed) timer = schedule(dismiss, duration()) }
  return {
    update(text) {
      hold()
      if (disposed) return
      // 自动刷新反复遇到同一问题时不重新弹出，避免轮询让提示一直占屏。
      if (!text || text === lastText && now() - lastShownAt < 30000) { visible = false; show(false); return }
      lastText = text; lastShownAt = now(); visible = true; show(true); resume()
    },
    hold, resume, dismiss,
    dispose() { hold(); disposed = true; visible = false },
  }
}
