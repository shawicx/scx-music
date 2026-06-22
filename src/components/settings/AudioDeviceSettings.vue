<script setup lang="ts">
import { useAudioDevice } from '../../composables/useAudioDevice'
import { useI18n } from '../../composables/useI18n'

const { t } = useI18n()
const { devices, defaultDeviceName, selectedDevice, selectDevice } = useAudioDevice()
</script>

<template>
  <v-card class="settings-card" variant="flat" color="surface">
    <div class="card-header">
      <v-icon icon="mdi-speaker" size="18" class="card-icon" />
      <span class="card-title">{{ t('settings.outputDevice') }}</span>
    </div>

    <div v-if="devices.length === 0" class="device-empty">{{ t('settings.noDevices') }}</div>
    <div v-else class="device-list">
      <button
        :class="['device-option', { active: selectedDevice === null }]"
        @click="selectDevice(null)"
      >
        <v-icon icon="mdi-speaker" size="16" />
        <span class="device-name">{{ t('settings.defaultDevice') }}<span v-if="defaultDeviceName" class="device-sub">（{{ defaultDeviceName }}）</span></span>
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
</template>

<style src="../../styles/settings-card.css"></style>
<style scoped>
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
