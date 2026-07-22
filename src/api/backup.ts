import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { BackupProgress } from '@/types/backup'

export interface StartBackupOptions {
  sourceRoot: string
  destPath: string
  files: Array<{ source_path: string; relative_path: string; file_size: number }>
  conflictStrategy?: string
  concurrency?: number
}

export interface BackupResult {
  total_files: number
  succeeded: number
  failed: number
  total_bytes: number
  copied_bytes: number
  duration_secs: number
  avg_speed_mbps: number
  errors: string[]
}

export async function startBackup(options: StartBackupOptions): Promise<BackupResult> {
  return invoke<BackupResult>('start_backup', { options })
}

export async function pauseBackup(): Promise<void> {
  return invoke<void>('pause_backup')
}

export async function resumeBackup(): Promise<void> {
  return invoke<void>('resume_backup')
}

export async function cancelBackup(): Promise<void> {
  return invoke<void>('cancel_backup')
}

export function onBackupProgress(
  callback: (progress: BackupProgress) => void
): Promise<() => void> {
  return listen<BackupProgress>('backup:progress', (event) => {
    callback(event.payload)
  })
}

export function onBackupLog(callback: (message: string) => void): Promise<() => void> {
  return listen<{ message: string }>('backup:log', (event) => {
    callback(event.payload.message)
  })
}

export function onBackupComplete(callback: (result: BackupResult) => void): Promise<() => void> {
  return listen<BackupResult>('backup:complete', (event) => {
    callback(event.payload)
  })
}
