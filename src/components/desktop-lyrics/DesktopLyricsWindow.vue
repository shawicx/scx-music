<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useDesktopLyrics } from '../../composables/useDesktopLyrics'
import LyricLinePair from './LyricLinePair.vue'
import LockBadge from './LockBadge.vue'

const {
  locked,
  config,
  currentSong,
  lines,
  currentLineIndex,
  toggleLock,
  restoreFromSettings,
} = useDesktopLyrics()

const currentLine = computed(() => {
  const idx = currentLineIndex.value
  if (idx < 0 || idx >= lines.value.length) return null
  return lines.value[idx]
})
const nextLine = computed(() => {
  const idx = currentLineIndex.value
  if (idx < 0 || idx + 1 >= lines.value.length) return null
  return lines.value[idx + 1]
})
const hasLyrics = computed(() => lines.value.length > 0)

const rootStyle = computed(() => ({
  background: `rgba(0, 0, 0, ${config.bgOpacity})`,
  opacity: locked.value ? '0.8' : '1',
}))

function onDragStart(e: MouseEvent) {
  if (locked.value) return
  if ((e.target as HTMLElement).closest('.lock-badge')) return
  getCurrentWindow().startDragging()
}

onMounted(async () => {
  await restoreFromSettings()
})
</script>

<template>
  <div class="dl-root" :class="{ locked }" :style="rootStyle" @mousedown="onDragStart">
    <LockBadge v-if="!locked" :locked="locked" @toggle="toggleLock" />

    <LyricLinePair
      v-if="hasLyrics"
      :current="currentLine"
      :next="nextLine"
      :font-size="config.fontSize"
      :color-current="config.colorCurrent"
      :color-next="config.colorNext"
      :glow="config.glowStrength"
    />
    <div v-else class="dl-fallback">
      {{ currentSong?.title ?? '—' }}
    </div>
  </div>
</template>

<style scoped>
.dl-root {
  position: relative;
  width: 100vw;
  height: 100vh;
  padding: 12px 24px;
  box-sizing: border-box;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  overflow: hidden;
  background-color: #fff;
  transition: opacity 0.2s ease, background 0.3s ease;
}
.dl-root:hover .lock-badge {
  opacity: 0.8;
}
.dl-fallback {
  font-size: 22px;
  color: rgba(255, 255, 255, 0.6);
  text-align: center;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  user-select: none;
}
</style>
