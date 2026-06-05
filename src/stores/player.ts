import { defineStore } from 'pinia'
import { usePlayer } from '../composables/usePlayer'

export const usePlayerStore = defineStore('player', () => {
  return usePlayer()
})
