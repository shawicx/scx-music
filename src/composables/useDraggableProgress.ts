import { computed, ref, type Ref } from 'vue'

/**
 * @description 进度条拖拽逻辑（PlayerBar 和 MiniPlayer 共用）。
 *
 * - `progressModel` 的 setter 在每次值变时**实时 seek**（非仅拖拽结束 seek）
 * - `isDragging` 控制显示：拖拽中显示 localProgress 对应的时间，否则显示真实 progress
 * - `@start="isDragging = true"` / `@end="isDragging = false"` 由模板直接赋值
 *
 * @param progress 播放进度（秒，来自 player store）
 * @param duration 总时长（秒）
 * @param seek seek 回调（接收秒数）
 */
export function useDraggableProgress(
  progress: Ref<number>,
  duration: Ref<number>,
  seek: (value: number) => void,
) {
  const isDragging = ref(false)
  const localProgress = ref(0)

  const progressModel = computed({
    get: () => {
      if (isDragging.value) {
        return isNaN(localProgress.value) ? 0 : localProgress.value
      }
      if (duration.value > 0) {
        const result = (progress.value / duration.value) * 100
        return isNaN(result) ? 0 : result
      }
      return 0
    },
    set: (val: number) => {
      localProgress.value = val
      if (duration.value > 0 && !isNaN(val)) {
        seek((val / 100) * duration.value)
      }
    },
  })

  const displayProgress = computed(() => {
    if (isDragging.value && duration.value > 0) {
      const result = (localProgress.value / 100) * duration.value
      return isNaN(result) ? progress.value : result
    }
    return progress.value
  })

  return { progressModel, displayProgress, isDragging }
}
