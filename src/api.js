import { reactive } from 'vue'
import { localInvoke } from './services/localChat'
import { getActiveAgentId } from './stores/agents'
import { clearConversationCache } from './stores/conversations'
export { AGENTS, DEFAULT_AGENTS, loadAgents, saveAgent, deleteAgent,
  loadActiveAgentId, getActiveAgentId, setActiveAgentId } from './stores/agents'

function readRemoteConfig() {
  try { return JSON.parse(localStorage.getItem('remote_config') || '{}') }
  catch { return {} }
}
export const appMode = reactive({
  current: localStorage.getItem('app_mode') || 'standalone',
  remoteConfig: { url: '', token: '', connected: false, ...readRemoteConfig() },
})
export function setAppMode(mode) {
  appMode.current = mode
  localStorage.setItem('app_mode', mode)
}
export function saveRemoteConfig(config) {
  Object.assign(appMode.remoteConfig, config)
  localStorage.setItem('remote_config', JSON.stringify(appMode.remoteConfig))
}

export const saveConfig = (key, value) => localInvoke('save_config', { key, value: String(value ?? '') })
export const getConfig = (key) => localInvoke('get_config', { key })
export const saveApiKey = (key) => localInvoke('save_api_key', { key })
export const getApiKey = () => localInvoke('get_api_key')

export function getRecentMemories(limit = 10, offset = 0, agentId = getActiveAgentId()) {
  return localInvoke('get_memories', { agentId, limit, offset })
}
export const getAllMemories = () => getRecentMemories(10000)
export const deleteMemory = (id) => localInvoke('delete_memory', { id, agentId: getActiveAgentId() })
export function getTopics(agentId = getActiveAgentId()) {
  return localInvoke('get_topics', { agentId })
}
export const updateTopicStatus = (id, revealed) => localInvoke('update_topic_status', { id, revealed })
export const deleteTopic = (id) => localInvoke('delete_topic', { id })

export async function resetAll() {
  await localInvoke('local_reset_data', { clearConfig: true })
  clearConversationCache()
  return { ok: true }
}
export const exportAllData = () => localInvoke('local_export_data')
export async function importAllData(data) {
  await localInvoke('local_import_data', { data })
  clearConversationCache()
  return { ok: true }
}
export async function uploadModel() {
  console.warn('移动端暂不支持本地模型上传')
  return null
}
