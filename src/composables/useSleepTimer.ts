import { ref } from 'vue'
import { usePlayer } from './usePlayer'
import { useToast } from './useToast'
import { useI18n } from './useI18n'

/**
 * 睡眠定时器 — 单例 composable（模块级状态，多组件共享同一计时器实例）。
 *
 * 行为：
 * - 启动后每秒更新 remainingSecs；进入最后 FADE_DURATION 秒时音量线性渐弱到 0。
 * - 到点调用 player.stop() 完全停止播放，并恢复原音量。
 * - 取消时立即恢复原音量，不停止播放。
 *
 * 不持久化：定时器是一次性操作，应用重启后清空（符合睡眠场景）。
 */

/** 最后 N 秒进入渐弱阶段。 */
const FADE_DURATION = 30

/** 渐弱 / 倒计时轮询间隔（毫秒）。 */
const TICK_INTERVAL = 1000

// ── 模块级单例状态 ──────────────────────────────────────────────────────────
const isActive = ref(false)
const remainingSecs = ref(0)
const totalMinutes = ref<number | null>(null)

let tickTimer: ReturnType<typeof setInterval> | null = null
let endTime = 0
/** 启动定时器时的原始音量快照，用于 cancel / 到点后恢复。 */
let originalVolume: number | null = null

/**
 * 把秒数格式化为 M:SS（tooltip 倒计时显示用）。
 */
function formatRemaining(secs: number): string {
  const m = Math.floor(secs / 60)
  const s = secs % 60
  return `${m}:${s.toString().padStart(2, '0')}`
}

export function useSleepTimer() {
  const player = usePlayer()
  const { showInfo, showSuccess } = useToast()
  const { t } = useI18n()

  /** 清理 interval，不触碰音量与播放状态。 */
  function clearTick() {
    if (tickTimer !== null) {
      clearInterval(tickTimer)
      tickTimer = null
    }
  }

  /** 恢复 originalVolume 到播放器（best-effort，失败静默）。 */
  function restoreVolume() {
    if (originalVolume === null) return
    try {
      player.setVolume(originalVolume).catch((e) => {
        console.warn('[sleepTimer] restore volume failed:', e)
      })
    } catch (e) {
      console.warn('[sleepTimer] restore volume threw:', e)
    }
  }

  /** 重置响应式状态与临时变量（不含 interval 与音量，由调用方处理）。 */
  function resetState() {
    isActive.value = false
    remainingSecs.value = 0
    totalMinutes.value = null
    originalVolume = null
  }

  /**
   * 停止定时器并完全停止播放（到点触发）。
   * 内部使用，不对外暴露。
   */
  async function finish() {
    clearTick()
    try {
      await player.stop()
    } catch (e) {
      console.warn('[sleepTimer] stop playback failed:', e)
    }
    restoreVolume()
    resetState()
    showSuccess(t('toast.sleepTimerStopped'))
  }

  /**
   * 启动睡眠定时器。
   *
   * 若已有定时器在运行，先取消（恢复音量）再重新启动。
   *
   * @param minutes 倒计时分钟数（正数）
   */
  function start(minutes: number) {
    if (minutes <= 0) return

    // 覆盖旧定时器：先清理 interval 并恢复音量，但不 stop 播放、不 toast
    if (isActive.value) {
      clearTick()
      restoreVolume()
    }

    // 快照当前音量（渐弱基准 + 恢复源）
    originalVolume = player.volume.value
    totalMinutes.value = minutes
    endTime = Date.now() + minutes * 60_000
    remainingSecs.value = Math.round((endTime - Date.now()) / 1000)
    isActive.value = true

    tickTimer = setInterval(() => {
      const remaining = Math.max(0, Math.round((endTime - Date.now()) / 1000))
      remainingSecs.value = remaining

      if (remaining <= 0) {
        finish()
        return
      }

      // 渐弱阶段：线性降低音量
      if (remaining <= FADE_DURATION && originalVolume !== null) {
        const ratio = remaining / FADE_DURATION // 1.0 → 0.0
        const targetVol = Math.max(0, Math.min(1, originalVolume * ratio))
        player.setVolume(targetVol).catch((e) => {
          console.warn('[sleepTimer] fade setVolume failed:', e)
        })
      }
    }, TICK_INTERVAL)

    showSuccess(t('toast.sleepTimerSet', { minutes }))
  }

  /**
   * 取消定时器：清 interval + 恢复音量 + 重置状态。不停止播放。
   */
  function cancel() {
    clearTick()
    restoreVolume()
    resetState()
    showInfo(t('toast.sleepTimerCanceled'))
  }

  return {
    isActive,
    remainingSecs,
    totalMinutes,
    formatRemaining,
    start,
    cancel,
  }
}
