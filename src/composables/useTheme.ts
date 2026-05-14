import { watch } from 'vue'
import { useStorage } from '@vueuse/core'
import { useTheme as useVuetifyTheme } from 'vuetify'
import type { ThemeName } from '../plugins/vuetify'

const storageKey = 'scx-music-theme'

const themeName = useStorage<ThemeName>(storageKey, 'teal')

export function useTheme() {
  const vuetifyTheme = useVuetifyTheme()

  function setTheme(name: ThemeName) {
    themeName.value = name
  }

  watch(
    themeName,
    (name) => {
      vuetifyTheme.global.name.value = name
    },
    { immediate: true },
  )

  return { themeName, setTheme }
}
