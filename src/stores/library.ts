import { defineStore } from 'pinia'
import { useLibrary } from '../composables/useLibrary'

export const useLibraryStore = defineStore('library', () => {
  return useLibrary()
})
