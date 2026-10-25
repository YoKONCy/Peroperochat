export function runTimestamp(value) {
  if (value === null || value === undefined || value === '') return null
  const number = typeof value === 'number' ? value : Date.parse(value)
  return Number.isFinite(number) && number > 0 ? number : null
}

export function elapsedLabel(milliseconds) {
  const seconds = Math.floor(Math.max(0, milliseconds) / 1000)
  if (seconds < 60) return `${seconds}s`
  const minutes = Math.floor(seconds / 60), rest = String(seconds % 60).padStart(2, '0')
  return minutes < 60 ? `${minutes}:${rest}` : `${Math.floor(minutes / 60)}:${String(minutes % 60).padStart(2, '0')}:${rest}`
}
