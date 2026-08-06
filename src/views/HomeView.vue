<script setup lang="ts">
import { useRouter } from 'vue-router'
import { IMAGE_FORMATS, VIDEO_FORMATS } from '@/constants/media'
import { useDevice } from '@/composables/useDevice'
import { formatBytes } from '@/utils/format'
import { countFiles } from '@/api/scan'

const router = useRouter()
const { devices: volumes, refresh, watchDevices } = useDevice()

onMounted(async () => {
  try {
    await refresh()
  } catch {
    // ignore device read errors, keep source path as-is
  }
  watchDevices()
})

const srcPath = ref('')
const srcStats = ref<{ count: number; bytes: number } | null>(null)
let srcStatsTimer: ReturnType<typeof setTimeout> | undefined

async function updateSrcStats() {
  const p = srcPath.value.trim()
  if (!p) {
    srcStats.value = null
    return
  }
  try {
    const res = await countFiles(p)
    srcStats.value = res
  } catch {
    srcStats.value = null
  }
}

watch(srcPath, () => {
  clearTimeout(srcStatsTimer)
  srcStatsTimer = setTimeout(updateSrcStats, 400)
})
const dstPath = ref('/Users/zhao/Pictures/SafeKeep')
const srcDev = ref(0)
const org = ref(1)
const minSz = ref('0')
const maxSz = ref('')
const fmts = reactive({
  jpg: true,
  heic: true,
  heif: true,
  png: true,
  raw: false,
  dng: false,
  webp: true,
  gif: true,
  tiff: false,
  avif: true,
  mp4: true,
  mov: true,
  mkv: false,
  m4v: true,
  webm: false,
  '3gp': false
})

function toggleFmt(k: keyof typeof fmts) {
  fmts[k] = !fmts[k]
}

const deviceIcons: Record<string, string> = {
  MTP: 'phone',
  RemovableDisk: 'sd',
  LocalDisk: 'hdd',
  Unknown: 'hdd'
}

const devices = computed(() =>
  volumes.value.map((v, i) => {
    const used = v.total_space > 0 ? Math.max(0, v.total_space - v.free_space) : 0
    const pct = v.total_space > 0 ? Math.round((used / v.total_space) * 100) : 0
    return {
      icon: deviceIcons[v.device_type] ?? 'hdd',
      label: v.name,
      usedBytes: used,
      pct,
      connected: true,
      index: i
    }
  })
)

const selectedDevice = computed(() => volumes.value[srcDev.value])

const selectedPct = computed(() => {
  const v = selectedDevice.value
  if (!v || v.total_space <= 0) return 0
  return Math.round(((v.total_space - v.free_space) / v.total_space) * 100)
})

// 按已用比例返回进度条颜色: 充足=绿, 偏高=黄, 紧张=红
function storageColor(pct: number): string {
  if (pct >= 90) return 'var(--failed)'
  if (pct >= 75) return 'var(--modified)'
  return 'var(--new)'
}

// 目标路径所在卷的空间
const dstVolume = computed(() => volumes.value.find((v) => dstPath.value.startsWith(v.path)))
const dstFree = computed(() => (dstVolume.value ? dstVolume.value.free_space : 0))
const dstTotal = computed(() => (dstVolume.value ? dstVolume.value.total_space : 0))
const dstPct = computed(() => {
  if (dstTotal.value <= 0) return 0
  return Math.round(((dstTotal.value - dstFree.value) / dstTotal.value) * 100)
})

function selectDev(i: number) {
  srcDev.value = i
  srcPath.value = volumes.value[i]?.path ?? ''
}

// 设备列表变化时只修正选中索引，不自动改写源路径,
// 避免启动时对整个系统盘触发全量统计导致卡顿
watch(volumes, (v: string | any[]) => {
  if (v.length === 0) {
    srcDev.value = 0
    return
  }
  if (srcDev.value >= v.length) {
    srcDev.value = v.length - 1
  }
})

type OrgOpt = 'YYYY/MM/DD' | 'YYYY/MM' | 'Flat'
const orgOpts: OrgOpt[] = ['YYYY/MM/DD', 'YYYY/MM', 'Flat']
const orgKeys: Record<OrgOpt, string> = {
  'YYYY/MM/DD': 'home.org_yyyy_mm_dd',
  'YYYY/MM': 'home.org_yyyy_mm',
  Flat: 'home.org_flat'
}

function startScan() {
  router.push({ name: 'preview' })
}
</script>

<template>
  <div class="home-view">
    <div class="page-header">
      <h1>{{ $t('home.new_backup') }}</h1>
      <p>{{ $t('home.configure_hint') }}</p>
    </div>

    <div class="config-grid">
      <!-- Source -->
      <DesignCard padding="18px">
        <div class="card-section">
          <div class="section-title">
            <span class="icon-accent">
              <svg width="13" height="13" viewBox="0 0 20 20" fill="currentColor">
                <path
                  d="M2 3a1 1 0 011-1h2.153a1 1 0 01.986.836l.74 4.435a1 1 0 01-.54 1.06l-1.548.773a11.037 11.037 0 006.105 6.105l.774-1.548a1 1 0 011.059-.54l4.435.74a1 1 0 01.836.986V17a1 1 0 01-1 1h-2C7.82 18 2 12.18 2 5V3z"
                />
              </svg>
            </span>
            <DesignLabel :text="$t('home.source_device')" />
          </div>

          <div class="device-grid">
            <button
              v-for="(d, i) in devices"
              :key="i"
              :class="['device-btn', { selected: srcDev === i }]"
              :disabled="!d.connected"
              @click="selectDev(i)"
            >
              <span class="device-name" :class="{ 'text-accent': srcDev === i }">
                <span class="device-icon">
                  <svg
                    v-if="d.icon === 'phone'"
                    width="13"
                    height="13"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      d="M2 3a1 1 0 011-1h2.153a1 1 0 01.986.836l.74 4.435a1 1 0 01-.54 1.06l-1.548.773a11.037 11.037 0 006.105 6.105l.774-1.548a1 1 0 011.059-.54l4.435.74a1 1 0 01.836.986V17a1 1 0 01-1 1h-2C7.82 18 2 12.18 2 5V3z"
                    />
                  </svg>
                  <svg
                    v-else-if="d.icon === 'sd'"
                    width="13"
                    height="13"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      fillRule="evenodd"
                      d="M3 5a2 2 0 012-2h6l4 4v8a2 2 0 01-2 2H5a2 2 0 01-2-2V5zm7 0v3h3l-3-3z"
                      clipRule="evenodd"
                    />
                  </svg>
                  <svg v-else width="13" height="13" viewBox="0 0 20 20" fill="currentColor">
                    <path
                      fillRule="evenodd"
                      d="M2 5a2 2 0 012-2h12a2 2 0 012 2v2a2 2 0 01-2 2H4a2 2 0 01-2-2V5zm14 1a1 1 0 11-2 0 1 1 0 012 0zM2 13a2 2 0 012-2h12a2 2 0 012 2v2a2 2 0 01-2 2H4a2 2 0 01-2-2v-2zm14 1a1 1 0 11-2 0 1 1 0 012 0z"
                      clipRule="evenodd"
                    />
                  </svg>
                </span>
                {{ d.label }}
              </span>
              <span class="device-sub">{{ formatBytes(d.usedBytes) }}</span>
            </button>
          </div>

          <DesignPathInput
            :label="$t('home.source_path')"
            :value="srcPath"
            :placeholder="$t('home.source_placeholder')"
            :hint="
              srcStats
                ? $t('home.scan_hint', {
                    files: srcStats.count.toLocaleString(),
                    size: formatBytes(srcStats.bytes)
                  })
                : undefined
            "
            @update:value="srcPath = $event"
          />

          <div class="storage-row">
            <div class="storage-row-header">
              <span>{{ $t('home.device_storage') }}</span>
              <span class="mono">
                {{
                  selectedDevice
                    ? `${formatBytes(selectedDevice.total_space - selectedDevice.free_space)} / ${formatBytes(selectedDevice.total_space)} · ${$t('home.available_space', { size: formatBytes(selectedDevice.free_space) })}`
                    : '--'
                }}
              </span>
            </div>
            <ProgressBar :value="selectedPct" :color="storageColor(selectedPct)" />
          </div>
        </div>
      </DesignCard>

      <!-- Destination -->
      <DesignCard padding="18px">
        <div class="card-section">
          <div class="section-title">
            <span class="icon-new">
              <svg width="13" height="13" viewBox="0 0 20 20" fill="currentColor">
                <path
                  fillRule="evenodd"
                  d="M2 5a2 2 0 012-2h12a2 2 0 012 2v2a2 2 0 01-2 2H4a2 2 0 01-2-2V5zm14 1a1 1 0 11-2 0 1 1 0 012 0zM2 13a2 2 0 012-2h12a2 2 0 012 2v2a2 2 0 01-2 2H4a2 2 0 01-2-2v-2zm14 1a1 1 0 11-2 0 1 1 0 012 0z"
                  clipRule="evenodd"
                />
              </svg>
            </span>
            <DesignLabel :text="$t('home.destination')" />
          </div>

          <DesignPathInput
            :label="$t('home.backup_path')"
            :value="dstPath"
            :placeholder="$t('home.backup_placeholder')"
            @update:value="dstPath = $event"
          />

          <div class="storage-row">
            <div class="storage-row-header">
              <span>{{ $t('home.free_space') }}</span>
              <span class="mono text-new" style="font-weight: 600">{{
                $t('home.available_space', { size: formatBytes(dstFree) })
              }}</span>
            </div>
            <ProgressBar :value="dstPct" :color="storageColor(dstPct)" />
          </div>

          <div class="org-section">
            <DesignLabel :text="$t('home.organize_by')" />
            <div class="toggle-row">
              <DesignToggle
                v-for="(o, i) in orgOpts"
                :key="o"
                :label="$t(orgKeys[o])"
                :on="org === i"
                @change="org = i"
              />
            </div>
          </div>
        </div>
      </DesignCard>
    </div>

    <!-- Filters -->
    <DesignCard padding="18px">
      <div class="filters-grid">
        <div class="filter-col">
          <div class="filter-header text-backed">
            <svg width="13" height="13" viewBox="0 0 20 20" fill="currentColor">
              <path
                fillRule="evenodd"
                d="M4 3a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V5a2 2 0 00-2-2H4zm12 12H4l4-8 3 6 2-4 3 6z"
                clipRule="evenodd"
              />
            </svg>
            <DesignLabel :text="$t('home.filter_images')" />
          </div>
          <div class="toggle-row">
            <DesignToggle
              v-for="f in IMAGE_FORMATS"
              :key="f"
              :label="f.toUpperCase()"
              :on="fmts[f]"
              @change="toggleFmt(f)"
            />
          </div>
        </div>
        <div class="filter-col">
          <div class="filter-header text-modified">
            <svg width="13" height="13" viewBox="0 0 20 20" fill="currentColor">
              <path
                d="M2 6a2 2 0 012-2h6a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V6zm12.553 1.106A1 1 0 0013 8v4a1 1 0 001.553.894l3-2a1 1 0 000-1.788l-3-2z"
              />
            </svg>
            <DesignLabel :text="$t('home.filter_videos')" />
          </div>
          <div class="toggle-row">
            <DesignToggle
              v-for="f in VIDEO_FORMATS"
              :key="f"
              :label="f.toUpperCase()"
              :on="fmts[f]"
              @change="toggleFmt(f)"
            />
          </div>
        </div>
        <div class="filter-col">
          <DesignLabel :text="$t('home.file_size_mb')" />
          <div class="size-grid">
            <div>
              <span class="size-label">{{ $t('home.filter_min_size') }}</span>
              <input v-model="minSz" placeholder="0" class="size-input" />
            </div>
            <div>
              <span class="size-label">{{ $t('home.filter_max_size') }}</span>
              <input v-model="maxSz" :placeholder="$t('home.no_limit')" class="size-input" />
            </div>
          </div>
        </div>
      </div>
    </DesignCard>

    <div class="page-actions">
      <button class="btn-secondary">{{ $t('home.save_preset') }}</button>
      <button class="btn-primary" @click="startScan">
        <svg width="15" height="15" viewBox="0 0 20 20" fill="currentColor">
          <path
            fillRule="evenodd"
            d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"
            clipRule="evenodd"
          />
        </svg>
        {{ $t('home.start_scan') }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.home-view {
  display: flex;
  height: 100%;
  padding: 26px 30px;
  overflow-y: auto;
  flex-direction: column;
  gap: 18px;
}

.page-header h1 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  letter-spacing: -0.01em;
  color: var(--text);
}

.page-header p {
  margin: 3px 0 0;
  font-size: 13px;
  color: var(--text-sec);
}

.config-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}

.card-section {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 7px;
}

.icon-accent {
  display: flex;
  color: var(--accent);
}

.icon-new {
  display: flex;
  color: var(--new);
}

.device-grid {
  display: flex;
  gap: 6px;
}

.device-btn {
  display: flex;
  padding: 9px 10px;
  text-align: left;
  cursor: pointer;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 7px;
  flex: 1;
  flex-direction: column;
  gap: 3px;
}

.device-btn.selected {
  background: var(--accent-dim);
  border-color: rgb(79 127 255 / 40%);
}

.device-btn:disabled {
  cursor: not-allowed;
  opacity: 0.35;
}

.device-name {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text);
}

.device-name.text-accent {
  color: var(--accent);
}

.device-icon {
  display: flex;
  color: var(--text-sec);
}

.device-name.text-accent .device-icon {
  color: var(--accent);
}

.device-sub {
  font-family: var(--mono);
  font-size: 10px;
  color: var(--text-muted);
}

.storage-row {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.storage-row-header {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  color: var(--text-sec);
}

.mono {
  font-family: var(--mono);
}

.text-new {
  color: var(--new);
}

.text-accent {
  color: var(--accent);
}

.text-backed {
  color: var(--backed);
}

.text-modified {
  color: var(--modified);
}

.org-section {
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.toggle-row {
  display: flex;
  gap: 5px;
  flex-wrap: wrap;
}

.filters-grid {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 18px;
}

.filter-col {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.filter-header {
  display: flex;
  align-items: center;
  gap: 5px;
}

.size-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.size-label {
  display: block;
  margin-bottom: 3px;
  font-size: 10px;
  color: var(--text-muted);
}

.size-input {
  width: 100%;
  padding: 7px 9px;
  font-family: var(--mono);
  font-size: 12px;
  color: var(--text);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
}

.page-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.btn-secondary {
  padding: 9px 18px;
  font-size: 13px;
  color: var(--text-sec);
  cursor: pointer;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 7px;
}

.btn-primary {
  display: flex;
  padding: 9px 22px;
  font-size: 13px;
  font-weight: 600;
  color: #fff;
  cursor: pointer;
  background: var(--accent);
  border: none;
  border-radius: 7px;
  box-shadow: 0 4px 14px rgb(79 127 255 / 30%);
  align-items: center;
  gap: 6px;
}
</style>
