<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from '../stores/settings'
import { useToast } from '../composables/useToast'
import { themeMeta } from '../plugins/vuetify'

const emit = defineEmits<{ back: [] }>()
const settingsStore = useSettingsStore()
const { showToast } = useToast()

const { colorName, mode, setColorTheme, setMode } = settingsStore

const themes = Object.entries(themeMeta) as Array<[string, { label: string; color: string }]>

const themeModes = [
  { value: 'light' as const, label: '浅色', icon: 'mdi-white-balance-sunny' },
  { value: 'system' as const, label: '跟随系统', icon: 'mdi-desktop-mac' },
  { value: 'dark' as const, label: '深色', icon: 'mdi-moon-waning-crescent' },
]

interface AudioDeviceInfo {
  name: string
  isDefault: boolean
}

interface AudioDevicesResponse {
  devices: AudioDeviceInfo[]
  defaultDeviceName: string | null
}

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
    showToast(String(e))
    await loadDevices()
  }
}

onMounted(loadDevices)
</script>

<template>
  <div class="settings-view">
    <div class="settings-header">
      <v-btn icon variant="plain" size="small" @click="emit('back')">
        <v-icon icon="mdi-arrow-left" size="20" />
      </v-btn>
      <h2 class="settings-title">设置</h2>
    </div>

    <v-card class="settings-card" variant="flat" color="surface">
      <div class="card-header">
        <v-icon icon="mdi-theme-light-dark" size="18" class="card-icon" />
        <span class="card-title">主题模式</span>
      </div>

      <div class="mode-toggle">
        <button
          v-for="modeOption in themeModes"
          :key="modeOption.value"
          :class="['mode-button', { active: mode === modeOption.value }]"
          @click="setMode(modeOption.value)"
        >
          <v-icon :icon="modeOption.icon" size="16" />
          <span class="mode-label">{{ modeOption.label }}</span>
        </button>
      </div>
    </v-card>

    <v-card class="settings-card" variant="flat" color="surface">
      <div class="card-header">
        <v-icon icon="mdi-palette" size="18" class="card-icon" />
        <span class="card-title">主题颜色</span>
      </div>

      <div class="theme-grid">
        <button
          v-for="[key, meta] in themes"
          :key="key"
          :class="['theme-option', { active: colorName === key }]"
          @click="setColorTheme(key as any)"
        >
          <span class="theme-swatch" :style="{ background: meta.color }" />
          <span class="theme-label">{{ meta.label }}</span>
        </button>
      </div>
    </v-card>

    <v-card class="settings-card" variant="flat" color="surface">
      <div class="card-header">
        <v-icon icon="mdi-speaker" size="18" class="card-icon" />
        <span class="card-title">输出设备</span>
      </div>

      <div v-if="devices.length === 0" class="device-empty">未检测到输出设备</div>
      <div v-else class="device-list">
        <button
          :class="['device-option', { active: selectedDevice === null }]"
          @click="selectDevice(null)"
        >
          <v-icon icon="mdi-speaker" size="16" />
          <span class="device-name">默认设备<span v-if="defaultDeviceName" class="device-sub">（{{ defaultDeviceName }}）</span></span>
        </button>
        <button
          v-for="device in devices"
          :key="device.name"
          :class="['device-option', { active: selectedDevice === device.name }]"
          @click="selectDevice(device.name)"
        >
          <v-icon :icon="device.isDefault ? 'mdi-star' : 'mdi-speaker'" size="16" />
          <span class="device-name">{{ device.name }}</span>
        </button>
      </div>
    </v-card>
  </div>
</template>

<style scoped>
.settings-view {
  padding: 32px;
  overflow-y: auto;
  height: 100%;
}

.settings-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 24px;
}

.settings-title {
  font-size: var(--text-xl);
  font-weight: 700;
  color: rgb(var(--v-theme-on-background));
}

.settings-card {
  padding: 20px;
  margin-bottom: 16px;
  border-radius: 12px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
}

.card-icon {
  color: rgb(var(--v-theme-primary));
}

.card-title {
  font-size: var(--text-md);
  font-weight: 600;
}

.mode-toggle {
  display: flex;
  gap: 8px;
  padding: 4px;
  background: rgb(var(--v-theme-surface-variant));
  border-radius: 10px;
  border: 1px solid rgb(var(--v-theme-border));
}

.mode-button {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 12px;
  border-radius: 8px;
  border: none;
  background: transparent;
  cursor: pointer;
  transition: all 0.2s;
  color: rgb(var(--v-theme-on-background));
  font-size: var(--text-sm);
  font-weight: 500;
}

.mode-button:hover {
  background: rgba(var(--v-theme-primary), 0.1);
}

.mode-button.active {
  background: rgb(var(--v-theme-primary));
  color: rgb(var(--v-theme-on-primary));
  font-weight: 600;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.theme-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.theme-option {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 14px 8px;
  border-radius: 12px;
  border: 2px solid transparent;
  background: transparent;
  cursor: pointer;
  transition: border-color 0.2s, background 0.2s;
}

.theme-option:hover {
  background: var(--v-accent-bg);
}

.theme-option.active {
  border-color: rgb(var(--v-theme-primary));
  background: var(--v-accent-bg);
}

.theme-swatch {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: block;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

.theme-label {
  font-size: var(--text-sm);
  color: var(--v-text-secondary);
}

.device-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.device-option {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border-radius: 8px;
  border: 2px solid transparent;
  background: transparent;
  cursor: pointer;
  transition: border-color 0.2s, background 0.2s;
  color: rgb(var(--v-theme-on-background));
  font-size: var(--text-md);
  text-align: left;
  width: 100%;
}

.device-option:hover {
  background: var(--v-accent-bg);
}

.device-option.active {
  border-color: rgb(var(--v-theme-primary));
  background: var(--v-accent-bg);
}

.device-name {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.device-sub {
  color: var(--v-text-secondary);
  font-size: var(--text-xs);
}

.device-empty {
  font-size: var(--text-sm);
  color: var(--v-text-secondary);
  text-align: center;
  padding: 16px;
}
</style>
