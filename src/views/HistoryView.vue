<script setup lang="ts">
const open = ref<number | null>(null)

const HISTORY = [
  {
    id: 1,
    time: '2024-03-15 14:30',
    source: '/Volumes/iPhone 15 Pro/DCIM',
    dest: '/Users/zhao/Pictures/SafeKeep',
    files: 47,
    size: '1.2 GB',
    status: 'success',
    duration: '3m 24s',
    newFiles: 32,
    backedFiles: 12,
    failedFiles: 0
  },
  {
    id: 2,
    time: '2024-03-12 09:15',
    source: 'D:\\DCIM\\Camera',
    dest: 'E:\\PhotoBackup\\2024',
    files: 128,
    size: '4.7 GB',
    status: 'success',
    duration: '12m 11s',
    newFiles: 89,
    backedFiles: 39,
    failedFiles: 0
  },
  {
    id: 3,
    time: '2024-03-08 17:45',
    source: '/gvfs/mtp:host=OnePlus/DCIM',
    dest: '/home/zhao/Photos',
    files: 23,
    size: '892 MB',
    status: 'partial',
    duration: '5m 03s',
    newFiles: 18,
    backedFiles: 3,
    failedFiles: 2
  },
  {
    id: 4,
    time: '2024-03-01 10:00',
    source: '/Volumes/SanDisk_32GB/DCIM',
    dest: '/Users/zhao/Pictures/SD',
    files: 312,
    size: '18.3 GB',
    status: 'success',
    duration: '28m 47s',
    newFiles: 312,
    backedFiles: 0,
    failedFiles: 0
  }
]

const statusColors: Record<string, string> = {
  success: 'var(--new)',
  partial: 'var(--modified)',
  error: 'var(--failed)'
}

const statusBgs: Record<string, string> = {
  success: 'var(--new-bg)',
  partial: 'var(--modified-bg)',
  error: 'var(--failed-bg)'
}
</script>

<template>
  <div class="history-view">
    <div class="page-header">
      <div>
        <h1>Backup History</h1>
        <p>Complete log of all past backup sessions.</p>
      </div>
      <button class="btn-secondary-sm">Export CSV</button>
    </div>

    <div class="history-list">
      <DesignCard v-for="h in HISTORY" :key="h.id" style="overflow: hidden">
        <div class="history-item" @click="open = open === h.id ? null : h.id">
          <div
            class="status-dot"
            :style="{
              background: statusColors[h.status] || 'var(--text-sec)',
              boxShadow: `0 0 0 3px ${statusBgs[h.status] || 'var(--surface)'}`
            }"
          />
          <div class="history-grid">
            <div>
              <div class="meta-label">Time</div>
              <div class="mono-sm">{{ h.time }}</div>
            </div>
            <div class="route-col">
              <div class="meta-label">Route</div>
              <div class="route-text">
                <span class="route-src">{{ h.source }}</span>
                <svg width="12" height="12" viewBox="0 0 20 20" fill="currentColor" class="chevron">
                  <path
                    fillRule="evenodd"
                    d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z"
                    clipRule="evenodd"
                  />
                </svg>
                <span class="route-dst">{{ h.dest }}</span>
              </div>
            </div>
            <div>
              <div class="meta-label">Files · Size · Duration</div>
              <div class="mono-sm">
                {{ h.files }} · {{ h.size }} ·
                <span class="muted">{{ h.duration }}</span>
              </div>
            </div>
            <StatusBadge :status="h.status" />
            <span class="expand-arrow">{{ open === h.id ? '▲' : '▼' }}</span>
          </div>
        </div>

        <div v-if="open === h.id" class="history-detail">
          <div class="detail-grid">
            <div
              v-for="s in [
                { lbl: 'New Files', val: h.newFiles, c: 'var(--new)', bg: 'var(--new-bg)' },
                { lbl: 'Skipped', val: h.backedFiles, c: 'var(--backed)', bg: 'var(--backed-bg)' },
                { lbl: 'Failed', val: h.failedFiles, c: 'var(--failed)', bg: 'var(--failed-bg)' },
                { lbl: 'Duration', val: h.duration, c: 'var(--text)', bg: 'var(--surface)' }
              ]"
              :key="s.lbl"
              class="detail-item"
              :style="{
                background: s.bg,
                borderColor: s.bg === 'var(--surface)' ? 'var(--border)' : 'transparent'
              }"
            >
              <div class="detail-label" :style="{ color: s.c }">{{ s.lbl }}</div>
              <div class="detail-val" :style="{ color: s.c }">{{ s.val }}</div>
            </div>
          </div>
          <div v-if="h.failedFiles > 0" class="failed-section">
            <div class="failed-title">Failed Files</div>
            <div class="failed-line">VID_20240308_142211.mp4 — Permission denied (EACCES)</div>
            <div class="failed-line">IMG_20240307_093422.heic — File corrupted (CRC mismatch)</div>
          </div>
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

.btn-secondary-sm {
  padding: 7px 14px;
  font-size: 12px;
  color: var(--text-sec);
  cursor: pointer;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 7px;
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
  grid-template-columns: 140px 1fr 140px 100px 30px;
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
  padding: 14px 18px;
  border-top: 1px solid var(--border);
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
}

.detail-item {
  padding: 10px 14px;
  border: 1px solid;
  border-radius: 7px;
}

.detail-label {
  margin-bottom: 4px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.detail-val {
  font-family: var(--mono);
  font-size: 22px;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.failed-section {
  padding: 10px 14px;
  margin-top: 10px;
  background: var(--failed-bg);
  border: 1px solid rgb(239 68 68 / 20%);
  border-radius: 7px;
  grid-column: 1 / -1;
}

.failed-title {
  margin-bottom: 6px;
  font-size: 11px;
  font-weight: 600;
  color: var(--failed);
}

.failed-line {
  padding-left: 10px;
  margin-bottom: 3px;
  font-family: var(--mono);
  font-size: 11px;
  line-height: 1.6;
  color: var(--text-sec);
  border-left: 2px solid var(--failed);
}
</style>
