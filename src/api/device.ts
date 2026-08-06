import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export type DeviceType = 'MTP' | 'RemovableDisk' | 'LocalDisk' | 'Unknown'

export interface DeviceVolume {
  path: string
  name: string
  device_type: DeviceType
  total_space: number
  free_space: number
}

export async function detectDevice(path: string): Promise<DeviceType> {
  return invoke<DeviceType>('detect_device', { path })
}

export async function listDevices(): Promise<DeviceVolume[]> {
  return invoke<DeviceVolume[]>('list_devices')
}

// 监听磁盘插入 / 拔出变化
export function onDevicesChanged(callback: (devices: DeviceVolume[]) => void): Promise<() => void> {
  return listen<DeviceVolume[]>('devices:changed', (event) => {
    callback(event.payload)
  })
}
