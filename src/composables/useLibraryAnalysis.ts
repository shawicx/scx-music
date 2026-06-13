import { ref, computed } from 'vue'
import { invokeCommand } from '../utils/errorHandler'
import type { LibraryStats } from '../types'

const stats = ref<LibraryStats | null>(null)
const loading = ref(false)

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`
}

function formatDuration(secs: number): string {
  const hours = Math.floor(secs / 3600)
  const minutes = Math.floor((secs % 3600) / 60)
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
}

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
