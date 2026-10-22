// 头像按文件头识别被动位图；不把 SVG、HTML 或错误响应当作可执行图片展示。
export function avatarImageMime(bytes) {
  const starts = values => values.every((value, index) => bytes[index] === value)
  const text = (start, end) => String.fromCharCode(...bytes.slice(start, end))
  if (bytes.length < 12) return ''
  if (starts([137, 80, 78, 71, 13, 10, 26, 10])) return 'image/png'
  if (starts([255, 216, 255])) return 'image/jpeg'
  if (['GIF87a', 'GIF89a'].includes(text(0, 6))) return 'image/gif'
  if (text(0, 4) === 'RIFF' && text(8, 12) === 'WEBP') return 'image/webp'
  if (text(4, 8) === 'ftyp' && ['avif', 'avis'].includes(text(8, 12))) return 'image/avif'
  if (starts([66, 77])) return 'image/bmp'
  if (starts([0, 0, 1, 0])) return 'image/x-icon'
  return ''
}
