<script setup lang="ts">
import { useRouter } from 'vue-router'
import { convertFileSrc } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import { useScanStore } from '@/stores/scan'
import { formatBytes, formatDateTime } from '@/utils/format'
import { i18n } from '@/locales'
import type { FileEntry } from '@/types/file'

const router = useRouter()
const scanStore = useScanStore()

onMounted(() => {
  if (scanStore.status === 'idle' && scanStore.options) {
    void scanStore.beginScan(scanStore.options)
  }
})

onBeforeUnmount(() => {
  if (scanStore.status === 'scanning') void scanStore.cancel()
})

const sel = ref(new Set<string>())
const page = ref(1)
type FileStatus = 'new' | 'backed_up' | 'changed'
type FilterKey = 'all' | FileStatus
const filter = ref<FilterKey>('all')
const PER = 7

const files = computed(() => scanStore.files)
const selectionInitialized = ref(false)

function selectable(f: FileEntry): boolean {
  return f.status === 'new' || f.status === 'changed'
}

// 默认勾选所有新增 / 已变更文件，已备份文件不重复复制
watch(
  () => [scanStore.status, scanStore.files] as const,
  ([status, list]) => {
    if (status === 'done' && !selectionInitialized.value) {
      sel.value = new Set(list.filter(selectable).map((f) => f.path))
      selectionInitialized.value = true
    }
  },
  { immediate: true }
)

const filtered = computed(() => {
  if (filter.value === 'all') return files.value
  return files.value.filter((f) => f.status === filter.value)
})

const paged = computed(() => filtered.value.slice((page.value - 1) * PER, page.value * PER))

const pages = computed(() => Math.max(1, Math.ceil(filtered.value.length / PER)))

const counts = computed(() => ({
  new: files.value.filter((f) => f.status === 'new').length,
  backed_up: files.value.filter((f) => f.status === 'backed_up').length,
  changed: files.value.filter((f) => f.status === 'changed').length
}))

const totalSize = computed(() => formatBytes(scanStore.totalSize))

const progressPct = computed(() => {
  const t = scanStore.progress.total
  if (t && t > 0) return Math.round((scanStore.progress.scanned / t) * 100)
  return null
})

const pageItems = computed<(number | 'left' | 'right')[]>(() => {
  const last = pages.value
  const cur = page.value
  if (last <= 7) {
    return Array.from({ length: last }, (_, i) => i + 1)
  }
  const items: (number | 'left' | 'right')[] = [1]
  const start = Math.max(2, cur - 2)
  const end = Math.min(last - 1, cur + 2)
  if (start > 2) items.push('left')
  for (let i = start; i <= end; i++) items.push(i)
  if (end < last - 1) items.push('right')
  items.push(last)
  return items
})

watch(filtered, () => {
  if (page.value > pages.value) page.value = pages.value
})

function selectFilter(f: FilterKey) {
  filter.value = f
  page.value = 1
}

function goPage(p: number) {
  if (p >= 1 && p <= pages.value) page.value = p
}

function toggleFile(path: string) {
  const n = new Set(sel.value)
  if (n.has(path)) n.delete(path)
  else n.add(path)
  sel.value = n
}

function toggleAll() {
  const allSelected =
    filtered.value.length > 0 && filtered.value.every((f) => sel.value.has(f.path))
  sel.value = allSelected ? new Set() : new Set(filtered.value.map((f) => f.path))
}

function startBackup() {
  const selected = files.value.filter((f) => sel.value.has(f.path))
  if (selected.length === 0) {
    ElMessage.warning(i18n.global.t('preview.no_selection'))
    return
  }
  scanStore.requestBackup(selected.map((f) => f.path))
  router.push({ name: 'backup' })
}

function retry() {
  selectionInitialized.value = false
  if (scanStore.options) void scanStore.beginScan(scanStore.options)
}

const previewing = ref<FileEntry | null>(null)
const previewSrc = computed(() => (previewing.value ? convertFileSrc(previewing.value.path) : ''))

function openPreview(f: FileEntry) {
  previewing.value = f
}
</script>

<template>
  <div class="preview-view">
    <div class="page-header">
      <div>
        <h1>{{ $t('preview.title') }}</h1>
        <p>{{ $t('preview.subtitle') }}</p>
      </div>
      <div class="header-actions">
        <button class="btn-secondary-sm" @click="router.push({ name: 'home' })">
          ← {{ $t('common.back') }}
        </button>
        <button
          class="btn-primary-sm"
          :disabled="scanStore.status !== 'done' || sel.size === 0"
          @click="startBackup"
        >
          <icon-local-upload />
          {{ $t('preview.start_backup') }} · {{ sel.size }}
          {{ $t('preview.files') }}
        </button>
      </div>
    </div>

    <div class="stats-grid">
      <DesignStatCard
        :label="$t('preview.total_files')"
        :value="files.length"
        :sub="$t('preview.total_size_hint', { size: totalSize })"
      />
      <DesignStatCard
        :label="$t('preview.new_files')"
        :value="counts.new"
        :sub="$t('preview.new_hint')"
        color="var(--new)"
      />
      <DesignStatCard
        :label="$t('preview.backed_up_files')"
        :value="counts.backed_up"
        :sub="$t('preview.backed_hint')"
        color="var(--backed)"
      />
      <DesignStatCard
        :label="$t('preview.changed_files')"
        :value="counts.changed"
        :sub="$t('preview.modified_hint')"
        color="var(--modified)"
      />
    </div>

    <template v-if="scanStore.status === 'scanning'">
      <DesignCard class="status-panel">
        <div class="status-body">
          <span class="status-title">{{ $t('preview.scanning') }}</span>
          <ProgressBar :value="progressPct ?? 0" :height="8" />
          <span class="status-sub">
            {{
              scanStore.progress.total
                ? $t('preview.scanned_files', {
                    scanned: scanStore.progress.scanned,
                    total: scanStore.progress.total
                  })
                : $t('preview.scanned_only', { scanned: scanStore.progress.scanned })
            }}
          </span>
          <span v-if="scanStore.progress.current_file" class="status-current mono-sm">
            {{ $t('preview.scan_current', { file: scanStore.progress.current_file }) }}
          </span>
          <button class="btn-secondary-sm" @click="scanStore.cancel()">
            {{ $t('common.cancel') }}
          </button>
        </div>
      </DesignCard>
    </template>

    <template v-else-if="scanStore.status === 'error'">
      <DesignCard class="status-panel">
        <div class="status-body">
          <span class="status-title text-failed">{{ $t('preview.scan_error') }}</span>
          <span class="status-sub mono-sm">{{ scanStore.error }}</span>
          <button class="btn-primary-sm" @click="retry">{{ $t('common.retry') }}</button>
        </div>
      </DesignCard>
    </template>

    <template v-else-if="files.length === 0">
      <DesignCard class="status-panel">
        <div class="status-body">
          <span class="status-title">{{ $t('preview.empty_title') }}</span>
          <span class="status-sub">{{ $t('preview.empty_hint') }}</span>
          <button class="btn-secondary-sm" @click="router.push({ name: 'home' })">
            {{ $t('common.back') }}
          </button>
        </div>
      </DesignCard>
    </template>

    <template v-else>
      <div class="filter-tabs">
        <button
          v-for="f in ['all', 'new', 'backed_up', 'changed'] as const"
          :key="f"
          :class="['filter-tab', { active: filter === f }]"
          @click="selectFilter(f)"
        >
          {{ f === 'all' ? $t('common.all') : $t(`status.${f}`) }}
          <span v-if="f !== 'all'" class="filter-count">{{
            counts[f as keyof typeof counts]
          }}</span>
        </button>
      </div>

      <DesignCard style="display: flex; overflow: hidden; flex: 1; flex-direction: column">
        <div class="table-wrapper">
          <table class="design-table">
            <thead>
              <tr>
                <th class="th-check">
                  <input
                    type="checkbox"
                    :checked="filtered.length > 0 && filtered.every((f) => sel.has(f.path))"
                    @change="toggleAll"
                  />
                </th>
                <th>{{ $t('preview.file_name') }}</th>
                <th class="th-type">{{ $t('preview.type') }}</th>
                <th class="th-size">{{ $t('preview.size') }}</th>
                <th class="th-modified">{{ $t('preview.modified_at') }}</th>
                <th class="th-status">{{ $t('preview.status') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="f in paged"
                :key="f.path"
                :class="['table-row', { selected: sel.has(f.path) }]"
                @click="toggleFile(f.path)"
              >
                <td class="td-check">
                  <input
                    type="checkbox"
                    :checked="sel.has(f.path)"
                    @click.stop="toggleFile(f.path)"
                  />
                </td>
                <td>
                  <button class="file-name" @click.stop="openPreview(f)">
                    {{ f.file_name }}
                  </button>
                </td>
                <td>
                  <span :class="['file-type', f.is_image ? 'text-backed' : 'text-modified']">
                    <icon-local-image v-if="f.is_image" />
                    <icon-local-video v-else />
                    {{ $t(f.is_image ? 'preview.type_image' : 'preview.type_video') }}
                  </span>
                </td>
                <td class="mono muted">{{ formatBytes(f.file_size) }}</td>
                <td class="mono muted">{{ formatDateTime(f.modified_at) }}</td>
                <td><StatusBadge :status="f.status" /></td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="table-footer">
          <span class="footer-info">
            <span v-if="sel.size > 0" class="selected-count">
              {{ $t('preview.selected', { count: sel.size }) }}
            </span>
            {{
              $t('preview.pagination', {
                from: (page - 1) * PER + 1,
                to: Math.min(page * PER, filtered.length),
                total: filtered.length
              })
            }}
          </span>
          <div class="pagination">
            <button class="page-nav" :disabled="page <= 1" @click="goPage(page - 1)"> ‹ </button>
            <template v-for="(it, i) in pageItems" :key="i">
              <span v-if="it === 'left' || it === 'right'" class="page-ellipsis">…</span>
              <button v-else :class="['page-btn', { active: page === it }]" @click="goPage(it)">
                {{ it }}
              </button>
            </template>
            <button class="page-nav" :disabled="page >= pages" @click="goPage(page + 1)">
              ›
            </button>
          </div>
        </div>
      </DesignCard>
    </template>

    <div v-if="previewing" class="lightbox-backdrop" @click.self="previewing = null">
      <div class="lightbox">
        <div class="lightbox-top">
          <span class="mono-sm">{{ previewing.file_name }}</span>
          <button class="btn-secondary-sm" @click="previewing = null">✕</button>
        </div>
        <div class="lightbox-media">
          <video
            v-if="!previewing.is_image"
            :src="previewSrc"
            controls
            autoplay
            class="lightbox-video"
          />
          <img v-else :src="previewSrc" class="lightbox-img" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.preview-view {
  display: flex;
  height: 100%;
  padding: 26px 30px;
  flex-direction: column;
  gap: 18px;
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

.page-header p {
  margin: 3px 0 0;
  font-size: 13px;
  color: var(--text-sec);
}

.header-actions {
  display: flex;
  gap: 7px;
}

.btn-secondary-sm {
  padding: 8px 16px;
  font-size: 12px;
  color: var(--text-sec);
  cursor: pointer;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 7px;
}

.btn-primary-sm {
  display: flex;
  padding: 8px 20px;
  font-size: 13px;
  font-weight: 600;
  color: #fff;
  cursor: pointer;
  background: var(--accent);
  border: none;
  border-radius: 7px;
  box-shadow: 0 4px 14px rgb(79 127 255 / 28%);
  align-items: center;
  gap: 6px;
}

.btn-primary-sm:disabled {
  cursor: not-allowed;
  opacity: 0.5;
  box-shadow: none;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
}

.filter-tabs {
  display: flex;
  width: fit-content;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 7px;
  gap: 0;
}

.filter-tab {
  padding: 6px 13px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-sec);
  cursor: pointer;
  background: transparent;
  border: none;
  border-radius: 6px;
}

.filter-tab.active {
  color: var(--text);
  background: var(--card);
}

.filter-count {
  margin-left: 5px;
  font-family: var(--mono);
  font-size: 10px;
  color: var(--text-muted);
}

.status-panel {
  display: flex;
  padding: 26px;
  flex: 1;
}

.status-body {
  display: flex;
  width: 100%;
  max-width: 520px;
  margin: 0 auto;
  flex-direction: column;
  gap: 12px;
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

.status-current {
  overflow: hidden;
  color: var(--text-muted);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.text-failed {
  color: var(--failed);
}

.table-wrapper {
  overflow-y: auto;
  flex: 1;
}

.design-table {
  width: 100%;
  border-collapse: collapse;
}

.design-table th {
  position: sticky;
  top: 0;
  z-index: 1;
  padding: 9px 14px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.07em;
  color: var(--text-sec);
  text-align: left;
  text-transform: uppercase;
  background: var(--card);
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

.th-type {
  width: 80px;
}

.th-size {
  width: 100px;
}

.th-modified {
  width: 150px;
}

.th-status {
  width: 120px;
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

.file-name {
  padding: 0;
  font-family: var(--mono);
  font-size: 12px;
  color: var(--accent);
  cursor: pointer;
  background: none;
  border: none;
}

.file-name:hover {
  text-decoration: underline;
}

.mono-sm {
  font-family: var(--mono);
  font-size: 12px;
}

.muted {
  color: var(--text-muted);
}

.file-type {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  font-weight: 500;
}

.text-backed {
  color: var(--backed);
}

.text-modified {
  color: var(--modified);
}

.table-footer {
  display: flex;
  padding: 9px 14px;
  border-top: 1px solid var(--border);
  justify-content: space-between;
  align-items: center;
}

.footer-info {
  font-size: 11px;
  color: var(--text-muted);
}

.selected-count {
  margin-right: 10px;
  color: var(--accent);
}

.pagination {
  display: flex;
  gap: 4px;
  align-items: center;
}

.page-btn {
  width: 26px;
  height: 26px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-sec);
  cursor: pointer;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 5px;
}

.page-btn.active {
  color: #fff;
  background: var(--accent);
  border-color: var(--accent);
}

.page-ellipsis {
  min-width: 20px;
  font-size: 12px;
  color: var(--text-muted);
  text-align: center;
}

.page-nav {
  width: 26px;
  height: 26px;
  font-size: 14px;
  line-height: 1;
  color: var(--text-sec);
  cursor: pointer;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 5px;
}

.page-nav:disabled {
  cursor: not-allowed;
  opacity: 0.4;
}

.lightbox-backdrop {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  padding: 40px;
  background: rgb(0 0 0 / 72%);
  align-items: center;
  justify-content: center;
}

.lightbox {
  display: flex;
  width: min(920px, 100%);
  max-height: 100%;
  overflow: hidden;
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: 10px;
  flex-direction: column;
}

.lightbox-top {
  display: flex;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border);
  justify-content: space-between;
  align-items: center;
  gap: 10px;
}

.lightbox-media {
  display: flex;
  min-height: 0;
  flex: 1;
  align-items: center;
  justify-content: center;
  overflow: auto;
}

.lightbox-img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}

.lightbox-video {
  width: 100%;
  max-height: 100%;
}
</style>
