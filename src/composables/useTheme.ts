import { ref, watch, computed } from 'vue'
import { useTheme as useVuetifyTheme } from 'vuetify'
import { invoke } from '@tauri-apps/api/core'
import { usePreferredDark } from '@vueuse/core'
import type { ThemeColor, ThemeMode } from '../plugins/vuetify'

// Color theme management
export function useColorTheme() {
  const colorName = ref<ThemeColor>('teal')

  async function loadColorFromDb() {
    const value = await invoke<string | null>('get_setting', { key: 'theme_color' })
    if (value) {
      colorName.value = value as ThemeColor
    } else {
      // Migrate old theme setting
      const oldTheme = await invoke<string | null>('get_setting', { key: 'theme' })
      if (oldTheme) {
        colorName.value = oldTheme as ThemeColor
        // Save to new key
        await invoke('set_setting', { key: 'theme_color', value: oldTheme })
      }
    }
  }

  function setColorTheme(name: ThemeColor) {
    colorName.value = name
    invoke('set_setting', { key: 'theme_color', value: name }).catch(console.error)
  }

  return { colorName, setColorTheme, loadColorFromDb }
}

// Mode management (light/dark/system)
export function useThemeMode() {
  const mode = ref<ThemeMode>('system')
  const preferredDark = usePreferredDark()

  async function loadModeFromDb() {
    const value = await invoke<string | null>('get_setting', { key: 'theme_mode' })
    if (value) {
      mode.value = value as ThemeMode
    } else {
      // First time user - check if they have old theme setting
      const oldTheme = await invoke<string | null>('get_setting', { key: 'theme' })
      if (oldTheme) {
        // Existing users default to dark mode
        mode.value = 'dark'
        await invoke('set_setting', { key: 'theme_mode', value: 'dark' })
      } else {
        // New users default to system mode
        mode.value = 'system'
      }
    }
  }

  function setMode(newMode: ThemeMode) {
    mode.value = newMode
    invoke('set_setting', { key: 'theme_mode', value: newMode }).catch(console.error)
  }

  const isDark = computed(() => {
    if (mode.value === 'system') {
      return preferredDark.value
    }
    return mode.value === 'dark'
  })

  return { mode, setMode, loadModeFromDb, isDark }
}

// Main theme orchestration
export function useTheme() {
  const vuetifyTheme = useVuetifyTheme()
  const { colorName, setColorTheme, loadColorFromDb } = useColorTheme()
  const { mode, setMode, loadModeFromDb, isDark } = useThemeMode()

  // Watch for changes and update Vuetify theme
  watch([colorName, isDark], ([color, dark]) => {
    const themeName = `${color}-${dark ? 'dark' : 'light'}`
    vuetifyTheme.global.name.value = themeName
  }, { immediate: true })

  async function loadThemeFromDb() {
    await Promise.all([loadColorFromDb(), loadModeFromDb()])
  }

  return {
    colorName,
    mode,
    isDark,
    setColorTheme,
    setMode,
    loadThemeFromDb,
  }
}
