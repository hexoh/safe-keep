import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import type { BackupFileRecord, BackupHistoryEntry, BackupTask, FailedFile } from '@/types/backup'

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

export interface TaskReport {
  task: BackupTask
  files: BackupFileRecord[]
  failed: FailedFile[]
}

function baseName(path: string): string {
  const parts = path.split(/[\\/]/)
  return parts[parts.length - 1] || path
}

function generateTaskFileCsv(report: TaskReport): string {
  const header =
    'relative_path,file_name,file_size,status,dest_path,backed_up_at,modified_at,error_message'
  const successRows = report.files.map((file) =>
    [
      escapeCsv(file.relative_path),
      escapeCsv(file.file_name),
      file.file_size,
      file.status,
      escapeCsv(file.dest_path),
      file.backed_up_at ?? '',
      file.modified_at,
      escapeCsv(file.error_message ?? '')
    ].join(',')
  )
  const failedRows = report.failed.map((file) =>
    [
      escapeCsv(file.relative_path),
      escapeCsv(baseName(file.relative_path)),
      file.file_size,
      'failed',
      escapeCsv(file.dest_path ?? ''),
      '',
      '',
      escapeCsv(file.error)
    ].join(',')
  )
  return [header, ...successRows, ...failedRows].join('\n')
}

/** 导出单个备份任务的文件明细报告（成功 + 失败）。 */
export async function exportTaskReportCsv(report: TaskReport): Promise<boolean> {
  const path = await save({
    filters: [{ name: 'CSV', extensions: ['csv'] }],
    defaultPath: `backup-task-${report.task.id}.csv`
  })
  if (!path) return false
  await writeFile(path, generateTaskFileCsv(report))
  return true
}

export async function exportTaskReportJson(report: TaskReport): Promise<boolean> {
  const path = await save({
    filters: [{ name: 'JSON', extensions: ['json'] }],
    defaultPath: `backup-task-${report.task.id}.json`
  })
  if (!path) return false
  await writeFile(path, JSON.stringify(report, null, 2))
  return true
}
