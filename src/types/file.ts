export interface FileEntry {
  path: string
  relative_path: string
  file_name: string
  file_size: number
  modified_at: number
  is_image: boolean
}

export interface ScanProgress {
  scanned: number
  total: number | null
  current_file: string
}

export interface ScanResult {
  files: FileEntry[]
  total_files: number
  total_size: number
}

export interface ScanOptions {
  source_path: string
  include_images?: boolean
  include_videos?: boolean
  min_size?: number
  max_size?: number
}
