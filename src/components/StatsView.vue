<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { storeToRefs } from 'pinia'
import { use } from 'echarts/core'
import { BarChart, PieChart, LineChart, HeatmapChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  GridComponent,
  LegendComponent,
  CalendarComponent,
  VisualMapComponent,
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import VChart from 'vue-echarts'
import { useStatsStore } from '../stores/stats'
import { useI18n } from '../composables/useI18n'
import type { ListeningRange } from '../types'

use([
  BarChart, PieChart, LineChart, HeatmapChart,
  TitleComponent, TooltipComponent, GridComponent, LegendComponent,
  CalendarComponent, VisualMapComponent, CanvasRenderer,
])

const emit = defineEmits<{ back: [] }>()
const statsStore = useStatsStore()
const { overview, topSongs, topArtists, genreDistribution, trend, heatmap, loading, currentRange, formattedTotalDuration } = storeToRefs(statsStore)
const { t } = useI18n()

onMounted(() => {
  statsStore.loadData('7d')
})

const primaryColor = getComputedStyle(document.documentElement).getPropertyValue('--v-theme-primary').trim()

function getThemeColors() {
  const style = getComputedStyle(document.documentElement)
  const p = style.getPropertyValue('--v-theme-primary').trim()
  const colors = [
    `rgb(${p})`,
    '#FF6384', '#36A2EB', '#FFCE56', '#4BC0C0',
    '#9966FF', '#FF9F40', '#C9CBCF', '#7BC8A4',
    '#E7E9ED', '#FF6B6B', '#4ECDC4', '#45B7D1',
  ]
  return colors
}

const textColor = computed(() => {
  const style = getComputedStyle(document.documentElement)
  return `rgb(${style.getPropertyValue('--v-theme-on-surface-variant').trim() || '128,128,128'})`
})

const topSongsOption = computed(() => {
  if (!topSongs.value.length) return {}
  const data = topSongs.value.slice().reverse()
  return {
    tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
    grid: { left: 120, right: 20, top: 10, bottom: 20 },
    xAxis: { type: 'value', axisLabel: { color: textColor.value } },
    yAxis: {
      type: 'category',
      data: data.map(d => `${d.title} - ${d.artist}`),
      axisLabel: { color: textColor.value, width: 110, overflow: 'truncate', fontSize: 11 },
    },
    series: [{
      type: 'bar',
      data: data.map(d => d.playCount),
      itemStyle: { color: `rgb(${primaryColor})`, borderRadius: [0, 4, 4, 0] },
      barWidth: '60%',
    }],
  }
})

const topArtistsOption = computed(() => {
  if (!topArtists.value.length) return {}
  const data = topArtists.value.slice().reverse()
  return {
    tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
    grid: { left: 100, right: 20, top: 10, bottom: 20 },
    xAxis: { type: 'value', axisLabel: { color: textColor.value } },
    yAxis: {
      type: 'category',
      data: data.map(d => d.artist),
      axisLabel: { color: textColor.value, width: 90, overflow: 'truncate', fontSize: 11 },
    },
    series: [{
      type: 'bar',
      data: data.map(d => d.playCount),
      itemStyle: { color: `rgb(${primaryColor})`, borderRadius: [0, 4, 4, 0] },
      barWidth: '60%',
    }],
  }
})

const genreOption = computed(() => {
  if (!genreDistribution.value.length) return {}
  const colors = getThemeColors()
  return {
    tooltip: { trigger: 'item', formatter: '{b}: {c}s ({d}%)' },
    legend: {
      orient: 'vertical', right: 10, top: 'center',
      textStyle: { color: textColor.value, fontSize: 11 },
      formatter: (name: string) => name.length > 10 ? name.slice(0, 10) + '...' : name,
    },
    series: [{
      type: 'pie',
      radius: ['40%', '70%'],
      center: ['35%', '50%'],
      avoidLabelOverlap: false,
      label: { show: false },
      data: genreDistribution.value.map((d, i) => ({
        name: d.genre,
        value: Math.round(d.durationSecs),
        itemStyle: { color: colors[i % colors.length] },
      })),
    }],
  }
})

const trendOption = computed(() => {
  if (!trend.value.length) return {}
  return {
    tooltip: { trigger: 'axis' },
    grid: { left: 50, right: 20, top: 10, bottom: 30 },
    xAxis: {
      type: 'category',
      data: trend.value.map(d => d.date),
      axisLabel: { color: textColor.value, fontSize: 10, rotate: 30 },
    },
    yAxis: { type: 'value', axisLabel: { color: textColor.value, formatter: '{value}m' } },
    series: [{
      type: 'line',
      data: trend.value.map(d => Math.round(d.durationSecs / 60)),
      smooth: true,
      areaStyle: { opacity: 0.3 },
      lineStyle: { color: `rgb(${primaryColor})` },
      itemStyle: { color: `rgb(${primaryColor})` },
    }],
  }
})

const heatmapOption = computed(() => {
  if (!heatmap.value.length) return {}
  const data: [string, number][] = heatmap.value.map(d => [d.date, Math.round(d.durationSecs / 60)])
  const maxVal = Math.max(60, ...data.map(d => d[1]))
  const firstDate = data[0]?.[0]
  const lastDate = data[data.length - 1]?.[0]
  if (!firstDate || !lastDate) return {}
  return {
    tooltip: { formatter: (p: any) => `${p.value[0]}: ${p.value[1]}min` },
    visualMap: {
      min: 0,
      max: maxVal,
      type: 'piecewise',
      orient: 'horizontal',
      left: 'center',
      top: 0,
      pieces: [
        { min: 0, max: 0, color: '#ebedf0', label: '0' },
        { min: 1, max: 15, color: '#9be9a8', label: '1-15' },
        { min: 16, max: 30, color: '#40c463', label: '16-30' },
        { min: 31, max: 60, color: '#30a14e', label: '31-60' },
        { min: 61, color: '#216e39', label: '60+' },
      ],
      textStyle: { color: textColor.value, fontSize: 10 },
    },
    calendar: {
      top: 60,
      left: 40,
      right: 20,
      bottom: 10,
      range: [firstDate, lastDate],
      cellSize: ['auto', 14],
      splitLine: { show: false },
      itemStyle: { borderWidth: 3, borderColor: '#fff' },
      yearLabel: { show: false },
      dayLabel: { firstDay: 1, color: textColor.value, fontSize: 10 },
      monthLabel: { color: textColor.value, fontSize: 10 },
    },
    series: [{
      type: 'heatmap',
      coordinateSystem: 'calendar',
      data,
    }],
  }
})

function switchRange(range: ListeningRange) {
  statsStore.loadData(range)
}
</script>

<template>
  <div class="stats-view">
    <div class="stats-header">
      <v-btn icon variant="plain" size="small" @click="emit('back')">
        <v-icon icon="mdi-arrow-left" size="20" />
      </v-btn>
      <h2 class="stats-title">{{ t('stats.title') }}</h2>
      <div class="range-selector">
        <v-btn-toggle v-model="currentRange" mandatory density="compact" variant="outlined" divided @update:model-value="switchRange">
          <v-btn value="7d" size="small">{{ t('stats.last7days') }}</v-btn>
          <v-btn value="30d" size="small">{{ t('stats.last30days') }}</v-btn>
          <v-btn value="all" size="small">{{ t('stats.all') }}</v-btn>
        </v-btn-toggle>
      </div>
    </div>

    <div v-if="loading" class="loading-state">
      <v-progress-circular indeterminate color="primary" size="32" />
      <span>{{ t('stats.loadingStats') }}</span>
    </div>

    <template v-else-if="overview">
      <!-- Overview Cards -->
      <div class="overview-grid">
        <div class="stat-card">
          <div class="stat-label">{{ t('stats.playCount') }}</div>
          <div class="stat-value">{{ overview.playCount.toLocaleString() }}</div>
          <div class="stat-unit">{{ t('stats.times') }}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">{{ t('stats.totalDuration') }}</div>
          <div class="stat-value">{{ formattedTotalDuration }}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">{{ t('stats.genresListened') }}</div>
          <div class="stat-value">{{ overview.genreCount }}</div>
          <div class="stat-unit">{{ t('stats.kinds') }}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">{{ t('stats.artistsListened') }}</div>
          <div class="stat-value">{{ overview.artistCount }}</div>
          <div class="stat-unit">{{ t('stats.artists') }}</div>
        </div>
      </div>

      <!-- Charts Grid -->
      <div class="charts-grid">
        <v-card class="chart-card" variant="flat" color="surface">
          <div class="card-header">
            <span class="card-title">{{ t('stats.topSongs') }}</span>
          </div>
          <VChart v-if="topSongs.length" :option="topSongsOption" autoresize style="height: 320px" />
          <div v-else class="empty-chart">{{ t('stats.emptyState') }}</div>
        </v-card>

        <v-card class="chart-card" variant="flat" color="surface">
          <div class="card-header">
            <span class="card-title">{{ t('stats.topArtists') }}</span>
          </div>
          <VChart v-if="topArtists.length" :option="topArtistsOption" autoresize style="height: 320px" />
          <div v-else class="empty-chart">{{ t('stats.emptyState') }}</div>
        </v-card>

        <v-card class="chart-card" variant="flat" color="surface">
          <div class="card-header">
            <span class="card-title">{{ t('stats.genreDistribution') }}</span>
          </div>
          <VChart v-if="genreDistribution.length" :option="genreOption" autoresize style="height: 320px" />
          <div v-else class="empty-chart">{{ t('stats.emptyState') }}</div>
        </v-card>

        <v-card class="chart-card" variant="flat" color="surface">
          <div class="card-header">
            <span class="card-title">{{ t('stats.trend') }}</span>
          </div>
          <VChart v-if="trend.length" :option="trendOption" autoresize style="height: 320px" />
          <div v-else class="empty-chart">{{ t('stats.emptyState') }}</div>
        </v-card>
      </div>

      <!-- Heatmap -->
      <v-card class="chart-card" variant="flat" color="surface">
        <div class="card-header">
          <span class="card-title">{{ t('stats.heatmap') }}</span>
        </div>
        <VChart v-if="heatmap.length" :option="heatmapOption" autoresize style="height: 200px" />
        <div v-else class="empty-chart">{{ t('stats.emptyState') }}</div>
      </v-card>
    </template>

    <div v-else class="empty-state">
      <p>{{ t('stats.emptyState') }}</p>
    </div>
  </div>
</template>

<style scoped>
.stats-view {
  padding: 32px;
  overflow-y: auto;
  height: 100%;
}

.stats-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 24px;
}

.stats-title {
  font-size: var(--text-xl);
  font-weight: 700;
  color: rgb(var(--v-theme-on-background));
  flex: 1;
}

.range-selector {
  margin-left: auto;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 64px 0;
  color: var(--v-text-secondary);
  font-size: var(--text-sm);
}

.overview-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-bottom: 20px;
}

.stat-card {
  background: rgb(var(--v-theme-surface));
  border-radius: 12px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  text-align: center;
}

.stat-label {
  font-size: var(--text-xs);
  color: var(--v-text-secondary);
}

.stat-value {
  font-size: var(--text-lg);
  font-weight: 700;
  color: rgb(var(--v-theme-on-background));
}

.stat-unit {
  font-size: var(--text-xs);
  color: var(--v-text-secondary);
}

.charts-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin-bottom: 16px;
}

.chart-card {
  padding: 16px;
  border-radius: 12px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.card-title {
  font-size: var(--text-md);
  font-weight: 600;
}

.empty-chart {
  height: 320px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--v-text-secondary);
  font-size: var(--text-sm);
}
</style>
