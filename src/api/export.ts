import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import type { BackupHistoryEntry } from '@/types/backup'

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
