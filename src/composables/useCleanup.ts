import { ref } from 'vue'
import type { CleanupFilter, DryRunResult, CleanupResult, CleanupProgress } from '@/types/cleanup'
import {
  dryRunCleanup as apiDryRun,
  executeCleanup as apiExecute,
  cancelCleanup as apiCancel,
  onCleanupProgress,
  onCleanupComplete
} from '@/api/cleanup'
import type { CleanupFile } from '@/types/cleanup'

export function useCleanup() {
  const loading = ref(false)
  const running = ref(false)
  const dryRunResult = ref<DryRunResult | null>(null)
  const cleanupResult = ref<CleanupResult | null>(null)
  const progress = ref<CleanupProgress | null>(null)
  const error = ref<string | null>(null)

  async function dryRun(filter: CleanupFilter) {
    loading.value = true
    error.value = null
    dryRunResult.value = null

    try {
      const result = await apiDryRun(filter)
      dryRunResult.value = result
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : (e?.message ?? 'Unknown error')
    } finally {
      loading.value = false
    }
  }

  async function execute(files: CleanupFile[], permanent: boolean) {
    running.value = true
    error.value = null
    cleanupResult.value = null
    progress.value = null

    const unlistenProgress = await onCleanupProgress((p) => {
      progress.value = p
    })

    const unlistenComplete = await onCleanupComplete((res) => {
      cleanupResult.value = res
      running.value = false
    })

    try {
      const result = await apiExecute(files, permanent)
      cleanupResult.value = result
      running.value = false
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : (e?.message ?? 'Unknown error')
      running.value = false
    } finally {
      unlistenProgress()
      unlistenComplete()
    }
  }

  async function cancel() {
    await apiCancel()
    running.value = false
  }

  return {
    loading,
    running,
    dryRunResult,
    cleanupResult,
    progress,
    error,
    dryRun,
    execute,
    cancel
  }
}
