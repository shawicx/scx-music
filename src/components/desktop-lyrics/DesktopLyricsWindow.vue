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
  isDark,
  themeColor,
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
  // 网易云风格:纯文字悬浮,窗口背景恒透明(窗口本身 transparent:true)。
  // bgOpacity 配置保留但不再生效(向后兼容旧 settings 值,避免读取报错)。
  background: 'transparent',
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
  <div class="dl-root" :class="{ locked, 'dl-dark': isDark, 'dl-light': !isDark }" :style="rootStyle" @mousedown="onDragStart">
    <LockBadge v-if="!locked" :locked="locked" :is-dark="isDark" @toggle="toggleLock" />

    <LyricLinePair
      v-if="hasLyrics"
      :current="currentLine"
      :next="nextLine"
      :font-size="config.fontSize"
      :color-current="config.colorCurrent"
      :color-next="config.colorNext"
      :glow="config.glowStrength"
      :theme-color="themeColor"
      :is-dark="isDark"
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
  border-radius: var(--radius-lg);
  overflow: hidden;
  transition: opacity 0.2s ease, background 0.3s ease;
}
/* fallback 文字色:仅当用户未自定义 colorCurrent 时由子组件继承。
   LyricLinePair 的 inline style 优先级更高,会覆盖此处。 */
.dl-dark { color: #fff; }
.dl-light { color: #111; }
.dl-root:hover .lock-badge {
  opacity: 0.8;
}
.dl-fallback {
  font-size: 22px;
  /* 继承明暗模式色,加透明度做次级文字 */
  opacity: 0.6;
  text-align: center;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  user-select: none;
}
</style>
