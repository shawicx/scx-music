import { ref, reactive } from 'vue'
import { invokeCommand } from '../utils/errorHandler'
import { useToast } from './useToast'
import { useI18n } from './useI18n'

/** 后端 LyricsCacheStats 的前端镜像（camelCase 由 serde rename 自动转换）。 */
export interface LyricsCacheStats {
  total: number
  sizeBytes: number
  orphanCount: number
  bySource: { embedded: number; lrclib: number; none: number }
}

/** 后端 PlayHistoryStats 的前端镜像。 */
export interface PlayHistoryStats {
  total: number
  oldestAt: string | null
  sizeBytes: number
}

interface ClearedResult {
  cleared: number
}

interface HistoryClearedResult {
  cleared: number
  scope: string
}

/**
 * 缓存与冗余数据清理。
 *
 * 持有统计状态（reactive），封装 5 个后端命令的调用，统一 toast 反馈。
 * 每个动作独立 loading 标记，避免并发误触。清空歌词缓存后不打断当前播放
 * （不调 useLyrics.refresh，下次切歌自然走三级回退重取）。
 */
export function useCache() {
  const { showSuccess, showError } = useToast()
  const { t } = useI18n()

  const lyricsStats = ref<LyricsCacheStats | null>(null)
  const historyStats = ref<PlayHistoryStats | null>(null)
  const loading = reactive({
    stats: false,
    lyrics: false,
    orphan: false,
    history: false,
  })

  /** 并行拉取两个统计命令。 */
  async function loadStats(): Promise<void> {
    loading.stats = true
    try {
      const [lyrics, history] = await Promise.all([
        invokeCommand<LyricsCacheStats>('get_lyrics_cache_stats'),
        invokeCommand<PlayHistoryStats>('get_play_history_stats'),
      ])
      lyricsStats.value = lyrics
      historyStats.value = history
    } catch (e) {
      showError(`${t('toast.statsLoadFailed')}: ${(e as Error).message}`)
    } finally {
      loading.stats = false
    }
  }

  /** 清空全部歌词缓存（含 source='none' 负缓存）。不打断当前播放。 */
  async function clearLyricsCache(): Promise<void> {
    loading.lyrics = true
    try {
      const result = await invokeCommand<ClearedResult>('clear_lyrics_cache')
      showSuccess(t('toast.lyricsCacheCleared', { count: result.cleared }))
      await loadStats()
    } catch (e) {
      showError(`${t('toast.cacheClearFailed')}: ${(e as Error).message}`)
    } finally {
      loading.lyrics = false
    }
  }

  /** 清理孤儿歌词（song_id 已不在 songs 表的残留）。 */
  async function clearOrphanLyrics(): Promise<void> {
    loading.orphan = true
    try {
      const result = await invokeCommand<ClearedResult>('clear_orphan_lyrics')
      showSuccess(t('toast.orphanLyricsCleared', { count: result.cleared }))
      await loadStats()
    } catch (e) {
      showError(`${t('toast.cacheClearFailed')}: ${(e as Error).message}`)
    } finally {
      loading.orphan = false
    }
  }

  /**
   * 将后端返回的 scope token 转为本地化文案用于 toast 展示。
   * - "all" → 全部清空文案
   * - "before_30d"/"before_90d" → 近 N 天文案
   * - "before_365d" → 近 1 年文案（与组件层 retentionOption 映射一致）
   */
  function scopeToLocalLabel(scope: string): string {
    if (scope === 'all') return t('settings.cacheManagement.playHistory.retentionOptions.all')
    const match = /^before_(\d+)d$/.exec(scope)
    if (match) {
      const days = Number(match[1])
      if (days === 365) return t('settings.cacheManagement.playHistory.retentionOptions.1y')
      if (days === 30) return t('settings.cacheManagement.playHistory.retentionOptions.30d')
      if (days === 90) return t('settings.cacheManagement.playHistory.retentionOptions.90d')
      return String(days)
    }
    return scope
  }

  /**
   * 按时间段清理播放历史。
   * @param beforeDays undefined=全部；正数=保留近 N 天
   */
  async function clearPlayHistory(beforeDays?: number): Promise<void> {
    loading.history = true
    try {
      const result = await invokeCommand<HistoryClearedResult>('clear_play_history', {
        beforeDays: beforeDays ?? null,
      })
      showSuccess(t('toast.historyCleared', { scope: scopeToLocalLabel(result.scope) }))
      await loadStats()
    } catch (e) {
      showError(`${t('toast.cacheClearFailed')}: ${(e as Error).message}`)
    } finally {
      loading.history = false
    }
  }

  return {
    lyricsStats,
    historyStats,
    loading,
    loadStats,
    clearLyricsCache,
    clearOrphanLyrics,
    clearPlayHistory,
  }
}
