import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { CleanupFilter, DryRunResult, CleanupResult, CleanupProgress } from '@/types/cleanup'

export async function dryRunCleanup(filter: CleanupFilter): Promise<DryRunResult> {
  return invoke<DryRunResult>('dry_run_cleanup', { filter })
}

export async function executeCleanup(
  files: Array<{
    file_id: number
    source_root: string
    relative_path: string
    dest_path: string
    file_name: string
    file_size: number
    modified_at: number
    backed_up_at: string | null
  }>,
  permanent: boolean
): Promise<CleanupResult> {
  return invoke<CleanupResult>('execute_cleanup', { files, permanent })
}

export async function cancelCleanup(): Promise<void> {
  return invoke<void>('cancel_cleanup')
}

export async function getSourceRoots(): Promise<string[]> {
  return invoke<string[]>('get_source_roots')
}

export function onCleanupProgress(
  callback: (progress: CleanupProgress) => void
): Promise<() => void> {
  return listen<CleanupProgress>('cleanup:progress', (event) => {
    callback(event.payload)
  })
}

export function onCleanupComplete(callback: (result: CleanupResult) => void): Promise<() => void> {
  return listen<CleanupResult>('cleanup:complete', (event) => {
    callback(event.payload)
  })
}
