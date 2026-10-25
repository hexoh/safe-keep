import { ref } from 'vue'
import { check } from '@tauri-apps/plugin-updater'

const checking = ref(false)
const updateAvailable = ref(false)
const updateVersion = ref('')
const updateBody = ref('')
const updateError = ref<string | null>(null)
const downloading = ref(false)
const downloadProgress = ref(0)

async function checkForUpdates() {
  checking.value = true
  updateError.value = null
  try {
    const update = await check()
    if (update) {
      updateAvailable.value = true
      updateVersion.value = update.version
      updateBody.value = update.body ?? ''
    } else {
      updateAvailable.value = false
      updateVersion.value = ''
      updateBody.value = ''
    }
  } catch (e: any) {
    updateError.value = typeof e === 'string' ? e : (e?.message ?? 'Unknown error')
  } finally {
    checking.value = false
  }
}

async function installUpdate() {
  downloading.value = true
  try {
    const update = await check()
    if (update) {
      await update.downloadAndInstall((event) => {
        if (event.event === 'Progress') {
          downloadProgress.value = event.data.chunkLength
        }
      })
    }
  } finally {
    downloading.value = false
  }
}

export function useUpdater() {
  return {
    checking,
    updateAvailable,
    updateVersion,
    updateBody,
    updateError,
    downloading,
    downloadProgress,
    checkForUpdates,
    installUpdate
  }
}
