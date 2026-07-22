export function formatBytes(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = bytes
  let unitIdx = 0
  while (size >= 1024 && unitIdx < units.length - 1) {
    size /= 1024
    unitIdx++
  }
  return `${size.toFixed(unitIdx > 0 ? 2 : 0)} ${units[unitIdx]}`
}

export function formatDate(ts: number): string {
  return new Date(ts * 1000).toLocaleDateString()
}

export function formatDuration(secs: number): string {
  if (secs < 60) return `${Math.round(secs)}s`
  const m = Math.floor(secs / 60)
  const s = Math.round(secs % 60)
  return `${m}m ${s}s`
}

export function formatSpeed(mbps: number): string {
  return `${mbps.toFixed(2)} MB/s`
}
