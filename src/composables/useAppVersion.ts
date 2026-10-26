import { ref } from 'vue'
import { getVersion } from '@tauri-apps/api/app'

const appVersion = ref('')
let requested = false

/** 运行时获取应用版本号，失败时保持为空（例如浏览器环境）。 */
export function useAppVersion() {
  if (!requested) {
    requested = true
    getVersion()
      .then((version) => {
        appVersion.value = version
      })
      .catch(() => {
        appVersion.value = ''
      })
  }

  return { appVersion }
}
