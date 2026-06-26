import { ref, computed } from 'vue'
import { invokeCommand } from '../utils/errorHandler'
import { formatHoursMinutes as formatDuration } from '../utils/format'
import type {
  ListeningOverview,
  TopSong,
  TopArtist,
  GenreDuration,
  DayDuration,
  ListeningRange,
} from '../types'

const overview = ref<ListeningOverview | null>(null)
const topSongs = ref<TopSong[]>([])
const topArtists = ref<TopArtist[]>([])
const genreDistribution = ref<GenreDuration[]>([])
const trend = ref<DayDuration[]>([])
const heatmap = ref<DayDuration[]>([])
const loading = ref(false)
const currentRange = ref<ListeningRange>('7d')

export function useListeningStats() {
  async function loadData(range?: ListeningRange) {
    if (range) currentRange.value = range
    loading.value = true
    try {
      // 单次 IPC 获取全部仪表盘数据，替代原 Promise.all 6 次扇出
      const dash = await invokeCommand<{
        overview: ListeningOverview
        topSongs: TopSong[]
        topArtists: TopArtist[]
        genreDistribution: GenreDuration[]
        trend: DayDuration[]
        heatmap: DayDuration[]
      }>('stats_dashboard', { range: currentRange.value, topLimit: 10 })
      overview.value = dash.overview
      topSongs.value = dash.topSongs
      topArtists.value = dash.topArtists
      genreDistribution.value = dash.genreDistribution
      trend.value = dash.trend
      heatmap.value = dash.heatmap
    } finally {
      loading.value = false
    }
  }

  const formattedTotalDuration = computed(() =>
    overview.value ? formatDuration(overview.value.totalDurationSecs) : '0m',
  )

  return {
    overview,
    topSongs,
    topArtists,
    genreDistribution,
    trend,
    heatmap,
    loading,
    currentRange,
    loadData,
    formattedTotalDuration,
    formatDuration,
  }
}
