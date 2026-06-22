<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useSettingsStore } from '../../stores/settings'
import { themeMeta } from '../../plugins/vuetify'
import { useI18n, type LocaleSetting } from '../../composables/useI18n'

const { t, setLocale, getLocaleSetting } = useI18n()
const settingsStore = useSettingsStore()
const { colorName, mode } = storeToRefs(settingsStore)
const { setColorTheme, setMode } = settingsStore

const themes = Object.entries(themeMeta) as Array<[string, { label: string; color: string }]>

const themeModes = [
  { value: 'light' as const, labelKey: 'settings.light', icon: 'mdi-white-balance-sunny' },
  { value: 'system' as const, labelKey: 'settings.system', icon: 'mdi-desktop-mac' },
  { value: 'dark' as const, labelKey: 'settings.dark', icon: 'mdi-moon-waning-crescent' },
]

const languageOptions: { value: LocaleSetting; labelKey: string }[] = [
  { value: 'system', labelKey: 'settings.system' },
  { value: 'zh-CN', labelKey: 'settings.chinese' },
  { value: 'en', labelKey: 'settings.english' },
]

const currentLanguage = ref<LocaleSetting>('system')

onMounted(async () => {
  currentLanguage.value = await getLocaleSetting()
})

async function handleSetLocale(value: LocaleSetting) {
  currentLanguage.value = value
  await setLocale(value)
}
</script>

<template>
  <v-card class="settings-card" variant="flat" color="surface">
    <div class="card-header">
      <v-icon icon="mdi-translate" size="18" class="card-icon" />
      <span class="card-title">{{ t('settings.language') }}</span>
    </div>
    <div class="mode-toggle">
      <button
        v-for="lang in languageOptions"
        :key="lang.value"
        :class="['mode-button', { active: currentLanguage === lang.value }]"
        @click="handleSetLocale(lang.value)"
      >
        <span class="mode-label">{{ t(lang.labelKey) }}</span>
      </button>
    </div>
  </v-card>

  <v-card class="settings-card" variant="flat" color="surface">
    <div class="card-header">
      <v-icon icon="mdi-theme-light-dark" size="18" class="card-icon" />
      <span class="card-title">{{ t('settings.themeMode') }}</span>
    </div>

    <div class="mode-toggle">
      <button
        v-for="modeOption in themeModes"
        :key="modeOption.value"
        :class="['mode-button', { active: mode === modeOption.value }]"
        @click="setMode(modeOption.value)"
      >
        <v-icon :icon="modeOption.icon" size="16" />
        <span class="mode-label">{{ t(modeOption.labelKey) }}</span>
      </button>
    </div>
  </v-card>

  <v-card class="settings-card" variant="flat" color="surface">
    <div class="card-header">
      <v-icon icon="mdi-palette" size="18" class="card-icon" />
      <span class="card-title">{{ t('settings.themeColor') }}</span>
    </div>

    <div class="theme-grid">
      <button
        v-for="[key, meta] in themes"
        :key="key"
        :class="['theme-option', { active: colorName === key }]"
        @click="setColorTheme(key as any)"
      >
        <span class="theme-swatch" :style="{ background: meta.color }" />
        <span class="theme-label">{{ t(`settings.${key}`) }}</span>
      </button>
    </div>
  </v-card>
</template>

<style src="../../styles/settings-card.css"></style>
<style scoped>
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
</style>
