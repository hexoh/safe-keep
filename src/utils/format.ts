// 十进制 (1000 进制) 容量格式化，与厂商 / 系统惯例一致
import { i18n } from '@/locales'

export function formatBytes(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = bytes
  let unitIdx = 0
  while (size >= 1000 && unitIdx < units.length - 1) {
    size /= 1000
    unitIdx++
  }
  return `${size.toFixed(unitIdx > 0 ? 2 : 0)} ${units[unitIdx]}`
}

function pad2(n: number): string {
  return String(n).padStart(2, '0')
}

export function formatDate(ts: number): string {
  return new Date(ts * 1000).toLocaleDateString()
}

export function formatDateTime(ts: number): string {
  const d = new Date(ts * 1000)
  return `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())} ${pad2(d.getHours())}:${pad2(d.getMinutes())}`
}

export function formatIsoDateTime(value?: string | null): string {
  if (!value) return '—'
  const normalized = value.includes('T') ? value : value.replace(' ', 'T')
  const date = new Date(normalized)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString()
}

export function formatDuration(secs: number): string {
  if (secs < 60) {
    return i18n.global.t('common.duration_seconds', { seconds: Math.round(secs) })
  }
  const m = Math.floor(secs / 60)
  const s = Math.round(secs % 60)
  return i18n.global.t('common.duration_minutes', { minutes: m, seconds: s })
}

export function formatSpeed(mbps: number): string {
  return `${mbps.toFixed(2)} MB/s`
}
