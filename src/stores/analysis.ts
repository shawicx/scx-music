import { defineStore } from 'pinia'
import { useLibraryAnalysis } from '../composables/useLibraryAnalysis'

export const useAnalysisStore = defineStore('analysis', () => {
  return useLibraryAnalysis()
})
