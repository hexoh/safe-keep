<script setup lang="ts">
import { ElMessage } from 'element-plus'
import { getBackupTasks, getFailedFiles, retryFailedBackup } from '@/api/backup'
import type { FailedFile } from '@/api/backup'
import { exportTasksCsv, exportTasksJson } from '@/api/export'
import { useSettings } from '@/composables/useSettings'
import { formatBytes, formatDuration, formatIsoDateTime, formatSpeed } from '@/utils/format'
import { translateError } from '@/utils/errors'
import { i18n } from '@/locales'
import type { BackupTask } from '@/types/backup'

const { settings, load } = useSettings()

const tasks = ref<BackupTask[]>([])
const loading = ref(false)
const error = ref('')
const openId = ref<number | null>(null)
const failedByTask = ref<Record<number, FailedFile[]>>({})
const failedLoading = ref(false)
const retryingId = ref<number | null>(null)
const exporting = ref(false)

const statusColor: Record<string, string> = {
  completed: 'var(--new)',
  cancelled: 'var(--modified)',
  failed: 'var(--failed)',
  running: 'var(--accent)',
  paused: 'var(--modified)'
}

const statusBg: Record<string, string> = {
  completed: 'var(--new-bg)',
  cancelled: 'var(--modified-bg)',
  failed: 'var(--failed-bg)',
  running: 'var(--accent-dim)',
  paused: 'var(--modified-bg)'
}

const stats = computed(() => ({
  tasks: tasks.value.length,
  files: tasks.value.reduce((sum, t) => sum + t.backed_up_files, 0),
  bytes: tasks.value.reduce((sum, t) => sum + t.backed_up_bytes, 0),
  failed: tasks.value.reduce((sum, t) => sum + t.failed_files, 0)
}))

const hasTasks = computed(() => tasks.value.length > 0)

function parseDbDate(value?: string | null): number | null {
  if (!value) return null
  const normalized = value.includes('T') ? value : value.replace(' ', 'T')
  const date = new Date(normalized)
  return Number.isNaN(date.getTime()) ? null : date.getTime()
}

function taskDuration(task: BackupTask): string {
  const started = parseDbDate(task.started_at)
  const completed = parseDbDate(task.completed_at)
  if (started == null || completed == null) return '—'
  return formatDuration(Math.max(0, (completed - started) / 1000))
}

function taskPercent(task: BackupTask): number {
  if (!task.total_files) return 0
  return Math.min(100, Math.round((task.backed_up_files / task.total_files) * 100))
}

async function loadTasks() {
  loading.value = true
  error.value = ''
  try {
    tasks.value = await getBackupTasks()
    if (openId.value != null && !tasks.value.some((t) => t.id === openId.value)) {
      openId.value = null
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : (e?.message ?? 'Unknown error')
  } finally {
    loading.value = false
  }
}

async function loadFailedFiles(task: BackupTask, force = false) {
  if (!force && failedByTask.value[task.id]) return
  failedLoading.value = true
  try {
    failedByTask.value = {
      ...failedByTask.value,
      [task.id]: await getFailedFiles(task.source_root)
    }
  } catch (e: any) {
    ElMessage.error(typeof e === 'string' ? e : (e?.message ?? 'Unknown error'))
  } finally {
    failedLoading.value = false
  }
}

async function toggleTask(task: BackupTask) {
  if (openId.value === task.id) {
    openId.value = null
    return
  }
  openId.value = task.id
  if (task.failed_files > 0) await loadFailedFiles(task)
}

async function retryTask(task: BackupTask) {
  await load()
  retryingId.value = task.id
  try {
    let files = failedByTask.value[task.id]
    if (!files) {
      files = await getFailedFiles(task.source_root)
      failedByTask.value = { ...failedByTask.value, [task.id]: files }
    }
    if (files.length === 0) {
      ElMessage.info(i18n.global.t('history.retry_empty'))
      return
    }

    const result = await retryFailedBackup({
      sourceRoot: task.source_root,
      destPath: task.dest_path,
      files: [...files],
      conflictStrategy: settings.value.conflict_strategy,
      concurrency: settings.value.concurrent_threads
    })

    ElMessage.success(
      i18n.global.t('history.retry_done', {
        succeeded: result.succeeded,
        failed: result.failed
      })
    )
    await loadTasks()
    await loadFailedFiles(task, true)
  } catch (e: any) {
    ElMessage.error(
      i18n.global.t('history.retry_error', {
        error: typeof e === 'string' ? e : (e?.message ?? 'Unknown error')
      })
    )
  } finally {
    retryingId.value = null
  }
}

async function exportCsv() {
  if (!hasTasks.value || exporting.value) return
  exporting.value = true
  try {
    if (await exportTasksCsv(tasks.value)) {
      ElMessage.success(i18n.global.t('history.export_done'))
    }
  } catch {
    ElMessage.error(i18n.global.t('history.export_failed'))
  } finally {
    exporting.value = false
  }
}

async function exportJson() {
  if (!hasTasks.value || exporting.value) return
  exporting.value = true
  try {
    if (await exportTasksJson(tasks.value)) {
      ElMessage.success(i18n.global.t('history.export_done'))
    }
  } catch {
    ElMessage.error(i18n.global.t('history.export_failed'))
  } finally {
    exporting.value = false
  }
}

onMounted(loadTasks)
</script>

<template>
  <div class="history-view">
    <div class="page-header">
      <div>
        <h1>{{ $t('history.title') }}</h1>
        <p>{{ $t('history.subtitle') }}</p>
      </div>
      <div class="header-actions">
        <button class="btn-secondary-sm" :disabled="!hasTasks || exporting" @click="exportCsv">
          {{ $t('history.export_csv') }}
        </button>
        <button class="btn-secondary-sm" :disabled="!hasTasks || exporting" @click="exportJson">
          {{ $t('history.export_json') }}
        </button>
      </div>
    </div>

    <div v-if="hasTasks" class="stats-grid">
      <DesignStatCard :label="$t('history.total_tasks')" :value="stats.tasks" />
      <DesignStatCard :label="$t('history.backed_up_files')" :value="stats.files" />
      <DesignStatCard :label="$t('history.backed_up_size')" :value="formatBytes(stats.bytes)" />
      <DesignStatCard
        :label="$t('history.failed_files')"
        :value="stats.failed"
        :color="stats.failed > 0 ? 'var(--failed)' : undefined"
      />
    </div>

    <DesignCard v-if="error" padding="14px" class="error-card">
      <span class="text-failed mono-sm">{{ error }}</span>
      <button class="btn-secondary-sm" @click="loadTasks">{{ $t('common.retry') }}</button>
    </DesignCard>

    <div v-if="loading" class="loading-state">
      <span class="spinner" />
      <span>{{ $t('common.loading') }}</span>
    </div>

    <div v-else-if="!hasTasks && !error" class="empty-state">
      <span class="empty-icon">🕘</span>
      <span class="empty-title">{{ $t('history.empty_title') }}</span>
      <span class="empty-hint">{{ $t('history.empty_hint') }}</span>
    </div>

    <div v-else class="history-list">
      <DesignCard v-for="task in tasks" :key="task.id" style="overflow: hidden">
        <div class="history-item" @click="toggleTask(task)">
          <div
            class="status-dot"
            :style="{
              background: statusColor[task.status] || 'var(--text-sec)',
              boxShadow: `0 0 0 3px ${statusBg[task.status] || 'var(--surface)'}`
            }"
          />

          <div class="history-grid">
            <div>
              <div class="meta-label">{{ $t('history.time') }}</div>
              <div class="mono-sm">{{ formatIsoDateTime(task.started_at) }}</div>
            </div>

            <div class="route-col">
              <div class="meta-label">{{ $t('history.route') }}</div>
              <div class="route-text">
                <span class="route-src">{{ task.source_root }}</span>
                <svg width="12" height="12" viewBox="0 0 20 20" fill="currentColor" class="chevron">
                  <path
                    fillRule="evenodd"
                    d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z"
                    clipRule="evenodd"
                  />
                </svg>
                <span class="route-dst">{{ task.dest_path }}</span>
              </div>
            </div>

            <div>
              <div class="meta-label">{{ $t('history.files_size') }}</div>
              <div class="mono-sm">
                {{ task.backed_up_files }} / {{ task.total_files }} ·
                {{ formatBytes(task.backed_up_bytes || task.total_bytes) }}
              </div>
            </div>

            <div>
              <div class="meta-label">{{ $t('history.speed') }}</div>
              <div class="mono-sm">
                {{ task.avg_speed != null ? formatSpeed(task.avg_speed) : '—' }}
              </div>
            </div>

            <StatusBadge :status="task.status" />
            <span class="expand-arrow">{{ openId === task.id ? '▲' : '▼' }}</span>
          </div>
        </div>

        <div v-if="openId === task.id" class="history-detail">
          <div class="detail-grid">
            <div class="detail-item detail-green">
              <div class="detail-label">{{ $t('history.detail_succeeded') }}</div>
              <div class="detail-val">{{ task.backed_up_files }}</div>
            </div>
            <div class="detail-item detail-red">
              <div class="detail-label">{{ $t('history.detail_failed') }}</div>
              <div class="detail-val">{{ task.failed_files }}</div>
            </div>
            <div class="detail-item detail-blue">
              <div class="detail-label">{{ $t('history.detail_size') }}</div>
              <div class="detail-val">{{ formatBytes(task.backed_up_bytes) }}</div>
            </div>
            <div class="detail-item">
              <div class="detail-label">{{ $t('history.detail_duration') }}</div>
              <div class="detail-val">{{ taskDuration(task) }}</div>
            </div>
          </div>

          <div class="progress-line">
            <span class="progress-label">{{ $t('history.completion') }}</span>
            <ProgressBar :value="taskPercent(task)" :height="4" />
            <span class="mono-sm">{{ taskPercent(task) }}%</span>
          </div>

          <div class="detail-meta">
            <span>{{ $t('history.started_at') }}: {{ formatIsoDateTime(task.started_at) }}</span>
            <span
              >{{ $t('history.completed_at') }}: {{ formatIsoDateTime(task.completed_at) }}</span
            >
          </div>

          <div v-if="task.failed_files > 0" class="failed-section">
            <div class="failed-header">
              <span class="failed-title">{{ $t('history.failed_files') }}</span>
              <button
                class="btn-danger-sm"
                :disabled="retryingId === task.id"
                @click.stop="retryTask(task)"
              >
                {{ retryingId === task.id ? $t('common.processing') : $t('history.retry_failed') }}
              </button>
            </div>
            <div v-if="failedLoading && !failedByTask[task.id]" class="failed-line muted">
              {{ $t('common.loading') }}
            </div>
            <div v-for="(file, i) in failedByTask[task.id] || []" :key="i" class="failed-line">
              <span class="failed-name">{{ file.relative_path }}</span>
              <span class="failed-error">{{ file.error }}</span>
            </div>
            <div
              v-if="failedByTask[task.id] && failedByTask[task.id].length === 0"
              class="failed-line muted"
            >
              {{ $t('history.retry_empty') }}
            </div>
          </div>
          <div v-else class="success-hint">{{ $t('history.no_failed') }}</div>
        </div>
      </DesignCard>
    </div>
  </div>
</template>

<style scoped>
.history-view {
  display: flex;
  height: 100%;
  padding: 26px 30px;
  overflow-y: auto;
  flex-direction: column;
  gap: 16px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 14px;
}

.page-header h1 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: var(--text);
}

.page-header p {
  margin: 3px 0 0;
  font-size: 13px;
  color: var(--text-sec);
}

.header-actions {
  display: flex;
  gap: 7px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
}

.btn-secondary-sm {
  padding: 7px 14px;
  font-size: 12px;
  color: var(--text-sec);
  cursor: pointer;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 7px;
}

.btn-secondary-sm:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.history-item {
  display: flex;
  padding: 14px 18px;
  cursor: pointer;
  align-items: center;
  gap: 14px;
}

.status-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  flex-shrink: 0;
}

.history-grid {
  flex: 1;
  display: grid;
  grid-template-columns: 150px 1fr 170px 110px 100px 20px;
  align-items: center;
  gap: 12px;
}

.meta-label {
  margin-bottom: 2px;
  font-size: 10px;
  color: var(--text-muted);
}

.mono-sm {
  font-family: var(--mono);
  font-size: 12px;
  color: var(--text);
}

.muted {
  color: var(--text-muted);
}

.route-col {
  min-width: 0;
}

.route-text {
  display: flex;
  overflow: hidden;
  font-family: var(--mono);
  font-size: 11px;
  align-items: center;
  gap: 4px;
}

.route-src {
  overflow: hidden;
  color: var(--text);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chevron {
  color: var(--text-muted);
  flex-shrink: 0;
}

.route-dst {
  overflow: hidden;
  color: var(--text-sec);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.expand-arrow {
  font-size: 11px;
  color: var(--text-muted);
  text-align: right;
}

.history-detail {
  padding: 14px 18px 16px;
  border-top: 1px solid var(--border);
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
}

.detail-item {
  padding: 10px 14px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 7px;
}

.detail-green {
  background: var(--new-bg);
  border-color: transparent;
}

.detail-red {
  background: var(--failed-bg);
  border-color: transparent;
}

.detail-blue {
  background: var(--backed-bg);
  border-color: transparent;
}

.detail-label {
  margin-bottom: 4px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.06em;
  color: var(--text-sec);
  text-transform: uppercase;
}

.detail-val {
  font-family: var(--mono);
  font-size: 20px;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--text);
}

.progress-line {
  display: grid;
  margin-top: 12px;
  grid-template-columns: auto 1fr auto;
  align-items: center;
  gap: 10px;
}

.progress-label {
  font-size: 11px;
  color: var(--text-sec);
}

.detail-meta {
  display: flex;
  margin-top: 10px;
  font-family: var(--mono);
  font-size: 11px;
  color: var(--text-muted);
  gap: 18px;
  flex-wrap: wrap;
}

.failed-section {
  padding: 10px 14px;
  margin-top: 12px;
  background: var(--failed-bg);
  border: 1px solid rgb(239 68 68 / 20%);
  border-radius: 7px;
}

.failed-header {
  display: flex;
  margin-bottom: 8px;
  justify-content: space-between;
  align-items: center;
}

.failed-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--failed);
}

.btn-danger-sm {
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 500;
  color: var(--failed);
  cursor: pointer;
  background: var(--surface);
  border: 1px solid var(--failed);
  border-radius: 5px;
}

.btn-danger-sm:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.failed-line {
  display: flex;
  padding: 3px 0 3px 10px;
  font-family: var(--mono);
  font-size: 11px;
  line-height: 1.6;
  color: var(--text-sec);
  border-left: 2px solid var(--failed);
  gap: 10px;
  flex-wrap: wrap;
}

.failed-name {
  color: var(--text);
}

.failed-error {
  color: var(--text-muted);
}

.success-hint {
  padding: 10px 0 0;
  font-size: 12px;
  color: var(--new);
}

.error-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.text-failed {
  color: var(--failed);
}

.loading-state,
.empty-state {
  display: flex;
  flex: 1;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  gap: 8px;
  font-size: 13px;
  color: var(--text-muted);
}

.empty-icon {
  font-size: 36px;
}

.empty-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-sec);
}

.empty-hint {
  font-size: 12px;
  color: var(--text-muted);
}

.spinner {
  width: 22px;
  height: 22px;
  border: 2px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
