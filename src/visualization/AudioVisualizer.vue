<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { VisualizationStyle } from '../types'
import { useI18n } from '../composables/useI18n'
import { useAudioAnalyzer } from './useAudioAnalyzer'
import { useVisualizationRenderer } from './useVisualizationRenderer'

const STYLE_OPTIONS: { key: VisualizationStyle; icon: string }[] = [
  { key: 'bar', icon: 'mdi-equalizer' },
  { key: 'mirror', icon: 'mdi-flip-vertical' },
  { key: 'circular', icon: 'mdi-circle-outline' },
  { key: 'radial', icon: 'mdi-ray-vertex' },
  { key: 'wave', icon: 'mdi-waves' },
  { key: 'wave-fill', icon: 'mdi-chart-line-variant' },
]

const { t } = useI18n()

const currentStyle = ref<VisualizationStyle>('bar')
const canvasRef = ref<HTMLCanvasElement | null>(null)

const { frequencyData, start: startAnalyzer, stop: stopAnalyzer } = useAudioAnalyzer()
const { start: startRenderer, stop: stopRenderer } = useVisualizationRenderer(canvasRef, currentStyle, frequencyData)

const VALID_STYLES: VisualizationStyle[] = ['bar', 'circular', 'wave', 'mirror', 'radial', 'wave-fill']

async function loadStyle() {
  try {
    const saved = await invoke<string | null>('get_setting', { key: 'visualization_style' })
    if (saved === 'particle') {
      // particle 已移除，迁移到 bar
      currentStyle.value = 'bar'
      invoke('set_setting', { key: 'visualization_style', value: 'bar' }).catch(() => {})
    } else if (saved && VALID_STYLES.includes(saved as VisualizationStyle)) {
      currentStyle.value = saved as VisualizationStyle
    }
  } catch { /* use default */ }
}

function setStyle(style: VisualizationStyle) {
  currentStyle.value = style
  invoke('set_setting', { key: 'visualization_style', value: style }).catch(() => {})
}

onMounted(async () => {
  await loadStyle()
  await startAnalyzer()
  startRenderer()
})

onUnmounted(() => {
  stopRenderer()
  stopAnalyzer()
})
</script>

<template>
  <canvas ref="canvasRef" class="visualization-canvas" />
  <div class="style-picker">
    <v-btn-toggle v-model="currentStyle" mandatory density="compact" variant="text" @update:model-value="setStyle">
      <v-btn
        v-for="opt in STYLE_OPTIONS"
        :key="opt.key"
        :value="opt.key"
        size="x-small"
        variant="plain"
        :class="{ 'style-active': currentStyle === opt.key }"
      >
        <v-tooltip activator="parent" location="bottom">{{ t(`player.visualization${opt.key.charAt(0).toUpperCase() + opt.key.slice(1)}`) }}</v-tooltip>
        <v-icon :icon="opt.icon" size="16" />
      </v-btn>
    </v-btn-toggle>
  </div>
</template>

<style scoped>
.visualization-canvas {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  z-index: 0;
  pointer-events: none;
}

.style-picker {
  position: absolute;
  bottom: 20px;
  right: 20px;
  z-index: 2;
  display: flex;
  gap: 2px;
  padding: 4px 8px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 20px;
  backdrop-filter: blur(10px);
}

.style-active {
  opacity: 1 !important;
}

.style-picker :deep(.v-btn) {
  opacity: 0.4;
  transition: opacity 0.2s;
}

.style-picker :deep(.v-btn--active) {
  opacity: 1;
}
</style>
