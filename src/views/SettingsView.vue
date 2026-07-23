<script setup lang="ts">
import { useTheme } from '@/composables/useTheme'
import { useLocale } from '@/composables/useLocale'

const { theme } = useTheme()
const { language } = useLocale()

const threads = ref(4)
const dupPol = ref<'skip' | 'overwrite' | 'rename'>('skip')
const delMode = ref<'recycle' | 'permanent'>('recycle')
const autoBack = ref(false)
const notifs = ref(true)

const themeOptions = [
  { value: 'light' as const, label: 'Light' },
  { value: 'dark' as const, label: 'Dark' },
  { value: 'system' as const, label: 'System' }
]

const langOptions = [
  { value: 'system' as const, label: 'System / 系统' },
  { value: 'en' as const, label: 'English' },
  { value: 'zh-CN' as const, label: '中文' }
]

const conflictOptions = [
  { value: 'skip' as const, label: 'Skip' },
  { value: 'overwrite' as const, label: 'Overwrite' },
  { value: 'rename' as const, label: 'Rename (_1, _2…)' }
]

const deleteOptions = [
  { value: 'recycle' as const, label: 'Recycle Bin' },
  { value: 'permanent' as const, label: 'Permanent' }
]
</script>

<template>
  <div class="settings-view">
    <div class="page-header">
      <h1>Settings</h1>
      <p>Configure Safe Keep behavior and preferences.</p>
    </div>

    <!-- Performance -->
    <DesignCard padding="20px">
      <DesignLabel text="Performance" />
      <div class="settings-section">
        <div class="settings-row">
          <div>
            <div class="row-label">Parallel Transfer Threads</div>
            <div class="row-desc">
              Higher values saturate I/O bandwidth faster but increase CPU load
            </div>
          </div>
          <div class="thread-control">
            <input
              type="range"
              min="1"
              max="8"
              :value="threads"
              @input="threads = +($event.target as HTMLInputElement).value"
              style="width: 100px"
            />
            <span class="thread-val">{{ threads }}</span>
          </div>
        </div>
        <div class="settings-row">
          <div>
            <div class="row-label">Auto-backup on Device Connect</div>
            <div class="row-desc">
              Start scanning automatically when a source device is detected
            </div>
          </div>
          <DesignSwitcher :on="autoBack" @change="autoBack = $event" />
        </div>
        <div class="settings-row">
          <div>
            <div class="row-label">Desktop Notifications</div>
            <div class="row-desc">
              Show system alerts on completion, errors, or low-space warnings
            </div>
          </div>
          <DesignSwitcher :on="notifs" @change="notifs = $event" />
        </div>
      </div>
    </DesignCard>

    <!-- Conflict Policy -->
    <DesignCard padding="20px">
      <DesignLabel text="Conflict Policy" />
      <div class="settings-section">
        <div class="settings-row">
          <div>
            <div class="row-label">Duplicate File Handling</div>
            <div class="row-desc">
              Action when a file with the same name already exists at destination
            </div>
          </div>
          <DesignRadioGroup
            :options="conflictOptions"
            :model-value="dupPol"
            @update:model-value="dupPol = $event"
          />
        </div>
      </div>
    </DesignCard>

    <!-- Delete Strategy -->
    <DesignCard padding="20px">
      <DesignLabel text="Delete Strategy" />
      <div class="settings-section">
        <div class="settings-row">
          <div>
            <div class="row-label">Source Deletion Method</div>
            <div class="row-desc">How source files are removed during Safety Cleanup</div>
          </div>
          <DesignRadioGroup
            :options="deleteOptions"
            :model-value="delMode"
            @update:model-value="delMode = $event"
          />
        </div>
        <div v-if="delMode === 'permanent'" class="warning-box">
          <svg
            width="13"
            height="13"
            viewBox="0 0 20 20"
            fill="currentColor"
            style="color: var(--modified); flex-shrink: 0"
          >
            <path
              fillRule="evenodd"
              d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
              clipRule="evenodd"
            />
          </svg>
          <span>
            Permanent deletion bypasses the recycle bin. Files cannot be recovered once deleted.
          </span>
        </div>
      </div>
    </DesignCard>

    <!-- Appearance -->
    <DesignCard padding="20px">
      <DesignLabel text="Appearance" />
      <div class="settings-section">
        <div class="settings-row">
          <div>
            <div class="row-label">Color Theme</div>
            <div class="row-desc">Controls the application color scheme</div>
          </div>
          <DesignRadioGroup
            :options="themeOptions"
            :model-value="theme"
            @update:model-value="theme = $event"
          />
        </div>
        <div class="settings-row">
          <div>
            <div class="row-label">Language / 语言</div>
            <div class="row-desc">Interface display language</div>
          </div>
          <DesignRadioGroup
            :options="langOptions"
            :model-value="language"
            @update:model-value="language = $event"
          />
        </div>
      </div>
    </DesignCard>

    <!-- About -->
    <DesignCard padding="13px 18px">
      <div class="about-row">
        <div>
          <div class="about-title">Safe Keep · 归影</div>
          <div class="about-version">v2.1.0 · Tauri 2.x · Built 2024-03-15 · MIT License</div>
        </div>
        <button class="btn-secondary-sm">Check for Updates</button>
      </div>
    </DesignCard>
  </div>
</template>

<style scoped>
.settings-view {
  display: flex;
  height: 100%;
  padding: 26px 30px;
  overflow-y: auto;
  flex-direction: column;
  gap: 16px;
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

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 14px;
  margin-top: 16px;
}

.settings-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.row-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text);
}

.row-desc {
  margin-top: 2px;
  font-size: 11px;
  color: var(--text-muted);
}

.thread-control {
  display: flex;
  align-items: center;
  gap: 10px;
}

.thread-val {
  width: 18px;
  font-family: var(--mono);
  font-size: 16px;
  font-weight: 700;
  color: var(--accent);
  text-align: center;
}

.warning-box {
  display: flex;
  padding: 10px 14px;
  font-size: 12px;
  line-height: 1.6;
  color: var(--modified);
  background: var(--modified-bg);
  border: 1px solid rgb(245 158 11 / 30%);
  border-radius: 7px;
  gap: 8px;
}

.about-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.about-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
}

.about-version {
  margin-top: 2px;
  font-family: var(--mono);
  font-size: 11px;
  color: var(--text-muted);
}

.btn-secondary-sm {
  padding: 6px 14px;
  font-size: 12px;
  color: var(--text-sec);
  cursor: pointer;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
}
</style>
