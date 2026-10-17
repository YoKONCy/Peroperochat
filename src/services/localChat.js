import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { createLocalChatClient, normalizeError } from './localChatClient'
import { assertToken, currentToken } from '../stores/mode'

async function standaloneInvoke(command, args) {
  const token = currentToken()
  assertToken(token, 'standalone')
  const result = await invoke(command, { ...args, modeToken: token })
  assertToken(token, 'standalone')
  return result
}

async function standaloneListen(event, callback) {
  const token = currentToken()
  assertToken(token, 'standalone')
  return listen(event, message => {
    try { assertToken(token, 'standalone') } catch { return }
    callback(message)
  })
}

export const localChat = createLocalChatClient({ invoke: standaloneInvoke, listen: standaloneListen })

export async function localInvoke(command, args) {
  try { return await standaloneInvoke(command, args) }
  catch (error) { throw normalizeError(error) }
}

export function publishCharacterState(state) {
  if (!state || state.agentId !== window.WAIFU_GET_AGENT_ID?.()) return
  window.dispatchEvent(new CustomEvent('ppc:pero-status-updated', { detail: state }))
  for (const key of ['mood', 'vibe', 'mind']) {
    if (typeof state[key] === 'string') window.dispatchEvent(new CustomEvent(`ppc:${key}`, { detail: state[key] }))
  }
  if (state.texts) window.dispatchEvent(new CustomEvent('ppc:waifu-texts-updated', { detail: state.texts }))
}
