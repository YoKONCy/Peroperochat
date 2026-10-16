import { localInvoke } from './localChat'

// 单机管理接口仅传递编辑意图，校验、编译和提交均由 Rust 完成。
export const getModelConfiguration = () => localInvoke('local_model_configuration')
export const saveApiConfiguration = (values) => localInvoke('local_save_settings', { section: 'api', values })
export const saveModelParameters = (values) => localInvoke('local_save_settings', { section: 'model', values })

export const getPromptWorkspace = () => localInvoke('local_prompt_workspace')
export const savePromptDraft = (expectedRevision, bundle) =>
  localInvoke('local_manage_prompts', { expectedRevision, action: 'draft', bundle })
export const publishPromptDraft = (expectedRevision, note = '') =>
  localInvoke('local_manage_prompts', { expectedRevision, action: 'publish', note })
export const rollbackPromptRelease = (expectedRevision, releaseId) =>
  localInvoke('local_manage_prompts', { expectedRevision, action: 'rollback', releaseId })
export const previewPrompt = (agentId, content, { channel = 'desktop', draft = true } = {}) =>
  localInvoke('local_prompt_preview', { agentId, content, channel, draft })

export const queryDiary = (agentId, query = {}) => localInvoke('local_diary_query', { agentId, query })
export const writeDiary = (agentId, draft) => localInvoke('local_manage_diary', { agentId, action: 'write', draft })
export const deleteDiary = (agentId, id) => localInvoke('local_manage_diary', { agentId, action: 'delete', id })
export const getDiaryDay = (agentId, date) => localInvoke('local_diary_day', { agentId, date })
