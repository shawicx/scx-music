<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { onKeyStroke } from '@vueuse/core'
import { usePlayerStore } from '../stores/player'
import { useMiniPlayer } from '../composables/useMiniPlayer'
import { useI18n } from '../composables/useI18n'
import { useDraggableProgress } from '../composables/useDraggableProgress'

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

onMounted(async () => {
  await restoreFromSettings()
  // 迷你窗口必须自己订阅 audio:* 事件（每个窗口是独立 JS 上下文）
  await playerStore.setupListeners()
  await playerStore.getState()
})
</script>

<template>
  <div
    class="mini-player"
    @mouseenter="hover = true"
    @mouseleave="hover = false"
  >
    <!-- 顶部条：拖动区域 + 右上角控件 -->
    <div class="top-bar" data-tauri-drag-region>
      <div class="window-controls" :class="{ visible: hover }" data-tauri-drag-region="false">
        <v-btn
          variant="text"
          size="x-small"
          icon
          @click.stop="toggleAlwaysOnTop"
          :title="t('miniPlayer.alwaysOnTop')"
        >
          <v-icon :icon="alwaysOnTop ? 'mdi-pin' : 'mdi-pin-off'" size="14" />
        </v-btn>
        <v-btn
          variant="text"
          size="x-small"
          icon
          @click.stop="exit"
          :title="t('miniPlayer.expand')"
        >
          <v-icon icon="mdi-arrow-expand" size="14" />
        </v-btn>
        <!-- 故意调用 exit() 而非 close()：避免误关闭应用，与展开按钮行为一致 -->
        <v-btn
          variant="text"
          size="x-small"
          icon
          @click.stop="exit"
          :title="t('miniPlayer.close')"
        >
          <v-icon icon="mdi-close" size="14" />
        </v-btn>
      </div>
    </div>

    <!-- 主内容 -->
    <div class="content">
      <div
        class="cover-art"
        :style="currentSong ? { background: currentSong.artGradient || 'var(--v-gradient-brand)' } : {}"
      >
        <v-icon v-if="!currentSong" icon="mdi-music-note" size="20" color="rgba(255,255,255,0.6)" />
      </div>

      <div class="info-and-controls">
        <div class="meta">
          <div class="title">{{ currentSong?.title ?? t('player.notPlaying') }}</div>
          <div class="artist">{{ currentSong?.artist ?? '--' }}</div>
        </div>

        <div class="controls">
          <v-btn
            variant="text"
            size="x-small"
            icon
            :disabled="!currentSong"
            @click.stop="previous"
          >
            <v-icon icon="mdi-skip-previous" size="18" />
          </v-btn>
          <v-btn
            variant="text"
            size="small"
            icon
            color="secondary"
            :disabled="!currentSong"
            @click.stop="togglePlayPause"
          >
            <v-icon :icon="isPlaying ? 'mdi-pause' : 'mdi-play'" size="20" />
          </v-btn>
          <v-btn
            variant="text"
            size="x-small"
            icon
            :disabled="!currentSong"
            @click.stop="next"
          >
            <v-icon icon="mdi-skip-next" size="18" />
          </v-btn>
        </div>
      </div>
    </div>

    <!-- 底部进度条 -->
    <div class="progress-row">
      <span class="time">{{ formatTime(displayProgress) }}</span>
      <v-slider
        v-model="progressModel"
        :max="100"
        :step="0.1"
        hide-details
        density="compact"
        color="secondary"
        track-color="surface-variant"
        class="progress-slider"
        :disabled="!currentSong"
        @start="isDragging = true"
        @end="isDragging = false"
      />
      <span class="time">{{ formatTime(duration) }}</span>
    </div>
  </div>
</template>

<style scoped>
.mini-player {
  width: 100%;
  height: 100%;
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  display: flex;
  flex-direction: column;
  border-radius: var(--radius-md);
  overflow: hidden;
  user-select: none;
}

.top-bar {
  height: 12px;
  width: 100%;
  position: relative;
}

.window-controls {
  position: absolute;
  top: 0;
  right: 4px;
  display: flex;
  opacity: 0;
  transition: opacity 0.15s;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 0 0 0 8px;
  padding: 0 2px;
}

.window-controls.visible {
  opacity: 1;
}

.content {
  flex: 1;
  display: flex;
  align-items: center;
  padding: 0 12px;
  gap: 10px;
  min-height: 0;
}

.cover-art {
  width: 56px;
  height: 56px;
  border-radius: 6px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--v-gradient-brand, linear-gradient(135deg, #667eea, #764ba2));
}

.info-and-controls {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.meta {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.title {
  font-size: 13px;
  font-weight: 500;
  color: rgb(var(--v-theme-on-surface));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.artist {
  font-size: 11px;
  color: var(--v-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.controls {
  display: flex;
  align-items: center;
}

.progress-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 12px 6px;
  height: 22px;
}

.time {
  font-size: 10px;
  color: var(--v-text-secondary);
  min-width: 28px;
  text-align: center;
}

.progress-slider {
  flex: 1;
}
</style>
