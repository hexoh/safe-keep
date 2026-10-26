<script setup lang="ts">
import { ElMessage } from 'element-plus'
import { useRestore } from '@/composables/useRestore'
import { useSettings } from '@/composables/useSettings'
import { formatBytes, formatDuration, formatIsoDateTime } from '@/utils/format'
import { i18n } from '@/locales'
import type { RestoreFile } from '@/types/restore'

type RestoreConflict = 'overwrite' | 'skip' | 'keep_both'

const { files, loading, running, progress, result, error, fetchFiles, start, cancel } = useRestore()
const { settings, load } = useSettings()

const selectedSource = ref('')
const selected = ref(new Set<number>())
const target = ref('')
const conflict = ref<RestoreConflict>('overwrite')

const groups = computed(() => {
  const map = new Map<
    string,
    { root: string; count: number; bytes: number; latest: string | null }
  >()
  for (const file of files.value) {
    const group = map.get(file.source_root) ?? {
      root: file.source_root,
      count: 0,
      bytes: 0,
      latest: null
    }
    group.count += 1
    group.bytes += file.file_size
    if (file.backed_up_at && (!group.latest || file.backed_up_at > group.latest)) {
      group.latest = file.backed_up_at
    }
    map.set(file.source_root, group)
  }
  return [...map.values()].toSorted((a, b) => (b.latest ?? '').localeCompare(a.latest ?? ''))
})

const sourceFiles = computed(() =>
  files.value.filter((file) => file.source_root === selectedSource.value)
)

const selectedFiles = computed(() =>
  sourceFiles.value.filter((file) => selected.value.has(file.file_id))
)

const selectedSize = computed(() =>
  selectedFiles.value.reduce((sum, file) => sum + file.file_size, 0)
)

const allSelected = computed(
  () =>
    sourceFiles.value.length > 0 &&
    sourceFiles.value.every((file) => selected.value.has(file.file_id))
)

const activeGroup = computed(() =>
  groups.value.find((group) => group.root === selectedSource.value)
)

const restorePct = computed(() => {
  const p = progress.value
  if (!p || p.total_files === 0) return result.value ? 100 : 0
  return Math.min(100, Math.round((p.processed / p.total_files) * 100))
})

const conflictOptions = computed<{ value: RestoreConflict; label: string }[]>(() => [
  { value: 'overwrite', label: i18n.global.t('restore.overwrite') },
  { value: 'skip', label: i18n.global.t('restore.skip') },
  { value: 'keep_both', label: i18n.global.t('restore.keep_both') }
])

function conflictFromSettings(strategy: string): RestoreConflict {
  if (strategy === 'skip') return 'skip'
  if (strategy === 'rename') return 'keep_both'
  return 'overwrite'
}

function originalPath(file: RestoreFile): string {
  return `${file.source_root.replace(/[\\/]+$/, '')}/${file.relative_path.replace(/^[\\/]+/, '')}`
}

function syncSelection() {
  selected.value = new Set(sourceFiles.value.map((file) => file.file_id))
}

function toggle(fileId: number) {
  if (running.value) return
  const next = new Set(selected.value)
  if (next.has(fileId)) next.delete(fileId)
  else next.add(fileId)
  selected.value = next
}

function toggleAll(event: Event) {
  if (running.value) return
  selected.value = (event.target as HTMLInputElement).checked
    ? new Set(sourceFiles.value.map((file) => file.file_id))
    : new Set()
}

async function loadFiles() {
  await fetchFiles()
  const roots = [...new Set(files.value.map((file) => file.source_root))]
  const next = roots.includes(selectedSource.value) ? selectedSource.value : (roots[0] ?? '')

  if (next !== selectedSource.value) {
    selectedSource.value = next
  } else {
    syncSelection()
  }
}

async function startRestore() {
  if (!selectedSource.value) {
    ElMessage.warning(i18n.global.t('restore.source_required'))
    return
  }
  if (!target.value.trim()) {
    ElMessage.warning(i18n.global.t('restore.target_required'))
    return
  }
  if (selectedFiles.value.length === 0) {
    ElMessage.warning(i18n.global.t('restore.selection_required'))
    return
  }

  const snapshot = [...selectedFiles.value]

  await start({
    sourceRoot: selectedSource.value,
    restoreTarget: target.value.trim(),
    files: snapshot,
    conflictStrategy: conflict.value
  })

  if (error.value) {
    ElMessage.error(i18n.global.t('restore.start_failed', { error: error.value }))
    return
  }

  if (result.value) {
    if (result.value.failed > 0) {
      ElMessage.warning(
        i18n.global.t('restore.partial', {
          succeeded: result.value.succeeded,
          failed: result.value.failed
        })
      )
    } else {
      ElMessage.success(i18n.global.t('restore.done', { count: result.value.succeeded }))
    }
  }

  await loadFiles()
}

async function stopRestore() {
  await cancel()
  ElMessage.info(i18n.global.t('restore.cancelled'))
}

watch(selectedSource, (source) => {
  target.value = source
  syncSelection()
})

onMounted(async () => {
  await load()
  conflict.value = conflictFromSettings(settings.value.conflict_strategy)
  await loadFiles()
})
</script>

<template>
  <div class="restore-view">
    <div class="page-header">
      <div>
        <h1>{{ $t('restore.title') }}</h1>
        <p>{{ $t('restore.subtitle') }}</p>
      </div>
      <button class="btn-secondary-sm" :disabled="running || loading" @click="loadFiles">
        {{ $t('restore.refresh') }}
      </button>
    </div>

    <DesignCard v-if="error" padding="14px" class="error-card">
      <span class="text-failed mono-sm">{{ error }}</span>
    </DesignCard>

    <div v-if="loading && files.length === 0" class="loading-state">
      <span class="spinner" />
      <span>{{ $t('common.loading') }}</span>
    </div>

    <template v-else>
      <DesignCard v-if="running || result" padding="16px">
        <div class="progress-block">
          <div class="progress-header">
            <span class="progress-title">
              {{ running ? $t('restore.running') : $t('restore.result_title') }}
            </span>
            <span class="mono-sm">{{ restorePct }}%</span>
          </div>
          <ProgressBar
            :value="restorePct"
            :color="result && !running ? 'var(--new)' : 'var(--accent)'"
            :height="7"
          />
          <div class="progress-meta">
            <span v-if="running && progress?.current_file" class="mono-sm muted">
              {{ progress.current_file }}
            </span>
            <span v-else-if="result" class="mono-sm">
              {{
                $t('restore.result_summary', {
                  succeeded: result.succeeded,
                  failed: result.failed,
                  size: formatBytes(result.restored_bytes)
                })
              }}
            </span>
            <button v-if="running" class="btn-danger-sm" @click="stopRestore">
              {{ $t('common.cancel') }}
            </button>
          </div>
          <div v-if="result && !running" class="result-meta">
            <span
              >{{ $t('restore.result_duration') }}: {{ formatDuration(result.duration_secs) }}</span
            >
            <span>{{ $t('restore.result_total') }}: {{ result.total_files }}</span>
          </div>
          <div v-if="result?.errors?.length" class="error-list">
            <div v-for="(message, i) in result.errors" :key="i" class="error-line">
              {{ message }}
            </div>
          </div>
        </div>
      </DesignCard>

      <div v-if="files.length === 0" class="empty-state">
        <span class="empty-icon">♻️</span>
        <span class="empty-title">{{ $t('restore.no_files_title') }}</span>
        <span class="empty-hint">{{ $t('restore.no_files_hint') }}</span>
        <button class="btn-secondary-sm" @click="loadFiles">{{ $t('restore.refresh') }}</button>
      </div>

      <template v-else>
        <div class="config-grid">
          <DesignCard padding="18px">
            <DesignLabel :text="$t('restore.select_source')" />
            <div class="source-section">
              <select v-model="selectedSource" class="select-input" :disabled="running">
                <option v-for="group in groups" :key="group.root" :value="group.root">
                  {{ group.root }}
                </option>
              </select>
              <div v-if="activeGroup" class="source-meta">
                <span>{{ $t('restore.session_files', { count: activeGroup.count }) }}</span>
                <span>{{ formatBytes(activeGroup.bytes) }}</span>
                <span v-if="activeGroup.latest">
                  {{ $t('restore.latest_backup') }}: {{ formatIsoDateTime(activeGroup.latest) }}
                </span>
              </div>
            </div>
          </DesignCard>

          <DesignCard padding="18px">
            <DesignLabel :text="$t('restore.restore_target')" />
            <div class="target-section">
              <DesignPathInput
                :label="$t('restore.target_path')"
                :value="target"
                :placeholder="$t('restore.target_placeholder')"
                :hint="$t('restore.target_hint')"
                @update:value="target = $event"
              />
              <div class="conflict-section">
                <DesignLabel :text="$t('restore.conflict')" />
                <DesignRadioGroup
                  :options="conflictOptions"
                  :model-value="conflict"
                  @update:model-value="conflict = $event"
                />
              </div>
            </div>
          </DesignCard>
        </div>

        <DesignCard style="overflow: hidden">
          <div class="table-header">
            <DesignLabel :text="$t('restore.file_list')" />
            <span class="file-count">
              {{
                $t('restore.selected_of', {
                  selected: selectedFiles.length,
                  total: sourceFiles.length,
                  size: formatBytes(selectedSize)
                })
              }}
            </span>
          </div>
          <table class="design-table">
            <thead>
              <tr>
                <th class="th-check">
                  <input
                    type="checkbox"
                    :checked="allSelected"
                    :disabled="running"
                    @change="toggleAll"
                  />
                </th>
                <th>{{ $t('restore.file_name') }}</th>
                <th>{{ $t('restore.original_path') }}</th>
                <th class="th-size">{{ $t('restore.size') }}</th>
                <th class="th-date">{{ $t('restore.backup_time') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="file in sourceFiles"
                :key="file.file_id"
                :class="['table-row', { selected: selected.has(file.file_id) }]"
                @click="toggle(file.file_id)"
              >
                <td class="td-check">
                  <input
                    type="checkbox"
                    :checked="selected.has(file.file_id)"
                    :disabled="running"
                    @click.stop="toggle(file.file_id)"
                  />
                </td>
                <td
                  ><span class="mono-sm">{{ file.file_name }}</span></td
                >
                <td>
                  <span class="mono-sm muted mono-path">{{ originalPath(file) }}</span>
                </td>
                <td class="mono muted">{{ formatBytes(file.file_size) }}</td>
                <td class="mono muted">{{ formatIsoDateTime(file.backed_up_at) }}</td>
              </tr>
              <tr v-if="sourceFiles.length === 0">
                <td colspan="5" class="empty-cell">{{ $t('restore.no_files_hint') }}</td>
              </tr>
            </tbody>
          </table>
        </DesignCard>

        <div class="action-row">
          <span class="selection-info">
            {{ $t('restore.selected_count', { count: selectedFiles.length }) }} ·
            {{ formatBytes(selectedSize) }}
          </span>
          <button
            class="btn-primary-sm"
            :disabled="selectedFiles.length === 0 || running"
            @click="startRestore"
          >
            {{ $t('restore.start_restore_count', { count: selectedFiles.length }) }}
          </button>
        </div>
      </template>
    </template>
  </div>
</template>

<style scoped>
.restore-view {
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

.config-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}

.source-section,
.target-section {
  display: flex;
  margin-top: 12px;
  flex-direction: column;
  gap: 12px;
}

.select-input {
  width: 100%;
  padding: 8px 11px;
  overflow: hidden;
  font-family: var(--mono);
  font-size: 12px;
  color: var(--text);
  text-overflow: ellipsis;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
}

.source-meta {
  display: flex;
  font-family: var(--mono);
  font-size: 11px;
  color: var(--text-muted);
  gap: 12px;
  flex-wrap: wrap;
}

.conflict-section {
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.progress-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
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

.progress-meta {
  display: flex;
  min-height: 20px;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
}

.result-meta {
  display: flex;
  font-family: var(--mono);
  font-size: 11px;
  color: var(--text-muted);
  gap: 16px;
}

.btn-danger-sm {
  padding: 5px 12px;
  font-size: 11px;
  font-weight: 500;
  color: var(--failed);
  cursor: pointer;
  background: var(--failed-bg);
  border: 1px solid var(--failed);
  border-radius: 5px;
}

.table-header {
  display: flex;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border);
  justify-content: space-between;
  align-items: center;
}

.file-count {
  font-family: var(--mono);
  font-size: 11px;
  color: var(--text-muted);
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

.mono {
  font-family: var(--mono);
}

.mono-sm {
  font-family: var(--mono);
  font-size: 12px;
}

.mono-path {
  display: inline-block;
  max-width: 380px;
  overflow: hidden;
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
  vertical-align: bottom;
}

.muted {
  color: var(--text-muted);
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

.btn-primary-sm {
  padding: 8px 20px;
  font-size: 13px;
  font-weight: 600;
  color: #fff;
  cursor: pointer;
  background: var(--accent);
  border: none;
  border-radius: 7px;
}

.btn-primary-sm:disabled {
  color: var(--text-muted);
  cursor: not-allowed;
  background: var(--surface);
  opacity: 0.6;
}

.error-card {
  color: var(--failed);
}

.error-list {
  padding: 10px 12px;
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

.loading-state,
.empty-state {
  display: flex;
  flex: 1;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  gap: 9px;
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
