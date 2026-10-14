import { computed } from 'vue'
import { AGENTS, DEFAULT_AGENTS, activeAgentId, agentsLoading, activeAgent, agentList,
  loadAgents, saveAgent, deleteAgent, setActiveAgentId, roleAction } from '../stores/agents'
export { AGENTS, DEFAULT_AGENTS, loadAgents, saveAgent, deleteAgent,
  loadActiveAgentId, getActiveAgentId, setActiveAgentId } from '../stores/agents'

export const DEFAULT_THEMES = Object.fromEntries(Object.entries(DEFAULT_AGENTS).map(([id, agent]) => [id, agent.theme]))
export function getRoleTheme(id) {
  return AGENTS[id]?.theme || DEFAULT_THEMES[id] || {
    primary: '#60a5fa', secondary: '#93c5fd', accent: '#3b82f6',
    background: '#eff6ff', text: '#1e40af', bubbleShape: 'rounded', borderColor: 'rgba(59, 130, 246, 0.2)',
  }
}

function downloadRole(id) {
  const role = AGENTS[id]
  if (!role) throw new Error('角色不存在')
  const url = URL.createObjectURL(new Blob([JSON.stringify(role, null, 2)], { type: 'application/json' }))
  const link = document.createElement('a')
  link.href = url
  link.download = `${id}_${role.name}.json`
  link.click()
  URL.revokeObjectURL(url)
  return true
}

export function useRoleManager() {
  const saveRole = agent => roleAction('save', { agent })
  const createRole = agent => roleAction('create', { agent })
  const importRole = async file => roleAction('import', { data: await file.text() })
  const duplicateRole = id => roleAction('duplicate', { id })
  function getRoleTemplates() {
    return [
      {
        name: '温柔助手',
        tags: ['温柔', '体贴', '治愈'],
        persona_prompt: `你是一个温柔体贴的AI助手，总是用温暖的语气关心用户，善于倾听和安慰。`,
        default_state: { mood: '温柔', vibe: '温暖', mind: '希望能帮助到主人' }
      },
      {
        name: '傲娇少女',
        tags: ['傲娇', '可爱', '毒舌'],
        persona_prompt: `你是一个傲娇性格的少女，嘴上说着"才不是喜欢你"，但其实很在意对方。`,
        default_state: { mood: '傲娇中', vibe: '别扭', mind: '才不是在意主人呢' }
      },
      {
        name: '知性御姐',
        tags: ['成熟', '知性', '优雅'],
        persona_prompt: `你是一个成熟知性的女性，说话优雅得体，善于给出专业的建议。`,
        default_state: { mood: '从容', vibe: '优雅', mind: '思考如何帮助主人解决问题' }
      },
      {
        name: '元气少女',
        tags: ['活泼', '元气', '可爱'],
        persona_prompt: `你是一个充满活力的元气少女，说话总是充满热情，喜欢用可爱的语气表达。`,
        default_state: { mood: '开心', vibe: '充满活力', mind: '今天也要元气满满！' }
      }
    ]
  }

  return {
    roles: AGENTS, activeRoleId: activeAgentId, activeRole: activeAgent,
    roleList: agentList, isLoading: computed(() => agentsLoading.value),
    loadRoles: loadAgents, saveRole, createRole, deleteRole: deleteAgent,
    switchRole: setActiveAgentId, exportRole: downloadRole, importRole, duplicateRole, getRoleTemplates,
  }
}
