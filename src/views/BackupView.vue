<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useScanStore } from '@/stores/scan'
import { useBackup } from '@/composables/useBackup'
import { useSettings } from '@/composables/useSettings'
import { formatBytes, formatDuration, formatSpeed } from '@/utils/format'
import { translateError } from '@/utils/errors'
import type { StartBackupOptions } from '@/api/backup'

const router = useRouter()
const scanStore = useScanStore()
const { settings, load } = useSettings()
const { running, paused, progress, result, logs, error, start, pause, resume, cancel } = useBackup()

const started = ref(false)
const cancelling = ref(false)
const cancelledByUser = ref(false)
const logRef = ref<HTMLDivElement | null>(null)

const files = computed(() => {
  const selected = new Set(scanStore.selectedPaths)
  return scanStore.files.filter((f) => selected.has(f.path))
})

const sourceRoot = computed(() => scanStore.options?.source_path ?? '')
const destPath = computed(() => scanStore.options?.dest_path ?? '')
const totalSize = computed(() => files.value.reduce((sum, f) => sum + f.file_size, 0))

function pad2(value: number): string {
  return String(value).padStart(2, '0')
}

/** 根据首页选择的“组织方式”计算目标目录内的相对路径。 */
function destRelativePath(file: { file_name: string; relative_path: string; modified_at: number }) {
  const mode = scanStore.options?.organize_by
  if (!mode) return file.relative_path

  const date = new Date(file.modified_at * 1000)
  if (Number.isNaN(date.getTime())) return file.relative_path

  const year = date.getFullYear()
  const month = pad2(date.getMonth() + 1)
  const day = pad2(date.getDate())

  if (mode === 'Flat') return file.file_name
  if (mode === 'YYYY/MM') return `${year}/${month}/${file.file_name}`
  return `${year}/${month}/${day}/${file.file_name}`
}

const canStart = computed(
  () => !started.value && files.value.length > 0 && !!sourceRoot.value && !!destPath.value
)

type Phase = 'ready' | 'running' | 'paused' | 'done' | 'cancelled' | 'error'

const phase = computed<Phase>(() => {
  if (error.value) return 'error'
  if (result.value) return cancelledByUser.value ? 'cancelled' : 'done'
  if (paused.value) return 'paused'
  if (running.value) return 'running'
  if (started.value && cancelling.value) return 'cancelled'
  if (started.value) return 'running'
  return 'ready'
})

const titleKey = computed(
  () =>
    ({
      ready: 'backup.title',
      running: 'backup.running_title',
      paused: 'backup.paused_title',
      done: 'backup.done_title',
      cancelled: 'backup.cancelled_title',
      error: 'backup.failed_title'
    })[phase.value]
)

function progressPct(): number {
  const p = progress.value
  if (!p) return 0
  if (p.total_bytes > 0) {
    return Math.min(100, Math.round((p.copied_bytes / p.total_bytes) * 100))
  }
  if (p.total_files > 0) {
    return Math.round((p.completed_files / p.total_files) * 100)
  }
  return 0
}

const overallPct = computed(() => {
  if (result.value && !cancelledByUser.value) return 100
  return progressPct()
})

const filePct = computed(() => Math.round((progress.value?.current_file_progress ?? 0) * 100))
const speed = computed(() => progress.value?.speed_mbps ?? result.value?.avg_speed_mbps ?? 0)
const doneCount = computed(() => progress.value?.completed_files ?? result.value?.succeeded ?? 0)
const totalCount = computed(() => progress.value?.total_files ?? files.value.length)
const copiedBytes = computed(() => progress.value?.copied_bytes ?? result.value?.copied_bytes ?? 0)
const totalBytes = computed(
  () => progress.value?.total_bytes ?? result.value?.total_bytes ?? totalSize.value
)
const eta = computed(() => progress.value?.remaining_secs ?? null)
const currentFile = computed(() => progress.value?.current_file ?? '')
const duration = computed(() => result.value?.duration_secs ?? 0)
const avgSpeed = computed(() => result.value?.avg_speed_mbps ?? 0)

async function begin() {
  if (files.value.length === 0) return
  started.value = true
  cancelling.value = false
  cancelledByUser.value = false

  await load()

  const options: StartBackupOptions = {
    sourceRoot: sourceRoot.value,
    destPath: destPath.value,
    files: files.value.map((f) => ({
      source_path: f.path,
      relative_path: f.relative_path,
      dest_relative_path: destRelativePath(f),
      file_size: f.file_size,
      // 带上扫描时的 mtime：慢速设备上省掉一次完整的串行 stat 遍历
      modified_at: f.modified_at
    })),
    conflictStrategy: settings.value.conflict_strategy,
    concurrency: settings.value.concurrent_threads
  }

  await start(options)
}

async function togglePause() {
  if (paused.value) await resume()
  else await pause()
}

async function cancelBackup() {
  cancelledByUser.value = true
  cancelling.value = true
  await cancel()
}

function clearLogs() {
  logs.value = []
}

function retry() {
  error.value = null
  result.value = null
  progress.value = null
  started.value = false
  cancelling.value = false
  cancelledByUser.value = false
  void begin()
}

onMounted(() => {
  if (scanStore.consumeBackupRequest()) void begin()
})

watch(
  logs,
  () => {
    nextTick(() => {
      if (logRef.value) logRef.value.scrollTop = logRef.value.scrollHeight
    })
  },
  { deep: true }
)
</script>

<template>
  <div class="backup-view">
    <div class="page-header">
      <div>
        <h1 :class="{ 'text-new': phase === 'done', 'text-failed': phase === 'error' }">
          {{ $t(titleKey) }}
        </h1>
        <p class="route-mono">
          {{ sourceRoot || $t('common.not_selected') }} →
          {{ destPath || $t('common.not_selected') }}
        </p>
      </div>
      <div class="header-actions">
        <template v-if="phase === 'running' || phase === 'paused'">
          <button class="btn-outline" @click="togglePause">
            {{ paused ? $t('backup.resume') : $t('backup.pause') }}
          </button>
          <button class="btn-danger-outline" @click="cancelBackup">
            {{ $t('backup.cancel') }}
          </button>
        </template>
        <template v-else-if="phase === 'done' || phase === 'cancelled'">
          <button class="btn-secondary-sm" @click="router.push({ name: 'preview' })">
            {{ $t('backup.back_to_preview') }}
          </button>
          <button class="btn-primary-sm" @click="router.push({ name: 'history' })">
            {{ $t('backup.view_details') }}
          </button>
        </template>
        <template v-else-if="phase === 'error'">
          <button class="btn-secondary-sm" @click="router.push({ name: 'preview' })">
            {{ $t('common.back') }}
          </button>
          <button class="btn-primary-sm" @click="retry">{{ $t('common.retry') }}</button>
        </template>
      </div>
    </div>

    <!-- No files selected -->
    <DesignCard v-if="files.length === 0" class="status-panel">
      <DesignEmptyState :title="$t('backup.no_files_title')" :hint="$t('backup.no_files_hint')">
        <button class="btn-primary-sm" @click="router.push({ name: 'preview' })">
          ← {{ $t('backup.back_to_preview') }}
        </button>
      </DesignEmptyState>
    </DesignCard>

    <!-- Ready to start (direct navigation) -->
    <DesignCard v-else-if="!started" class="status-panel">
      <div class="status-body">
        <span class="status-title">{{ $t('backup.title') }}</span>
        <span class="status-sub">
          {{ $t('backup.ready_hint', { files: files.length, size: formatBytes(totalSize) }) }}
        </span>
        <button class="btn-primary-sm" :disabled="!canStart" @click="begin">
          {{ $t('preview.start_backup') }}
        </button>
      </div>
    </DesignCard>

    <template v-else>
      <!-- Error -->
      <DesignCard v-if="phase === 'error'" class="status-panel">
        <div class="status-body">
          <span class="status-title text-failed">{{ $t('backup.failed_title') }}</span>
          <span class="status-sub mono-sm">{{ error ? translateError(error) : '' }}</span>
          <div class="status-actions">
            <button class="btn-secondary-sm" @click="router.push({ name: 'preview' })">
              {{ $t('common.back') }}
            </button>
            <button class="btn-primary-sm" @click="retry">{{ $t('common.retry') }}</button>
          </div>
        </div>
      </DesignCard>

      <template v-else>
        <!-- Global progress -->
        <DesignCard padding="20px">
          <div class="progress-section">
            <div class="progress-header">
              <span class="progress-title">{{ $t('backup.overall_progress') }}</span>
              <span :class="['progress-pct', { 'text-new': phase === 'done' }]">
                {{ overallPct }}%
              </span>
            </div>
            <ProgressBar
              :value="overallPct"
              :color="phase === 'done' ? 'var(--new)' : 'var(--accent)'"
              :height="10"
            />
            <div class="speed-grid">
              <div class="speed-item">
                <DesignLabel :text="$t('backup.transfer_speed')" />
                <span class="speed-val" style="color: var(--accent)">
                  {{ formatSpeed(speed) }}
                </span>
              </div>
              <div class="speed-item">
                <DesignLabel :text="$t('backup.transferred')" />
                <span class="speed-val">
                  {{ formatBytes(copiedBytes) }} / {{ formatBytes(totalBytes) }}
                </span>
              </div>
              <div class="speed-item">
                <DesignLabel :text="$t('backup.files_done')" />
                <span class="speed-val">{{ doneCount }} / {{ totalCount }}</span>
              </div>
              <div class="speed-item">
                <DesignLabel :text="$t('backup.eta')" />
                <span
                  class="speed-val"
                  :style="{ color: phase === 'done' ? 'var(--new)' : 'var(--modified)' }"
                >
                  {{
                    phase === 'done'
                      ? $t('backup.complete_short')
                      : eta != null
                        ? formatDuration(eta)
                        : $t('backup.eta_unknown')
                  }}
                </span>
              </div>
            </div>
          </div>
        </DesignCard>

        <!-- Current file -->
        <DesignCard v-if="phase === 'running' || phase === 'paused'" padding="14px">
          <div class="current-file-header">
            <div class="file-info">
              <span class="file-name">{{ currentFile || $t('backup.waiting') }}</span>
              <span v-if="filePct > 0" class="file-pct">{{ filePct }}%</span>
            </div>
            <span v-if="paused" class="paused-badge">{{ $t('backup.paused_title') }}</span>
          </div>
          <ProgressBar :value="filePct" :height="3" />
        </DesignCard>

        <!-- Completion -->
        <DesignCard v-if="phase === 'done' || phase === 'cancelled'" padding="18px">
          <div class="summary-header">
            <DesignLabel
              :text="phase === 'done' ? $t('backup.completed') : $t('backup.cancelled_title')"
            />
            <span class="summary-state" :class="phase === 'done' ? 'text-new' : 'text-modified'">
              {{ phase === 'done' ? $t('status.success') : $t('backup.cancelled_hint') }}
            </span>
          </div>
          <div class="summary-grid">
            <div class="summary-item">
              <span class="summary-label">{{ $t('backup.success_count') }}</span>
              <span class="summary-value text-new">{{ result?.succeeded ?? 0 }}</span>
            </div>
            <div class="summary-item">
              <span class="summary-label">{{ $t('backup.failed_count') }}</span>
              <span class="summary-value text-failed">{{ result?.failed ?? 0 }}</span>
            </div>
            <div class="summary-item">
              <span class="summary-label">{{ $t('backup.duration') }}</span>
              <span class="summary-value">{{ formatDuration(duration) }}</span>
            </div>
            <div class="summary-item">
              <span class="summary-label">{{ $t('backup.avg_speed') }}</span>
              <span class="summary-value">{{ formatSpeed(avgSpeed) }}</span>
            </div>
          </div>
          <div v-if="result?.errors?.length" class="error-list">
            <div class="error-title">{{ $t('backup.error_title') }}</div>
            <div v-for="(message, i) in result.errors.slice(0, 5)" :key="i" class="error-line">
              {{ message }}
            </div>
          </div>
        </DesignCard>

        <!-- Log -->
        <DesignCard
          style="display: flex; min-height: 0; overflow: hidden; flex: 1; flex-direction: column"
        >
          <div class="log-header">
            <DesignLabel :text="$t('backup.live_log')" />
            <div class="log-header-right">
              <span v-if="running && !paused" class="live-indicator">
                <span class="live-dot" />
                {{ $t('backup.live') }}
              </span>
              <button class="clear-btn" @click="clearLogs">{{ $t('backup.clear_log') }}</button>
            </div>
          </div>
          <div ref="logRef" class="log-content">
            <div v-if="logs.length === 0" class="log-line muted">
              {{ $t('backup.log_empty') }}
            </div>
            <div
              v-for="(line, i) in logs"
              :key="i"
              :class="[
                'log-line',
                {
                  'log-success': line.includes('✓') || line.includes('complete'),
                  'log-transfer': line.includes('→') || line.includes('Copying')
                }
              ]"
            >
              {{ line }}
            </div>
          </div>
        </DesignCard>
      </template>
    </template>
  </div>
</template>

<style scoped>
.backup-view {
  display: flex;
  height: 100%;
  padding: 26px 30px;
  flex-direction: column;
  gap: 16px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.page-header h1 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: var(--text);
}

.text-new {
  color: var(--new);
}

.text-failed {
  color: var(--failed);
}

.text-modified {
  color: var(--modified);
}

.route-mono {
  margin: 3px 0 0;
  overflow: hidden;
  font-family: var(--mono);
  font-size: 12px;
  color: var(--text-sec);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.header-actions {
  display: flex;
  gap: 7px;
}

.btn-outline,
.btn-secondary-sm {
  display: flex;
  padding: 7px 14px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text);
  cursor: pointer;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 7px;
  align-items: center;
  gap: 5px;
}

.btn-secondary-sm {
  color: var(--text-sec);
}

.btn-danger-outline {
  display: flex;
  padding: 7px 14px;
  font-size: 12px;
  font-weight: 500;
  color: var(--failed);
  cursor: pointer;
  background: var(--failed-bg);
  border: 1px solid var(--failed);
  border-radius: 7px;
  align-items: center;
  gap: 5px;
}

.btn-primary-sm {
  padding: 7px 16px;
  font-size: 13px;
  font-weight: 600;
  color: #fff;
  cursor: pointer;
  background: var(--accent);
  border: none;
  border-radius: 7px;
}

.btn-primary-sm:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.progress-section {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
}

.progress-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
}

.progress-pct {
  font-family: var(--mono);
  font-size: 26px;
  font-weight: 700;
  letter-spacing: -0.03em;
  color: var(--accent);
}

.speed-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
}

.speed-item {
  display: flex;
  padding: 11px 14px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  flex-direction: column;
  gap: 3px;
}

.speed-val {
  font-family: var(--mono);
  font-size: 15px;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--text);
}

.current-file-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.file-info {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 8px;
}

.file-name {
  overflow: hidden;
  font-family: var(--mono);
  font-size: 13px;
  font-weight: 500;
  color: var(--text);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-pct {
  font-family: var(--mono);
  font-size: 12px;
  color: var(--text-sec);
}

.paused-badge {
  font-size: 11px;
  color: var(--modified);
}

.summary-header {
  display: flex;
  margin-bottom: 12px;
  justify-content: space-between;
  align-items: center;
}

.summary-state {
  font-size: 12px;
  font-weight: 500;
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
}

.summary-item {
  display: flex;
  padding: 10px 14px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  flex-direction: column;
  gap: 3px;
}

.summary-label {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.06em;
  color: var(--text-sec);
  text-transform: uppercase;
}

.summary-value {
  font-family: var(--mono);
  font-size: 20px;
  font-weight: 700;
  color: var(--text);
}

.error-list {
  padding: 10px 12px;
  margin-top: 12px;
  background: var(--failed-bg);
  border: 1px solid rgb(239 68 68 / 25%);
  border-radius: 7px;
}

.error-title {
  margin-bottom: 4px;
  font-size: 11px;
  font-weight: 600;
  color: var(--failed);
}

.error-line {
  font-family: var(--mono);
  font-size: 11px;
  line-height: 1.6;
  color: var(--text-sec);
}

.status-panel {
  display: flex;
  padding: 26px;
  flex: 1;
}

.status-body {
  display: flex;
  width: 100%;
  max-width: 580px;
  margin: 0 auto;
  flex-direction: column;
  gap: 12px;
  justify-content: center;
}

/* 面板内的操作按钮按内容宽度收缩，避免拉成整行长条 */
.status-body > button {
  width: fit-content;
  min-width: 96px;
  align-self: flex-start;
  justify-content: center;
}

.status-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--text);
}

.status-sub {
  font-size: 12px;
  color: var(--text-sec);
}

.status-actions {
  display: flex;
  gap: 8px;
}

.log-header {
  display: flex;
  padding: 9px 14px;
  border-bottom: 1px solid var(--border);
  justify-content: space-between;
  align-items: center;
}

.log-header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.live-indicator {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 10px;
  color: var(--accent);
}

.live-dot {
  width: 6px;
  height: 6px;
  background: var(--accent);
  border-radius: 50%;
  animation: pulse-dot 1.2s ease-in-out infinite;
}

.clear-btn {
  font-size: 11px;
  color: var(--text-muted);
  cursor: pointer;
  background: none;
  border: none;
}

.log-content {
  display: flex;
  padding: 10px 14px;
  overflow-y: auto;
  flex: 1;
  flex-direction: column;
  gap: 2px;
}

.log-line {
  font-family: var(--mono);
  font-size: 11px;
  line-height: 1.6;
  color: var(--text-sec);
}

.log-success {
  color: var(--new);
}

.log-transfer {
  color: var(--text);
}

.muted {
  color: var(--text-muted);
}

.mono-sm {
  font-family: var(--mono);
  font-size: 12px;
}
</style>
