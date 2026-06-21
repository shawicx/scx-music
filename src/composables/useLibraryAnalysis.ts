import { ref, computed } from 'vue'
import { invokeCommand } from '../utils/errorHandler'
import { formatHoursMinutes as formatDuration, formatFileSize } from '../utils/format'
import type { LibraryStats } from '../types'

const stats = ref<LibraryStats | null>(null)
const loading = ref(false)

export function useLibraryAnalysis() {
  async function loadStats() {
    loading.value = true
    try {
      stats.value = await invokeCommand<LibraryStats>('get_library_stats')
    } finally {
      loading.value = false
    }
  }

  const formattedTotalSize = computed(() =>
    stats.value ? formatFileSize(stats.value.totalFileSize) : '0 B'
  )

  const formattedTotalDuration = computed(() =>
    stats.value ? formatDuration(stats.value.totalDurationSecs) : '0m'
  )

  return {
    stats,
    loading,
    loadStats,
    formattedTotalSize,
    formattedTotalDuration,
    formatFileSize,
    formatDuration,
  }
}
