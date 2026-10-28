// 十进制 (1000 进制) 容量格式化，与厂商 / 系统惯例一致
import { i18n } from '@/locales'

function formatNumber(value: number, format: 'decimal' | 'integer' = 'decimal'): string {
  return i18n.global.n(value, format)
}

export function formatBytes(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = bytes
  let unitIdx = 0
  while (size >= 1000 && unitIdx < units.length - 1) {
    size /= 1000
    unitIdx++
  }
  const digits = unitIdx > 0 ? 2 : 0
  return `${formatNumber(Number(size.toFixed(digits)), digits > 0 ? 'decimal' : 'integer')} ${units[unitIdx]}`
}

export function formatDate(ts: number): string {
  return i18n.global.d(new Date(ts * 1000), 'short') as string
}

export function formatDateTime(ts: number): string {
  return i18n.global.d(new Date(ts * 1000), 'long') as string
}

export function formatIsoDateTime(value?: string | null): string {
  if (!value) return '—'
  const normalized = value.includes('T') ? value : value.replace(' ', 'T')
  const date = new Date(normalized)
  if (Number.isNaN(date.getTime())) return value
  return i18n.global.d(date, 'long') as string
}

export function formatDuration(secs: number): string {
  if (secs < 60) {
    return i18n.global.t('common.duration_seconds', {
      seconds: formatNumber(Math.round(secs), 'integer')
    })
  }
  const m = Math.floor(secs / 60)
  const s = Math.round(secs % 60)
  return i18n.global.t('common.duration_minutes', {
    minutes: formatNumber(m, 'integer'),
    seconds: formatNumber(s, 'integer')
  })
}

export function formatSpeed(mbps: number): string {
  return `${formatNumber(Number(mbps.toFixed(2)))} MB/s`
}

/** 供模板直接使用，避免每次重复 toLocaleString 造成的 locale 不一致。 */
export function formatCount(value: number): string {
  return formatNumber(value, 'integer')
}
