import { ref } from 'vue'
import type { RestoreFile, RestoreResult, RestoreProgress } from '@/types/restore'
import {
  getRestorableFiles as apiGetFiles,
  startRestore as apiStartRestore,
  cancelRestore as apiCancel,
  onRestoreProgress,
  onRestoreComplete
} from '@/api/restore'

export function useRestore() {
  const files = ref<RestoreFile[]>([])
  const loading = ref(false)
  const running = ref(false)
  const progress = ref<RestoreProgress | null>(null)
  const result = ref<RestoreResult | null>(null)
  const error = ref<string | null>(null)

  async function fetchFiles() {
    loading.value = true
    error.value = null
    try {
      files.value = await apiGetFiles()
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : (e?.message ?? 'Unknown error')
    } finally {
      loading.value = false
    }
  }

  async function start(options: {
    sourceRoot: string
    restoreTarget: string
    files: RestoreFile[]
    conflictStrategy?: string
  }) {
    running.value = true
    error.value = null
    result.value = null
    progress.value = null

    const unlistenProgress = await onRestoreProgress((p) => {
      progress.value = p
    })

    const unlistenComplete = await onRestoreComplete((res) => {
      result.value = res
      running.value = false
    })

    try {
      const res = await apiStartRestore(options)
      result.value = res
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
    files,
    loading,
    running,
    progress,
    result,
    error,
    fetchFiles,
    start,
    cancel
  }
}
