import { invoke } from '@tauri-apps/api/core'

export type DeviceType = 'MTP' | 'RemovableDisk' | 'LocalDisk' | 'Unknown'

export async function detectDevice(path: string): Promise<DeviceType> {
  return invoke<DeviceType>('detect_device', { path })
}
