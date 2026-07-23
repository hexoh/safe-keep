<script setup lang="ts">
const ran = ref(false)
const confirm = ref(false)
const sel = ref(new Set([1, 2, 4]))
const age = ref('90')
const minSz = ref('100')

const CLEANUP = [
  {
    id: 1,
    name: 'VID_20231015_143221.mp4',
    path: '/Users/zhao/Pictures/Backup/2023/10',
    size: '512.0 MB',
    age: '163 days'
  },
  {
    id: 2,
    name: 'VID_20231102_091135.mov',
    path: '/Users/zhao/Pictures/Backup/2023/11',
    size: '1.2 GB',
    age: '145 days'
  },
  {
    id: 3,
    name: 'IMG_20231201_175423.jpg',
    path: '/Users/zhao/Pictures/Backup/2023/12',
    size: '3.8 MB',
    age: '105 days'
  },
  {
    id: 4,
    name: 'VID_20231205_220018.mp4',
    path: '/Users/zhao/Pictures/Backup/2023/12',
    size: '347.2 MB',
    age: '101 days'
  },
  {
    id: 5,
    name: 'IMG_20231220_090812.png',
    path: '/Users/zhao/Pictures/Backup/2023/12',
    size: '8.1 MB',
    age: '86 days'
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
  sel.value = checked ? new Set(CLEANUP.map((f) => f.id)) : new Set()
}
</script>

<template>
  <div class="cleanup-view">
    <div class="page-header">
      <div class="header-left">
        <h1>Safety Cleanup</h1>
        <p>Remove source files only after confirming they are safely backed up.</p>
      </div>
      <div class="warning-badge">
        <svg width="13" height="13" viewBox="0 0 20 20" fill="currentColor">
          <path
            fillRule="evenodd"
            d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
            clipRule="evenodd"
          />
        </svg>
        Destructive — review carefully
      </div>
    </div>

    <DesignCard padding="18px">
      <div class="filter-row">
        <div class="filter-field">
          <DesignLabel text="Older than (days)" />
          <input v-model="age" placeholder="90" class="filter-input" />
        </div>
        <div class="filter-field">
          <DesignLabel text="Min size (MB)" />
          <input v-model="minSz" placeholder="100" class="filter-input" />
        </div>
        <div class="filter-field">
          <DesignLabel text="Source scope" />
          <input value="/Users/zhao/Pictures" class="filter-input" />
        </div>
        <button
          class="sim-btn"
          @click="
            ran = true
            confirm = false
          "
          >Run Simulation</button
        >
      </div>
    </DesignCard>

    <template v-if="ran">
      <div class="results-header">
        <div class="results-title">
          <DesignLabel text="Simulation Results" />
          <span class="dry-run-badge">DRY RUN</span>
        </div>
        <span class="results-summary">
          {{ sel.size }} files ·
          <span class="text-new mono" style="font-weight: 600">2.07 GB to free</span>
        </span>
      </div>

      <DesignCard style="overflow: hidden">
        <table class="design-table">
          <thead>
            <tr>
              <th class="th-check">
                <input type="checkbox" :checked="sel.size === CLEANUP.length" @change="toggleAll" />
              </th>
              <th>File</th>
              <th>Location</th>
              <th class="th-size">Size</th>
              <th class="th-age">Age</th>
              <th class="th-backup">Backup</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="f in CLEANUP"
              :key="f.id"
              :class="['table-row', { selected: sel.has(f.id) }]"
              @click="toggle(f.id)"
            >
              <td class="td-check">
                <input type="checkbox" :checked="sel.has(f.id)" @click.stop="toggle(f.id)" />
              </td>
              <td
                ><span class="mono-sm">{{ f.name }}</span></td
              >
              <td
                ><span class="mono-sm muted mono-path">{{ f.path }}</span></td
              >
              <td class="mono muted">{{ f.size }}</td>
              <td class="mono muted">{{ f.age }}</td>
              <td>
                <span class="verified-badge">
                  <svg width="11" height="11" viewBox="0 0 20 20" fill="currentColor">
                    <path
                      fillRule="evenodd"
                      d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                      clipRule="evenodd"
                    />
                  </svg>
                  Verified
                </span>
              </td>
            </tr>
          </tbody>
        </table>
      </DesignCard>

      <div class="action-row">
        <template v-if="confirm">
          <span class="confirm-warning">⚠ Permanently delete {{ sel.size }} source files?</span>
          <button class="btn-secondary-sm" @click="confirm = false">Cancel</button>
          <button class="btn-danger">Delete Now</button>
        </template>
        <button
          v-else
          class="btn-danger"
          :disabled="sel.size === 0"
          @click="confirm = true"
          :class="{ disabled: sel.size === 0 }"
        >
          <svg width="15" height="15" viewBox="0 0 20 20" fill="currentColor">
            <path
              fillRule="evenodd"
              d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v3a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v3a1 1 0 102 0V8a1 1 0 00-1-1z"
              clipRule="evenodd"
            />
          </svg>
          Delete {{ sel.size }} Files
        </button>
      </div>
    </template>

    <div v-else class="empty-state">
      <span class="empty-icon">🧹</span>
      <span class="empty-text"
        >Configure filters and run a simulation to preview cleanup candidates</span
      >
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
  display: flex;
  padding: 7px 13px;
  font-size: 11px;
  font-weight: 500;
  color: var(--modified);
  background: var(--modified-bg);
  border: 1px solid rgb(245 158 11 / 35%);
  border-radius: 7px;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.filter-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 14px;
  align-items: end;
}

.filter-field {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.filter-input {
  padding: 8px 11px;
  font-family: var(--mono);
  font-size: 13px;
  color: var(--text);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
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

.text-new {
  color: var(--new);
}

.mono {
  font-family: var(--mono);
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

.th-age {
  width: 100px;
}

.th-backup {
  width: 110px;
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

.verified-badge {
  display: flex;
  font-size: 11px;
  font-weight: 500;
  color: var(--new);
  align-items: center;
  gap: 4px;
}

.action-row {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 10px;
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

.btn-danger.disabled {
  color: var(--text-muted);
  cursor: not-allowed;
  background: var(--surface);
  border-color: var(--border);
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  gap: 8px;
  opacity: 0.4;
}

.empty-icon {
  font-size: 36px;
}

.empty-text {
  font-size: 13px;
  color: var(--text-muted);
}
</style>
