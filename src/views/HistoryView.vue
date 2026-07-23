<script setup lang="ts">
import { getBackupHistory, getFailedFiles, retryFailedBackup } from '@/api/backup'
import { exportCsv, exportJson } from '@/api/export'
import { formatBytes } from '@/utils/format'
import type { BackupHistoryEntry, FailedFile } from '@/types/backup'

const entries = ref<BackupHistoryEntry[]>([])
const loading = ref(false)
const exporting = ref(false)
const error = ref<string | null>(null)

const failedDialogVisible = ref(false)
const currentFailedFiles = ref<FailedFile[]>([])
const currentRetrySource = ref('')
const currentRetryDest = ref('')
const retrying = ref(false)

async function fetchHistory() {
  loading.value = true
  error.value = null
  try {
    entries.value = await getBackupHistory()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : (e?.message ?? 'Unknown error')
  } finally {
    loading.value = false
  }
}

async function handleExportCsv() {
  exporting.value = true
  try {
    await exportCsv(entries.value)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : (e?.message ?? 'Unknown error')
  } finally {
    exporting.value = false
  }
}

async function handleExportJson() {
  exporting.value = true
  try {
    await exportJson(entries.value)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : (e?.message ?? 'Unknown error')
  } finally {
    exporting.value = false
  }
}

async function showFailedFiles(entry: BackupHistoryEntry) {
  currentRetrySource.value = entry.source_root
  currentRetryDest.value = entry.dest_path
  try {
    currentFailedFiles.value = await getFailedFiles(entry.source_root)
    failedDialogVisible.value = true
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : (e?.message ?? 'Unknown error')
  }
}

async function handleRetry() {
  retrying.value = true
  try {
    const result = await retryFailedBackup({
      sourceRoot: currentRetrySource.value,
      destPath: currentRetryDest.value,
      files: currentFailedFiles.value
    })
    failedDialogVisible.value = false
    ElMessage.success(`Retry complete: ${result.succeeded} succeeded, ${result.failed} failed`)
    await fetchHistory()
  } catch (e: any) {
    ElMessage.error(typeof e === 'string' ? e : (e?.message ?? 'Unknown error'))
  } finally {
    retrying.value = false
  }
}

onMounted(fetchHistory)
</script>

<template>
  <div class="history-view">
    <div class="page-header">
      <h1>{{ $t('history.title') }}</h1>
      <div v-if="entries.length > 0" class="export-actions">
        <el-button size="small" :loading="exporting" @click="handleExportCsv">
          {{ $t('history.export_csv') }}
        </el-button>
        <el-button size="small" :loading="exporting" @click="handleExportJson">
          {{ $t('history.export_json') }}
        </el-button>
      </div>
    </div>

    <div v-if="loading" class="loading-state">
      <el-skeleton :rows="5" animated />
    </div>

    <div v-else-if="error" class="error-state">
      <el-result status="error" :title="$t('common.error')" :sub-title="error">
        <template #extra>
          <el-button @click="fetchHistory">{{ $t('common.retry') }}</el-button>
        </template>
      </el-result>
    </div>

    <template v-else-if="entries.length > 0">
      <div class="history-list">
        <el-card v-for="(entry, i) in entries" :key="i" shadow="hover" class="history-card">
          <div class="history-card-header">
            <div class="history-source">
              <el-tag type="primary" size="small">{{ $t('history.source') }}</el-tag>
              <span class="path-text">{{ entry.source_root }}</span>
            </div>
            <div class="history-dest">
              <el-tag type="success" size="small">{{ $t('history.dest') }}</el-tag>
              <span class="path-text">{{ entry.dest_path }}</span>
            </div>
          </div>
          <div class="history-card-stats">
            <div class="stat-item">
              <span class="stat-label">{{ $t('history.files') }}</span>
              <span class="stat-value">{{ entry.total_files }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">{{ $t('history.size') }}</span>
              <span class="stat-value">{{ formatBytes(entry.total_bytes) }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">{{ $t('history.status') }}</span>
              <span class="stat-value">
                <el-tag type="success" size="small">
                  {{ entry.backed_up_count }}/{{ entry.total_files }}
                </el-tag>
              </span>
            </div>
          </div>
          <div class="history-card-footer">
            <span class="time-label">{{ $t('history.time') }}:</span>
            <span class="time-value">{{ entry.last_backed_up_at ?? '--' }}</span>
            <el-button size="small" class="retry-btn" @click="showFailedFiles(entry)">
              {{ $t('history.retry_failed') }}
            </el-button>
          </div>
        </el-card>
      </div>
    </template>

    <div v-else class="empty-state">
      <el-empty :description="$t('common.no_data')" />
    </div>

    <el-dialog v-model="failedDialogVisible" :title="$t('history.retry_failed')" width="600px">
      <div v-if="currentFailedFiles.length === 0" class="no-failed">
        <el-empty :description="$t('common.no_data')" />
      </div>
      <el-table v-else :data="currentFailedFiles" size="small" max-height="400">
        <el-table-column
          prop="relative_path"
          :label="$t('history.source')"
          min-width="200"
          show-overflow-tooltip
        />
        <el-table-column prop="file_size" label="Size" width="100">
          <template #default="{ row }: { row: FailedFile }">
            {{ formatBytes(row.file_size) }}
          </template>
        </el-table-column>
        <el-table-column prop="error" label="Error" min-width="200" show-overflow-tooltip />
      </el-table>
      <template #footer>
        <el-button @click="failedDialogVisible = false">{{ $t('common.cancel') }}</el-button>
        <el-button
          type="primary"
          :loading="retrying"
          :disabled="currentFailedFiles.length === 0"
          @click="handleRetry"
        >
          {{ $t('common.confirm') }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.history-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.history-card-header {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
}

.history-source,
.history-dest {
  display: flex;
  align-items: center;
  gap: 8px;
}

.path-text {
  font-family: monospace;
  font-size: 13px;
  word-break: break-all;
}

.history-card-stats {
  display: flex;
  gap: 24px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.stat-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.stat-value {
  font-size: 14px;
  font-weight: 600;
}

.history-card-footer {
  display: flex;
  margin-top: 12px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  align-items: center;
  gap: 8px;
}

.time-label {
  margin-right: 4px;
}

.retry-btn {
  margin-left: auto;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.export-actions {
  display: flex;
  gap: 8px;
}

.no-failed {
  padding: 24px 0;
}
</style>
