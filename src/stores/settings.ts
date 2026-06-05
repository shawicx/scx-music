import { defineStore } from 'pinia'
import { useTheme } from '../composables/useTheme'

export const useSettingsStore = defineStore('settings', () => {
  return useTheme()
})
