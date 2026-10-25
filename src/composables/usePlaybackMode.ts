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
    const nextMode = modes[(idxOf(currentKey) + 1) % modes.length]
    await setPlaybackMode(nextMode)
  }

  function idxOf(key: PlaybackModeKey): number {
    const modes: readonly PlaybackModeKey[] = ['sequential', 'repeat_all', 'repeat_one', 'shuffle']
    const i = modes.indexOf(key)
    return i < 0 ? 0 : i
  }

  /** 设定播放模式并重建队列（cycleMode 的原子版本，供双按钮 UI 使用）。 */
  async function setPlaybackMode(mode: PlaybackModeKey) {
    await playerStore.setMode(mode)
    await playerStore.regenerateQueue()
  }

  /**
   * 双按钮 UI：shuffle 独立开关；
   * repeat 在 关 → 列表循环 → 单曲循环 间循环（shuffle 开启时点 repeat 切到列表循环）。
   */
  async function toggleShuffle() {
    await setPlaybackMode(playbackMode.value === 'shuffle' ? 'sequential' : 'shuffle')
  }

  async function cycleRepeat() {
    const cur = playbackMode.value
    if (cur === 'repeat_all') await setPlaybackMode('repeat_one')
    else await setPlaybackMode('repeat_all')
  }

  return {
    playbackMode,
    modeConfig,
    modeIcon,
    modeLabel,
    isModeActive,
    cycleMode,
    setPlaybackMode,
    toggleShuffle,
    cycleRepeat,
  }
}
