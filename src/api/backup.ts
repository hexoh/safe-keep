import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { BackupProgress } from '@/types/backup'
import type { BackupFileRecord, BackupHistoryEntry, BackupTask } from '@/types/backup'

export async function getBackupHistory(): Promise<BackupHistoryEntry[]> {
  return invoke<BackupHistoryEntry[]>('get_backup_history')
}

export async function getBackupTasks(): Promise<BackupTask[]> {
  return invoke<BackupTask[]>('get_backup_tasks')
}

export interface StartBackupOptions {
  sourceRoot: string
  destPath: string
  files: Array<{
    source_path: string
    relative_path: string
    dest_relative_path?: string
    file_size: number
  }>
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

/** 查询某源目录下指定状态的文件记录（历史详情页的成功文件列表）。 */
export async function getSourceFiles(
  sourceRoot: string,
  statuses: string[] = ['backed_up', 'verified', 'restored'],
  from?: string | null,
  to?: string | null
): Promise<BackupFileRecord[]> {
  return invoke<BackupFileRecord[]>('get_source_files', {
    sourceRoot,
    statuses,
    from: from ?? null,
    to: to ?? null
  })
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
