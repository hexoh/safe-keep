import { ref } from 'vue'
import { check } from '@tauri-apps/plugin-updater'

const checking = ref(false)
const updateAvailable = ref(false)
const updateVersion = ref('')
const updateBody = ref('')
const downloading = ref(false)
const downloadProgress = ref(0)

async function checkForUpdates() {
  checking.value = true
  try {
    const update = await check()
    if (update) {
      updateAvailable.value = true
      updateVersion.value = update.version
      updateBody.value = update.body ?? ''
    }
  } catch {
    // silent fail
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
    downloading,
    downloadProgress,
    checkForUpdates,
    installUpdate
  }
}
