import { defineStore } from 'pinia'
import { cancelScan, onScanError, onScanProgress, startScan } from '@/api/scan'
import type { FileEntry, ScanOptions, ScanProgress } from '@/types/file'

type ScanStatus = 'idle' | 'scanning' | 'done' | 'error'

interface ScanState {
  options: ScanOptions | null
  files: FileEntry[]
  totalSize: number
  status: ScanStatus
  progress: ScanProgress
  error: string
  selectedPaths: string[]
  pendingBackup: boolean
}

export const useScanStore = defineStore('scan', {
  state: (): ScanState => ({
    options: null,
    files: [],
    totalSize: 0,
    status: 'idle',
    progress: { scanned: 0, total: null, current_file: '' },
    error: '',
    selectedPaths: [],
    pendingBackup: false
  }),
  actions: {
    setOptions(options: ScanOptions) {
      this.options = options
    },
    reset() {
      this.files = []
      this.totalSize = 0
      this.status = 'idle'
      this.progress = { scanned: 0, total: null, current_file: '' }
      this.error = ''
      this.selectedPaths = []
      this.pendingBackup = false
    },
    requestBackup(paths: string[]) {
      this.selectedPaths = [...paths]
      this.pendingBackup = true
    },
    consumeBackupRequest(): boolean {
      if (!this.pendingBackup) return false
      this.pendingBackup = false
      return true
    },
    async beginScan(options: ScanOptions) {
      this.reset()
      this.options = options
      this.status = 'scanning'
      const unProgress = await onScanProgress((p) => {
        this.progress = p
      })
      const unError = await onScanError((p) => {
        this.error = p.error
      })
      try {
        const res = await startScan(options)
        this.files = res.files
        this.totalSize = res.total_size
        this.status = 'done'
      } catch (e) {
        if (this.status === 'scanning') {
          this.status = 'error'
          this.error = this.error || (e instanceof Error ? e.message : String(e))
        }
      } finally {
        unProgress()
        unError()
      }
    },
    async cancel() {
      try {
        await cancelScan()
      } catch {
        // ignore
      }
      if (this.status === 'scanning') this.status = 'idle'
    }
  }
})
