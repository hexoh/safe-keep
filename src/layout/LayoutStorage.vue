<script setup lang="ts">
import { homeDir } from '@tauri-apps/api/path'
import { useDevice } from '@/composables/useDevice'
import { useSettings } from '@/composables/useSettings'
import { useScanStore } from '@/stores/scan'
import { formatBytes } from '@/utils/format'
import { sortCopy } from '@/utils/array'

const { settings, load } = useSettings()
const { devices, refresh } = useDevice()
const scanStore = useScanStore()

const fallbackPath = ref('')

const targetPath = computed(
  () => scanStore.options?.dest_path || settings.value.default_dest || fallbackPath.value
)

const volume = computed(() => volumeForPath(targetPath.value))

const used = computed(() =>
  volume.value ? Math.max(0, volume.value.total_space - volume.value.free_space) : 0
)

const pct = computed(() =>
  volume.value && volume.value.total_space > 0
    ? Math.round((used.value / volume.value.total_space) * 100)
    : 0
)

// 取路径所在的最长挂载点，避免根卷 "/" 抢先匹配 /Volumes/xxx。
function volumeForPath(path: string) {
  if (!path) return null
  const normalized = path.replace(/\\/g, '/')

  return (
    sortCopy(
      devices.value
        .map((device) => ({
          device,
          root: device.path.replace(/\\/g, '/').replace(/\/+$/, '') || '/'
        }))
        .filter(({ root }) =>
          root === '/' ? normalized.startsWith('/') : normalized.startsWith(`${root}/`)
        ),
      (a, b) => b.root.length - a.root.length
    )[0]?.device ?? null
  )
}

onMounted(async () => {
  await load()
  try {
    await refresh()
  } catch {
    // 设备信息读取失败时保持占位状态，不阻塞界面。
  }
  try {
    fallbackPath.value = await homeDir()
  } catch {
    // ignore
  }
})
</script>

<template>
  <div class="layout-storage">
    <div class="storage-header">
      <span class="storage-label">{{ $t('storage.backup_drive') }}</span>
      <span class="storage-free">
        {{
          volume
            ? $t('storage.free_space', { size: formatBytes(volume.free_space) })
            : $t('storage.not_detected')
        }}
      </span>
    </div>
    <div class="storage-bar">
      <div class="storage-bar-fill" :style="{ width: `${pct}%` }" />
    </div>
    <div class="storage-footer">
      {{
        volume
          ? $t('storage.used_space', {
              used: formatBytes(used),
              total: formatBytes(volume.total_space)
            })
          : '--'
      }}
    </div>
  </div>
</template>

<style scoped>
.layout-storage {
  padding: 9px 10px;
  margin: 8px 2px 0;
  background: var(--card);
  border-radius: 7px;
}

.storage-header {
  display: flex;
  margin-bottom: 5px;
  justify-content: space-between;
  gap: 6px;
}

.storage-label {
  font-size: 10px;
  color: var(--text-muted);
}

.storage-free {
  overflow: hidden;
  font-family: var(--mono);
  font-size: 10px;
  color: var(--text-sec);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.storage-bar {
  height: 3px;
  overflow: hidden;
  background: var(--border);
  border-radius: 3px;
}

.storage-bar-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 3px;
  transition: width 0.35s ease;
}

.storage-footer {
  margin-top: 4px;
  overflow: hidden;
  font-family: var(--mono);
  font-size: 10px;
  color: var(--text-muted);
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
