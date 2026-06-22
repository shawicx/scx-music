import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AudioDevicesResponse } from '../types'
import { useToast } from './useToast'

type AudioDeviceInfo = AudioDevicesResponse['devices'][number]

/**
 * 音频输出设备管理
 *
 * - `loadDevices()`：拉取设备列表 + 当前选中设备（onMounted 自动调用）
 * - `selectDevice(name|null)`：乐观更新 selectedDevice → invoke → 始终 loadDevices 重同步；
 *   失败时 showToast + loadDevices 回滚
 *
 * selectDevice(null) 表示用默认设备。
 */
export function useAudioDevice() {
  const toast = useToast()
  const devices = ref<AudioDeviceInfo[]>([])
  const defaultDeviceName = ref<string | null>(null)
  const selectedDevice = ref<string | null>(null)

  async function loadDevices() {
    const res = await invoke<AudioDevicesResponse>('player_get_output_devices')
    devices.value = res.devices
    defaultDeviceName.value = res.defaultDeviceName
    selectedDevice.value = await invoke<string | null>('player_get_current_device')
  }

  async function selectDevice(name: string | null) {
    try {
      selectedDevice.value = name
      await invoke('player_set_output_device', { deviceName: name })
      await loadDevices()
    } catch (e) {
      toast.showToast(String(e))
      await loadDevices()
    }
  }

  onMounted(loadDevices)

  return { devices, defaultDeviceName, selectedDevice, loadDevices, selectDevice }
}
