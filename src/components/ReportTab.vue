<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { use } from 'echarts/core'
import { BarChart } from 'echarts/charts'
import { GridComponent, TooltipComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import VChart from 'vue-echarts'
import { useListeningReport } from '../composables/useListeningReport'
import { useI18n } from '../composables/useI18n'

use([BarChart, GridComponent, TooltipComponent, CanvasRenderer])

const { t } = useI18n()
const {
  periodState,
  overview,
  hourlyDistribution,
  loading,
  periodLabel,
  isInProgress,
  canGoForward,
  dominantSlot,
  peakHourRange,
  formattedTotalDuration,
  loadReport,
  shiftPeriod,
  setKind,
  TIME_SLOTS,
} = useListeningReport()

onMounted(() => {
  loadReport()
})

const primaryColor = getComputedStyle(document.documentElement)
  .getPropertyValue('--v-theme-primary')
  .trim()

const textColor = computed(() => {
  const style = getComputedStyle(document.documentElement)
  return `rgb(${style.getPropertyValue('--v-theme-on-surface').trim() || '128,128,128'})`
})

const hourlyOption = computed(() => {
  const slotColor = (hour: number) => {
    const slot = TIME_SLOTS.find(s => hour >= s.range[0] && hour <= s.range[1])
    return slot?.color || primaryColor
  }
  return {
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'shadow' },
      formatter: (params: any) => {
        const p = params[0]
        const mins = Math.round(p.value / 60)
        return `${p.name}:00<br/>${mins}m`
      },
    },
    grid: { left: 40, right: 16, top: 10, bottom: 30 },
    xAxis: {
      type: 'category',
      data: hourlyDistribution.value.map(h => String(h.hour)),
      axisLabel: { color: textColor.value, fontSize: 10 },
    },
    yAxis: {
      type: 'value',
      axisLabel: {
        color: textColor.value,
        formatter: '{value}m',
        fontSize: 10,
      },
      splitLine: { lineStyle: { color: 'rgba(128,128,128,0.15)' } },
    },
    series: [
      {
        type: 'bar',
        data: hourlyDistribution.value.map(h => ({
          value: Math.round(h.durationSecs / 60),
          itemStyle: { color: slotColor(h.hour), borderRadius: [3, 3, 0, 0] },
        })),
        barWidth: '70%',
      },
    ],
  }
})

const insightText = computed(() => {
  const slot = dominantSlot.value
  if (slot.total === 0 || !peakHourRange.value) return ''
  const slotLabelMap: Record<string, string> = {
    dawn: t('report.slotDawn'),
    morning: t('report.slotMorning'),
    afternoon: t('report.slotAfternoon'),
    night: t('report.slotNight'),
  }
  if (slot.key === 'night') {
    return t('report.insightNight', { range: peakHourRange.value })
  }
  if (slot.key === 'morning') {
    return t('report.insightMorning', { range: peakHourRange.value })
  }
  return t('report.insightGeneric', { slot: slotLabelMap[slot.key] })
})

const hasData = computed(
  () => overview.value !== null && overview.value.totalDurationSecs > 0,
)
</script>

<template>
  <div class="report-tab">
    <!-- Period Selector -->
    <div class="period-bar">
      <div class="period-kind">
        <v-btn-toggle
          :model-value="periodState.kind"
          mandatory
          density="compact"
          variant="outlined"
          divided
          @update:model-value="setKind($event as any)"
        >
          <v-btn value="week" size="small">{{ t('report.week') }}</v-btn>
          <v-btn value="month" size="small">{{ t('report.month') }}</v-btn>
          <v-btn value="year" size="small">{{ t('report.year') }}</v-btn>
        </v-btn-toggle>
      </div>
      <div class="period-nav">
        <v-btn
          icon
          variant="plain"
          size="small"
          :disabled="!canGoForward"
          @click="shiftPeriod(1)"
        >
          <v-icon icon="mdi-chevron-left" size="22" />
        </v-btn>
        <div class="period-label">
          {{ periodLabel }}
          <span v-if="isInProgress" class="in-progress-tag">
            {{ t('report.inProgress') }}
          </span>
        </div>
        <v-btn
          icon
          variant="plain"
          size="small"
          :disabled="periodState.offset >= 0"
          @click="shiftPeriod(-1)"
        >
          <v-icon icon="mdi-chevron-right" size="22" />
        </v-btn>
      </div>
    </div>

    <div v-if="loading" class="loading-state">
      <v-progress-circular indeterminate color="primary" size="32" />
    </div>

    <template v-else-if="hasData">
      <!-- Overview Cards -->
      <div class="overview-grid">
        <div class="stat-card">
          <div class="stat-label">{{ t('report.totalDuration') }}</div>
          <div class="stat-value">{{ formattedTotalDuration }}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">{{ t('report.playCount') }}</div>
          <div class="stat-value">
            {{ overview?.playCount.toLocaleString() }}
          </div>
          <div class="stat-unit">{{ t('report.times') }}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">{{ t('report.uniqueSongs') }}</div>
          <div class="stat-value">{{ overview?.uniqueSongCount ?? 0 }}</div>
          <div class="stat-unit">{{ t('report.songs') }}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">{{ t('report.uniqueArtists') }}</div>
          <div class="stat-value">{{ overview?.artistCount ?? 0 }}</div>
          <div class="stat-unit">{{ t('report.artists') }}</div>
        </div>
      </div>

      <!-- Hourly Distribution -->
      <v-card class="chart-card" variant="flat" color="surface">
        <div class="card-header">
          <span class="card-title">{{ t('report.hourlyDistribution') }}</span>
          <span v-if="insightText" class="insight-tag">{{ insightText }}</span>
        </div>
        <VChart
          v-if="hourlyDistribution.length"
          :option="hourlyOption"
          autoresize
          style="height: 280px"
        />
      </v-card>
    </template>

    <div v-else class="empty-state">
      <v-icon icon="mdi-chart-line-variant" size="48" class="empty-icon" />
      <p>{{ t('report.emptyState') }}</p>
    </div>
  </div>
</template>

<style scoped>
.report-tab {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.period-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.period-nav {
  display: flex;
  align-items: center;
  gap: 8px;
}

.period-label {
  font-size: var(--text-lg);
  font-weight: 700;
  color: rgb(var(--v-theme-on-background));
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 160px;
  justify-content: center;
}

.in-progress-tag {
  font-size: 10px;
  font-weight: 500;
  color: rgb(var(--v-theme-primary));
  background: rgba(var(--v-theme-primary), 0.1);
  padding: 2px 8px;
  border-radius: 10px;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 64px 0;
  color: rgb(var(--v-theme-on-surface));
  font-size: var(--text-sm);
}

.empty-icon {
  opacity: 0.4;
  margin-bottom: 8px;
}

.overview-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}

.stat-card {
  background: rgb(var(--v-theme-surface));
  border-radius: 12px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  text-align: center;
}

.stat-label {
  font-size: var(--text-xs);
  color: rgb(var(--v-theme-on-surface));
}

.stat-value {
  font-size: var(--text-xl);
  font-weight: 700;
  color: rgb(var(--v-theme-on-background));
}

.stat-unit {
  font-size: var(--text-xs);
  color: rgb(var(--v-theme-on-surface));
}

.chart-card {
  padding: 16px;
  border-radius: 12px;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}

.card-title {
  font-size: var(--text-md);
  font-weight: 600;
}

.insight-tag {
  font-size: var(--text-xs);
  color: rgb(var(--v-theme-primary));
  background: rgba(var(--v-theme-primary), 0.1);
  padding: 4px 12px;
  border-radius: 12px;
}
</style>
