import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import type { BackupHistoryEntry, BackupTask } from '@/types/backup'

function escapeCsv(value: string): string {
  if (value.includes(',') || value.includes('"') || value.includes('\n')) {
    return `"${value.replace(/"/g, '""')}"`
  }
  return value
}

function generateCsv(entries: BackupHistoryEntry[]): string {
  const header = 'source,destination,files,bytes,backed_up_count,last_backed_up_at'
  const rows = entries.map((e) =>
    [
      escapeCsv(e.source_root),
      escapeCsv(e.dest_path),
      e.total_files,
      e.total_bytes,
      e.backed_up_count,
      e.last_backed_up_at ?? ''
    ].join(',')
  )
  return [header, ...rows].join('\n')
}

function generateJson(entries: BackupHistoryEntry[]): string {
  return JSON.stringify(entries, null, 2)
}

async function writeFile(path: string, content: string): Promise<void> {
  return invoke<void>('write_text_file', { path, content })
}

export async function exportCsv(entries: BackupHistoryEntry[]): Promise<void> {
  const path = await save({
    filters: [{ name: 'CSV', extensions: ['csv'] }],
    defaultPath: 'backup-history.csv'
  })
  if (!path) return
  const content = generateCsv(entries)
  await writeFile(path, content)
}

export async function exportJson(entries: BackupHistoryEntry[]): Promise<void> {
  const path = await save({
    filters: [{ name: 'JSON', extensions: ['json'] }],
    defaultPath: 'backup-history.json'
  })
  if (!path) return
  const content = generateJson(entries)
  await writeFile(path, content)
}

function generateTaskCsv(tasks: BackupTask[]): string {
  const header =
    'id,source_root,dest_path,status,total_files,total_bytes,backed_up_files,failed_files,started_at,completed_at,avg_speed'
  const rows = tasks.map((task) =>
    [
      task.id,
      escapeCsv(task.source_root),
      escapeCsv(task.dest_path),
      task.status,
      task.total_files,
      task.total_bytes,
      task.backed_up_files,
      task.failed_files,
      task.started_at ?? '',
      task.completed_at ?? '',
      task.avg_speed ?? ''
    ].join(',')
  )
  return [header, ...rows].join('\n')
}

export async function exportTasksCsv(tasks: BackupTask[]): Promise<boolean> {
  const path = await save({
    filters: [{ name: 'CSV', extensions: ['csv'] }],
    defaultPath: 'backup-tasks.csv'
  })
  if (!path) return false
  await writeFile(path, generateTaskCsv(tasks))
  return true
}

export async function exportTasksJson(tasks: BackupTask[]): Promise<boolean> {
  const path = await save({
    filters: [{ name: 'JSON', extensions: ['json'] }],
    defaultPath: 'backup-tasks.json'
  })
  if (!path) return false
  await writeFile(path, JSON.stringify(tasks, null, 2))
  return true
}
