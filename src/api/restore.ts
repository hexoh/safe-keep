import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { RestoreFile, RestoreResult, RestoreProgress } from '@/types/restore'

export async function getRestorableFiles(): Promise<RestoreFile[]> {
  return invoke<RestoreFile[]>('get_restorable_files')
}

export interface StartRestoreOptions {
  sourceRoot: string
  restoreTarget: string
  files: RestoreFile[]
  conflictStrategy?: string
}

export async function startRestore(options: StartRestoreOptions): Promise<RestoreResult> {
  return invoke<RestoreResult>('start_restore', { options })
}

export async function cancelRestore(): Promise<void> {
  return invoke<void>('cancel_restore')
}

export function onRestoreProgress(
  callback: (progress: RestoreProgress) => void
): Promise<() => void> {
  return listen<RestoreProgress>('restore:progress', (event) => {
    callback(event.payload)
  })
}

export function onRestoreLog(callback: (message: string) => void): Promise<() => void> {
  return listen<{ message: string }>('restore:log', (event) => {
    callback(event.payload.message)
  })
}

export function onRestoreComplete(callback: (result: RestoreResult) => void): Promise<() => void> {
  return listen<RestoreResult>('restore:complete', (event) => {
    callback(event.payload)
  })
}
