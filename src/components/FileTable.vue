<script setup lang="ts">
import type { FileEntry } from '@/types/file'

defineProps<{
  files: FileEntry[]
  loading?: boolean
}>()

const selected = defineModel<string[]>('selected', { default: () => [] })

const statusTag = {
  new: 'success',
  backed_up: 'info',
  changed: 'warning'
} as const

function formatSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = bytes
  let unitIdx = 0
  while (size >= 1024 && unitIdx < units.length - 1) {
    size /= 1024
    unitIdx++
  }
  return `${size.toFixed(unitIdx > 0 ? 2 : 0)} ${units[unitIdx]}`
}

function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString()
}
</script>

<template>
  <el-table
    :data="files"
    :loading="loading"
    style="width: 100%"
    size="small"
    @selection-change="(rows: FileEntry[]) => (selected = rows.map((r) => r.path))"
  >
    <el-table-column type="selection" width="40" />
    <el-table-column
      prop="file_name"
      :label="$t('preview.file_name')"
      min-width="200"
      show-overflow-tooltip
    />
    <el-table-column :label="$t('preview.size')" width="100" sortable>
      <template #default="{ row }: { row: FileEntry }">
        {{ formatSize(row.file_size) }}
      </template>
    </el-table-column>
    <el-table-column :label="$t('preview.modified_at')" width="180" sortable>
      <template #default="{ row }: { row: FileEntry }">
        {{ formatDate(row.modified_at) }}
      </template>
    </el-table-column>
    <el-table-column :label="$t('preview.status')" width="100" sortable>
      <template #default="{ row }: { row: FileEntry }">
        <el-tag :type="statusTag[row.status] || 'info'" size="small">
          {{ $t(`status.${row.status}`) }}
        </el-tag>
      </template>
    </el-table-column>
  </el-table>
</template>
