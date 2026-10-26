<script setup lang="ts">
import { ElMessage } from 'element-plus'
import { useTheme } from '@/composables/useTheme'
import type { ThemeMode } from '@/composables/useTheme'
import { useLocale } from '@/composables/useLocale'
import type { Language } from '@/composables/useLocale'
import { useSettings } from '@/composables/useSettings'
import { useUpdater } from '@/composables/useUpdater'
import { useAppVersion } from '@/composables/useAppVersion'
import { i18n } from '@/locales'
import type { AppSettings } from '@/types/settings'

const { theme } = useTheme()
const { language } = useLocale()
const { settings, loading, load, save } = useSettings()
const { checking, updateAvailable, updateVersion, updateError, checkForUpdates } = useUpdater()
const { appVersion } = useAppVersion()

const ready = ref(false)
const saving = ref(false)
const saved = ref(false)

let saveTimer: ReturnType<typeof setTimeout> | undefined
let saveQueued = false

type CompareStrategy = AppSettings['compare_strategy']
type ConflictStrategy = AppSettings['conflict_strategy']
type DeleteStrategy = AppSettings['delete_strategy']
type UpdateChannel = AppSettings['update_channel']

const compareOptions = computed<{ value: CompareStrategy; label: string }[]>(() => [
  { value: 'size_time', label: i18n.global.t('settings.compare_size_time') },
  { value: 'fast_hash', label: i18n.global.t('settings.compare_fast_hash') },
  { value: 'sha256', label: i18n.global.t('settings.compare_sha256') }
])

const conflictOptions = computed<{ value: ConflictStrategy; label: string }[]>(() => [
  { value: 'rename', label: i18n.global.t('settings.conflict_rename') },
  { value: 'skip', label: i18n.global.t('settings.conflict_skip') },
  { value: 'overwrite', label: i18n.global.t('settings.conflict_overwrite') }
])

const deleteOptions = computed<{ value: DeleteStrategy; label: string }[]>(() => [
  { value: 'recycle', label: i18n.global.t('settings.delete_recycle') },
  { value: 'permanent', label: i18n.global.t('settings.delete_permanent') }
])

const themeOptions = computed<{ value: ThemeMode; label: string }[]>(() => [
  { value: 'light', label: i18n.global.t('settings.theme_light') },
  { value: 'dark', label: i18n.global.t('settings.theme_dark') },
  { value: 'system', label: i18n.global.t('settings.theme_system') }
])

const languageOptions = computed<{ value: Language; label: string }[]>(() => [
  { value: 'system', label: i18n.global.t('settings.language_system') },
  { value: 'zh-CN', label: i18n.global.t('settings.language_zh') },
  { value: 'en', label: i18n.global.t('settings.language_en') }
])

const channelOptions = computed<{ value: UpdateChannel; label: string }[]>(() => [
  { value: 'stable', label: i18n.global.t('settings.channel_stable') },
  { value: 'beta', label: i18n.global.t('settings.channel_beta') }
])

function normalizeLanguage(value: string): Language {
  return value === 'en' || value === 'zh-CN' || value === 'system' ? value : 'system'
}

function normalizeTheme(value: string): ThemeMode {
  return value === 'light' || value === 'dark' || value === 'system' ? value : 'system'
}

function applyTheme(value: ThemeMode) {
  theme.value = value
  settings.value.theme = value
}

function applyLanguage(value: Language) {
  language.value = value
  settings.value.language = value
}

function scheduleSave() {
  saved.value = false
  clearTimeout(saveTimer)
  saveTimer = setTimeout(() => {
    void saveNow()
  }, 500)
}

async function saveNow() {
  clearTimeout(saveTimer)
  if (saving.value) {
    saveQueued = true
    return
  }
  saving.value = true
  try {
    await save()
    saved.value = true
  } catch (e: any) {
    ElMessage.error(
      i18n.global.t('settings.save_failed', {
        error: typeof e === 'string' ? e : (e?.message ?? 'Unknown error')
      })
    )
  } finally {
    saving.value = false
    if (saveQueued) {
      saveQueued = false
      void saveNow()
    }
  }
}

async function checkUpdate() {
  await checkForUpdates()
  if (updateError.value) {
    ElMessage.error(i18n.global.t('settings.update_check_failed', { error: updateError.value }))
    return
  }
  if (updateAvailable.value) {
    ElMessage.success(i18n.global.t('settings.update_available', { version: updateVersion.value }))
    return
  }
  ElMessage.info(i18n.global.t('settings.up_to_date'))
}

watch(
  settings,
  () => {
    if (ready.value) scheduleSave()
  },
  { deep: true }
)

onMounted(async () => {
  await load()

  settings.value.concurrent_threads = Math.min(
    16,
    Math.max(1, Number(settings.value.concurrent_threads) || 4)
  )
  const loadedTheme = normalizeTheme(settings.value.theme)
  const loadedLanguage = normalizeLanguage(settings.value.language)
  theme.value = loadedTheme
  settings.value.theme = loadedTheme
  language.value = loadedLanguage
  settings.value.language = loadedLanguage

  ready.value = true
})

onBeforeUnmount(() => {
  clearTimeout(saveTimer)
  if (ready.value && !saved.value && !saving.value) void save().catch(() => {})
})
</script>

<template>
  <div class="settings-view">
    <div class="page-header">
      <div>
        <h1>{{ $t('settings.title') }}</h1>
        <p>{{ $t('settings.subtitle') }}</p>
      </div>
      <div class="header-actions">
        <span v-if="saving" class="save-state">{{ $t('settings.saving') }}</span>
        <span v-else-if="saved" class="save-state saved">{{ $t('settings.saved') }}</span>
        <button class="btn-primary-sm" :disabled="saving || loading" @click="saveNow">
          {{ $t('common.save') }}
        </button>
      </div>
    </div>

    <DesignCard padding="20px">
      <DesignLabel :text="$t('settings.performance')" />
      <div class="settings-section">
        <DesignPathInput
          :label="$t('settings.default_dest')"
          :value="settings.default_dest"
          :placeholder="$t('settings.default_dest_placeholder')"
          :hint="$t('settings.default_dest_desc')"
          @update:value="settings.default_dest = $event"
        />

        <div class="settings-row">
          <div class="row-text">
            <div class="row-label">{{ $t('settings.concurrent_threads') }}</div>
            <div class="row-desc">{{ $t('settings.concurrent_threads_desc') }}</div>
          </div>
          <div class="thread-control">
            <input
              v-model.number="settings.concurrent_threads"
              type="range"
              min="1"
              max="16"
              class="thread-range"
            />
            <input
              v-model.number="settings.concurrent_threads"
              type="number"
              min="1"
              max="16"
              class="thread-number"
            />
          </div>
        </div>

        <div class="settings-row">
          <div class="row-text">
            <div class="row-label">{{ $t('settings.compare_strategy') }}</div>
            <div class="row-desc">{{ $t('settings.compare_strategy_desc') }}</div>
          </div>
          <DesignRadioGroup
            :options="compareOptions"
            :model-value="settings.compare_strategy"
            @update:model-value="settings.compare_strategy = $event"
          />
        </div>
      </div>
    </DesignCard>

    <DesignCard padding="20px">
      <DesignLabel :text="$t('settings.backup_cleanup')" />
      <div class="settings-section">
        <div class="settings-row">
          <div class="row-text">
            <div class="row-label">{{ $t('settings.conflict_strategy') }}</div>
            <div class="row-desc">{{ $t('settings.conflict_strategy_desc') }}</div>
          </div>
          <DesignRadioGroup
            :options="conflictOptions"
            :model-value="settings.conflict_strategy"
            @update:model-value="settings.conflict_strategy = $event"
          />
        </div>

        <div class="settings-row">
          <div class="row-text">
            <div class="row-label">{{ $t('settings.delete_strategy') }}</div>
            <div class="row-desc">{{ $t('settings.delete_strategy_desc') }}</div>
          </div>
          <DesignRadioGroup
            :options="deleteOptions"
            :model-value="settings.delete_strategy"
            @update:model-value="settings.delete_strategy = $event"
          />
        </div>

        <div v-if="settings.delete_strategy === 'permanent'" class="warning-box">
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
          <span>{{ $t('settings.permanent_warning') }}</span>
        </div>

        <div class="settings-row">
          <div class="row-text">
            <div class="row-label">{{ $t('settings.dry_run_default') }}</div>
            <div class="row-desc">{{ $t('settings.dry_run_default_desc') }}</div>
          </div>
          <DesignSwitcher
            :on="settings.dry_run_default"
            @change="settings.dry_run_default = $event"
          />
        </div>

        <div class="settings-row">
          <div class="row-text">
            <div class="row-label">{{ $t('settings.auto_cleanup_reminder') }}</div>
            <div class="row-desc">{{ $t('settings.auto_cleanup_reminder_desc') }}</div>
          </div>
          <DesignSwitcher
            :on="settings.auto_cleanup_reminder"
            @change="settings.auto_cleanup_reminder = $event"
          />
        </div>
      </div>
    </DesignCard>

    <DesignCard padding="20px">
      <DesignLabel :text="$t('settings.appearance')" />
      <div class="settings-section">
        <div class="settings-row">
          <div class="row-text">
            <div class="row-label">{{ $t('settings.theme') }}</div>
            <div class="row-desc">{{ $t('settings.theme_desc') }}</div>
          </div>
          <DesignRadioGroup
            :options="themeOptions"
            :model-value="theme"
            @update:model-value="applyTheme"
          />
        </div>

        <div class="settings-row">
          <div class="row-text">
            <div class="row-label">{{ $t('settings.language') }}</div>
            <div class="row-desc">{{ $t('settings.language_desc') }}</div>
          </div>
          <DesignRadioGroup
            :options="languageOptions"
            :model-value="language"
            @update:model-value="applyLanguage"
          />
        </div>
      </div>
    </DesignCard>

    <DesignCard padding="20px">
      <DesignLabel :text="$t('settings.updates')" />
      <div class="settings-section">
        <div class="settings-row">
          <div class="row-text">
            <div class="row-label">{{ $t('settings.auto_update') }}</div>
            <div class="row-desc">{{ $t('settings.auto_update_desc') }}</div>
          </div>
          <DesignSwitcher :on="settings.auto_update" @change="settings.auto_update = $event" />
        </div>

        <div class="settings-row">
          <div class="row-text">
            <div class="row-label">{{ $t('settings.update_channel') }}</div>
            <div class="row-desc">{{ $t('settings.update_channel_desc') }}</div>
          </div>
          <DesignRadioGroup
            :options="channelOptions"
            :model-value="settings.update_channel"
            @update:model-value="settings.update_channel = $event"
          />
        </div>
      </div>
    </DesignCard>

    <DesignCard padding="13px 18px">
      <div class="about-row">
        <div>
          <div class="about-title">{{ $t('settings.about_title') }}</div>
          <div class="about-version">
            <span v-if="appVersion">v{{ appVersion }} · </span>
            Tauri 2.x · {{ $t('settings.about_license') }}
          </div>
        </div>
        <button class="btn-secondary-sm" :disabled="checking" @click="checkUpdate">
          {{ checking ? $t('settings.checking') : $t('settings.check_update') }}
        </button>
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

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 14px;
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
  align-items: center;
  gap: 10px;
}

.save-state {
  font-size: 11px;
  color: var(--text-muted);
}

.save-state.saved {
  color: var(--new);
}

.btn-primary-sm {
  padding: 7px 16px;
  font-size: 12px;
  font-weight: 600;
  color: #fff;
  cursor: pointer;
  background: var(--accent);
  border: none;
  border-radius: 7px;
}

.btn-primary-sm:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.settings-section {
  display: flex;
  margin-top: 16px;
  flex-direction: column;
  gap: 16px;
}

.settings-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.row-text {
  min-width: 0;
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
  flex-shrink: 0;
  align-items: center;
  gap: 10px;
}

.thread-range {
  width: 120px;
  accent-color: var(--accent);
}

.thread-number {
  width: 58px;
  padding: 6px 8px;
  font-family: var(--mono);
  font-size: 13px;
  color: var(--text);
  text-align: center;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
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
  gap: 14px;
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
  white-space: nowrap;
  cursor: pointer;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
}

.btn-secondary-sm:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}
</style>
