export interface AppSettings {
  default_dest: string
  concurrent_threads: number
  compare_strategy: 'size_time' | 'fast_hash' | 'sha256'
  conflict_strategy: 'rename' | 'skip' | 'overwrite'
  delete_strategy: 'recycle' | 'permanent'
  dry_run_default: boolean
  auto_cleanup_reminder: boolean
  language: string
  theme: 'light' | 'dark' | 'system'
  auto_update: boolean
  update_channel: 'stable' | 'beta'
}
