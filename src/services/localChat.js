import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { createLocalChatClient, normalizeError } from './localChatClient'

export const localChat = createLocalChatClient({ invoke, listen })

export async function localInvoke(command, args) {
  try { return await invoke(command, args) }
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
