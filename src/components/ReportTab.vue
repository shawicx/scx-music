<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { storeToRefs } from 'pinia'
import { use } from 'echarts/core'
import { BarChart } from 'echarts/charts'
import { GridComponent, TooltipComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import VChart from 'vue-echarts'
import { useListeningReport } from '../composables/useListeningReport'
import { useSettingsStore } from '../stores/settings'
import { useI18n } from '../composables/useI18n'
import SegControl from './common/SegControl.vue'

use([BarChart, GridComponent, TooltipComponent, CanvasRenderer])

const { t } = useI18n()
const settingsStore = useSettingsStore()
const { accentHex, isDark } = storeToRefs(settingsStore)
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
} = useListeningReport()

onMounted(() => {
  loadReport()
})

const textColor = computed(() => (isDark.value ? 'rgba(242,242,245,0.58)' : 'rgba(29,29,31,0.64)'))
const hairlineColor = computed(() => (isDark.value ? 'rgba(255,255,255,0.085)' : 'rgba(0,0,0,0.09)'))

const hourlyOption = computed(() => {
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
      axisLine: { lineStyle: { color: hairlineColor.value } },
    },
    yAxis: {
      type: 'value',
      axisLabel: {
        color: textColor.value,
        formatter: '{value}m',
        fontSize: 10,
      },
      splitLine: { lineStyle: { color: hairlineColor.value } },
    },
    series: [
      {
        type: 'bar',
        data: hourlyDistribution.value.map(h => ({
          value: Math.round(h.durationSecs / 60),
          itemStyle: { color: accentHex.value, borderRadius: [3, 3, 0, 0] },
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
    <!-- 周期选择 + 期次导航 -->
    <div class="period-bar">
      <SegControl
        :model-value="periodState.kind"
        :options="[
          { value: 'week', label: t('report.week') },
          { value: 'month', label: t('report.month') },
          { value: 'year', label: t('report.year') },
        ]"
        :aria-label="t('report.tab')"
        @update:model-value="setKind($event as any)"
      />
      <div class="period-nav">
        <button class="nav-btn" :disabled="!canGoForward" :aria-label="t('common.back')" @click="shiftPeriod(1)">
          <v-icon icon="mdi-chevron-left" size="20"></v-icon>
        </button>
        <div class="period-label num">
          {{ periodLabel }}
          <span v-if="isInProgress" class="in-progress-tag">{{ t('report.inProgress') }}</span>
        </div>
        <button class="nav-btn" :disabled="periodState.offset >= 0" :aria-label="t('common.next')" @click="shiftPeriod(-1)">
          <v-icon icon="mdi-chevron-right" size="20"></v-icon>
        </button>
      </div>
    </div>

    <div v-if="loading" class="loading-state">
      <v-progress-circular indeterminate color="primary" size="32" />
    </div>

    <template v-else-if="hasData">
      <!-- 概览数字格 -->
      <div class="ov">
        <div class="ov-cell">
          <b class="num">{{ formattedTotalDuration }}</b>
          <span>{{ t('report.totalDuration') }}</span>
        </div>
        <div class="ov-cell">
          <b class="num">{{ overview?.playCount.toLocaleString() }}</b>
          <span>{{ t('report.playCount') }}</span>
        </div>
        <div class="ov-cell">
          <b class="num">{{ overview?.uniqueSongCount ?? 0 }}</b>
          <span>{{ t('report.uniqueSongs') }}</span>
        </div>
        <div class="ov-cell">
          <b class="num">{{ overview?.artistCount ?? 0 }}</b>
          <span>{{ t('report.uniqueArtists') }}</span>
        </div>
      </div>

      <!-- 时段分布 -->
      <div class="chart">
        <h4>
          {{ t('report.hourlyDistribution') }}
          <span v-if="insightText" class="insight-tag">{{ insightText }}</span>
        </h4>
        <VChart
          v-if="hourlyDistribution.length"
          :option="hourlyOption"
          autoresize
          style="height: 280px"
        />
      </div>
    </template>

    <div v-else class="empty-state">
      <v-icon icon="mdi-chart-line-variant" size="40" class="empty-icon" />
      <p>{{ t('report.emptyState') }}</p>
    </div>
  </div>
</template>

<style scoped>
.report-tab {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.period-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.period-nav {
  display: flex;
  align-items: center;
  gap: 6px;
}
.nav-btn {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  display: grid;
  place-items: center;
  color: var(--text-2);
  background: none;
  border: none;
  cursor: pointer;
}
.nav-btn:hover:not(:disabled) {
  background: var(--surface-2);
  color: var(--text-1);
}
.nav-btn:disabled {
  opacity: 0.35;
  cursor: default;
}

.period-label {
  font-size: 12px;
  color: var(--text-2);
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 160px;
  justify-content: center;
}

.in-progress-tag {
  font-size: 10.5px;
  font-style: normal;
  color: var(--text-3);
  border: var(--hairline-w) solid var(--hairline);
  padding: 2px 9px;
  border-radius: 99px;
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

.empty-icon {
  opacity: 0.4;
  margin-bottom: 8px;
}

/* 概览数字格 */
.ov {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
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

.chart {
  border: var(--hairline-w) solid var(--hairline);
  border-radius: 12px;
  padding: 14px 16px 12px;
}
.chart h4 {
  font-size: 12.5px;
  font-weight: 640;
  margin-bottom: 12px;
  color: var(--text-1);
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.insight-tag {
  margin-left: auto;
  font-size: 10.5px;
  font-weight: 500;
  color: var(--accent);
  background: rgb(var(--accent-rgb) / 0.1);
  padding: 3px 10px;
  border-radius: 99px;
}

@media (max-width: 960px) {
  .ov {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
