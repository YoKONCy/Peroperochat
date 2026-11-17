import test from 'node:test'
import assert from 'node:assert/strict'
import { createRemoteSurfaceState, applyRemoteFrame, installRemoteProjection, remoteUserSubmissionId } from '../src/services/remoteSurface.js'

function snapshot(generation, messages, deletedMessageIds = []) {
  return {
    protocolVersion: 1, threadId: 'thread-a', threadGeneration: generation, messages, deletedMessageIds,
    surfaces: messages.map(message => ({
      surfaceId: `message:${message.messageId}`, messageId: message.messageId, generation: 'publication',
      threadId: 'thread-a', principalId: 'pero', state: 'committed', revision: message.revision,
      sequence: 1, nodes: [],
    })),
  }
}
function frame(type, projection) {
  return { protocolVersion: 1, surfaceId: 'execution:a', generation: 'execution', threadId: 'thread-a',
    threadGeneration: projection.threadGeneration, operationId: `${type}:1`, sequence: 1, revision: 1,
    operation: { type, snapshot: projection } }
}

test('主线发布与侧线消息对均按提交身份确认用户输入，失败结算也能确认', () => {
  const state = createRemoteSurfaceState()
  const user = { messageId: '1', threadId: 'thread-a', role: 'user', pairId: null,
    publicationId: 'conversation:thread-a:3:request-a:input', revision: 1, content: '你好呀' }
  const failure = { messageId: 'execution:failed', threadId: 'thread-a', role: 'assistant',
    revision: 1, content: '', failureMessage: 'HTTP_400_INVALID_TOOL_SCHEMA' }
  applyRemoteFrame(state, frame('surface.settled', snapshot(3, [user, failure])))
  const confirmed = [...state.messages.values()].map(message => remoteUserSubmissionId(message, 'thread-a', 3))
  assert.deepEqual(confirmed, ['request-a', null])
  assert.equal(remoteUserSubmissionId({ ...user, pairId: 'queue_side-request', publicationId: null }, 'thread-a', 3), 'side-request')
  assert.equal(remoteUserSubmissionId({ ...user, publicationId: null }, 'thread-a', 3), null)
  assert.equal(remoteUserSubmissionId({ ...user, role: 'assistant' }, 'thread-a', 3), null)
  assert.equal(remoteUserSubmissionId(user, 'thread-b', 3), null)
  assert.equal(remoteUserSubmissionId(user, 'thread-a', 4), null)
  assert.equal(remoteUserSubmissionId({ ...user, publicationId: 'conversation:thread-a:x:request-a:input' }, 'thread-a', 3), null)
  assert.equal(remoteUserSubmissionId({ ...user, publicationId: 'conversation:thread-a:3:request-a:final' }, 'thread-a', 3), null)
})

test('发布与结算保留消息身份、角色及修订，重复回执不会生成重复记录', () => {
  const state = createRemoteSurfaceState()
  applyRemoteFrame(state, {
    protocolVersion: 1, surfaceId: 'execution:a', generation: 'execution', threadGeneration: 1,
    operationId: 'open:1', sequence: 1, revision: 1,
    operation: { type: 'surface.open', threadId: 'thread-a', principalId: 'pero', nodes: [] },
  })
  const user = { messageId: '1', role: 'user', pairId: 'queue_original', revision: 1, content: '原输入' }
  const reply = { messageId: '2', role: 'assistant', revision: 1, content: '第一份发布' }
  applyRemoteFrame(state, frame('surface.publish', snapshot(1, [user, reply])))
  applyRemoteFrame(state, frame('surface.settled', snapshot(1, [user, { ...reply, revision: 2, content: '完整发布' }])))
  applyRemoteFrame(state, frame('surface.publish', snapshot(1, [user, reply])))
  assert.equal(state.messages.size, 2)
  assert.equal(state.surfaces.size, 2)
  assert.deepEqual(state.messages.get('1'), user)
  assert.equal(state.messages.get('2').content, '完整发布')
})

test('撤回墓碑和会话代次拒绝过期历史，其他会话不能混入当前记录', () => {
  const state = createRemoteSurfaceState()
  const old = { messageId: '1', role: 'user', revision: 1, content: '已撤回' }
  installRemoteProjection(state, snapshot(1, [old]))
  applyRemoteFrame(state, frame('surface.settled', snapshot(1, [], ['1'])))
  installRemoteProjection(state, snapshot(1, [old]), { merge: true })
  assert.equal(state.messages.has('1'), false)
  assert.equal(state.surfaces.has('message:1'), false)
  applyRemoteFrame(state, frame('surface.publish', snapshot(2, [{ ...old, messageId: '2', content: '新会话代次' }])))
  applyRemoteFrame(state, frame('surface.publish', snapshot(1, [old])))
  assert.deepEqual([...state.messages.keys()], ['2'])
  assert.throws(() => installRemoteProjection(state, { ...snapshot(2, []), threadId: 'thread-b' }), { code: 'REMOTE_PROTOCOL' })
})

test('迟到的提交回执保留较新修订，并遵守已经确认的删除记录', () => {
  const state = createRemoteSurfaceState()
  const original = { messageId: '1', role: 'assistant', revision: 1, content: '旧回复' }
  const old = snapshot(1, [original])
  installRemoteProjection(state, snapshot(1, [{ ...original, revision: 2, content: '已修订' }]))
  const commit = {
    ...frame('surface.commit', old),
    operation: { type: 'surface.commit', surface: old.surfaces[0], message: original, snapshot: old },
  }
  applyRemoteFrame(state, commit)
  assert.equal(state.messages.get('1').content, '已修订')
  assert.equal(state.surfaces.get('message:1').revision, 2)
  installRemoteProjection(state, snapshot(1, [], ['1']))
  applyRemoteFrame(state, { ...commit, surfaceId: 'execution:b' })
  assert.equal(state.messages.has('1'), false)
  assert.equal(state.surfaces.has('message:1'), false)
})
