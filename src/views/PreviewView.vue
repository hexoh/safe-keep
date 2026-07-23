<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

const MOCK_FILES = [
  {
    id: 1,
    name: 'IMG_20240315_142301.jpg',
    sizeStr: '4.1 MB',
    modified: '2024-03-15 14:23',
    status: 'new',
    type: 'image'
  },
  {
    id: 2,
    name: 'VID_20240315_091022.mp4',
    sizeStr: '128.5 MB',
    modified: '2024-03-15 09:10',
    status: 'new',
    type: 'video'
  },
  {
    id: 3,
    name: 'IMG_20240314_180534.jpg',
    sizeStr: '3.8 MB',
    modified: '2024-03-14 18:05',
    status: 'backed',
    type: 'image'
  },
  {
    id: 4,
    name: 'IMG_20240314_155203.heic',
    sizeStr: '6.1 MB',
    modified: '2024-03-14 15:52',
    status: 'modified',
    type: 'image'
  },
  {
    id: 5,
    name: 'VID_20240313_221045.mov',
    sizeStr: '256.3 MB',
    modified: '2024-03-13 22:10',
    status: 'new',
    type: 'video'
  },
  {
    id: 6,
    name: 'IMG_20240313_102234.png',
    sizeStr: '7.0 MB',
    modified: '2024-03-13 10:22',
    status: 'backed',
    type: 'image'
  },
  {
    id: 7,
    name: 'IMG_20240312_195511.jpg',
    sizeStr: '3.5 MB',
    modified: '2024-03-12 19:55',
    status: 'new',
    type: 'image'
  },
  {
    id: 8,
    name: 'VID_20240312_154323.mp4',
    sizeStr: '100.0 MB',
    modified: '2024-03-12 15:43',
    status: 'failed',
    type: 'video'
  },
  {
    id: 9,
    name: 'IMG_20240311_083412.heic',
    sizeStr: '5.0 MB',
    modified: '2024-03-11 08:34',
    status: 'new',
    type: 'image'
  },
  {
    id: 10,
    name: 'IMG_20240310_171922.jpg',
    sizeStr: '2.5 MB',
    modified: '2024-03-10 17:19',
    status: 'backed',
    type: 'image'
  },
  {
    id: 11,
    name: 'VID_20240309_143021.mp4',
    sizeStr: '87.3 MB',
    modified: '2024-03-09 14:30',
    status: 'new',
    type: 'video'
  },
  {
    id: 12,
    name: 'IMG_20240308_092145.jpg',
    sizeStr: '3.2 MB',
    modified: '2024-03-08 09:21',
    status: 'modified',
    type: 'image'
  }
]

const sel = ref(new Set<number>())
const page = ref(1)
const filter = ref<'all' | 'new' | 'backed' | 'modified' | 'failed'>('all')
const PER = 7

const filtered = computed(() => {
  return filter.value === 'all' ? MOCK_FILES : MOCK_FILES.filter((f) => f.status === filter.value)
})

const paged = computed(() => filtered.value.slice((page.value - 1) * PER, page.value * PER))

const pages = computed(() => Math.ceil(filtered.value.length / PER))

const counts = computed(() => ({
  new: MOCK_FILES.filter((f) => f.status === 'new').length,
  backed: MOCK_FILES.filter((f) => f.status === 'backed').length,
  modified: MOCK_FILES.filter((f) => f.status === 'modified').length,
  failed: MOCK_FILES.filter((f) => f.status === 'failed').length
}))

function toggleFile(id: number) {
  const n = new Set(sel.value)
  if (n.has(id)) n.delete(id)
  else n.add(id)
  sel.value = n
}

function toggleAll() {
  sel.value =
    sel.value.size === paged.value.length && paged.value.length > 0
      ? new Set()
      : new Set(paged.value.map((f) => f.id))
}
</script>

<template>
  <div class="preview-view">
    <div class="page-header">
      <div>
        <h1>Scan Preview</h1>
        <p>Review files before backup. Deselect to skip.</p>
      </div>
      <div class="header-actions">
        <button class="btn-secondary-sm" @click="router.push({ name: 'home' })">← Back</button>
        <button class="btn-primary-sm" @click="router.push({ name: 'backup' })">
          <svg width="15" height="15" viewBox="0 0 20 20" fill="currentColor">
            <path
              fillRule="evenodd"
              d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM6.293 6.707a1 1 0 010-1.414l3-3a1 1 0 011.414 0l3 3a1 1 0 01-1.414 1.414L11 5.414V13a1 1 0 11-2 0V5.414L7.707 6.707a1 1 0 01-1.414 0z"
              clipRule="evenodd"
            />
          </svg>
          Start Backup · {{ counts.new + counts.modified }} files
        </button>
      </div>
    </div>

    <div class="stats-grid">
      <DesignStatCard label="Total Files" :value="MOCK_FILES.length" sub="527.8 MB total" />
      <DesignStatCard label="New" :value="counts.new" sub="Will be copied" color="var(--new)" />
      <DesignStatCard
        label="Already Backed"
        :value="counts.backed"
        sub="Skip (identical)"
        color="var(--backed)"
      />
      <DesignStatCard
        label="Modified"
        :value="counts.modified"
        sub="Changed on device"
        color="var(--modified)"
      />
      <DesignStatCard
        label="Failed"
        :value="counts.failed"
        sub="Cannot read"
        color="var(--failed)"
      />
    </div>

    <div class="filter-tabs">
      <button
        v-for="f in ['all', 'new', 'backed', 'modified', 'failed'] as const"
        :key="f"
        :class="['filter-tab', { active: filter === f }]"
        @click="
          filter = f
          page = 1
        "
      >
        {{ f.charAt(0).toUpperCase() + f.slice(1) }}
        <span v-if="f !== 'all'" class="filter-count">{{ counts[f as keyof typeof counts] }}</span>
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
                  :checked="sel.size === paged.length && paged.length > 0"
                  @change="toggleAll"
                />
              </th>
              <th>File Name</th>
              <th class="th-type">Type</th>
              <th class="th-size">Size</th>
              <th class="th-modified">Modified</th>
              <th class="th-status">Status</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="f in paged"
              :key="f.id"
              :class="['table-row', { selected: sel.has(f.id) }]"
              @click="toggleFile(f.id)"
            >
              <td class="td-check">
                <input type="checkbox" :checked="sel.has(f.id)" @click.stop="toggleFile(f.id)" />
              </td>
              <td>
                <span class="mono-sm">{{ f.name }}</span>
              </td>
              <td>
                <span :class="['file-type', f.type === 'image' ? 'text-backed' : 'text-modified']">
                  <svg
                    v-if="f.type === 'image'"
                    width="13"
                    height="13"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      fillRule="evenodd"
                      d="M4 3a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V5a2 2 0 00-2-2H4zm12 12H4l4-8 3 6 2-4 3 6z"
                      clipRule="evenodd"
                    />
                  </svg>
                  <svg v-else width="13" height="13" viewBox="0 0 20 20" fill="currentColor">
                    <path
                      d="M2 6a2 2 0 012-2h6a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V6zm12.553 1.106A1 1 0 0013 8v4a1 1 0 001.553.894l3-2a1 1 0 000-1.788l-3-2z"
                    />
                  </svg>
                  {{ f.type.toUpperCase() }}
                </span>
              </td>
              <td class="mono muted">{{ f.sizeStr }}</td>
              <td class="mono muted">{{ f.modified }}</td>
              <td><StatusBadge :status="f.status" /></td>
            </tr>
          </tbody>
        </table>
      </div>

      <div class="table-footer">
        <span class="footer-info">
          <span v-if="sel.size > 0" class="selected-count">{{ sel.size }} selected</span>
          {{ (page - 1) * PER + 1 }}–{{ Math.min(page * PER, filtered.length) }} of
          {{ filtered.length }}
        </span>
        <div class="pagination">
          <button
            v-for="p in pages"
            :key="p"
            :class="['page-btn', { active: page === p }]"
            @click="page = p"
          >
            {{ p }}
          </button>
        </div>
      </div>
    </DesignCard>
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

.stats-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
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
</style>
