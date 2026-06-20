<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useGlobalShortcuts } from '../../composables/useGlobalShortcuts'
import { useToast } from '../../composables/useToast'
import { useI18n } from '../../composables/useI18n'
import KeyCaptureField from './KeyCaptureField.vue'

interface ShortcutDefault {
  id: string
  combo: string
  enabled: boolean
}

const { t } = useI18n()
const { showToast } = useToast()
const { rebind, setEnabled, isComboRegistered, resetAll } = useGlobalShortcuts()

const defaults = ref<ShortcutDefault[]>([])
const storedCombos = ref<Record<string, string>>({})
const storedEnabled = ref<Record<string, boolean>>({})

const mediaActions = computed(() => defaults.value.filter(d => d.id.startsWith('media.')))
const appActions = computed(() => defaults.value.filter(d => d.id.startsWith('app.')))

function comboOf(actionId: string): string {
  return storedCombos.value[`shortcut.${actionId}`] ?? defaults.value.find(d => d.id === actionId)?.combo ?? ''
}
function enabledOf(actionId: string): boolean {
  const v = storedEnabled.value[`shortcut.${actionId}.enabled`]
  if (v === undefined) return defaults.value.find(d => d.id === actionId)?.enabled ?? false
  return v
}
function actionLabel(actionId: string): string {
  return t(`settings.shortcuts.action.${actionId}`)
}

async function loadAll() {
  defaults.value = await invoke<ShortcutDefault[]>('shortcuts_list_defaults')
  const stored = await invoke<Record<string, string>>('get_all_settings')
  storedCombos.value = stored
  const enabledMap: Record<string, boolean> = {}
  for (const k of Object.keys(stored)) {
    if (k.endsWith('.enabled')) {
      enabledMap[k] = stored[k] === 'true'
    }
  }
  storedEnabled.value = enabledMap
}

async function onRebind(actionId: string, newCombo: string) {
  // In-app conflict detection
  if (newCombo) {
    for (const def of defaults.value) {
      if (def.id === actionId) continue
      if (comboOf(def.id) === newCombo) {
        showToast(t('settings.shortcuts.capture.conflictApp', { action: actionLabel(def.id) }))
        return
      }
    }
    // System-level conflict check (warning only, doesn't block)
    const isReg = await isComboRegistered(newCombo)
    if (isReg) {
      console.warn(`[shortcuts] combo ${newCombo} may be system-registered`)
    }
  }

  const result = await rebind(actionId, newCombo)
  if (result.ok) {
    if (newCombo) {
      storedCombos.value = { ...storedCombos.value, [`shortcut.${actionId}`]: newCombo }
    } else {
      const next = { ...storedCombos.value }
      delete next[`shortcut.${actionId}`]
      storedCombos.value = next
    }
    showToast(t('settings.shortcuts.saved'))
  } else {
    showToast(t('settings.shortcuts.capture.registerFailed', { error: result.error }))
  }
}

async function onToggleEnabled(actionId: string, enabled: boolean) {
  try {
    await setEnabled(actionId, enabled)
    storedEnabled.value = {
      ...storedEnabled.value,
      [`shortcut.${actionId}.enabled`]: enabled,
    }
  } catch (e: any) {
    showToast(String(e))
  }
}

async function onResetAll() {
  await resetAll()
  await loadAll()
  showToast(t('settings.shortcuts.resetDone'))
}

onMounted(loadAll)
</script>

<template>
  <v-card class="settings-card" variant="flat" color="surface">
    <div class="card-header">
      <v-icon icon="mdi-keyboard-settings" size="18" class="card-icon" />
      <span class="card-title">{{ t('settings.shortcuts.title') }}</span>
    </div>

    <v-list density="compact" bg-color="transparent" class="shortcuts-list">
      <v-list-subheader>{{ t('settings.shortcuts.group.media') }}</v-list-subheader>
      <KeyCaptureField
        v-for="action in mediaActions"
        :key="action.id"
        :action-id="action.id"
        :action-label="actionLabel(action.id)"
        :combo="comboOf(action.id)"
        :enabled="enabledOf(action.id)"
        @rebind="onRebind"
        @toggle-enabled="onToggleEnabled"
      />

      <v-divider class="my-2" />

      <v-list-subheader>{{ t('settings.shortcuts.group.app') }}</v-list-subheader>
      <KeyCaptureField
        v-for="action in appActions"
        :key="action.id"
        :action-id="action.id"
        :action-label="actionLabel(action.id)"
        :combo="comboOf(action.id)"
        :enabled="enabledOf(action.id)"
        @rebind="onRebind"
        @toggle-enabled="onToggleEnabled"
      />
    </v-list>

    <div class="shortcuts-hint">{{ t('settings.shortcuts.subtitle') }}</div>

    <div class="action-row">
      <v-btn color="error" variant="text" prepend-icon="mdi-restore" @click="onResetAll">
        {{ t('settings.shortcuts.resetAll') }}
      </v-btn>
    </div>
  </v-card>
</template>

<style scoped>
/* 与 SettingsView 中其他 settings-card 保持视觉一致 */
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

.shortcuts-list {
  padding: 0;
}

.shortcuts-hint {
  margin-top: 16px;
  font-size: var(--text-sm);
  color: var(--v-text-secondary);
  line-height: 1.5;
}

.action-row {
  display: flex;
  gap: 12px;
  margin-top: 12px;
}
</style>
