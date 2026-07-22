import { ref, readonly } from 'vue'
import type { ScanOptions, ScanProgress, ScanResult } from '@/types/file'
import { startScan, cancelScan, onScanProgress, onScanComplete, onScanError } from '@/api/scan'

export function useScan() {
  const scanning = ref(false)
  const cancelled = ref(false)
  const progress = ref<ScanProgress>({ scanned: 0, total: null, current_file: '' })
  const result = ref<ScanResult | null>(null)
  const error = ref<string | null>(null)

  async function scan(options: ScanOptions) {
    scanning.value = true
    cancelled.value = false
    error.value = null
    result.value = null
    progress.value = { scanned: 0, total: null, current_file: '' }

    const unlistenProgress = await onScanProgress((p) => {
      progress.value = p
    })

    const unlistenComplete = await onScanComplete(() => {})

    const unlistenError = await onScanError((e) => {
      error.value = e.error
    })

    try {
      const scanResult = await startScan(options)
      result.value = scanResult
    } catch (e: any) {
      if (e === 'cancelled') {
        cancelled.value = true
      } else {
        error.value = typeof e === 'string' ? e : (e?.message ?? 'Unknown error')
      }
    } finally {
      scanning.value = false
      unlistenProgress()
      unlistenComplete()
      unlistenError()
    }
  }

  async function cancel() {
    cancelled.value = true
    await cancelScan()
  }

  return {
    scanning: readonly(scanning),
    cancelled: readonly(cancelled),
    progress: readonly(progress),
    result: readonly(result),
    error: readonly(error),
    scan,
    cancel
  }
}
