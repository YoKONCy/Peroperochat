export const ttsAdapterLabels = { edge_tts: 'Edge TTS', openai: 'OpenAI', mimo: 'MiMo', bailian: '阿里百炼', minimax: 'MiniMax', stepfun: '阶跃星辰', custom: '自定义 HTTP' }
export function ttsChain(workspace, presetId) {
  const preset = workspace?.presets?.find(item => item.id === presetId)
  const model = workspace?.models?.find(item => item.id === preset?.modelId)
  return { preset, model, profile: workspace?.profiles?.find(item => item.id === model?.profileId), connection: workspace?.connections?.find(item => item.id === model?.connectionId) }
}
export function addTtsPreset(workspace, adapter, name) {
  const id = crypto.randomUUID()
  workspace.connections.push({ id, name, apiBase: adapter.apiBase, apiKey: '', headers: {}, timeoutMs: 60000 })
  workspace.profiles.push({ id, adapter: adapter.id, ...(adapter.id === 'custom' ? { custom: { path: '/audio/speech', method: 'POST', bodyType: 'json', body: { model: '{{model}}', input: '{{text}}', voice: '{{voice}}', response_format: '{{format}}' }, response: { mode: 'binary', format: 'mp3' } } } : {}) })
  workspace.models.push({ id, connectionId: id, profileId: id, modelId: adapter.models[0] || (adapter.id === 'custom' ? 'custom' : '') })
  workspace.presets.push({ id, name, modelId: id, voice: adapter.voice, format: adapter.formats[0], speed: 1, rate: '+25%', pitch: '+5Hz', language: '', instructions: '', emotion: '', referenceText: '', extra: {} })
  return id
}
export function removeTtsPreset(workspace, id) {
  if (workspace.presets.length < 2) return
  workspace.presets = workspace.presets.filter(item => item.id !== id)
  if (workspace.defaultPresetId === id) workspace.defaultPresetId = workspace.presets[0].id
  for (const [agent, preset] of Object.entries(workspace.bindings)) if (preset === id) delete workspace.bindings[agent]
}
