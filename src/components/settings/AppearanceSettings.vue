<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useSettingsStore } from '../../stores/settings'
import { themeMeta } from '../../plugins/vuetify'
import { useI18n, type LocaleSetting } from '../../composables/useI18n'

const { t, setLocale, getLocaleSetting } = useI18n()
const settingsStore = useSettingsStore()
const { colorName, mode, dynamicColor } = storeToRefs(settingsStore)
const { setColorTheme, setMode, setDynamicColor } = settingsStore

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
  <v-card class="settings-card" variant="flat">
    <div class="card-header">
      <v-icon icon="mdi-translate" size="16" class="card-icon" />
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

  <v-card class="settings-card" variant="flat">
    <div class="card-header">
      <v-icon icon="mdi-theme-light-dark" size="16" class="card-icon" />
      <span class="card-title">{{ t('settings.themeMode') }}</span>
    </div>

    <div class="mode-toggle">
      <button
        v-for="modeOption in themeModes"
        :key="modeOption.value"
        :class="['mode-button', { active: mode === modeOption.value }]"
        @click="setMode(modeOption.value)"
      >
        <v-icon :icon="modeOption.icon" size="14" />
        <span class="mode-label">{{ t(modeOption.labelKey) }}</span>
      </button>
    </div>
  </v-card>

  <v-card class="settings-card" variant="flat">
    <div class="card-header">
      <v-icon icon="mdi-palette" size="16" class="card-icon" />
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
    <p class="theme-hint">{{ t('settings.dynamicColorDesc') }}</p>
    <div class="switch-row">
      <div>
        <div class="switch-label">{{ t('settings.dynamicColor') }}</div>
        <div class="switch-desc">{{ t('settings.dynamicColorDesc') }}</div>
      </div>
      <v-switch
        :model-value="dynamicColor"
        color="primary"
        hide-details
        density="compact"
        :aria-label="t('settings.dynamicColor')"
        @update:model-value="(v: boolean | null) => v !== null && setDynamicColor(v)"
      />
    </div>
  </v-card>
</template>

<style src="../../styles/settings-card.css"></style>
<style scoped>
.theme-grid {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 10px;
  padding: 6px 0 2px;
}

@media (max-width: 720px) {
  .theme-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

.theme-option {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 7px;
  padding: 10px 4px 8px;
  border-radius: 10px;
  border: 1.5px solid transparent;
  background: transparent;
  cursor: pointer;
  transition: border-color 0.2s, background 0.2s;
}

.theme-option:hover {
  background: var(--surface-2);
}

.theme-option.active {
  border-color: var(--accent);
  background: rgb(var(--accent-rgb) / 0.08);
}

.theme-swatch {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  display: block;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.25);
}

.theme-option.active .theme-swatch {
  outline: 1.5px solid var(--text-1);
  outline-offset: 3px;
}

.theme-label {
  font-size: 11px;
  color: var(--text-2);
}

.theme-hint {
  font-size: 11px;
  color: var(--text-3);
  margin: 8px 0 2px;
}
</style>
