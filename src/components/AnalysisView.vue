<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { storeToRefs } from 'pinia'
import { use } from 'echarts/core'
import { BarChart, PieChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  GridComponent,
  LegendComponent,
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import VChart from 'vue-echarts'
import { useAnalysisStore } from '../stores/analysis'
import { useI18n } from '../composables/useI18n'

use([BarChart, PieChart, TitleComponent, TooltipComponent, GridComponent, LegendComponent, CanvasRenderer])

const emit = defineEmits<{ back: [] }>()
const analysisStore = useAnalysisStore()
const { stats, loading, formattedTotalSize, formattedTotalDuration } = storeToRefs(analysisStore)
const { t } = useI18n()

onMounted(() => {
  analysisStore.loadStats()
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
  return `rgb(${style.getPropertyValue('--v-theme-on-surface').trim() || '128,128,128'})`
})

const artistChartOption = computed(() => {
  if (!stats.value) return {}
  const data = stats.value.artistRanking.slice(0, 15).reverse()
  return {
    tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
    grid: { left: 100, right: 20, top: 10, bottom: 20 },
    xAxis: { type: 'value', axisLabel: { color: textColor.value } },
    yAxis: {
      type: 'category',
      data: data.map(d => d.artist),
      axisLabel: {
        color: textColor.value,
        width: 90,
        overflow: 'truncate',
        fontSize: 11,
      },
    },
    series: [{
      type: 'bar',
      data: data.map(d => d.songCount),
      itemStyle: {
        color: `rgb(${primaryColor})`,
        borderRadius: [0, 4, 4, 0],
      },
      barWidth: '60%',
    }],
  }
})

const genreChartOption = computed(() => {
  if (!stats.value) return {}
  const colors = getThemeColors()
  return {
    tooltip: { trigger: 'item', formatter: '{b}: {c} ({d}%)' },
    legend: {
      orient: 'vertical',
      right: 10,
      top: 'center',
      textStyle: { color: textColor.value, fontSize: 11 },
      formatter: (name: string) => name.length > 10 ? name.slice(0, 10) + '...' : name,
    },
    series: [{
      type: 'pie',
      radius: ['40%', '70%'],
      center: ['35%', '50%'],
      avoidLabelOverlap: false,
      label: { show: false },
      data: stats.value.genreDistribution.map((d, i) => ({
        name: d.genre,
        value: d.songCount,
        itemStyle: { color: colors[i % colors.length] },
      })),
    }],
  }
})

const qualityChartOption = computed(() => {
  if (!stats.value) return {}
  const colors = getThemeColors()
  return {
    tooltip: { trigger: 'item', formatter: '{b}: {c} ({d}%)' },
    legend: {
      bottom: 0,
      textStyle: { color: textColor.value, fontSize: 11 },
    },
    series: [{
      type: 'pie',
      radius: ['40%', '70%'],
      center: ['50%', '45%'],
      label: { show: false },
      data: stats.value.qualityDistribution.map((d, i) => ({
        name: d.quality,
        value: d.songCount,
        itemStyle: { color: colors[i % colors.length] },
      })),
    }],
  }
})

const durationChartOption = computed(() => {
  if (!stats.value) return {}
  const bucketOrder = ['0-2min', '2-5min', '5-10min', '10min+']
  const data = bucketOrder
    .map(label => {
      const found = stats.value!.durationDistribution.find(d => d.label === label)
      return { label, count: found ? found.songCount : 0 }
    })
  return {
    tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
    grid: { left: 50, right: 20, top: 10, bottom: 30 },
    xAxis: {
      type: 'category',
      data: data.map(d => d.label),
      axisLabel: { color: textColor.value },
    },
    yAxis: { type: 'value', axisLabel: { color: textColor.value } },
    series: [{
      type: 'bar',
      data: data.map(d => d.count),
      itemStyle: {
        color: `rgb(${primaryColor})`,
        borderRadius: [4, 4, 0, 0],
      },
      barWidth: '50%',
    }],
  }
})
</script>

<template>
  <div class="analysis-view">
    <div class="analysis-header">
      <v-btn icon variant="plain" size="small" @click="emit('back')">
        <v-icon icon="mdi-arrow-left" size="20" />
      </v-btn>
      <h2 class="analysis-title">{{ t('analysis.title') }}</h2>
    </div>

    <div v-if="loading" class="loading-state">
      <v-progress-circular indeterminate color="primary" size="32" />
      <span>{{ t('analysis.loadingStats') }}</span>
    </div>

    <template v-else-if="stats">
      <!-- Overview Cards -->
      <div class="overview-grid">
        <div class="stat-card">
          <v-icon icon="mdi-music-note" size="20" class="stat-icon" />
          <div class="stat-value">{{ stats.totalSongs }}</div>
          <div class="stat-label">{{ t('analysis.totalSongs') }}</div>
        </div>
        <div class="stat-card">
          <v-icon icon="mdi-account-music" size="20" class="stat-icon" />
          <div class="stat-value">{{ stats.totalArtists }}</div>
          <div class="stat-label">{{ t('analysis.totalArtists') }}</div>
        </div>
        <div class="stat-card">
          <v-icon icon="mdi-album" size="20" class="stat-icon" />
          <div class="stat-value">{{ stats.totalAlbums }}</div>
          <div class="stat-label">{{ t('analysis.totalAlbums') }}</div>
        </div>
        <div class="stat-card">
          <v-icon icon="mdi-clock-outline" size="20" class="stat-icon" />
          <div class="stat-value">{{ formattedTotalDuration }}</div>
          <div class="stat-label">{{ t('analysis.totalDuration') }}</div>
        </div>
        <div class="stat-card">
          <v-icon icon="mdi-harddisk" size="20" class="stat-icon" />
          <div class="stat-value">{{ formattedTotalSize }}</div>
          <div class="stat-label">{{ t('analysis.storageSize') }}</div>
        </div>
      </div>

      <!-- Charts Grid -->
      <div class="charts-grid">
        <v-card class="chart-card" variant="flat" color="surface">
          <div class="card-header">
            <v-icon icon="mdi-account-music" size="18" class="card-icon" />
            <span class="card-title">{{ t('analysis.artistRanking') }}</span>
          </div>
          <VChart :option="artistChartOption" autoresize style="height: 320px" />
        </v-card>

        <v-card class="chart-card" variant="flat" color="surface">
          <div class="card-header">
            <v-icon icon="mdi-tag-multiple" size="18" class="card-icon" />
            <span class="card-title">{{ t('analysis.genreDistribution') }}</span>
          </div>
          <VChart :option="genreChartOption" autoresize style="height: 320px" />
        </v-card>

        <v-card class="chart-card" variant="flat" color="surface">
          <div class="card-header">
            <v-icon icon="mdi-tune-variant" size="18" class="card-icon" />
            <span class="card-title">{{ t('analysis.qualityDistribution') }}</span>
          </div>
          <VChart :option="qualityChartOption" autoresize style="height: 320px" />
        </v-card>

        <v-card class="chart-card" variant="flat" color="surface">
          <div class="card-header">
            <v-icon icon="mdi-timer-sand" size="18" class="card-icon" />
            <span class="card-title">{{ t('analysis.durationDistribution') }}</span>
          </div>
          <VChart :option="durationChartOption" autoresize style="height: 320px" />
        </v-card>
      </div>

      <!-- Album Ranking -->
      <v-card class="chart-card" variant="flat" color="surface">
        <div class="card-header">
          <v-icon icon="mdi-album" size="18" class="card-icon" />
          <span class="card-title">{{ t('analysis.albumRanking') }}</span>
        </div>
        <div class="album-list">
          <div
            v-for="(album, i) in stats.albumRanking.slice(0, 10)"
            :key="i"
            class="album-item"
          >
            <span class="album-rank">{{ i + 1 }}</span>
            <div class="album-info">
              <span class="album-name">{{ album.album }}</span>
              <span class="album-artist">{{ album.artist }}</span>
            </div>
            <span class="album-count">{{ t('common.songs', { count: album.songCount }) }}</span>
          </div>
        </div>
      </v-card>
    </template>
  </div>
</template>

<style scoped>
.analysis-view {
  padding: 32px;
  overflow-y: auto;
  height: 100%;
}

.analysis-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 24px;
}

.analysis-title {
  font-size: var(--text-xl);
  font-weight: 700;
  color: rgb(var(--v-theme-on-background));
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 64px 0;
  color: rgb(var(--v-theme-on-surface));
  font-size: var(--text-sm);
}

.overview-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
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

.stat-icon {
  color: rgb(var(--v-theme-on-surface));
  margin-bottom: 4px;
}

.stat-value {
  font-size: var(--text-lg);
  font-weight: 700;
  color: rgb(var(--v-theme-on-background));
}

.stat-label {
  font-size: var(--text-xs);
  color: rgb(var(--v-theme-on-surface));
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

.card-icon {
  color: rgb(var(--v-theme-on-surface));
}

.card-title {
  font-size: var(--text-md);
  font-weight: 600;
}

.album-list {
  display: flex;
  flex-direction: column;
}

.album-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 0;
  border-bottom: 1px solid var(--v-border-color);
}

.album-item:last-child {
  border-bottom: none;
}

.album-rank {
  width: 24px;
  text-align: center;
  font-weight: 700;
  font-size: var(--text-sm);
  color: rgb(var(--v-theme-primary));
}

.album-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.album-name {
  font-size: var(--text-sm);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.album-artist {
  font-size: var(--text-xs);
  color: rgb(var(--v-theme-on-surface));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.album-count {
  font-size: var(--text-xs);
  color: rgb(var(--v-theme-on-surface));
  white-space: nowrap;
}
</style>
