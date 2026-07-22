export interface RestoreFile {
  file_id: number
  source_root: string
  relative_path: string
  dest_path: string
  file_name: string
  file_size: number
  modified_at: number
  backed_up_at: string | null
}

export interface RestoreProgress {
  total_files: number
  processed: number
  succeeded: number
  failed: number
  current_file: string
}

export interface RestoreResult {
  total_files: number
  succeeded: number
  failed: number
  total_bytes: number
  restored_bytes: number
  duration_secs: number
  errors: string[]
}
