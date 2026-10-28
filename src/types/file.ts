export type FileStatus = 'new' | 'backed_up' | 'changed'

export interface FileEntry {
  path: string
  relative_path: string
  file_name: string
  file_size: number
  modified_at: number
  is_image: boolean
  status: FileStatus
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

export type OrganizeMode = 'YYYY/MM/DD' | 'YYYY/MM' | 'Flat'

export interface ScanOptions {
  source_path: string
  dest_path?: string
  include_images?: boolean
  include_videos?: boolean
  min_size?: number
  max_size?: number
  /** 目标目录组织方式：按日期归档或平铺。 */
  organize_by?: OrganizeMode
  /** 增量模式：扫描阶段直接过滤已备份且未变更的文件。 */
  exclude_backed_up?: boolean
}
