<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()
const status = ref<'running' | 'paused' | 'done'>('running')
const overallPct = ref(0)
const filePct = ref(0)
const fileIdx = ref(0)
const logs = ref([
  '[14:23:01] Starting backup session — 9 files, 527.8 MB',
  '[14:23:01] ✓ Output directory ready: /Users/zhao/Pictures/SafeKeep/2024/03/',
  '[14:23:02] → Copying VID_20240315_091022.mp4  (128.5 MB)'
])
const logRef = ref<HTMLDivElement | null>(null)

const mockFiles = [
  { name: 'VID_20240315_091022.mp4', sizeStr: '128.5 MB', type: 'video' as const },
  { name: 'VID_20240313_221045.mov', sizeStr: '256.3 MB', type: 'video' as const },
  { name: 'IMG_20240315_142301.jpg', sizeStr: '4.1 MB', type: 'image' as const }
]

const cur = computed(() => mockFiles[Math.min(fileIdx.value, mockFiles.length - 1)])
const speed = computed(() =>
  status.value === 'running' ? (42.8 + Math.sin(Date.now() / 900) * 6).toFixed(1) : '0.0'
)
const done = computed(() => Math.min(fileIdx.value, mockFiles.length))

let intervalId: ReturnType<typeof setInterval> | null = null

function advanceFile() {
  const nextIdx = fileIdx.value + 1
  fileIdx.value = nextIdx
  const increment = Math.floor(100 / mockFiles.length)
  overallPct.value = Math.min(100, overallPct.value + increment)
  filePct.value = 0

  if (nextIdx >= mockFiles.length || overallPct.value >= 100) {
    status.value = 'done'
    overallPct.value = 100
    logs.value = [
      ...logs.value,
      '[14:26:25] ✓ Backup complete — 9 files, 527.8 MB transferred in 3m 24s'
    ]
    if (intervalId) clearInterval(intervalId)
    return
  }

  const nf = mockFiles[nextIdx]
  if (nf) {
    logs.value = [
      ...logs.value.slice(-30),
      `[14:${23 + nextIdx}:${String((nextIdx * 7) % 60).padStart(2, '0')}] → Copying ${nf.name}  (${nf.sizeStr})`
    ]
  }
}

function tick() {
  if (status.value !== 'running') return
  const next = filePct.value + 3
  if (next >= 100) {
    filePct.value = 100
    advanceFile()
  } else {
    filePct.value = next
  }
}

onMounted(() => {
  intervalId = setInterval(tick, 90)
})

onUnmounted(() => {
  if (intervalId) clearInterval(intervalId)
})

watch(logs, () => {
  nextTick(() => {
    if (logRef.value) logRef.value.scrollTop = logRef.value.scrollHeight
  })
})
</script>

<template>
  <div class="backup-view">
    <div class="page-header">
      <div>
        <h1 :class="{ 'text-new': status === 'done' }">
          {{
            status === 'done'
              ? '✓ Backup Complete'
              : status === 'paused'
                ? '⏸ Paused'
                : 'Backup Running'
          }}
        </h1>
        <p class="route-mono">iPhone 15 Pro → /Users/zhao/Pictures/SafeKeep</p>
      </div>
      <div class="header-actions">
        <template v-if="status !== 'done'">
          <button class="btn-outline" @click="status = status === 'paused' ? 'running' : 'paused'">
            <svg
              v-if="status === 'paused'"
              width="14"
              height="14"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fillRule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z"
                clipRule="evenodd"
              />
            </svg>
            <svg v-else width="14" height="14" viewBox="0 0 20 20" fill="currentColor">
              <path
                fillRule="evenodd"
                d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zM7 8a1 1 0 012 0v4a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v4a1 1 0 102 0V8a1 1 0 00-1-1z"
                clipRule="evenodd"
              />
            </svg>
            {{ status === 'paused' ? 'Resume' : 'Pause' }}
          </button>
          <button class="btn-danger-outline">
            <svg width="14" height="14" viewBox="0 0 20 20" fill="currentColor">
              <path
                fillRule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zM8 7a1 1 0 00-1 1v4a1 1 0 001 1h4a1 1 0 001-1V8a1 1 0 00-1-1H8z"
                clipRule="evenodd"
              />
            </svg>
            Cancel
          </button>
        </template>
        <button v-else class="btn-primary-sm" @click="router.push({ name: 'history' })">
          View History →
        </button>
      </div>
    </div>

    <!-- Global progress -->
    <DesignCard padding="20px">
      <div class="progress-section">
        <div class="progress-header">
          <span class="progress-title">Overall Progress</span>
          <span :class="['progress-pct', { 'text-new': status === 'done' }]">
            {{ overallPct }}%
          </span>
        </div>
        <ProgressBar
          :value="overallPct"
          :color="status === 'done' ? 'var(--new)' : 'var(--accent)'"
          :height="10"
        />
        <div class="speed-grid">
          <div
            v-for="s in [
              { lbl: 'Transfer Speed', val: `${speed} MB/s`, c: 'var(--accent)' },
              { lbl: 'Transferred', val: `${(overallPct * 5.27).toFixed(0)} MB`, c: 'var(--text)' },
              { lbl: 'Files Done', val: `${done} / ${mockFiles.length}`, c: 'var(--text)' },
              {
                lbl: 'ETA',
                val:
                  status === 'done'
                    ? 'Complete'
                    : `${Math.max(0, Math.ceil(((100 - overallPct) / (100 / mockFiles.length)) * 0.09 * 60))}s`,
                c: status === 'done' ? 'var(--new)' : 'var(--modified)'
              }
            ]"
            :key="s.lbl"
            class="speed-item"
          >
            <DesignLabel :text="s.lbl" />
            <span class="speed-val" :style="{ color: s.c }">{{ s.val }}</span>
          </div>
        </div>
      </div>
    </DesignCard>

    <!-- Current file -->
    <DesignCard v-if="status !== 'done' && cur" padding="14px">
      <div class="current-file-header">
        <div class="file-info">
          <span :class="['file-icon', cur.type === 'video' ? 'text-modified' : 'text-backed']">
            <svg
              v-if="cur.type === 'video'"
              width="13"
              height="13"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                d="M2 6a2 2 0 012-2h6a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V6zm12.553 1.106A1 1 0 0013 8v4a1 1 0 001.553.894l3-2a1 1 0 000-1.788l-3-2z"
              />
            </svg>
            <svg v-else width="13" height="13" viewBox="0 0 20 20" fill="currentColor">
              <path
                fillRule="evenodd"
                d="M4 3a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V5a2 2 0 00-2-2H4zm12 12H4l4-8 3 6 2-4 3 6z"
                clipRule="evenodd"
              />
            </svg>
          </span>
          <span class="file-name">{{ cur.name }}</span>
          <span class="file-size">{{ cur.sizeStr }}</span>
        </div>
        <span class="file-pct">{{ filePct }}%</span>
      </div>
      <ProgressBar :value="filePct" :height="3" />
    </DesignCard>

    <!-- Log -->
    <DesignCard
      style="display: flex; min-height: 0; overflow: hidden; flex: 1; flex-direction: column"
    >
      <div class="log-header">
        <DesignLabel text="Live Log" />
        <div class="log-header-right">
          <span v-if="status === 'running'" class="live-indicator">
            <span class="live-dot" />
            LIVE
          </span>
          <button class="clear-btn">Clear</button>
        </div>
      </div>
      <div ref="logRef" class="log-content">
        <div
          v-for="(line, i) in logs"
          :key="i"
          :class="[
            'log-line',
            { 'log-success': line.includes('✓'), 'log-transfer': line.includes('→') }
          ]"
        >
          {{ line }}
        </div>
      </div>
    </DesignCard>
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

.route-mono {
  margin: 3px 0 0;
  font-family: var(--mono);
  font-size: 12px;
  color: var(--text-sec);
}

.header-actions {
  display: flex;
  gap: 7px;
}

.btn-outline {
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
  font-size: 16px;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.current-file-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 7px;
}

.file-icon {
  display: flex;
}

.file-name {
  font-family: var(--mono);
  font-size: 13px;
  font-weight: 500;
  color: var(--text);
}

.file-size {
  font-family: var(--mono);
  font-size: 11px;
  color: var(--text-muted);
}

.file-pct {
  font-family: var(--mono);
  font-size: 12px;
  color: var(--text-sec);
}

.text-backed {
  color: var(--backed);
}

.text-modified {
  color: var(--modified);
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
</style>
