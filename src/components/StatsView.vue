<script setup lang="ts">
import { onMounted, computed, ref } from 'vue'
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
import { useSettingsStore } from '../stores/settings'
import { useI18n } from '../composables/useI18n'
import type { ListeningRange } from '../types'
import SegControl from './common/SegControl.vue'
import ReportTab from './ReportTab.vue'

use([
  BarChart, PieChart, LineChart, HeatmapChart,
  TitleComponent, TooltipComponent, GridComponent, LegendComponent,
  CalendarComponent, VisualMapComponent, CanvasRenderer,
])

const emit = defineEmits<{ back: [] }>()
const statsStore = useStatsStore()
const settingsStore = useSettingsStore()
const { overview, topSongs, topArtists, genreDistribution, trend, heatmap, loading, currentRange, formattedTotalDuration } = storeToRefs(statsStore)
const { accentHex, isDark } = storeToRefs(settingsStore)
const { t } = useI18n()

const activeTab = ref<'stats' | 'report'>('stats')

onMounted(() => {
  statsStore.loadData('7d')
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
/** 单色阶梯（环形图）：accent 依序递减透明度。 */
function monoLadder(count: number): string[] {
  return Array.from({ length: count }, (_, i) => accentRgba(Math.max(0.18, 1 - i * 0.14)))
}

const textColor = computed(() => (isDark.value ? 'rgba(242,242,245,0.58)' : 'rgba(29,29,31,0.64)'))
const hairlineColor = computed(() => (isDark.value ? 'rgba(255,255,255,0.085)' : 'rgba(0,0,0,0.09)'))

const topSongsOption = computed(() => {
  if (!topSongs.value.length) return {}
  const data = topSongs.value.slice().reverse()
  return {
    tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
    grid: { left: 130, right: 20, top: 10, bottom: 20 },
    xAxis: { type: 'value', axisLabel: { color: textColor.value }, splitLine: { lineStyle: { color: hairlineColor.value } } },
    yAxis: {
      type: 'category',
      data: data.map(d => `${d.title} · ${d.artist}`),
      axisLabel: { color: textColor.value, width: 120, overflow: 'truncate', fontSize: 11 },
      axisLine: { lineStyle: { color: hairlineColor.value } },
    },
    series: [{
      type: 'bar',
      data: data.map(d => d.playCount),
      itemStyle: { color: accentHex.value, borderRadius: [0, 4, 4, 0] },
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
    xAxis: { type: 'value', axisLabel: { color: textColor.value }, splitLine: { lineStyle: { color: hairlineColor.value } } },
    yAxis: {
      type: 'category',
      data: data.map(d => d.artist),
      axisLabel: { color: textColor.value, width: 90, overflow: 'truncate', fontSize: 11 },
      axisLine: { lineStyle: { color: hairlineColor.value } },
    },
    series: [{
      type: 'bar',
      data: data.map(d => d.playCount),
      itemStyle: { color: accentHex.value, borderRadius: [0, 4, 4, 0] },
      barWidth: '60%',
    }],
  }
})

const genreOption = computed(() => {
  if (!genreDistribution.value.length) return {}
  const colors = monoLadder(genreDistribution.value.length)
  return {
    tooltip: { trigger: 'item', formatter: '{b}: {c}s ({d}%)' },
    legend: {
      orient: 'vertical', right: 10, top: 'center',
      textStyle: { color: textColor.value, fontSize: 11 },
      formatter: (name: string) => name.length > 10 ? name.slice(0, 10) + '...' : name,
    },
    series: [{
      type: 'pie',
      radius: ['52%', '72%'],
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
      axisLine: { lineStyle: { color: hairlineColor.value } },
    },
    yAxis: { type: 'value', axisLabel: { color: textColor.value, formatter: '{value}m' }, splitLine: { lineStyle: { color: hairlineColor.value } } },
    series: [{
      type: 'line',
      data: trend.value.map(d => Math.round(d.durationSecs / 60)),
      smooth: true,
      areaStyle: { opacity: 0.16, color: accentHex.value },
      lineStyle: { color: accentHex.value, width: 2 },
      itemStyle: { color: accentHex.value },
      symbolSize: 5,
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
  const emptyCell = isDark.value ? 'rgba(255,255,255,0.05)' : 'rgba(0,0,0,0.05)'
  const cellBorder = isDark.value ? '#1b1b23' : '#f5f5f4'
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
        { min: 0, max: 0, color: emptyCell, label: '0' },
        { min: 1, max: 15, color: accentRgba(0.25), label: '1-15' },
        { min: 16, max: 30, color: accentRgba(0.45), label: '16-30' },
        { min: 31, max: 60, color: accentRgba(0.7), label: '31-60' },
        { min: 61, color: accentRgba(1), label: '60+' },
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
      itemStyle: { borderWidth: 3, borderColor: cellBorder, color: emptyCell },
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

function switchRange(range: string | number) {
  statsStore.loadData(range as ListeningRange)
}
</script>

<template>
  <div class="stats-view">
    <!-- 页头：返回 + 衬线标题 + 分页 + 时间范围 -->
    <div class="page-head">
      <button class="ph-back" :aria-label="t('common.back')" @click="emit('back')">
        <v-icon icon="mdi-chevron-left" size="18"></v-icon>
      </button>
      <h1 class="ph-title">{{ t('stats.title') }}</h1>
      <div class="page-tools">
        <SegControl
          v-model="activeTab"
          :options="[
            { value: 'stats', label: t('stats.title') },
            { value: 'report', label: t('report.tab') },
          ]"
          :aria-label="t('stats.title')"
        />
        <SegControl
          v-if="activeTab === 'stats'"
          :model-value="currentRange"
          :options="[
            { value: '7d', label: t('stats.last7days') },
            { value: '30d', label: t('stats.last30days') },
            { value: 'all', label: t('stats.all') },
          ]"
          :aria-label="t('stats.all')"
          @update:model-value="switchRange"
        />
      </div>
    </div>

    <div v-if="activeTab === 'stats'" class="statpage">
      <div v-if="loading" class="loading-state">
        <v-progress-circular indeterminate color="primary" size="32" />
        <span>{{ t('stats.loadingStats') }}</span>
      </div>

      <template v-else-if="overview">
        <!-- 概览数字格 -->
        <div class="ov">
          <div class="ov-cell">
            <b class="num">{{ overview.playCount.toLocaleString() }}</b>
            <span>{{ t('stats.playCount') }}</span>
          </div>
          <div class="ov-cell">
            <b class="num">{{ formattedTotalDuration }}</b>
            <span>{{ t('stats.totalDuration') }}</span>
          </div>
          <div class="ov-cell">
            <b class="num">{{ overview.genreCount }}</b>
            <span>{{ t('stats.genresListened') }}</span>
          </div>
          <div class="ov-cell">
            <b class="num">{{ overview.artistCount }}</b>
            <span>{{ t('stats.artistsListened') }}</span>
          </div>
        </div>

        <!-- 图表 -->
        <div class="charts2">
          <div class="chart">
            <h4>{{ t('stats.topSongs') }}</h4>
            <VChart v-if="topSongs.length" :option="topSongsOption" autoresize class="chart-canvas" />
            <div v-else class="empty-chart">{{ t('stats.emptyState') }}</div>
          </div>
          <div class="chart">
            <h4>{{ t('stats.topArtists') }}</h4>
            <VChart v-if="topArtists.length" :option="topArtistsOption" autoresize class="chart-canvas" />
            <div v-else class="empty-chart">{{ t('stats.emptyState') }}</div>
          </div>
          <div class="chart">
            <h4>{{ t('stats.genreDistribution') }}</h4>
            <VChart v-if="genreDistribution.length" :option="genreOption" autoresize class="chart-canvas" />
            <div v-else class="empty-chart">{{ t('stats.emptyState') }}</div>
          </div>
          <div class="chart">
            <h4>{{ t('stats.trend') }}</h4>
            <VChart v-if="trend.length" :option="trendOption" autoresize class="chart-canvas" />
            <div v-else class="empty-chart">{{ t('stats.emptyState') }}</div>
          </div>
        </div>

        <div class="chart">
          <h4>{{ t('stats.heatmap') }}</h4>
          <VChart v-if="heatmap.length" :option="heatmapOption" autoresize style="height: 200px" />
          <div v-else class="empty-chart">{{ t('stats.emptyState') }}</div>
        </div>
      </template>

      <div v-else class="empty-state">
        <p>{{ t('stats.emptyState') }}</p>
      </div>
    </div>

    <div v-else class="statpage">
      <ReportTab />
    </div>
  </div>
</template>

<style scoped>
.stats-view {
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
.page-tools {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 10px;
  padding-bottom: 4px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.statpage {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 14px 28px 28px;
}

.loading-state,
.empty-state {
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
.empty-chart {
  height: 300px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-3);
  font-size: 12.5px;
}

@media (max-width: 960px) {
  .charts2 {
    grid-template-columns: 1fr;
  }
  .ov {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
