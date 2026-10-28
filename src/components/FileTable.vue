<script setup lang="ts">
import type { FileEntry } from '@/types/file'
import { formatBytes, formatDateTime } from '@/utils/format'

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
        {{ formatBytes(row.file_size) }}
      </template>
    </el-table-column>
    <el-table-column :label="$t('preview.modified_at')" width="180" sortable>
      <template #default="{ row }: { row: FileEntry }">
        {{ formatDateTime(row.modified_at) }}
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
