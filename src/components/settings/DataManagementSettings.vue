<script setup lang="ts">
import { ref } from 'vue'
import { useLibraryStore } from '../../stores/library'
import { useToast } from '../../composables/useToast'
import { useI18n } from '../../composables/useI18n'
import { useImportExport } from '../../composables/useImportExport'

const { t } = useI18n()
const toast = useToast()
const libraryStore = useLibraryStore()
const { exportBackup, importBackup, exportSettings, importSettings } = useImportExport()

const showRestoreDialog = ref(false)
const restoreFilePath = ref('')
const restoreStrategy = ref<'replace' | 'merge'>('replace')
const isBackupLoading = ref(false)

async function handleExportBackup() {
  isBackupLoading.value = true
  try {
    await exportBackup()
  } finally {
    isBackupLoading.value = false
  }
}

async function handleRestoreBackup() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      multiple: false,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (!selected) return
    restoreFilePath.value = selected
    restoreStrategy.value = 'replace'
    showRestoreDialog.value = true
  } catch {
    toast.showToast(t('toast.backupImportFailed'))
  }
}

async function confirmRestore() {
  showRestoreDialog.value = false
  isBackupLoading.value = true
  try {
    const result = await importBackup(restoreFilePath.value, restoreStrategy.value)
    if (result) {
      await libraryStore.loadFromDb()
    }
  } finally {
    isBackupLoading.value = false
  }
}
</script>

<template>
  <v-card class="settings-card" variant="flat" color="surface">
    <div class="card-header">
      <v-icon icon="mdi-database" size="18" class="card-icon" />
      <span class="card-title">{{ t('settings.dataManagement') }}</span>
    </div>
    <div class="action-row">
      <v-btn
        variant="outlined"
        prepend-icon="mdi-upload"
        :loading="isBackupLoading"
        @click="handleExportBackup"
      >
        {{ t('settings.backupLibrary') }}
      </v-btn>
      <v-btn
        variant="outlined"
        prepend-icon="mdi-download"
        :loading="isBackupLoading"
        @click="handleRestoreBackup"
      >
        {{ t('settings.restoreLibrary') }}
      </v-btn>
    </div>
  </v-card>

  <v-card class="settings-card" variant="flat" color="surface">
    <div class="card-header">
      <v-icon icon="mdi-cog-transfer" size="18" class="card-icon" />
      <span class="card-title">{{ t('settings.settingsTransfer') }}</span>
    </div>
    <div class="action-row">
      <v-btn variant="outlined" prepend-icon="mdi-upload" @click="exportSettings">
        {{ t('settings.exportSettingsBtn') }}
      </v-btn>
      <v-btn variant="outlined" prepend-icon="mdi-download" @click="importSettings">
        {{ t('settings.importSettingsBtn') }}
      </v-btn>
    </div>
  </v-card>

  <v-dialog v-model="showRestoreDialog" width="400">
    <v-card>
      <v-card-title>{{ t('settings.restoreLibrary') }}</v-card-title>
      <v-card-text>
        <p class="restore-warning">{{ t('importExport.replaceWarning') }}</p>
        <div class="mode-toggle" style="margin-top: 16px">
          <button
            :class="['mode-button', { active: restoreStrategy === 'replace' }]"
            @click="restoreStrategy = 'replace'"
          >
            <v-icon icon="mdi-refresh" size="16" />
            <span class="mode-label">{{ t('importExport.replaceLabel') }}</span>
          </button>
          <button
            :class="['mode-button', { active: restoreStrategy === 'merge' }]"
            @click="restoreStrategy = 'merge'"
          >
            <v-icon icon="mdi-merge" size="16" />
            <span class="mode-label">{{ t('importExport.mergeLabel') }}</span>
          </button>
        </div>
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn variant="text" @click="showRestoreDialog = false">{{ t('common.cancel') }}</v-btn>
        <v-btn variant="flat" color="primary" @click="confirmRestore">{{ t('common.confirm') }}</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style src="../../styles/settings-card.css"></style>
<style scoped>
.restore-warning {
  font-size: var(--text-sm);
  color: var(--v-text-secondary);
  line-height: 1.5;
}
</style>
