import { ref, computed, watch } from 'vue'
import { invokeCommand } from '../utils/errorHandler'
import { formatHoursMinutes as formatDuration } from '../utils/format'
import type {
  ListeningOverview,
  HourDuration,
  PeriodKind,
  PeriodState,
} from '../types'

const periodState = ref<PeriodState>({ kind: 'month', offset: 0 })
const overview = ref<ListeningOverview | null>(null)
const hourlyDistribution = ref<HourDuration[]>([])
const loading = ref(false)

function formatDateTimeUTC(d: Date): string {
  return d.toISOString().slice(0, 19).replace('T', ' ')
}

function fillMissingHours(data: HourDuration[]): HourDuration[] {
  const map = new Map(data.map(h => [h.hour, h.durationSecs]))
  return Array.from({ length: 24 }, (_, hour) => ({
    hour,
    durationSecs: map.get(hour) ?? 0,
  }))
}

const periodRange = computed(() => {
  const { kind, offset } = periodState.value
  const now = new Date()
  let startDate: Date
  let endDate: Date

  if (kind === 'week') {
    const day = now.getDay()
    startDate = new Date(now)
    startDate.setDate(now.getDate() - (day === 0 ? 6 : day - 1) + offset * 7)
    startDate.setHours(0, 0, 0, 0)
    endDate = new Date(startDate)
    endDate.setDate(startDate.getDate() + 7)
  } else if (kind === 'month') {
    startDate = new Date(now.getFullYear(), now.getMonth() + offset, 1)
    endDate = new Date(now.getFullYear(), now.getMonth() + offset + 1, 1)
  } else {
    const year = now.getFullYear() + offset
    startDate = new Date(year, 0, 1)
    endDate = new Date(year + 1, 0, 1)
  }

  return {
    startDate,
    endDate,
    start: formatDateTimeUTC(startDate),
    end: formatDateTimeUTC(endDate),
  }
})

const periodLabel = computed(() => {
  const { kind, offset } = periodState.value
  const now = new Date()

  if (kind === 'week') {
    const { startDate } = periodRange.value
    const weekEnd = new Date(startDate)
    weekEnd.setDate(weekEnd.getDate() + 6)
    const fmt = (d: Date) =>
      `${String(d.getMonth() + 1).padStart(2, '0')}.${String(d.getDate()).padStart(2, '0')}`
    return `${fmt(startDate)} - ${fmt(weekEnd)}`
  }

  if (kind === 'month') {
    const target = new Date(now.getFullYear(), now.getMonth() + offset, 1)
    return `${target.getFullYear()}.${String(target.getMonth() + 1).padStart(2, '0')}`
  }

  return `${now.getFullYear() + offset}`
})

const isInProgress = computed(
  () => periodState.value.offset === 0 && new Date() < periodRange.value.endDate,
)

const canGoForward = computed(() => periodState.value.offset < 0)

async function loadReport() {
  loading.value = true
  try {
    const { start, end } = periodRange.value
    const [ov, hourly] = await Promise.all([
      invokeCommand<ListeningOverview>('stats_listening_overview', {
        start,
        end,
      }),
      invokeCommand<HourDuration[]>('stats_hourly_distribution', {
        start,
        end,
      }),
    ])
    overview.value = ov
    hourlyDistribution.value = fillMissingHours(hourly)
  } finally {
    loading.value = false
  }
}

function shiftPeriod(delta: number) {
  const newOffset = periodState.value.offset + delta
  if (newOffset > 0) return
  periodState.value = { ...periodState.value, offset: newOffset }
}

function setKind(kind: PeriodKind) {
  periodState.value = { kind, offset: 0 }
}

const TIME_SLOTS = [
  { key: 'dawn' as const, range: [0, 5] as const, color: '#c5cae9' },
  { key: 'morning' as const, range: [6, 11] as const, color: '#90caf9' },
  { key: 'afternoon' as const, range: [12, 17] as const, color: '#66bb6a' },
  { key: 'night' as const, range: [18, 23] as const, color: '#ff9800' },
]

const dominantSlot = computed(() => {
  const sums = TIME_SLOTS.map(slot => ({
    ...slot,
    total: hourlyDistribution.value
      .filter(h => h.hour >= slot.range[0] && h.hour <= slot.range[1])
      .reduce((sum, h) => sum + h.durationSecs, 0),
  }))
  return sums.sort((a, b) => b.total - a.total)[0]
})

const peakHourRange = computed(() => {
  const slot = dominantSlot.value
  if (slot.total === 0) return ''
  return `${slot.range[0]}-${slot.range[1]}`
})

const formattedTotalDuration = computed(() =>
  overview.value ? formatDuration(overview.value.totalDurationSecs) : '0m',
)

watch(periodState, loadReport, { deep: true })

export function useListeningReport() {
  return {
    periodState,
    overview,
    hourlyDistribution,
    loading,
    periodLabel,
    isInProgress,
    canGoForward,
    periodRange,
    dominantSlot,
    peakHourRange,
    formattedTotalDuration,
    formatDuration,
    loadReport,
    shiftPeriod,
    setKind,
    TIME_SLOTS,
  }
}
