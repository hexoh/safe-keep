export type FileStatus =
  | 'new'
  | 'backed_up'
  | 'changed'
  | 'deleted'
  | 'failed'
  | 'pending'
  | 'backing_up'
  | 'verified'
  | 'restored'

export type BackupStatus = 'running' | 'paused' | 'completed' | 'cancelled' | 'failed'

export interface BackupTask {
  id: number
  source_root: string
  dest_path: string
  total_files: number
  total_bytes: number
  backed_up_files: number
  backed_up_bytes: number
  failed_files: number
  status: BackupStatus
  started_at: string | null
  completed_at: string | null
  avg_speed: number | null
}

export interface BackupHistoryEntry {
  source_root: string
  dest_path: string
  total_files: number
  total_bytes: number
  backed_up_count: number
  last_backed_up_at: string | null
}

export interface BackupProgress {
  total_files: number
  completed_files: number
  total_bytes: number
  copied_bytes: number
  current_file: string
  current_file_progress: number
  speed_mbps: number
  remaining_secs: number | null
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
  failed_files?: FailedFile[]
}

export interface FailedFile {
  source_path: string
  relative_path: string
  file_size: number
  error: string
}
