import { computed, reactive, ref } from 'vue'
import defaults from '../../shared/defaultAgents.json'
import { localInvoke } from '../services/localChat'
import { onModeInvalidated, currentToken, assertToken } from './mode'

export const DEFAULT_AGENTS = defaults
export const AGENTS = reactive({})
export const activeAgentId = ref('pero')
export const agentsLoading = ref(false)
export const activeAgent = computed(() => AGENTS[activeAgentId.value] || null)
export const agentList = computed(() => Object.values(AGENTS))
let loading = null
let mutations = Promise.resolve()
let selections = Promise.resolve()
let selectionVersion = 0

onModeInvalidated(() => {
  selectionVersion++
  replaceAgents({})
  activeAgentId.value = null
  loading = null
  agentsLoading.value = false
})

function replaceAgents(agents) {
  for (const id of Object.keys(AGENTS)) if (!agents[id]) delete AGENTS[id]
  Object.assign(AGENTS, agents)
}

function announce() {
  window.dispatchEvent(new CustomEvent('ppc:agent-switched', { detail: { agentId: activeAgentId.value, roleId: activeAgentId.value } }))
}

export async function loadAgents() {
  if (loading) return loading
  agentsLoading.value = true
  const operation = (async () => {
    const version = selectionVersion
    replaceAgents(await localInvoke('local_agents'))
    const saved = await localInvoke('get_config', { key: 'ppc.activeAgent' })
    if (version === selectionVersion) activeAgentId.value = AGENTS[saved] ? saved : Object.keys(AGENTS)[0]
    return AGENTS
  })()
  loading = operation
  try { return await operation }
  finally { if (loading === operation) { loading = null; agentsLoading.value = false } }
}

export async function loadActiveAgentId() {
  if (!Object.keys(AGENTS).length) await loadAgents()
  return activeAgentId.value
}

export function getActiveAgentId() { return activeAgentId.value }

export function setActiveAgentId(id) {
  const token = currentToken()
  selectionVersion++
  const selected = selections.then(async () => {
    assertToken(token, 'standalone')
    if (!AGENTS[id]) throw new Error('角色不存在')
    await localInvoke('save_config', { key: 'ppc.activeAgent', value: id })
    activeAgentId.value = id
    announce()
    return true
  })
  selections = selected.catch(() => {})
  return selected
}

function mutate(action) {
  const token = currentToken()
  const next = mutations.then(() => { assertToken(token, 'standalone'); return action() })
  mutations = next.catch(() => {})
  return next
}

export function roleAction(action, args = {}) {
  return mutate(async () => {
    const result = await localInvoke('local_role_action', { action, ...args })
    replaceAgents(result.agents)
    announce()
    return AGENTS[result.agentId]
  })
}

export async function saveAgent(id, agent, create = false) {
  await roleAction(create ? 'create' : 'save', { agent: { ...agent, id } })
  return true
}

export function deleteAgent(id) {
  return mutate(async () => {
    const agents = await localInvoke('local_delete_agent', { id })
    replaceAgents(agents)
    if (!AGENTS[activeAgentId.value]) await setActiveAgentId(Object.keys(AGENTS)[0])
    else announce()
    return true
  })
}
