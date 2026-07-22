import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { ScanOptions, ScanProgress, ScanResult } from '@/types/file'

export async function startScan(options: ScanOptions): Promise<ScanResult> {
  return invoke<ScanResult>('start_scan', { options })
}

export async function cancelScan(): Promise<void> {
  return invoke<void>('cancel_scan')
}

export function onScanProgress(callback: (progress: ScanProgress) => void): Promise<() => void> {
  return listen<ScanProgress>('scan:progress', (event) => {
    callback(event.payload)
  })
}

export function onScanComplete(
  callback: (payload: { total: number }) => void
): Promise<() => void> {
  return listen<{ total: number }>('scan:complete', (event) => {
    callback(event.payload)
  })
}

export function onScanError(callback: (payload: { error: string }) => void): Promise<() => void> {
  return listen<{ error: string }>('scan:error', (event) => {
    callback(event.payload)
  })
}
