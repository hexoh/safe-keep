export interface CleanupFile {
  file_id: number
  source_root: string
  relative_path: string
  dest_path: string
  file_name: string
  file_size: number
  modified_at: number
  backed_up_at: string | null
}

export interface DryRunResult {
  files: CleanupFile[]
  total_files: number
  total_size: number
}

export interface CleanupProgress {
  total_files: number
  processed: number
  succeeded: number
  failed: number
  current_file: string
}

export interface CleanupResult {
  total_files: number
  succeeded: number
  failed: number
  total_size: number
  freed_size: number
  errors: string[]
}

export interface CleanupFilter {
  source_root: string
  before_days?: number
  is_image?: boolean
  min_size?: number
  max_size?: number
}
