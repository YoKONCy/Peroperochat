import { remoteClient } from './remote'
import { assertToken } from '../stores/mode'

// 按调用顺序写入同一原生连接；每次保留调用时的令牌，切换后不把旧草稿写进新分区。
let writes = Promise.resolve()
export async function readPresentation(scope, token) {
  assertToken(token, 'remote')
  // 读取等待此前排队的保存，避免恢复到尚未落盘的旧草稿。
  await writes.catch(() => {})
  assertToken(token, 'remote')
  return (await remoteClient.uiRead(scope, token)).data
}
export function writePresentation(scope, value, token) {
  const snapshot = JSON.parse(JSON.stringify(value))
  const pending = writes.catch(() => {}).then(() => {
    assertToken(token, 'remote')
    return remoteClient.uiWrite(scope, snapshot, token)
  })
  writes = pending
  return pending
}
