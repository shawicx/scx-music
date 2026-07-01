<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useStartupOptions } from '../../composables/useStartupOptions'
import { useToast } from '../../composables/useToast'
import { useI18n } from '../../composables/useI18n'

const { t } = useI18n()
const { getAutostart, setAutostart, isRestoreEnabled, setRestoreEnabled } = useStartupOptions()
const { showSuccess, showError } = useToast()

const autostart = ref(false)
const restorePlayback = ref(false)
const loading = ref(true)

onMounted(async () => {
  try {
    // 自启读系统真实状态；恢复播放读 settings
    const [autoVal, restoreVal] = await Promise.all([getAutostart(), isRestoreEnabled()])
    autostart.value = autoVal
    restorePlayback.value = restoreVal
  } catch (e) {
    console.warn('[startup settings] load failed:', e)
  } finally {
    loading.value = false
  }
})

async function onAutostartChange(val: boolean | null) {
  const enabled = val === true
  const prev = autostart.value
  autostart.value = enabled
  try {
    await setAutostart(enabled)
    showSuccess(t('settings.autostartUpdated'))
  } catch (e) {
    autostart.value = prev // 回滚
    showError(t('settings.autostartEnableFailed'))
    console.error('[startup settings] set autostart failed:', e)
  }
}

async function onRestoreChange(val: boolean | null) {
  const enabled = val === true
  const prev = restorePlayback.value
  restorePlayback.value = enabled
  try {
    await setRestoreEnabled(enabled)
  } catch (e) {
    restorePlayback.value = prev // 回滚
    console.error('[startup settings] set restore failed:', e)
  }
}
</script>

<template>
  <v-card class="settings-card" variant="flat" color="surface">
    <div class="card-header">
      <v-icon icon="mdi-power" size="18" class="card-icon" />
      <span class="card-title">{{ t('settings.startupTitle') }}</span>
    </div>

    <div class="switch-row">
      <div class="switch-text">
        <div class="switch-label">{{ t('settings.autostart') }}</div>
        <div class="switch-desc">{{ t('settings.autostartDesc') }}</div>
      </div>
      <v-switch
        :model-value="autostart"
        :loading="loading"
        :disabled="loading"
        color="primary"
        hide-details
        density="compact"
        @update:model-value="onAutostartChange"
      />
    </div>

    <v-divider class="switch-divider" />

    <div class="switch-row">
      <div class="switch-text">
        <div class="switch-label">{{ t('settings.restorePlayback') }}</div>
        <div class="switch-desc">{{ t('settings.restorePlaybackDesc') }}</div>
      </div>
      <v-switch
        :model-value="restorePlayback"
        :loading="loading"
        :disabled="loading"
        color="primary"
        hide-details
        density="compact"
        @update:model-value="onRestoreChange"
      />
    </div>
  </v-card>
</template>

<style src="../../styles/settings-card.css"></style>
<style scoped>
.switch-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 10px 0;
}

.switch-text {
  flex: 1;
  min-width: 0;
}

.switch-label {
  font-size: var(--text-md);
  font-weight: 500;
  color: rgb(var(--v-theme-on-background));
}

.switch-desc {
  font-size: var(--text-xs);
  color: var(--v-text-secondary);
  margin-top: 2px;
}

.switch-divider {
  margin: 0;
}
</style>
