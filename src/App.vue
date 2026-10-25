<script setup lang="ts">
import { ElMessage, ElNotification } from 'element-plus'
import { useRouter } from 'vue-router'
import { useUpdater } from '@/composables/useUpdater'
import { useTheme } from '@/composables/useTheme'
import { useLocale } from '@/composables/useLocale'
import { useSettings } from '@/composables/useSettings'
import { getSourceRoots, dryRunCleanup } from '@/api/cleanup'
import { formatBytes } from '@/utils/format'
import { i18n } from '@/locales'
import LayoutDefault from '@/layout/LayoutDefault.vue'

const router = useRouter()
const { checking, updateAvailable, updateVersion, installUpdate, checkForUpdates } = useUpdater()
const { settings, load } = useSettings()
useTheme()
useLocale()

const REMINDER_DAYS = 90
let reminderChecked = false

async function checkCleanupReminder() {
  if (reminderChecked) return
  reminderChecked = true

  try {
    const roots = await getSourceRoots()
    const results = await Promise.all(
      roots.map((sourceRoot) =>
        dryRunCleanup({ source_root: sourceRoot, before_days: REMINDER_DAYS })
      )
    )

    const files = results.reduce((sum, result) => sum + result.total_files, 0)
    const size = results.reduce((sum, result) => sum + result.total_size, 0)

    if (files === 0) return

    ElNotification({
      title: i18n.global.t('cleanup.reminder_title'),
      message: i18n.global.t('cleanup.reminder_message', {
        count: files,
        size: formatBytes(size)
      }),
      type: 'info',
      duration: 8000,
      onClick: () => router.push({ name: 'cleanup' })
    })
  } catch {
    // Cleanup reminder is best-effort; never block startup.
  }
}

onMounted(async () => {
  await load()

  if (settings.value.auto_update) void checkForUpdates()
  if (settings.value.auto_cleanup_reminder) void checkCleanupReminder()
})

async function installUpdateSafely() {
  try {
    await installUpdate()
  } catch (e: any) {
    ElMessage.error(
      i18n.global.t('updater.install_failed', {
        error: typeof e === 'string' ? e : (e?.message ?? 'Unknown error')
      })
    )
  }
}
</script>

<template>
  <el-alert
    v-if="checking"
    type="info"
    show-icon
    :closable="false"
    :title="$t('updater.checking')"
  />
  <el-alert v-if="updateAvailable" type="warning" show-icon :closable="false">
    <template #title>
      {{ $t('updater.available', { version: updateVersion }) }}
    </template>
    <template #default>
      <el-button size="small" type="primary" @click="installUpdateSafely">
        {{ $t('updater.install') }}
      </el-button>
    </template>
  </el-alert>

  <LayoutDefault />
</template>
