import { ref, computed } from 'vue'
import { invokeCommand } from '../utils/errorHandler'
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

function formatDuration(secs: number): string {
  const hours = Math.floor(secs / 3600)
  const minutes = Math.floor((secs % 3600) / 60)
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
}

export function useListeningStats() {
  async function loadData(range?: ListeningRange) {
    if (range) currentRange.value = range
    loading.value = true
    try {
      const r = currentRange.value
      const [ov, songs, artists, genres, trendData, heatmapData] = await Promise.all([
        invokeCommand<ListeningOverview>('stats_listening_overview', { range: r }),
        invokeCommand<TopSong[]>('stats_top_songs', { range: r, limit: 10 }),
        invokeCommand<TopArtist[]>('stats_top_artists', { range: r, limit: 10 }),
        invokeCommand<GenreDuration[]>('stats_genre_distribution', { range: r }),
        invokeCommand<DayDuration[]>('stats_trend', { range: r }),
        invokeCommand<DayDuration[]>('stats_heatmap'),
      ])
      overview.value = ov
      topSongs.value = songs
      topArtists.value = artists
      genreDistribution.value = genres
      trend.value = trendData
      heatmap.value = heatmapData
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
