import { ref } from 'vue'
import type { BackupProgress } from '@/types/backup'
import {
  startBackup as apiStartBackup,
  pauseBackup as apiPause,
  resumeBackup as apiResume,
  cancelBackup as apiCancel,
  onBackupProgress,
  onBackupLog,
  onBackupComplete
} from '@/api/backup'
import type { StartBackupOptions, BackupResult } from '@/api/backup'

export function useBackup() {
  const running = ref(false)
  const paused = ref(false)
  const progress = ref<BackupProgress | null>(null)
  const result = ref<BackupResult | null>(null)
  const logs = ref<string[]>([])
  const error = ref<string | null>(null)

  async function start(options: StartBackupOptions) {
    running.value = true
    paused.value = false
    error.value = null
    result.value = null
    progress.value = null
    logs.value = []

    const unlistenProgress = await onBackupProgress((p) => {
      progress.value = p
    })

    const unlistenLog = await onBackupLog((msg) => {
      logs.value.push(msg)
    })

    const unlistenComplete = await onBackupComplete((res) => {
      result.value = res
      running.value = false
    })

    try {
      const res = await apiStartBackup(options)
      result.value = res
      running.value = false
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : (e?.message ?? 'Unknown error')
      running.value = false
    } finally {
      unlistenProgress()
      unlistenLog()
      unlistenComplete()
    }
  }

  async function pause() {
    paused.value = true
    await apiPause()
  }

  async function resume() {
    paused.value = false
    await apiResume()
  }

  async function cancel() {
    await apiCancel()
    running.value = false
    paused.value = false
  }

  return {
    running,
    paused,
    progress,
    result,
    logs,
    error,
    start,
    pause,
    resume,
    cancel
  }
}
