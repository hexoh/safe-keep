<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

const srcPath = ref('/Volumes/iPhone 15 Pro/DCIM')
const dstPath = ref('/Users/zhao/Pictures/SafeKeep')
const srcDev = ref(0)
const org = ref(1)
const minSz = ref('0')
const maxSz = ref('')
const fmts = reactive({
  jpg: true,
  heic: true,
  png: true,
  raw: false,
  mp4: true,
  mov: true,
  mkv: false
})

function toggleFmt(k: keyof typeof fmts) {
  fmts[k] = !fmts[k]
}

const devices = [
  { icon: 'phone', label: 'iPhone 15 Pro', sub: '47.2 GB used', pct: 37, connected: true },
  { icon: 'sd', label: 'SanDisk 32 GB', sub: '18.1 GB used', pct: 56, connected: true },
  { icon: 'hdd', label: 'USB Drive', sub: 'Not detected', pct: 0, connected: false }
]

const orgOpts = ['YYYY/MM/DD', 'YYYY/MM', 'Flat']

function startScan() {
  router.push({ name: 'preview' })
}
</script>

<template>
  <div class="home-view">
    <div class="page-header">
      <h1>New Backup</h1>
      <p>Configure source, destination, and filters — then scan.</p>
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
            <DesignLabel text="Source Device" />
          </div>

          <div class="device-grid">
            <button
              v-for="(d, i) in devices"
              :key="i"
              :class="['device-btn', { selected: srcDev === i }]"
              :disabled="!d.connected"
              @click="srcDev = i"
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
              <span class="device-sub">{{ d.sub }}</span>
            </button>
          </div>

          <DesignPathInput
            label="Source Path"
            :value="srcPath"
            placeholder="/Volumes/Device/DCIM"
            hint="1,247 files found · 4.3 GB estimated"
            @update:value="srcPath = $event"
          />

          <div class="storage-row">
            <div class="storage-row-header">
              <span>Device storage</span>
              <span class="mono">47.2 / 128 GB</span>
            </div>
            <ProgressBar :value="37" />
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
            <DesignLabel text="Destination" />
          </div>

          <DesignPathInput
            label="Backup Path"
            :value="dstPath"
            placeholder="/path/to/backup"
            @update:value="dstPath = $event"
          />

          <div class="storage-row">
            <div class="storage-row-header">
              <span>Free space</span>
              <span class="mono text-new" style="font-weight: 600">412.8 GB available</span>
            </div>
            <ProgressBar :value="41" :color="'var(--new)'" />
          </div>

          <div class="org-section">
            <DesignLabel text="Organize output by" />
            <div class="toggle-row">
              <DesignToggle
                v-for="(o, i) in orgOpts"
                :key="o"
                :label="o"
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
            <DesignLabel text="Images" />
          </div>
          <div class="toggle-row">
            <DesignToggle
              v-for="f in ['jpg', 'heic', 'png', 'raw'] as const"
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
            <DesignLabel text="Videos" />
          </div>
          <div class="toggle-row">
            <DesignToggle
              v-for="f in ['mp4', 'mov', 'mkv'] as const"
              :key="f"
              :label="f.toUpperCase()"
              :on="fmts[f]"
              @change="toggleFmt(f)"
            />
          </div>
        </div>
        <div class="filter-col">
          <DesignLabel text="File Size (MB)" />
          <div class="size-grid">
            <div>
              <span class="size-label">Min</span>
              <input v-model="minSz" placeholder="0" class="size-input" />
            </div>
            <div>
              <span class="size-label">Max</span>
              <input v-model="maxSz" placeholder="No limit" class="size-input" />
            </div>
          </div>
        </div>
      </div>
    </DesignCard>

    <div class="page-actions">
      <button class="btn-secondary">Save Preset</button>
      <button class="btn-primary" @click="startScan">
        <svg width="15" height="15" viewBox="0 0 20 20" fill="currentColor">
          <path
            fillRule="evenodd"
            d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"
            clipRule="evenodd"
          />
        </svg>
        Start Scan
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
