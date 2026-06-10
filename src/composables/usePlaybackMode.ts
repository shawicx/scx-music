import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { usePlayerStore } from '../stores/player'
import { useI18n } from './useI18n'

const MODE_CONFIG = {
  sequential: { icon: 'mdi-arrow-right', labelKey: 'playbackMode.sequential' },
  repeat_all: { icon: 'mdi-repeat', labelKey: 'playbackMode.repeatAll' },
  repeat_one: { icon: 'mdi-repeat-once', labelKey: 'playbackMode.repeatOne' },
  shuffle: { icon: 'mdi-shuffle', labelKey: 'playbackMode.shuffle' },
} as const

export type PlaybackModeKey = keyof typeof MODE_CONFIG

export function usePlaybackMode() {
  const playerStore = usePlayerStore()
  const { playbackMode } = storeToRefs(playerStore)
  const { t } = useI18n()

  const modeConfig = computed(() => {
    const key = playbackMode.value as PlaybackModeKey
    return MODE_CONFIG[key] ?? MODE_CONFIG.sequential
  })

  const modeIcon = computed(() => modeConfig.value.icon)
  const modeLabel = computed(() => t(modeConfig.value.labelKey))

  const isModeActive = computed(() => playbackMode.value !== 'sequential')

  async function cycleMode() {
    const modes: readonly PlaybackModeKey[] = ['sequential', 'repeat_all', 'repeat_one', 'shuffle']
    const currentKey = playbackMode.value as PlaybackModeKey
    const idx = modes.indexOf(currentKey)
    const nextMode = modes[(idx + 1) % modes.length]
    await playerStore.setMode(nextMode)
    await playerStore.regenerateQueue()
  }

  return {
    playbackMode,
    modeConfig,
    modeIcon,
    modeLabel,
    isModeActive,
    cycleMode,
  }
}
