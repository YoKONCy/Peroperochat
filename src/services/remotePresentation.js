import { remoteClient } from './remote'
import { assertToken } from '../stores/mode'

// 同一分区按调用顺序写入同一原生连接；不同分区互不排队，切换会话时读取新分区不再等待旧分区落盘。
// 每次保留调用时的令牌，切换后不把旧草稿写进新分区。
const writes = new Map()
function scopeKey(scope) {
  return JSON.stringify(Object.entries(scope || {}).sort(([left], [right]) => left < right ? -1 : left > right ? 1 : 0))
}
export async function readPresentation(scope, token) {
  assertToken(token, 'remote')
  // 读取等待同一分区此前排队的保存，避免恢复到尚未落盘的旧草稿。
  await writes.get(scopeKey(scope))?.catch(() => {})
  assertToken(token, 'remote')
  return (await remoteClient.uiRead(scope, token)).data
}
export function writePresentation(scope, value, token) {
  const key = scopeKey(scope), snapshot = JSON.parse(JSON.stringify(value))
  const pending = (writes.get(key) || Promise.resolve()).catch(() => {}).then(() => {
    assertToken(token, 'remote')
    return remoteClient.uiWrite(scope, snapshot, token)
  })
  writes.set(key, pending)
  // 分区落盘后移除记录，避免随打开过的会话数量增长。
  pending.finally(() => { if (writes.get(key) === pending) writes.delete(key) }).catch(() => {})
  return pending
}
