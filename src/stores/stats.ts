import { defineStore } from 'pinia'
import { useListeningStats } from '../composables/useListeningStats'

export const useStatsStore = defineStore('stats', () => {
  return useListeningStats()
})
