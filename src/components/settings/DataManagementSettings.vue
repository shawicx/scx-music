<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useLibraryStore } from '../../stores/library'
import { useToast } from '../../composables/useToast'
import { useI18n } from '../../composables/useI18n'
import { useImportExport } from '../../composables/useImportExport'
import { useCache } from '../../composables/useCache'
import { formatFileSize } from '../../utils/format'

const { t } = useI18n()
const toast = useToast()
const libraryStore = useLibraryStore()
const { exportBackup, importBackup, exportSettings, importSettings } = useImportExport()

const {
  lyricsStats,
  historyStats,
  loading,
  loadStats,
  clearLyricsCache,
  clearOrphanLyrics,
  clearPlayHistory,
} = useCache()

type RetentionOption = '30d' | '90d' | '1y' | 'all'
const historyRetention = ref<RetentionOption>('30d')

// 统一确认对话框（复用于歌词缓存与播放历史清理）
const cacheConfirm = ref<{
  open: boolean
  bodyKey: string
  params: Record<string, unknown>
  action: (() => Promise<void>) | null
}>({
  open: false,
  bodyKey: '',
  params: {},
  action: null,
})

// v-select 选项：预解析 i18n 标签（避免 v-select 直接渲染 key 字符串）
const retentionOptions = computed(() => [
  { value: '30d' as RetentionOption, title: t('settings.cacheManagement.playHistory.retentionOptions.30d') },
  { value: '90d' as RetentionOption, title: t('settings.cacheManagement.playHistory.retentionOptions.90d') },
  { value: '1y' as RetentionOption, title: t('settings.cacheManagement.playHistory.retentionOptions.1y') },
  { value: 'all' as RetentionOption, title: t('settings.cacheManagement.playHistory.retentionOptions.all') },
])

onMounted(loadStats)

/** 将 retention 选项转为 beforeDays（all → undefined = 全部清空）。 */
function retentionToBeforeDays(opt: RetentionOption): number | undefined {
  if (opt === 'all') return undefined
  if (opt === '1y') return 365
  return Number(opt) // '30d'/'90d' → 30/90
}

function openLyricsConfirm() {
  const count = lyricsStats.value?.total ?? 0
  cacheConfirm.value = {
    open: true,
    bodyKey: 'settings.cacheManagement.confirm.lyricsBody',
    params: { count },
    action: clearLyricsCache,
  }
}

function openHistoryConfirm() {
  const beforeDays = retentionToBeforeDays(historyRetention.value)
  const scope =
    historyRetention.value === 'all'
      ? t('settings.cacheManagement.playHistory.retentionOptions.all')
      : historyRetention.value
  cacheConfirm.value = {
    open: true,
    bodyKey: 'settings.cacheManagement.confirm.historyBody',
    params: { scope },
    action: () => clearPlayHistory(beforeDays),
  }
}

async function confirmCacheAction() {
  const action = cacheConfirm.value.action
  cacheConfirm.value.open = false
  cacheConfirm.value.action = null
  if (action) await action()
}

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

  <v-card class="settings-card" variant="flat" color="surface">
    <div class="card-header">
      <v-icon icon="mdi-broom" size="18" class="card-icon" />
      <span class="card-title">{{ t('settings.cacheManagement.title') }}</span>
    </div>

    <div class="cache-item">
      <div class="cache-item-info">
        <span class="cache-item-title">{{ t('settings.cacheManagement.lyricsCache.title') }}</span>
        <span class="cache-item-desc">
          {{
            lyricsStats && lyricsStats.total > 0
              ? t('settings.cacheManagement.lyricsCache.desc', {
                  count: lyricsStats.total,
                  size: formatFileSize(lyricsStats.sizeBytes),
                })
              : t('settings.cacheManagement.lyricsCache.empty')
          }}
        </span>
      </div>
      <v-btn
        variant="outlined"
        :loading="loading.lyrics"
        :disabled="!lyricsStats || lyricsStats.total === 0"
        @click="openLyricsConfirm"
      >
        {{ t('settings.cacheManagement.lyricsCache.clearBtn') }}
      </v-btn>
    </div>

    <v-divider />

    <div class="cache-item">
      <div class="cache-item-info">
        <span class="cache-item-title">{{ t('settings.cacheManagement.orphanLyrics.title') }}</span>
        <span class="cache-item-desc">
          {{
            lyricsStats && lyricsStats.orphanCount > 0
              ? t('settings.cacheManagement.orphanLyrics.desc', { count: lyricsStats.orphanCount })
              : t('settings.cacheManagement.orphanLyrics.empty')
          }}
        </span>
      </div>
      <v-btn
        variant="outlined"
        :loading="loading.orphan"
        :disabled="!lyricsStats || lyricsStats.orphanCount === 0"
        @click="clearOrphanLyrics"
      >
        {{ t('settings.cacheManagement.orphanLyrics.clearBtn') }}
      </v-btn>
    </div>

    <v-divider />

    <div class="cache-item">
      <div class="cache-item-info">
        <span class="cache-item-title">{{ t('settings.cacheManagement.playHistory.title') }}</span>
        <span class="cache-item-desc">
          {{
            historyStats && historyStats.total > 0
              ? historyStats.oldestAt
                ? t('settings.cacheManagement.playHistory.desc', {
                    count: historyStats.total,
                    date: historyStats.oldestAt.slice(0, 10),
                  })
                : t('settings.cacheManagement.playHistory.descNoDate', { count: historyStats.total })
              : t('settings.cacheManagement.playHistory.empty')
          }}
        </span>
      </div>
      <div class="cache-item-actions">
        <v-select
          v-model="historyRetention"
          :items="retentionOptions"
          item-value="value"
          item-title="title"
          density="compact"
          variant="outlined"
          hide-details
          class="retention-select"
        />
        <v-btn variant="outlined" :loading="loading.history" @click="openHistoryConfirm">
          {{ t('settings.cacheManagement.playHistory.clearBtn') }}
        </v-btn>
      </div>
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

  <v-dialog v-model="cacheConfirm.open" width="420">
    <v-card>
      <v-card-title>{{ t('settings.cacheManagement.confirm.title') }}</v-card-title>
      <v-card-text>
        <p class="restore-warning">{{ t(cacheConfirm.bodyKey, cacheConfirm.params) }}</p>
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn variant="text" @click="cacheConfirm.open = false">{{ t('common.cancel') }}</v-btn>
        <v-btn variant="flat" color="primary" @click="confirmCacheAction">
          {{ t('settings.cacheManagement.confirm.confirmBtn') }}
        </v-btn>
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
.cache-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 0;
}
.cache-item-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.cache-item-title {
  font-size: var(--text-sm);
  font-weight: 500;
}
.cache-item-desc {
  font-size: var(--text-xs);
  color: var(--v-text-secondary);
  line-height: 1.4;
}
.cache-item-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
.retention-select {
  max-width: 140px;
}
</style>
