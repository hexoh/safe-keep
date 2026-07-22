import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { BackupProgress } from '@/types/backup'
import type { BackupHistoryEntry } from '@/types/backup'

export async function getBackupHistory(): Promise<BackupHistoryEntry[]> {
  return invoke<BackupHistoryEntry[]>('get_backup_history')
}

export interface StartBackupOptions {
  sourceRoot: string
  destPath: string
  files: Array<{ source_path: string; relative_path: string; file_size: number }>
  conflictStrategy?: string
  concurrency?: number
}

export interface FailedFile {
  source_path: string
  relative_path: string
  file_size: number
  error: string
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
  failed_files: FailedFile[]
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

export async function getFailedFiles(sourceRoot: string): Promise<FailedFile[]> {
  return invoke<FailedFile[]>('get_failed_files', { sourceRoot })
}

export interface RetryBackupOptions {
  sourceRoot: string
  destPath: string
  files: FailedFile[]
  conflictStrategy?: string
  concurrency?: number
}

export async function retryFailedBackup(options: RetryBackupOptions): Promise<BackupResult> {
  return invoke<BackupResult>('retry_failed_backup', { ...options })
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
