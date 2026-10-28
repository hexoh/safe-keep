<script setup lang="ts">
import { ElMessage } from 'element-plus'
import { useCleanup } from '@/composables/useCleanup'
import { useSettings } from '@/composables/useSettings'
import { getSourceRoots } from '@/api/cleanup'
import { detectDevice } from '@/api/device'
import type { DeviceType } from '@/api/device'
import { formatBytes, formatIsoDateTime } from '@/utils/format'
import { i18n } from '@/locales'
import type { CleanupFile, CleanupFilter } from '@/types/cleanup'

const { loading, running, dryRunResult, cleanupResult, progress, error, dryRun, execute, cancel } =
  useCleanup()
const { settings, load } = useSettings()

const sources = ref<string[]>([])
const sourceRoot = ref('')
const timeMode = ref<'days' | 'before'>('days')
const days = ref('90')
const beforeDate = ref('')
const typeFilter = ref<'all' | 'image' | 'video'>('all')
const minMb = ref('')
const maxMb = ref('')
const selected = ref(new Set<number>())
const confirmStep = ref(false)
const sourcesLoading = ref(false)

const files = computed(() => dryRunResult.value?.files ?? [])
const selectedFiles = computed(() => files.value.filter((f) => selected.value.has(f.file_id)))
const selectedSize = computed(() => selectedFiles.value.reduce((sum, f) => sum + f.file_size, 0))
const allSelected = computed(
  () => files.value.length > 0 && files.value.every((f) => selected.value.has(f.file_id))
)

// 结果列表分页（feature 3.1.7 / 3.2.6）：勾选状态跨页保留，全选作用于全部候选文件。
const PER = 10
const page = ref(1)
const pages = computed(() => Math.max(1, Math.ceil(files.value.length / PER)))
const pagedFiles = computed(() => files.value.slice((page.value - 1) * PER, page.value * PER))

function goPage(target: number) {
  if (target >= 1 && target <= pages.value) page.value = target
}

watch(files, () => {
  if (page.value > pages.value) page.value = 1
})

const cleanupPct = computed(() => {
  const p = progress.value
  if (!p || p.total_files === 0) return cleanupResult.value ? 100 : 0
  return Math.min(100, Math.round((p.processed / p.total_files) * 100))
})

const deviceType = ref<DeviceType | null>(null)

const deleteStrategy = computed(() => settings.value.delete_strategy)

// MTP 设备不支持回收站，必须使用永久删除。
const effectivePermanent = computed(
  () => deleteStrategy.value === 'permanent' || deviceType.value === 'MTP'
)

async function updateDeviceType() {
  if (!sourceRoot.value) {
    deviceType.value = null
    return
  }
  try {
    deviceType.value = await detectDevice(sourceRoot.value)
  } catch {
    deviceType.value = null
  }
}

function mbToBytes(value: string): number | undefined {
  const n = Number(value)
  return Number.isFinite(n) && n > 0 ? Math.round(n * 1024 * 1024) : undefined
}

function filePath(file: CleanupFile): string {
  return `${file.source_root.replace(/[\\/]+$/, '')}/${file.relative_path}`
}

function toggle(fileId: number) {
  if (running.value) return
  const next = new Set(selected.value)
  if (next.has(fileId)) next.delete(fileId)
  else next.add(fileId)
  selected.value = next
  confirmStep.value = false
}

function toggleAll() {
  if (running.value) return
  selected.value = allSelected.value ? new Set() : new Set(files.value.map((f) => f.file_id))
  confirmStep.value = false
}

async function loadSources() {
  sourcesLoading.value = true
  try {
    sources.value = await getSourceRoots()
    if (!sources.value.length) {
      ElMessage.warning(i18n.global.t('cleanup.no_sources'))
      return
    }
    if (!sourceRoot.value || !sources.value.includes(sourceRoot.value)) {
      sourceRoot.value = sources.value[0] ?? ''
    }
  } catch {
    ElMessage.error(i18n.global.t('cleanup.load_sources_failed'))
  } finally {
    sourcesLoading.value = false
  }
}

async function runSimulation() {
  if (!sourceRoot.value) {
    ElMessage.warning(i18n.global.t('cleanup.source_required'))
    return
  }

  const filter: CleanupFilter = {
    source_root: sourceRoot.value,
    is_image: typeFilter.value === 'all' ? undefined : typeFilter.value === 'image',
    min_size: mbToBytes(minMb.value),
    max_size: mbToBytes(maxMb.value),
    before_days: timeMode.value === 'days' ? Number(days.value) || undefined : undefined,
    before_date: timeMode.value === 'before' && beforeDate.value ? beforeDate.value : undefined
  }

  confirmStep.value = false
  page.value = 1
  await dryRun(filter)

  if (dryRunResult.value) {
    selected.value = new Set(dryRunResult.value.files.map((f) => f.file_id))
  } else if (error.value) {
    ElMessage.error(i18n.global.t('cleanup.dry_run_failed'))
  }
}

function requestExecute() {
  if (!selectedFiles.value.length) return
  confirmStep.value = true
}

function cancelConfirm() {
  confirmStep.value = false
}

async function confirmExecute() {
  if (!selectedFiles.value.length) return
  confirmStep.value = false
  await execute([...selectedFiles.value], effectivePermanent.value)
  if (error.value) {
    ElMessage.error(i18n.global.t('cleanup.execute_failed'))
    return
  }
  selected.value = new Set()
  if (!cleanupResult.value?.failed) void runSimulation()
}

async function stopCleanup() {
  await cancel()
  ElMessage.info(i18n.global.t('cleanup.cancelled'))
}

onMounted(async () => {
  await load()
  await loadSources()
  if (settings.value.dry_run_default && sourceRoot.value) {
    await runSimulation()
  }
})

watch(sourceRoot, () => {
  dryRunResult.value = null
  cleanupResult.value = null
  selected.value = new Set()
  confirmStep.value = false
  void updateDeviceType()
})
</script>

<template>
  <div class="cleanup-view">
    <div class="page-header">
      <div class="header-left">
        <h1>{{ $t('cleanup.title') }}</h1>
        <p>{{ $t('cleanup.subtitle') }}</p>
      </div>
      <div class="warning-badge">{{ $t('cleanup.warning') }}</div>
    </div>

    <div v-if="deviceType === 'MTP'" class="mtp-warning">
      {{ $t('cleanup.mtp_warning') }}
    </div>

    <DesignCard padding="18px">
      <div class="filter-grid">
        <div class="filter-field">
          <DesignLabel :text="$t('cleanup.source_scope')" />
          <select v-model="sourceRoot" class="filter-input" :disabled="sourcesLoading">
            <option v-if="sources.length === 0" value="">{{ $t('cleanup.no_sources') }}</option>
            <option v-for="source in sources" :key="source" :value="source">{{ source }}</option>
          </select>
        </div>

        <div class="filter-field">
          <DesignLabel :text="$t('cleanup.time_range')" />
          <div class="mode-row">
            <DesignToggle
              :label="$t('cleanup.mode_days')"
              :on="timeMode === 'days'"
              @change="timeMode = 'days'"
            />
            <DesignToggle
              :label="$t('cleanup.mode_before')"
              :on="timeMode === 'before'"
              @change="timeMode = 'before'"
            />
          </div>
          <input
            v-if="timeMode === 'days'"
            v-model="days"
            type="number"
            min="0"
            class="filter-input"
            :placeholder="$t('cleanup.older_than_days')"
          />
          <input
            v-else
            v-model="beforeDate"
            type="date"
            class="filter-input"
            :placeholder="$t('cleanup.before_date')"
          />
        </div>

        <div class="filter-field">
          <DesignLabel :text="$t('cleanup.file_type')" />
          <select v-model="typeFilter" class="filter-input">
            <option value="all">{{ $t('common.all') }}</option>
            <option value="image">{{ $t('common.images') }}</option>
            <option value="video">{{ $t('common.videos') }}</option>
          </select>
          <div class="size-grid">
            <input
              v-model="minMb"
              type="number"
              min="0"
              class="filter-input"
              :placeholder="$t('cleanup.min_size_mb')"
            />
            <input
              v-model="maxMb"
              type="number"
              min="0"
              class="filter-input"
              :placeholder="$t('cleanup.max_size_mb')"
            />
          </div>
        </div>

        <div class="filter-action">
          <button
            class="sim-btn"
            :disabled="loading || sources.length === 0"
            @click="runSimulation"
          >
            {{ loading ? $t('common.loading') : $t('cleanup.run_simulation') }}
          </button>
        </div>
      </div>
    </DesignCard>

    <DesignCard v-if="error" padding="14px" class="error-card">
      <span class="text-failed">{{ error }}</span>
    </DesignCard>

    <template v-if="dryRunResult">
      <div class="results-header">
        <div class="results-title">
          <DesignLabel :text="$t('cleanup.simulation_results')" />
          <span class="dry-run-badge">{{ $t('cleanup.dry_run_badge') }}</span>
        </div>
        <span class="results-summary">
          {{
            $t('cleanup.files_to_free', {
              count: files.length,
              size: formatBytes(dryRunResult.total_size)
            })
          }}
        </span>
      </div>

      <DesignCard style="overflow: hidden">
        <table class="design-table">
          <thead>
            <tr>
              <th class="th-check">
                <input type="checkbox" :checked="allSelected" @change="toggleAll" />
              </th>
              <th>{{ $t('cleanup.file') }}</th>
              <th>{{ $t('cleanup.location') }}</th>
              <th class="th-size">{{ $t('cleanup.size') }}</th>
              <th class="th-date">{{ $t('cleanup.backed_up_at') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="file in pagedFiles"
              :key="file.file_id"
              :class="['table-row', { selected: selected.has(file.file_id) }]"
              @click="toggle(file.file_id)"
            >
              <td class="td-check">
                <input
                  type="checkbox"
                  :checked="selected.has(file.file_id)"
                  @click.stop="toggle(file.file_id)"
                />
              </td>
              <td
                ><span class="mono-sm">{{ file.file_name }}</span></td
              >
              <td
                ><span class="mono-sm muted mono-path">{{ filePath(file) }}</span></td
              >
              <td class="mono muted">{{ formatBytes(file.file_size) }}</td>
              <td class="mono muted">{{ formatIsoDateTime(file.backed_up_at) }}</td>
            </tr>
            <tr v-if="files.length === 0">
              <td colspan="5" class="empty-cell">{{ $t('cleanup.no_candidates') }}</td>
            </tr>
          </tbody>
        </table>

        <div v-if="files.length > 0" class="table-footer">
          <span class="footer-info">
            {{
              $t('cleanup.pagination', {
                from: (page - 1) * PER + 1,
                to: Math.min(page * PER, files.length),
                total: files.length
              })
            }}
          </span>
          <div class="pager">
            <button class="page-nav" :disabled="page <= 1" @click="goPage(page - 1)">
              ‹ {{ $t('common.prev') }}
            </button>
            <span class="page-current">{{ page }} / {{ pages }}</span>
            <button class="page-nav" :disabled="page >= pages" @click="goPage(page + 1)">
              {{ $t('common.next') }} ›
            </button>
          </div>
        </div>
      </DesignCard>

      <div class="action-row">
        <span class="selection-info">
          {{ $t('cleanup.selected_count', { count: selectedFiles.length }) }} ·
          {{ formatBytes(selectedSize) }}
        </span>
        <template v-if="confirmStep">
          <span class="confirm-warning">
            {{ $t('cleanup.confirm_warning', { count: selectedFiles.length }) }}
          </span>
          <button class="btn-secondary-sm" @click="cancelConfirm">
            {{ $t('common.cancel') }}
          </button>
          <button class="btn-danger" :disabled="running" @click="confirmExecute">
            {{ $t('cleanup.delete_now') }}
          </button>
        </template>
        <button
          v-else
          class="btn-danger"
          :disabled="selectedFiles.length === 0 || running"
          @click="requestExecute"
        >
          {{ $t('cleanup.execute_n', { count: selectedFiles.length }) }}
        </button>
      </div>

      <DesignCard v-if="running" padding="14px">
        <div class="progress-block">
          <div class="progress-header">
            <span>{{ $t('cleanup.progress') }}</span>
            <div class="progress-actions">
              <span class="mono">{{ cleanupPct }}%</span>
              <button class="btn-secondary-sm" @click="stopCleanup">
                {{ $t('common.cancel') }}
              </button>
            </div>
          </div>
          <ProgressBar :value="cleanupPct" :height="6" />
          <span v-if="progress?.current_file" class="mono-sm muted">
            {{ progress.current_file }}
          </span>
        </div>
      </DesignCard>
    </template>

    <DesignCard v-if="cleanupResult" padding="18px">
      <div class="result-header">
        <DesignLabel :text="$t('cleanup.result_title')" />
        <span class="mono-sm muted">
          {{ effectivePermanent ? $t('cleanup.permanent_hint') : $t('cleanup.recycle_hint') }}
        </span>
      </div>
      <div class="result-grid">
        <div class="result-item">
          <span class="result-label">{{ $t('cleanup.result_success') }}</span>
          <span class="result-value text-new">{{ cleanupResult.succeeded }}</span>
        </div>
        <div class="result-item">
          <span class="result-label">{{ $t('cleanup.result_failed') }}</span>
          <span class="result-value text-failed">{{ cleanupResult.failed }}</span>
        </div>
        <div class="result-item">
          <span class="result-label">{{ $t('cleanup.result_released') }}</span>
          <span class="result-value">{{ formatBytes(cleanupResult.freed_size) }}</span>
        </div>
      </div>
      <div v-if="cleanupResult.errors.length" class="error-list">
        <div v-for="(message, i) in cleanupResult.errors" :key="i" class="error-line">
          {{ message }}
        </div>
      </div>
      <button class="btn-secondary-sm result-refresh" @click="runSimulation">
        {{ $t('cleanup.refresh') }}
      </button>
    </DesignCard>

    <div v-if="!dryRunResult && !cleanupResult && !loading" class="empty-state">
      <span class="empty-icon">🧹</span>
      <span class="empty-text">{{ $t('cleanup.empty_hint') }}</span>
    </div>
  </div>
</template>

<style scoped>
.cleanup-view {
  display: flex;
  height: 100%;
  padding: 26px 30px;
  overflow-y: auto;
  flex-direction: column;
  gap: 18px;
}

.page-header {
  display: flex;
  align-items: flex-start;
  gap: 14px;
}

.header-left {
  flex: 1;
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

.warning-badge {
  padding: 7px 13px;
  font-size: 11px;
  font-weight: 500;
  color: var(--modified);
  background: var(--modified-bg);
  border: 1px solid rgb(245 158 11 / 35%);
  border-radius: 7px;
  flex-shrink: 0;
}

.mtp-warning {
  padding: 10px 14px;
  font-size: 12px;
  line-height: 1.6;
  color: var(--modified);
  background: var(--modified-bg);
  border: 1px solid rgb(245 158 11 / 30%);
  border-radius: 7px;
}

.filter-grid {
  display: grid;
  grid-template-columns: 1.2fr 1fr 1fr auto;
  gap: 14px;
  align-items: end;
}

.filter-field {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 6px;
}

.filter-input {
  width: 100%;
  padding: 8px 11px;
  font-family: var(--mono);
  font-size: 12px;
  color: var(--text);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
}

.mode-row {
  display: flex;
  gap: 5px;
  flex-wrap: wrap;
}

.size-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}

.filter-action {
  display: flex;
  align-items: flex-end;
}

.sim-btn {
  padding: 9px 16px;
  font-size: 13px;
  font-weight: 500;
  color: var(--accent);
  cursor: pointer;
  background: var(--accent-dim);
  border: 1px solid rgb(79 127 255 / 40%);
  border-radius: 6px;
}

.sim-btn:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.error-card {
  color: var(--failed);
}

.results-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.results-title {
  display: flex;
  align-items: center;
  gap: 9px;
}

.dry-run-badge {
  padding: 2px 7px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.05em;
  color: var(--accent);
  background: var(--accent-dim);
  border-radius: 4px;
}

.results-summary {
  font-size: 12px;
  color: var(--text-sec);
}

.design-table {
  width: 100%;
  border-collapse: collapse;
}

.design-table th {
  padding: 9px 14px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.07em;
  color: var(--text-sec);
  text-align: left;
  text-transform: uppercase;
  border-bottom: 1px solid var(--border);
}

.design-table td {
  padding: 9px 14px;
  font-size: 12px;
  color: var(--text);
  border-bottom: 1px solid var(--border-subtle);
}

.th-check,
.td-check {
  width: 42px;
}

.th-size {
  width: 100px;
}

.th-date {
  width: 170px;
}

.table-row {
  cursor: pointer;
}

.table-row.selected {
  background: var(--accent-dim);
}

.table-row:hover {
  background: var(--card-hover);
}

.empty-cell {
  padding: 24px !important;
  color: var(--text-muted) !important;
  text-align: center;
}

.table-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border-top: 1px solid var(--border);
}

.footer-info {
  font-family: var(--mono);
  font-size: 11px;
  color: var(--text-sec);
}

.pager {
  display: flex;
  align-items: center;
  gap: 8px;
}

.page-nav {
  padding: 5px 10px;
  font-size: 11px;
  color: var(--text-sec);
  cursor: pointer;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
}

.page-nav:disabled {
  cursor: not-allowed;
  opacity: 0.4;
}

.page-current {
  font-family: var(--mono);
  font-size: 11px;
  color: var(--text-sec);
}

.mono {
  font-family: var(--mono);
}

.mono-sm {
  font-family: var(--mono);
  font-size: 12px;
}

.mono-path {
  display: inline-block;
  max-width: 360px;
  overflow: hidden;
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
  vertical-align: bottom;
}

.muted {
  color: var(--text-muted);
}

.text-new {
  color: var(--new);
}

.text-failed {
  color: var(--failed);
}

.action-row {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 10px;
}

.selection-info {
  margin-right: auto;
  font-size: 12px;
  color: var(--text-sec);
}

.confirm-warning {
  font-size: 12px;
  font-weight: 500;
  color: var(--modified);
}

.btn-secondary-sm {
  padding: 7px 14px;
  font-size: 12px;
  color: var(--text-sec);
  cursor: pointer;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
}

.btn-danger {
  display: flex;
  padding: 8px 18px;
  font-size: 13px;
  font-weight: 600;
  color: var(--failed);
  cursor: pointer;
  background: var(--failed-bg);
  border: 1px solid var(--failed);
  border-radius: 7px;
  align-items: center;
  gap: 6px;
}

.btn-danger:disabled {
  color: var(--text-muted);
  cursor: not-allowed;
  background: var(--surface);
  border-color: var(--border);
}

.progress-block {
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.progress-header {
  display: flex;
  font-size: 12px;
  color: var(--text-sec);
  justify-content: space-between;
  align-items: center;
}

.progress-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.result-header {
  display: flex;
  margin-bottom: 12px;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.result-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}

.result-item {
  display: flex;
  padding: 10px 14px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  flex-direction: column;
  gap: 3px;
}

.result-label {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.06em;
  color: var(--text-sec);
  text-transform: uppercase;
}

.result-value {
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

.error-line {
  font-family: var(--mono);
  font-size: 11px;
  line-height: 1.6;
  color: var(--text-sec);
}

.result-refresh {
  margin-top: 12px;
}

.empty-state {
  display: flex;
  flex: 1;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  gap: 8px;
  opacity: 0.5;
}

.empty-icon {
  font-size: 36px;
}

.empty-text {
  font-size: 13px;
  color: var(--text-muted);
}
</style>
