import { defineStore } from 'pinia'
import { useAudioEffects } from '../composables/useAudioEffects'

export const useAudioEffectsStore = defineStore('audioEffects', () => useAudioEffects())
