import { ref } from 'vue'
import type { AppSettings } from '@/types/settings'
import { getSettings as apiGetSettings, saveSettings as apiSaveSettings } from '@/api/settings'

const settings = ref<AppSettings>({
  default_dest: '',
  concurrent_threads: 4,
  compare_strategy: 'size_time',
  conflict_strategy: 'rename',
  delete_strategy: 'recycle',
  dry_run_default: false,
  auto_cleanup_reminder: true,
  language: 'zh-CN',
  theme: 'system',
  auto_update: true,
  update_channel: 'stable'
})

const loaded = ref(false)
const loading = ref(false)

async function load() {
  if (loaded.value) return
  loading.value = true
  try {
    const s = await apiGetSettings()
    settings.value = s
    loaded.value = true
  } catch {
    // use defaults
  } finally {
    loading.value = false
  }
}

async function save() {
  await apiSaveSettings(settings.value)
}

export function useSettings() {
  return {
    settings,
    loaded,
    loading,
    load,
    save
  }
}
