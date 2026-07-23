<script setup lang="ts">
const sel = ref(new Set<number>())
const path = ref('/Users/zhao/Desktop/Restored')
const conflict = ref<'skip' | 'overwrite' | 'rename'>('rename')
const srcId = ref(1)

const HISTORY = [
  {
    id: 1,
    time: '2024-03-15 14:30',
    source: '/Volumes/iPhone 15 Pro/DCIM',
    dest: '/Users/zhao/Pictures/SafeKeep',
    files: 47,
    size: '1.2 GB'
  },
  {
    id: 2,
    time: '2024-03-12 09:15',
    source: 'D:\\DCIM\\Camera',
    dest: 'E:\\PhotoBackup\\2024',
    files: 128,
    size: '4.7 GB'
  },
  {
    id: 3,
    time: '2024-03-01 10:00',
    source: '/Volumes/SanDisk_32GB/DCIM',
    dest: '/Users/zhao/Pictures/SD',
    files: 312,
    size: '18.3 GB'
  }
]

const RESTORED = [
  {
    id: 1,
    name: 'IMG_20240201_084512.jpg',
    sizeStr: '3.9 MB',
    deletedAt: '2024-03-01 10:28',
    path: '/Users/zhao/Pictures/Backup/2024/02'
  },
  {
    id: 2,
    name: 'VID_20240201_091234.mp4',
    sizeStr: '215.4 MB',
    deletedAt: '2024-03-01 10:29',
    path: '/Users/zhao/Pictures/Backup/2024/02'
  },
  {
    id: 3,
    name: 'IMG_20240130_162011.jpg',
    sizeStr: '4.4 MB',
    deletedAt: '2024-03-01 10:28',
    path: '/Users/zhao/Pictures/Backup/2024/01'
  },
  {
    id: 4,
    name: 'IMG_20240128_200345.heic',
    sizeStr: '5.8 MB',
    deletedAt: '2024-02-15 09:14',
    path: '/Users/zhao/Pictures/SD_Backup/2024/01'
  }
]

function toggle(id: number) {
  const n = new Set(sel.value)
  if (n.has(id)) n.delete(id)
  else n.add(id)
  sel.value = n
}

function toggleAll(e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  sel.value = checked ? new Set(RESTORED.map((f) => f.id)) : new Set()
}

const conflictOptions = [
  { value: 'skip' as const, label: 'Skip existing files' },
  { value: 'overwrite' as const, label: 'Overwrite existing files' },
  { value: 'rename' as const, label: 'Rename with suffix (_1, _2…)' }
]
</script>

<template>
  <div class="restore-view">
    <div class="page-header">
      <h1>Restore Files</h1>
      <p>Recover files deleted from source that still exist in your backup.</p>
    </div>

    <div class="config-grid">
      <DesignCard padding="18px">
        <DesignLabel text="Backup Source Session" />
        <div class="session-list">
          <label
            v-for="h in HISTORY"
            :key="h.id"
            :class="['session-item', { selected: srcId === h.id }]"
          >
            <input type="radio" name="src" :checked="srcId === h.id" @change="srcId = h.id" />
            <div>
              <div class="session-time">{{ h.time }}</div>
              <div class="session-meta">{{ h.files }} files · {{ h.size }}</div>
            </div>
          </label>
        </div>
      </DesignCard>

      <DesignCard padding="18px">
        <DesignLabel text="Restore Settings" />
        <div class="settings-section">
          <DesignPathInput
            label="Target Path"
            :value="path"
            placeholder="/path/to/restore"
            @update:value="path = $event"
          />
          <div class="conflict-section">
            <DesignLabel text="Conflict Resolution" />
            <div class="conflict-list">
              <label
                v-for="opt in conflictOptions"
                :key="opt.value"
                :class="['conflict-item', { selected: conflict === opt.value }]"
              >
                <input
                  type="radio"
                  name="conflict"
                  :value="opt.value"
                  :checked="conflict === opt.value"
                  @change="conflict = opt.value"
                />
                <span>{{ opt.label }}</span>
              </label>
            </div>
          </div>
        </div>
      </DesignCard>
    </div>

    <DesignCard style="overflow: hidden">
      <div class="table-header">
        <DesignLabel text="Deleted files available for restore" />
        <span class="file-count">{{ RESTORED.length }} files found</span>
      </div>
      <table class="design-table">
        <thead>
          <tr>
            <th class="th-check">
              <input type="checkbox" :checked="sel.size === RESTORED.length" @change="toggleAll" />
            </th>
            <th>File Name</th>
            <th>Backup Path</th>
            <th class="th-size">Size</th>
            <th class="th-date">Deleted At</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="f in RESTORED"
            :key="f.id"
            :class="['table-row', { selected: sel.has(f.id) }]"
            @click="toggle(f.id)"
          >
            <td class="td-check">
              <input type="checkbox" :checked="sel.has(f.id)" @click.stop="toggle(f.id)" />
            </td>
            <td>
              <span class="mono-sm">{{ f.name }}</span>
            </td>
            <td>
              <span class="mono-sm muted mono-path">{{ f.path }}</span>
            </td>
            <td class="mono muted">{{ f.sizeStr }}</td>
            <td class="mono muted">{{ f.deletedAt }}</td>
          </tr>
        </tbody>
      </table>
    </DesignCard>

    <div class="action-row">
      <button class="btn-secondary-sm">Cancel</button>
      <button :class="['btn-primary-sm', { disabled: sel.size === 0 }]" :disabled="sel.size === 0">
        <svg width="15" height="15" viewBox="0 0 20 20" fill="currentColor">
          <path
            fillRule="evenodd"
            d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0015 13h-2.101a1 1 0 110-2H18a1 1 0 011 1v4a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z"
            clipRule="evenodd"
          />
        </svg>
        Restore {{ sel.size > 0 ? `${sel.size} Files` : 'Files' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.restore-view {
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

.session-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 12px;
}

.session-item {
  display: flex;
  padding: 10px 13px;
  cursor: pointer;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 7px;
  align-items: center;
  gap: 10px;
}

.session-item.selected {
  background: var(--accent-dim);
  border-color: rgb(79 127 255 / 40%);
}

.session-time {
  font-size: 12px;
  font-weight: 500;
  color: var(--text);
}

.session-meta {
  font-family: var(--mono);
  font-size: 11px;
  color: var(--text-muted);
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 14px;
  margin-top: 12px;
}

.conflict-section {
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.conflict-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.conflict-item {
  display: flex;
  padding: 6px 9px;
  cursor: pointer;
  border-radius: 6px;
  align-items: center;
  gap: 8px;
}

.conflict-item.selected {
  background: var(--accent-dim);
}

.conflict-item span {
  font-size: 12px;
  color: var(--text);
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
  width: 90px;
}

.th-date {
  width: 140px;
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

.mono-sm {
  font-family: var(--mono);
  font-size: 12px;
}

.mono-path {
  font-size: 11px;
}

.muted {
  color: var(--text-muted);
}

.action-row {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
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
  align-items: center;
  gap: 6px;
}

.btn-primary-sm.disabled {
  color: var(--text-muted);
  cursor: not-allowed;
  background: var(--surface);
  opacity: 0.5;
}
</style>
