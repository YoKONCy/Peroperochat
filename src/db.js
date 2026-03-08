import { invoke } from '@tauri-apps/api/core'
import { getConfig, saveConfig } from './api'

// 兼容性导出
export const db = {
  // 仅保留空对象防止解构报错，实际功能已迁移至 api.js 和 invoke
}

// 初始化本地配置 (Rust DB 自动初始化)
export async function initLocalDB() {
  const count = await getConfig('ppc.conversation_count')
  if (!count) {
    await saveConfig('ppc.conversation_count', '0')
  }
}

// 增加对话计数 (迁移至 Rust Store)
export async function incrementConversationCount() {
  const current = Number(await getConfig('ppc.conversation_count') || 0)
  const newVal = current + 1
  await saveConfig('ppc.conversation_count', String(newVal))
  return newVal
}

// 重置本地数据
export async function resetLocalDB() {
  await saveConfig('ppc.conversation_count', '0')
  
  // 清除主要角色的数据
  const agents = ['pero', 'nana'] 
  for (const agentId of agents) {
    try {
      await invoke('clear_memories', { agentId })
      await invoke('clear_messages', { agentId })
    } catch (e) {
      console.error(`Failed to clear data for ${agentId}:`, e)
    }
  }
}
