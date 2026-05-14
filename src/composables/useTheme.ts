import { ref } from 'vue'
import { watch } from 'vue'
import { useTheme as useVuetifyTheme } from 'vuetify'
import { invoke } from '@tauri-apps/api/core'
import type { ThemeName } from '../plugins/vuetify'

const themeName = ref<ThemeName>('teal')

async function loadThemeFromDb() {
  const value = await invoke<string | null>('get_setting', { key: 'theme' })
  if (value) {
    themeName.value = value as ThemeName
  }
}

function saveThemeToDb(name: ThemeName) {
  invoke('set_setting', { key: 'theme', value: name }).catch(console.error)
}

export function useTheme() {
  const vuetifyTheme = useVuetifyTheme()

  function setTheme(name: ThemeName) {
    themeName.value = name
    saveThemeToDb(name)
  }

  watch(
    themeName,
    (name) => {
      vuetifyTheme.global.name.value = name
    },
    { immediate: true },
  )

  return { themeName, setTheme, loadThemeFromDb }
}
