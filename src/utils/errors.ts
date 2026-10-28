import { i18n } from '@/locales'
import { formatBytes } from '@/utils/format'

/**
 * 将后端返回的错误码转换为当前语言的提示文本。
 * 未识别的错误原样返回，便于排查问题。
 */
export function translateError(error: string): string {
  if (error.startsWith('ERR_DISK_FULL')) {
    const match = error.match(/need (\d+) bytes, only (\d+) bytes available/)
    return i18n.global.t('errors.disk_full', {
      required: formatBytes(Number(match?.[1] ?? 0)),
      available: formatBytes(Number(match?.[2] ?? 0))
    })
  }
  return error
}
