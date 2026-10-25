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
import { useSettingsStore } from '../stores/settings'
import { useI18n } from '../composables/useI18n'

use([BarChart, PieChart, TitleComponent, TooltipComponent, GridComponent, LegendComponent, CanvasRenderer])

const emit = defineEmits<{ back: [] }>()
const analysisStore = useAnalysisStore()
const settingsStore = useSettingsStore()
const { stats, loading, formattedTotalSize, formattedTotalDuration } = storeToRefs(analysisStore)
const { accentHex, isDark } = storeToRefs(settingsStore)
const { t } = useI18n()

onMounted(() => {
  analysisStore.loadStats()
})

// ===== 图表配色：跟随动态强调色 =====

function hexToRgbArr(hex: string): [number, number, number] {
  const v = hex.replace('#', '')
  return [parseInt(v.slice(0, 2), 16), parseInt(v.slice(2, 4), 16), parseInt(v.slice(4, 6), 16)]
}
function accentRgba(alpha: number): string {
  const [r, g, b] = hexToRgbArr(accentHex.value)
  return `rgba(${r}, ${g}, ${b}, ${alpha})`
}
function monoLadder(count: number): string[] {
  return Array.from({ length: count }, (_, i) => accentRgba(Math.max(0.18, 1 - i * 0.14)))
}

const textColor = computed(() => (isDark.value ? 'rgba(242,242,245,0.58)' : 'rgba(29,29,31,0.64)'))
const hairlineColor = computed(() => (isDark.value ? 'rgba(255,255,255,0.085)' : 'rgba(0,0,0,0.09)'))

const artistChartOption = computed(() => {
  if (!stats.value) return {}
  const data = stats.value.artistRanking.slice(0, 15).reverse()
  return {
    tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
    grid: { left: 100, right: 20, top: 10, bottom: 20 },
    xAxis: { type: 'value', axisLabel: { color: textColor.value }, splitLine: { lineStyle: { color: hairlineColor.value } } },
    yAxis: {
      type: 'category',
      data: data.map(d => d.artist),
      axisLabel: {
        color: textColor.value,
        width: 90,
        overflow: 'truncate',
        fontSize: 11,
      },
      axisLine: { lineStyle: { color: hairlineColor.value } },
    },
    series: [{
      type: 'bar',
      data: data.map(d => d.songCount),
      itemStyle: {
        color: accentHex.value,
        borderRadius: [0, 4, 4, 0],
      },
      barWidth: '60%',
    }],
  }
})

const genreChartOption = computed(() => {
  if (!stats.value) return {}
  const colors = monoLadder(stats.value.genreDistribution.length)
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
      radius: ['52%', '72%'],
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
  const colors = monoLadder(stats.value.qualityDistribution.length)
  return {
    tooltip: { trigger: 'item', formatter: '{b}: {c} ({d}%)' },
    legend: {
      bottom: 0,
      textStyle: { color: textColor.value, fontSize: 11 },
    },
    series: [{
      type: 'pie',
      radius: ['52%', '72%'],
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
      axisLine: { lineStyle: { color: hairlineColor.value } },
    },
    yAxis: { type: 'value', axisLabel: { color: textColor.value }, splitLine: { lineStyle: { color: hairlineColor.value } } },
    series: [{
      type: 'bar',
      data: data.map(d => d.count),
      itemStyle: {
        color: accentHex.value,
        borderRadius: [4, 4, 0, 0],
      },
      barWidth: '50%',
    }],
  }
})

// 专辑排行（横条）
const albumMax = computed(() => Math.max(1, ...(stats.value?.albumRanking.slice(0, 10).map(a => a.songCount) ?? [1])))
</script>

<template>
  <div class="analysis-view">
    <!-- 页头 -->
    <div class="page-head">
      <button class="ph-back" :aria-label="t('common.back')" @click="emit('back')">
        <v-icon icon="mdi-chevron-left" size="18"></v-icon>
      </button>
      <h1 class="ph-title">{{ t('analysis.title') }}</h1>
    </div>

    <div class="statpage">
      <div v-if="loading" class="loading-state">
        <v-progress-circular indeterminate color="primary" size="32" />
        <span>{{ t('analysis.loadingStats') }}</span>
      </div>

      <template v-else-if="stats">
        <!-- 概览数字格 -->
        <div class="ov five">
          <div class="ov-cell">
            <b class="num">{{ stats.totalSongs }}</b>
            <span>{{ t('analysis.totalSongs') }}</span>
          </div>
          <div class="ov-cell">
            <b class="num">{{ stats.totalArtists }}</b>
            <span>{{ t('analysis.totalArtists') }}</span>
          </div>
          <div class="ov-cell">
            <b class="num">{{ stats.totalAlbums }}</b>
            <span>{{ t('analysis.totalAlbums') }}</span>
          </div>
          <div class="ov-cell">
            <b class="num">{{ formattedTotalDuration }}</b>
            <span>{{ t('analysis.totalDuration') }}</span>
          </div>
          <div class="ov-cell">
            <b class="num">{{ formattedTotalSize }}</b>
            <span>{{ t('analysis.storageSize') }}</span>
          </div>
        </div>

        <!-- 图表 -->
        <div class="charts2">
          <div class="chart">
            <h4>{{ t('analysis.artistRanking') }}</h4>
            <VChart :option="artistChartOption" autoresize class="chart-canvas" />
          </div>
          <div class="chart">
            <h4>{{ t('analysis.genreDistribution') }}</h4>
            <VChart :option="genreChartOption" autoresize class="chart-canvas" />
          </div>
          <div class="chart">
            <h4>{{ t('analysis.qualityDistribution') }}</h4>
            <VChart :option="qualityChartOption" autoresize class="chart-canvas" />
          </div>
          <div class="chart">
            <h4>{{ t('analysis.durationDistribution') }}</h4>
            <VChart :option="durationChartOption" autoresize class="chart-canvas" />
          </div>
        </div>

        <!-- 专辑排行 -->
        <div class="chart">
          <h4>{{ t('analysis.albumRanking') }}</h4>
          <div
            v-for="(album, i) in stats.albumRanking.slice(0, 10)"
            :key="i"
            class="album-item"
          >
            <span class="album-rank num">{{ String(i + 1).padStart(2, '0') }}</span>
            <span class="album-info">
              <span class="album-name">{{ album.album }}</span>
              <span class="album-artist">{{ album.artist }}</span>
            </span>
            <span class="album-bar" aria-hidden="true">
              <b :style="{ width: Math.max(4, Math.round((album.songCount / albumMax) * 100)) + '%' }"></b>
            </span>
            <span class="album-count num">{{ t('common.songs', { count: album.songCount }) }}</span>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.analysis-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* 页头 */
.page-head {
  display: flex;
  align-items: flex-end;
  gap: 12px;
  padding: 20px 28px 0;
  flex-shrink: 0;
}
.ph-back {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  margin-bottom: 4px;
  display: grid;
  place-items: center;
  color: var(--text-2);
  flex-shrink: 0;
  background: none;
  border: none;
  cursor: pointer;
}
.ph-back:hover {
  background: var(--surface-2);
  color: var(--text-1);
}
.ph-title {
  font-family: var(--font-display);
  font-size: 26px;
  font-weight: 560;
  line-height: 1.15;
  color: var(--text-1);
}

.statpage {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 14px 28px 28px;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 64px 0;
  color: var(--text-3);
  font-size: 12.5px;
}

/* 概览数字格 */
.ov {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-bottom: 16px;
}
.ov.five {
  grid-template-columns: repeat(5, 1fr);
}
.ov-cell {
  border: var(--hairline-w) solid var(--hairline);
  border-radius: 12px;
  padding: 14px 8px 12px;
  text-align: center;
}
.ov-cell b {
  display: block;
  font-family: var(--font-display);
  font-size: 20px;
  font-weight: 560;
  line-height: 1.25;
  color: var(--text-1);
}
.ov-cell span {
  font-size: 11px;
  color: var(--text-3);
  margin-top: 3px;
  display: block;
}

/* 图表卡片 */
.charts2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}
.chart {
  border: var(--hairline-w) solid var(--hairline);
  border-radius: 12px;
  padding: 14px 16px 12px;
  margin-bottom: 14px;
}
.chart h4 {
  font-size: 12.5px;
  font-weight: 640;
  margin-bottom: 12px;
  color: var(--text-1);
}
.chart-canvas {
  height: 300px;
}

/* 专辑排行 */
.album-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 9px 0;
  border-bottom: var(--hairline-w) solid var(--hairline);
}
.album-item:last-child {
  border-bottom: none;
}
.album-rank {
  width: 26px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-3);
  flex-shrink: 0;
}
.album-info {
  width: 200px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}
.album-name {
  font-size: 12.5px;
  font-weight: 580;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.album-artist {
  font-size: 10.5px;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.album-bar {
  flex: 1;
  height: 6px;
  border-radius: 99px;
  background: var(--surface-2);
  position: relative;
  overflow: hidden;
}
.album-bar b {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  border-radius: 99px;
  background: var(--accent);
  display: block;
}
.album-count {
  width: 56px;
  font-size: 11px;
  color: var(--text-3);
  text-align: right;
  flex-shrink: 0;
}

@media (max-width: 960px) {
  .charts2 {
    grid-template-columns: 1fr;
  }
  .ov.five {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
