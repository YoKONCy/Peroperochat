// infOS Surface v1 的展示合成器。只合并服务端帧，不解释或执行工具参数与可执行节点。
const operationTypes = new Set(['surface.open', 'surface.append-text', 'surface.append-tool-arguments', 'surface.upsert-node', 'surface.patch-node', 'surface.commit', 'surface.fail', 'surface.dispose'])
const supportedKinds = new Set(['markdown', 'native-reasoning', 'thinking', 'model-retry', 'code', 'math', 'mermaid', 'tool-call', 'tool-result', 'status', 'error', 'attachment', 'approval', 'progress', 'input', 'panel', 'table', 'notification', 'document'])

function protocolError() { return Object.assign(new Error('远程 Surface 数据不兼容，请刷新权威投影'), { code: 'REMOTE_PROTOCOL' }) }
function nodeView(node) {
  if (!node || typeof node.nodeId !== 'string' || typeof node.kind !== 'string' || !node.props || typeof node.props !== 'object') throw protocolError()
  return { ...node, props: { ...node.props }, unsupported: !supportedKinds.has(node.kind) }
}
function descriptorView(surface) {
  if (!surface || typeof surface.surfaceId !== 'string' || typeof surface.generation !== 'string' || typeof surface.threadId !== 'string' || typeof surface.principalId !== 'string' ||
      !Number.isSafeInteger(surface.revision) || !Number.isSafeInteger(surface.sequence) || !['open', 'committed', 'failed', 'disposed'].includes(surface.state) ||
      !Array.isArray(surface.nodes) || surface.nodes.length > 1000) throw protocolError()
  return { ...surface, nodes: surface.nodes.map(nodeView), operationIds: new Set(), needsRefresh: false }
}
export function createRemoteSurfaceState() { return { surfaces: new Map(), retired: new Map(), needsRefresh: false } }

export function installRemoteProjection(state, projection, { merge = false } = {}) {
  if (projection?.protocolVersion !== 1 || !Array.isArray(projection.surfaces) || projection.surfaces.length > 500) throw protocolError()
  const next = merge ? new Map(state.surfaces) : new Map()
  for (const descriptor of projection.surfaces) {
    const surface = descriptorView(descriptor)
    const current = next.get(surface.surfaceId)
    if (current?.generation === surface.generation && current.revision > surface.revision) continue
    next.set(surface.surfaceId, surface)
  }
  state.surfaces = next
  state.needsRefresh = false
}
function upsert(surface, node) {
  const view = nodeView(node)
  const index = surface.nodes.findIndex(item => item.nodeId === node.nodeId)
  if (index < 0) surface.nodes.push(view)
  else surface.nodes[index] = view
  if (surface.nodes.length > 1000) throw protocolError()
}

export function applyRemoteFrame(state, frame) {
  if (frame?.protocolVersion !== 1 || typeof frame.surfaceId !== 'string' || typeof frame.generation !== 'string' || typeof frame.operationId !== 'string' ||
      !Number.isSafeInteger(frame.sequence) || !Number.isSafeInteger(frame.revision) || !operationTypes.has(frame.operation?.type)) throw protocolError()
  const operation = frame.operation
  if (state.retired.get(frame.surfaceId) === frame.generation) return
  let current = state.surfaces.get(frame.surfaceId)
  if (current && current.generation !== frame.generation) {
    if (operation.type !== 'surface.open') { state.needsRefresh = true; return }
    current = null
  }
  if (current && (current.operationIds.has(frame.operationId) || frame.sequence <= current.sequence)) return
  if (current && frame.sequence !== current.sequence + 1) {
    state.needsRefresh = true
    // 出现缺帧时只接受带完整服务端描述的 commit，其他增量不拼成貌似完整的内容。
    if (operation.type !== 'surface.commit') return
  }
  if (!current && operation.type !== 'surface.open' && operation.type !== 'surface.commit') { state.needsRefresh = true; return }
  let surface
  if (operation.type === 'surface.open') {
    surface = descriptorView({ surfaceId: frame.surfaceId, generation: frame.generation, threadId: operation.threadId, principalId: operation.principalId,
      executionId: frame.executionId, state: 'open', nodes: operation.nodes || [], sequence: frame.sequence, revision: frame.revision })
  } else if (operation.type === 'surface.commit') {
    surface = descriptorView(operation.surface)
    if (!['committed', 'failed'].includes(surface.state) || (current && (current.threadId !== surface.threadId || current.principalId !== surface.principalId))) throw protocolError()
    // 流式预览与持久消息使用不同身份；最终描述的序号、版本不可被预览帧覆盖。
    if (frame.surfaceId !== surface.surfaceId) {
      state.surfaces.delete(frame.surfaceId)
      state.retired.set(frame.surfaceId, frame.generation)
      if (state.retired.size > 500) state.retired.delete(state.retired.keys().next().value)
    }
    const committed = state.surfaces.get(surface.surfaceId)
    if (!committed || committed.revision <= surface.revision) state.surfaces.set(surface.surfaceId, surface)
    if (state.surfaces.size > 500) { state.surfaces.delete(state.surfaces.keys().next().value); state.needsRefresh = true }
    return
  } else {
    surface = { ...current, nodes: current.nodes.slice(), operationIds: new Set(current.operationIds) }
    const existing = surface.nodes.find(node => node.nodeId === operation.nodeId)
    if (operation.type === 'surface.append-text') {
      if (typeof operation.delta !== 'string') throw protocolError()
      const source = (existing?.props.source || '') + operation.delta
      if (source.length > 2 * 1024 * 1024) throw protocolError()
      upsert(surface, { ...(existing || { nodeId: operation.nodeId, kind: 'markdown', lifecycle: 'stable' }), revision: frame.revision,
        props: { ...(existing?.props || { phase: 'preview' }), source } })
    } else if (operation.type === 'surface.append-tool-arguments') {
      if (!existing || existing.kind !== 'tool-call') { state.needsRefresh = true; return }
      const combined = (existing.props.args || '') + (operation.argumentsDelta || '')
      const args = combined.slice(0, 64000)
      upsert(surface, { ...existing, revision: frame.revision, props: { ...existing.props, name: (existing.props.name || '') + (operation.nameDelta || ''),
        args, argsPreview: args, receivedChars: operation.receivedChars, previewTruncated: existing.props.previewTruncated || operation.previewTruncated || combined.length > 64000 } })
    } else if (operation.type === 'surface.upsert-node') {
      const previous = surface.nodes.find(node => node.nodeId === operation.node?.nodeId)
      upsert(surface, { ...operation.node, props: { ...previous?.props, ...operation.node?.props } })
    } else if (operation.type === 'surface.patch-node') {
      if (!existing) { state.needsRefresh = true; return }
      upsert(surface, { ...existing, revision: frame.revision, props: { ...existing.props, ...operation.patch } })
    } else if (operation.type === 'surface.fail') {
      surface.state = 'failed'
      surface.nodes = surface.nodes.filter(node => node.kind !== 'status').map(node => node.kind === 'tool-call' && !['completed', 'failed'].includes(node.props.state)
        ? { ...node, props: { ...node.props, state: 'failed' } } : node)
      upsert(surface, { nodeId: `${surface.surfaceId}:error`, kind: 'error', revision: frame.revision, lifecycle: 'stable', props: { code: operation.code, message: operation.message } })
    } else if (operation.type === 'surface.dispose') {
      state.surfaces.delete(frame.surfaceId)
      return
    }
  }
  surface.sequence = frame.sequence
  surface.revision = frame.revision
  surface.operationIds.add(frame.operationId)
  if (surface.operationIds.size > 2048) surface.operationIds.delete(surface.operationIds.values().next().value)
  state.surfaces.set(frame.surfaceId, surface)
  if (state.surfaces.size > 500) { state.surfaces.delete(state.surfaces.keys().next().value); state.needsRefresh = true }
}
