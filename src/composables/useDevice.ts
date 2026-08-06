import { ref } from 'vue'
import { listDevices, onDevicesChanged } from '@/api/device'
import type { DeviceVolume } from '@/api/device'

export function useDevice() {
  const devices = ref<DeviceVolume[]>([])

  async function refresh() {
    devices.value = await listDevices()
  }

  function watchDevices() {
    return onDevicesChanged((volumes) => {
      devices.value = volumes
    })
  }

  return {
    devices,
    refresh,
    watchDevices
  }
}
