import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { usePlayerStore } from '../stores/player'

const MODE_CONFIG = {
  sequential: { icon: 'mdi-arrow-right', label: '顺序播放' },
  repeat_all: { icon: 'mdi-repeat', label: '列表循环' },
  repeat_one: { icon: 'mdi-repeat-once', label: '单曲循环' },
  shuffle: { icon: 'mdi-shuffle', label: '随机播放' },
} as const

export type PlaybackModeKey = keyof typeof MODE_CONFIG

export function usePlaybackMode() {
  const playerStore = usePlayerStore()
  const { playbackMode } = storeToRefs(playerStore)

  const modeConfig = computed(() => {
    const key = playbackMode.value as PlaybackModeKey
    return MODE_CONFIG[key] ?? MODE_CONFIG.sequential
  })

  const modeIcon = computed(() => modeConfig.value.icon)
  const modeLabel = computed(() => modeConfig.value.label)

  const isModeActive = computed(() => playbackMode.value !== 'sequential')

  function cycleMode() {
    const modes: readonly PlaybackModeKey[] = ['sequential', 'repeat_all', 'repeat_one', 'shuffle']
    const currentKey = playbackMode.value as PlaybackModeKey
    const idx = modes.indexOf(currentKey)
    const nextMode = modes[(idx + 1) % modes.length]
    playerStore.setMode(nextMode)
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
