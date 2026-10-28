<script setup lang="ts">
import { useRouter } from 'vue-router'
import { IMAGE_FORMATS, VIDEO_FORMATS } from '@/constants/media'
import { useDevice } from '@/composables/useDevice'
import { formatBytes } from '@/utils/format'
import { countFiles, cancelCount } from '@/api/scan'
import { homeDir } from '@tauri-apps/api/path'
import { ElMessage } from 'element-plus'
import { i18n } from '@/locales'
import { useScanStore } from '@/stores/scan'
import { useSettings } from '@/composables/useSettings'

const STATE_KEY = 'safe-keep.last-device'
const PRESET_KEY = 'safe-keep.last-preset'
const LAST_DRIVE_KEY = 'safe-keep.last-drive'

// 每个驱动器的记忆对象: { src: 上次源路径, dst: 上次目标路径 }
type DriveState = { src?: string; dst?: string }

function loadState(): Record<string, DriveState> {
  try {
    const parsed = JSON.parse(localStorage.getItem(STATE_KEY) || '{}') as Record<string, DriveState>
    return typeof parsed === 'object' && parsed !== null ? parsed : {}
  } catch {
    return {}
  }
}

function saveState(map: Record<string, DriveState>) {
  try {
    localStorage.setItem(STATE_KEY, JSON.stringify(map))
  } catch {
    // ignore
  }
}

// 找出路径所属的驱动器根路径（/、/Volumes/xxx 等），用于按驱动器分别记忆
function driveRootOf(path: string): string {
  let best = ''
  for (const v of volumes.value) {
    if (v.path && path.startsWith(v.path) && v.path.length > best.length) {
      best = v.path
    }
  }
  return best
}

function mbToBytes(v: string): number | undefined {
  const n = Number(v)
  return Number.isFinite(n) && n > 0 ? Math.round(n * 1024 * 1024) : undefined
}

const router = useRouter()
const scanStore = useScanStore()
const { devices: volumes, refresh, watchDevices } = useDevice()
const { settings: appSettings, load: loadAppSettings } = useSettings()

onMounted(async () => {
  await loadAppSettings()
  try {
    await refresh()
  } catch {
    // ignore device read errors, keep source path as-is
  }
  restoreLastDevice()
  loadPreset()
  await initSrcPath()
  await initDstPath()
  watchDevices()
})

// 启动时恢复上次选择的设备，而不是固定落到系统盘。
function restoreLastDevice() {
  let last = ''
  try {
    last = localStorage.getItem(LAST_DRIVE_KEY) || ''
  } catch {
    // ignore
  }
  const index = volumes.value.findIndex((v) => v.path === last)
  srcDev.value = index >= 0 ? index : 0
}

const srcPath = ref('')
const srcStats = ref<{ count: number; bytes: number }>({ count: 0, bytes: 0 })
const counting = ref(false)
let srcStatsSeq = 0
let srcStatsTimer: ReturnType<typeof setTimeout> | undefined

async function updateSrcStats() {
  const p = srcPath.value.trim()
  const seq = ++srcStatsSeq
  if (!p) {
    srcStats.value = { count: 0, bytes: 0 }
    return
  }
  srcStats.value = { count: 0, bytes: 0 }
  counting.value = true
  try {
    const res = await countFiles(p)
    if (seq === srcStatsSeq) {
      counting.value = false
      srcStats.value = res
    }
  } catch {
    if (seq === srcStatsSeq) {
      counting.value = false
      srcStats.value = { count: 0, bytes: 0 }
    }
  }
}

// 切换源路径: 立即清零并停掉上一次遍历, 稍作防抖后开始统计新的路径
watch(srcPath, () => {
  srcStatsSeq++
  clearTimeout(srcStatsTimer)
  srcStats.value = { count: 0, bytes: 0 }
  counting.value = false
  void cancelCount()
  srcStatsTimer = setTimeout(updateSrcStats, 300)
})

// 初始化默认路径: 优先恢复当前驱动器上次备份目录, 否则使用用户主目录
async function initSrcPath() {
  const drivePath = volumes.value[srcDev.value]?.path
  const last = loadLastSrcPath(drivePath)
  if (last) {
    srcPath.value = last
    return
  }
  try {
    srcPath.value = await homeDir()
  } catch {
    // ignore, keep empty
  }
}

function loadLastSrcPath(drivePath: string | undefined): string {
  if (!drivePath) return ''
  return loadState()[drivePath]?.src || ''
}

function saveSrcPath() {
  const p = srcPath.value.trim()
  const drivePath = driveRootOf(p)
  if (!p || !drivePath) return
  const map = loadState()
  map[drivePath] = { ...map[drivePath], src: p }
  saveState(map)
}

function loadLastDstPath(drivePath: string | undefined): string {
  if (!drivePath) return ''
  return loadState()[drivePath]?.dst || ''
}

// 目标路径与左边源设备联动: 按源设备所属驱动器分别记忆上次备份目录, 无记录时默认个人文件夹
async function initDstPath() {
  const drivePath = volumes.value[srcDev.value]?.path
  const last = loadLastDstPath(drivePath)
  if (last) {
    dstPath.value = last
    return
  }
  if (appSettings.value.default_dest) {
    dstPath.value = appSettings.value.default_dest
    return
  }
  try {
    dstPath.value = await homeDir()
  } catch {
    // ignore, keep empty
  }
}

function saveDstPath() {
  const dst = dstPath.value.trim()
  const drivePath = driveRootOf(srcPath.value)
  if (!dst || !drivePath) return
  const map = loadState()
  map[drivePath] = { ...map[drivePath], dst }
  saveState(map)
}
const dstPath = ref('')
const srcDev = ref(0)
const org = ref(1)
const minSz = ref('0')
const maxSz = ref('')
const excludeBackedUp = ref(true)
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

// 切换源路径: 本机盘若未记忆过上次目录, 默认落到用户主目录, 避免对整个系统盘全量统计
async function selectDev(i: number) {
  srcDev.value = i
  const device = volumes.value[i]
  const drivePath = device?.path
  if (drivePath) {
    try {
      localStorage.setItem(LAST_DRIVE_KEY, drivePath)
    } catch {
      // ignore
    }
  }
  let p = loadLastSrcPath(drivePath)
  if (!p && device?.device_type === 'LocalDisk' && drivePath) {
    try {
      const home = await homeDir()
      if (home.startsWith(drivePath)) p = home
    } catch {
      // ignore, keep drive root
    }
  }
  srcPath.value = p || drivePath || ''

  const lastDst = loadLastDstPath(drivePath)
  if (lastDst) {
    dstPath.value = lastDst
  } else {
    try {
      dstPath.value = await homeDir()
    } catch {
      // ignore, keep as-is
    }
  }
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
  const source = srcPath.value.trim()
  const dest = dstPath.value.trim()
  if (!source) {
    ElMessage.error(i18n.global.t('home.source_required'))
    return
  }
  if (!dest) {
    ElMessage.error(i18n.global.t('home.dest_required'))
    return
  }
  saveSrcPath()
  saveDstPath()
  const imagesOn = IMAGE_FORMATS.some((fmt) => fmts[fmt])
  const videosOn = VIDEO_FORMATS.some((fmt) => fmts[fmt])
  scanStore.setOptions({
    source_path: source,
    dest_path: dest,
    include_images: imagesOn,
    include_videos: videosOn,
    min_size: mbToBytes(minSz.value),
    max_size: mbToBytes(maxSz.value),
    organize_by: orgOpts[org.value],
    exclude_backed_up: excludeBackedUp.value
  })
  router.push({ name: 'preview' })
}

type Preset = {
  org: number
  minSz: string
  maxSz: string
  excludeBackedUp: boolean
  fmts: typeof fmts
}

function savePreset() {
  const preset: Preset = {
    org: org.value,
    minSz: minSz.value,
    maxSz: maxSz.value,
    excludeBackedUp: excludeBackedUp.value,
    fmts: { ...fmts }
  }
  try {
    localStorage.setItem(PRESET_KEY, JSON.stringify(preset))
    ElMessage.success(i18n.global.t('home.preset_saved'))
  } catch {
    ElMessage.error(i18n.global.t('home.preset_save_failed'))
  }
}

// 读取上次保存的过滤条件预设，使“保存预设”真正可复用。
function loadPreset() {
  let raw: string | null = null
  try {
    raw = localStorage.getItem(PRESET_KEY)
  } catch {
    return
  }
  if (!raw) return

  try {
    const preset = JSON.parse(raw) as Partial<Preset>
    if (typeof preset.org === 'number' && preset.org >= 0 && preset.org < orgOpts.length) {
      org.value = preset.org
    }
    if (typeof preset.minSz === 'string') minSz.value = preset.minSz
    if (typeof preset.maxSz === 'string') maxSz.value = preset.maxSz
    if (typeof preset.excludeBackedUp === 'boolean') {
      excludeBackedUp.value = preset.excludeBackedUp
    }
    if (preset.fmts && typeof preset.fmts === 'object') {
      for (const key of Object.keys(fmts) as (keyof typeof fmts)[]) {
        const value = preset.fmts[key]
        if (typeof value === 'boolean') fmts[key] = value
      }
    }
  } catch {
    // ignore malformed preset
  }
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
            <span class="icon-accent"><icon-local-phone /></span>
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
                  <icon-local-phone v-if="d.icon === 'phone'" />
                  <icon-local-sd v-else-if="d.icon === 'sd'" />
                  <icon-local-hdd v-else />
                </span>
                {{ d.label }}
              </span>
              <span class="device-sub">{{
                $t('home.used_space', { size: formatBytes(d.usedBytes) })
              }}</span>
            </button>
          </div>

          <DesignPathInput
            :label="$t('home.source_path')"
            :value="srcPath"
            :placeholder="$t('home.source_placeholder')"
            :hint="
              srcPath.trim()
                ? counting
                  ? $t('home.scanning')
                  : $t('home.scan_hint', {
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
            <span class="icon-new"><icon-local-hdd /></span>
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
            <icon-local-image />
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
            <icon-local-video />
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
          <label class="exclude-row">
            <input v-model="excludeBackedUp" type="checkbox" class="exclude-check" />
            <span>{{ $t('home.exclude_backed_up') }}</span>
          </label>
        </div>
      </div>
    </DesignCard>

    <div class="page-actions">
      <button class="btn-secondary" @click="savePreset">{{ $t('home.save_preset') }}</button>
      <button class="btn-primary" @click="startScan">
        <icon-local-search />
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

.exclude-row {
  display: flex;
  margin-top: 2px;
  font-size: 11px;
  color: var(--text-sec);
  cursor: pointer;
  align-items: center;
  gap: 6px;
}

.exclude-check {
  accent-color: var(--accent);
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
