<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { onKeyStroke } from '@vueuse/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { usePlayerStore } from '../stores/player'
import { useMiniPlayer } from '../composables/useMiniPlayer'
import { useI18n } from '../composables/useI18n'
import { useDraggableProgress } from '../composables/useDraggableProgress'
import { applyAccentBroadcast, type AccentBroadcast } from '../composables/useTheme'
import CoverArt from '../components/player/CoverArt.vue'
import AppSlider from '../components/common/AppSlider.vue'

const playerStore = usePlayerStore()
const { t } = useI18n()
const { exit, toggleAlwaysOnTop, alwaysOnTop, restoreFromSettings } = useMiniPlayer()

// Cmd/Ctrl+Shift+M 退出迷你模式（与主窗口的进入快捷键对称；避开 macOS Cmd+M 默认加速器）
onKeyStroke('m', (e) => {
  if ((e.metaKey || e.ctrlKey) && e.shiftKey) {
    e.preventDefault()
    exit()
  }
})

const { currentSong, isPlaying, progress, duration } = storeToRefs(playerStore)
const { togglePlayPause, next, previous, seek, formatTime } = playerStore

const hover = ref(false)

const { progressModel, displayProgress, isDragging } = useDraggableProgress(progress, duration, seek)

// 跟随主窗口动态强调色（独立 webview 需自行监听同步）
let unlistenAccent: UnlistenFn | null = null

onMounted(async () => {
  await restoreFromSettings()
  // 迷你窗口必须自己订阅 audio:* 事件（每个窗口是独立 JS 上下文）
  await playerStore.setupListeners()
  await playerStore.getState()
  unlistenAccent = await listen<AccentBroadcast>('app:accent-changed', (e) => {
    applyAccentBroadcast(e.payload)
    // 同步主题标记类（tokens.css 的 scx-dark/scx-light 挂载点）
    document.documentElement.classList.toggle('scx-dark', e.payload.dark)
    document.documentElement.classList.toggle('scx-light', !e.payload.dark)
  })
})

onUnmounted(() => {
  unlistenAccent?.()
})
</script>

<template>
  <div class="mini-player" @mouseenter="hover = true" @mouseleave="hover = false">
    <!-- 顶部进度线（全宽，兼作拖拽区） -->
    <div class="m-progress" data-tauri-drag-region>
      <AppSlider
        v-model="progressModel"
        variant="line"
        class="m-slider"
        :aria-label="t('player.play')"
        :disabled="!currentSong"
        @start="isDragging = true"
        @end="isDragging = false"
      />
    </div>

    <!-- hover 时右上角窗口控件（置于进度线上方） -->
    <div class="window-controls" :class="{ visible: hover }">
      <button :title="t('miniPlayer.alwaysOnTop')" @click.stop="toggleAlwaysOnTop">
        <v-icon :icon="alwaysOnTop ? 'mdi-pin' : 'mdi-pin-off'" size="13"></v-icon>
      </button>
      <button :title="t('miniPlayer.expand')" @click.stop="exit">
        <v-icon icon="mdi-arrow-expand" size="13"></v-icon>
      </button>
      <!-- 故意调用 exit() 而非 close()：避免误关闭应用，与展开按钮行为一致 -->
      <button :title="t('miniPlayer.close')" @click.stop="exit">
        <v-icon icon="mdi-close" size="13"></v-icon>
      </button>
    </div>

    <!-- 主行：封面 + 元信息 + 控制 -->
    <div class="m-row">
      <span class="m-cover"><CoverArt :song-id="currentSong?.id" /></span>
      <span class="m-meta">
        <span class="m-title">{{ currentSong?.title ?? t('player.notPlaying') }}</span>
        <span class="m-artist num">{{
          currentSong ? `${currentSong.artist} · ${formatTime(displayProgress)} / ${formatTime(duration)}` : ''
        }}</span>
      </span>
      <span class="m-ctl">
        <button class="m-btn" :aria-label="t('player.previous')" :disabled="!currentSong" @click.stop="previous">
          <v-icon icon="mdi-skip-previous" size="17"></v-icon>
        </button>
        <button class="m-play" :aria-label="t('player.play')" :disabled="!currentSong" @click.stop="togglePlayPause">
          <v-icon :icon="isPlaying ? 'mdi-pause' : 'mdi-play'" size="15"></v-icon>
        </button>
        <button class="m-btn" :aria-label="t('player.next')" :disabled="!currentSong" @click.stop="next">
          <v-icon icon="mdi-skip-next" size="17"></v-icon>
        </button>
      </span>
    </div>
  </div>
</template>

<style scoped>
.mini-player {
  width: 100%;
  height: 100%;
  background: #16161d;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  user-select: none;
  position: relative;
  font-family: var(--font-ui);
  color: #f2f2f5;
}

/* 顶部进度线 */
.m-progress {
  height: 10px;
  display: flex;
  align-items: flex-start;
  cursor: default;
}
.m-slider {
  width: 100%;
  height: 10px;
}
.m-slider :deep(.track) {
  border-radius: 0;
}

/* hover 窗口控件 */
.window-controls {
  position: absolute;
  top: 4px;
  right: 4px;
  display: flex;
  opacity: 0;
  transition: opacity 0.15s;
  background: rgba(0, 0, 0, 0.35);
  -webkit-backdrop-filter: blur(10px);
  backdrop-filter: blur(10px);
  border-radius: 0 0 0 8px;
  padding: 0 2px;
  z-index: 5;
}
.window-controls.visible {
  opacity: 1;
}
.window-controls button {
  width: 22px;
  height: 20px;
  display: grid;
  place-items: center;
  color: rgba(242, 242, 245, 0.7);
  background: none;
  border: none;
  cursor: pointer;
}
.window-controls button:hover {
  color: #fff;
}

/* 主行 */
.m-row {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 14px 10px 16px;
  min-height: 0;
}
.m-cover {
  width: 52px;
  height: 52px;
  border-radius: 10px;
  overflow: hidden;
  flex-shrink: 0;
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.4);
  background: var(--surface-2, #23232e);
}
.m-meta {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.m-title {
  font-size: 13px;
  font-weight: 620;
  color: #f2f2f5;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.m-artist {
  font-size: 11px;
  color: rgba(242, 242, 245, 0.56);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 控制按钮 */
.m-ctl {
  display: flex;
  align-items: center;
  gap: 2px;
}
.m-btn {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  display: grid;
  place-items: center;
  color: rgba(242, 242, 245, 0.72);
  background: none;
  border: none;
  cursor: pointer;
}
.m-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.08);
  color: #fff;
}
.m-btn:disabled {
  opacity: 0.4;
  cursor: default;
}
.m-play {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--accent, #009688);
  color: var(--on-accent, #fff);
  display: grid;
  place-items: center;
  border: none;
  cursor: pointer;
  flex-shrink: 0;
}
.m-play:hover:not(:disabled) {
  filter: brightness(1.08);
}
.m-play:disabled {
  opacity: 0.5;
  cursor: default;
}
</style>
