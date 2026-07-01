<script setup lang="ts">
import { ref, computed } from 'vue'
import { storeToRefs } from 'pinia'
import { usePlayerStore } from '../stores/player'
import { useLibraryStore } from '../stores/library'
import IconButtonWithTooltip from './IconButtonWithTooltip.vue'
import PulseDots from './common/PulseDots.vue'
import { usePlaybackMode } from '../composables/usePlaybackMode'
import { useDesktopLyrics } from '../composables/useDesktopLyrics'
import { useDraggableProgress } from '../composables/useDraggableProgress'
import { useMiniPlayer } from '../composables/useMiniPlayer'
import { useSleepTimer } from '../composables/useSleepTimer'
import { useI18n } from '../composables/useI18n'

const emit = defineEmits<{ expand: []; toggleQueue: [] }>()

const playerStore = usePlayerStore()
const libraryStore = useLibraryStore()
const { modeIcon, modeLabel, isModeActive, cycleMode } = usePlaybackMode()
const { visible: desktopLyricsVisible, toggle: toggleDesktopLyrics } = useDesktopLyrics()
const { enter: enterMini } = useMiniPlayer()
const { isActive: sleepTimerActive, remainingSecs, totalMinutes, formatRemaining, start: startSleepTimer, cancel: cancelSleepTimer } = useSleepTimer()
const { t } = useI18n()

// ── 睡眠定时器 ──────────────────────────────────────────────────────────────
const sleepMenuOpen = ref(false)
const durationOptions = [
  { minutes: 15 },
  { minutes: 30 },
  { minutes: 45 },
  { minutes: 60 },
]
const sleepTooltip = computed(() =>
  sleepTimerActive.value
    ? t('player.sleepTimer.remaining', { time: formatRemaining(remainingSecs.value) })
    : t('player.sleepTimer.title'),
)
function onSelectSleepDuration(minutes: number) {
  startSleepTimer(minutes)
  sleepMenuOpen.value = false
}
function onCancelSleepTimer() {
  cancelSleepTimer()
  sleepMenuOpen.value = false
}

const {
  currentSong,
  isPlaying,
  progress,
  duration,
  volume,
} = storeToRefs(playerStore)

const {
  togglePlayPause,
  seek,
  setVolume,
  next,
  previous,
  formatTime,
} = playerStore

const { playlistSongs } = storeToRefs(libraryStore)
const { addSongToPlaylist: addSong, removeSongFromPlaylist: removeSong } = libraryStore

const isLiked = computed(() => {
  if (!currentSong.value) return false
  const favIds = playlistSongs.value['fav']
  return favIds?.includes(currentSong.value.id) ?? false
})

async function toggleLike() {
  if (!currentSong.value) return
  if (isLiked.value) {
    await removeSong('fav', currentSong.value.id)
  } else {
    await addSong('fav', currentSong.value.id)
  }
}

const volumeModel = computed({
  get: () => Math.round(volume.value * 100),
  set: (val: number) => setVolume(val / 100),
})

const { progressModel, displayProgress, isDragging } = useDraggableProgress(progress, duration, seek)
</script>

<template>
  <div class="player-bar">
    <div class="player-left" @click="$emit('expand')">
      <div class="cover-art">
        <v-icon v-if="!currentSong" icon="mdi-music-note" size="20" color="rgba(255,255,255,0.6)"></v-icon>
      </div>
      <div class="song-meta">
        <div class="song-name">{{ currentSong?.title ?? t('player.notPlaying') }}</div>
        <div class="song-artist">
          {{ currentSong?.artist ?? '--' }}
          <PulseDots v-if="isPlaying && currentSong" :size="4" :gap="3" class="pulse-inline" />
        </div>
      </div>
      <IconButtonWithTooltip
        :icon="isLiked ? 'mdi-heart' : 'mdi-heart-outline'"
        icon-active="mdi-heart"
        :active="isLiked"
        :tooltip="() => isLiked ? t('player.unlike') : t('player.addToFavorite')"
        :disabled="!currentSong"
        size="x-small"
        class="like-btn"
        @click.stop="toggleLike"
      />
    </div>
    <div class="player-center">
      <div class="controls">
        <IconButtonWithTooltip
        icon="mdi-skip-previous"
        :tooltip="t('player.previous')"
        @click.stop="previous"
      />

      <IconButtonWithTooltip
        :icon="isPlaying ? 'mdi-pause' : 'mdi-play'"
        icon-active="mdi-pause"
        :active="isPlaying"
        :tooltip="t('player.play')"
        color="secondary"
        size="small"
        class="play-btn"
        :disabled="!currentSong"
        @click.stop="togglePlayPause"
      />

      <IconButtonWithTooltip
        icon="mdi-skip-next"
        :tooltip="t('player.next')"
        @click.stop="next"
      />

      <IconButtonWithTooltip
        :icon="modeIcon"
        :active="isModeActive"
        :tooltip="modeLabel"
        color="secondary"
        @click.stop="cycleMode"
      />
      </div>
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
          @start="isDragging = true"
          @end="isDragging = false"
        />
        <span class="time">{{ formatTime(duration) }}</span>
      </div>
    </div>
    <div class="player-right">
      <IconButtonWithTooltip
        icon="mdi-chevron-double-up"
        :tooltip="t('miniPlayer.enter')"
        size="small"
        @click.stop="enterMini"
      />
      <IconButtonWithTooltip
        icon="mdi-monitor-eye"
        icon-active="mdi-monitor-eye"
        :active="desktopLyricsVisible"
        :tooltip="t('lyrics.desktopLyrics.toggle')"
        color="secondary"
        size="small"
        @click.stop="toggleDesktopLyrics"
      />
      <IconButtonWithTooltip
        icon="mdi-playlist-music"
        :tooltip="t('player.playlist')"
        size="small"
        @click.stop="emit('toggleQueue')"
      />
      <v-menu v-model="sleepMenuOpen" :close-on-content-click="false" location="top">
        <template #activator="{ props: menuProps }">
          <IconButtonWithTooltip
            v-bind="menuProps"
            icon="mdi-power-sleep"
            :active="sleepTimerActive"
            :tooltip="sleepTooltip"
            color="secondary"
            size="small"
          />
        </template>
        <v-list density="compact" min-width="160">
          <v-list-subheader>{{ t('player.sleepTimer.title') }}</v-list-subheader>
          <v-list-item
            v-for="opt in durationOptions"
            :key="opt.minutes"
            :title="t('player.sleepTimer.minutes', { count: opt.minutes })"
            :active="totalMinutes === opt.minutes"
            @click="onSelectSleepDuration(opt.minutes)"
          />
          <v-divider />
          <v-list-item
            v-if="sleepTimerActive"
            prepend-icon="mdi-close"
            :title="t('player.sleepTimer.cancel')"
            base-color="error"
            @click="onCancelSleepTimer"
          />
        </v-list>
      </v-menu>
      <v-slider
        v-model="volumeModel"
        :max="100"
        hide-details
        density="compact"
        color="secondary"
        track-color="surface-variant"
        class="volume-slider"
      />
    </div>
  </div>
</template>

<style scoped>
.player-bar {
  height: 72px;
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  border-top: 1px solid var(--glass-border);
  display: flex;
  align-items: center;
  padding: 0 16px;
  z-index: 10;
}

.player-left {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 240px;
  cursor: pointer;
}

.cover-art {
  width: 48px; height: 48px;
  background: var(--v-gradient-brand);
  border-radius: var(--radius-md);
  box-shadow: 0 4px 12px var(--v-accent-shadow);
  flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
}

.song-meta { display: flex; flex-direction: column; gap: 0.2rem; overflow: hidden; }
.song-name {
  font-size: var(--text-md); font-weight: 500; color: rgb(var(--v-theme-on-background));
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.song-artist {
  font-size: var(--text-sm); color: var(--v-text-secondary);
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  display: flex; align-items: center; gap: 6px;
}
.pulse-inline { flex-shrink: 0; }
.like-btn { margin-left: 4px; }

.player-center { display: flex; flex-direction: column; align-items: center; flex: 1; gap: 0; padding: 0 24px; }
.controls { display: flex; align-items: center; gap: 0.5rem; }
.play-btn { box-shadow: var(--shadow-accent); transition: transform 0.2s; }
.play-btn:hover { transform: scale(1.1); }

.progress-row { display: flex; align-items: center; gap: 8px; width: 100%; max-width: 600px; }
.time { font-size: var(--text-xs); color: var(--v-text-secondary); min-width: 32px; text-align: center; }
.progress-slider { flex: 1; }

.player-right {
  display: flex; align-items: center; gap: var(--space-sm);
  min-width: 180px; justify-content: flex-end;
}
/* 音量滑块与功能按钮组分隔:竖向分隔线,视觉上"按钮 | 音量" */
.volume-slider {
  width: 80px;
  margin-left: var(--space-xs);
  padding-left: var(--space-sm);
  border-left: 1px solid var(--glass-border);
}

/* 窄屏:音量滑块收窄 */
@media (max-width: 900px) {
  .volume-slider { width: 64px; }
}
/* 极窄屏:隐藏音量滑块(音量用 ArrowUp/Down 快捷键调节) */
@media (max-width: 720px) {
  .volume-slider { display: none; }
}
</style>
