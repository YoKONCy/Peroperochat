// 位移以裁切框边长为单位；预览与导出的 PNG 使用同一块原图区域。
export function clampAvatarCrop(crop) {
  const zoom = Math.max(1, Math.min(3, Number.isFinite(crop.zoom) ? crop.zoom : 1))
  const edge = Math.min(crop.width, crop.height)
  const boundX = (crop.width / edge * zoom - 1) / 2, boundY = (crop.height / edge * zoom - 1) / 2
  return { ...crop, zoom, x: Math.max(-boundX, Math.min(boundX, crop.x)), y: Math.max(-boundY, Math.min(boundY, crop.y)) }
}
export function avatarCropStyle(value) {
  const crop = clampAvatarCrop(value), edge = Math.min(crop.width, crop.height)
  return { width: `${crop.width / edge * crop.zoom * 100}%`, height: `${crop.height / edge * crop.zoom * 100}%`, left: `${50 + crop.x * 100}%`, top: `${50 + crop.y * 100}%` }
}
export function avatarCropRect(value) {
  const crop = clampAvatarCrop(value), size = Math.min(crop.width, crop.height) / crop.zoom
  return { x: (crop.width - size) / 2 - crop.x * size, y: (crop.height - size) / 2 - crop.y * size, size }
}
export function isLocalOwnerMessage(message) {
  return message?.role === 'user' && (message.channel === 'desktop' || message.channel === 'group' && message.platform === 'stronghold' && message.senderId === 'user')
}
