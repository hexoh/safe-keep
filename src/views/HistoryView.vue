<script setup lang="ts">
import { getBackupHistory } from '@/api/backup'
import { formatBytes, formatDuration } from '@/utils/format'
import type { BackupHistoryEntry } from '@/types/backup'

const entries = ref<BackupHistoryEntry[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

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

onMounted(fetchHistory)
</script>

<template>
  <div class="history-view">
    <h1>{{ $t('history.title') }}</h1>

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
          </div>
        </el-card>
      </div>
    </template>

    <div v-else class="empty-state">
      <el-empty :description="$t('common.no_data')" />
    </div>
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
  margin-top: 12px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.time-label {
  margin-right: 4px;
}
</style>
