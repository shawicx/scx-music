<script setup lang="ts">
import { ref, computed } from 'vue'
import { storeToRefs } from 'pinia'
import { usePlayerStore } from '../stores/player'
import { useLibraryStore } from '../stores/library'
import IconButtonWithTooltip from './IconButtonWithTooltip.vue'
import CoverArt from './player/CoverArt.vue'
import AppSlider from './common/AppSlider.vue'
import { usePlaybackMode } from '../composables/usePlaybackMode'
import { useDesktopLyrics } from '../composables/useDesktopLyrics'
import { useDraggableProgress } from '../composables/useDraggableProgress'
import { useMiniPlayer } from '../composables/useMiniPlayer'
import { useSleepTimer } from '../composables/useSleepTimer'
import { useI18n } from '../composables/useI18n'

const emit = defineEmits<{ expand: []; toggleQueue: [] }>()

const playerStore = usePlayerStore()
const libraryStore = useLibraryStore()
const { playbackMode, toggleShuffle, cycleRepeat } = usePlaybackMode()
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

// 静音切换：记住静音前音量，取消静音时恢复（音量为 0 视为已静音）
const lastVolume = ref(0.8)
const isMuted = computed(() => volume.value === 0)
function toggleMute() {
  if (volume.value > 0) {
    lastVolume.value = volume.value
    setVolume(0)
  } else {
    setVolume(lastVolume.value || 0.8)
  }
}

const { progressModel, displayProgress, isDragging } = useDraggableProgress(progress, duration, seek)

// 播放模式双按钮状态
const shuffleOn = computed(() => playbackMode.value === 'shuffle')
const repeatLabel = computed(() =>
  playbackMode.value === 'repeat_one'
    ? t('playbackMode.repeatOne')
    : playbackMode.value === 'repeat_all'
      ? t('playbackMode.repeatAll')
      : t('playbackMode.off'),
)
</script>

<template>
  <footer class="player-bar" aria-label="播放控制">
    <!-- 左：封面 + 元信息 + 收藏 -->
    <div class="pb-left">
      <button class="pb-cover" :disabled="!currentSong" aria-label="展开正在播放" @click.stop="emit('expand')">
        <CoverArt :song-id="currentSong?.id" class="cover-fill" />
      </button>
      <button class="pb-meta" :disabled="!currentSong" @click.stop="emit('expand')">
        <span class="pb-title">{{ currentSong?.title ?? t('player.notPlaying') }}</span>
        <span class="pb-artist">{{ currentSong?.artist ?? t('player.notPlaying') }}</span>
      </button>
      <IconButtonWithTooltip
        :icon="isLiked ? 'mdi-heart' : 'mdi-heart-outline'"
        icon-active="mdi-heart"
        :active="isLiked"
        :tooltip="() => (isLiked ? t('player.unlike') : t('player.addToFavorite'))"
        :disabled="!currentSong"
        size="x-small"
        class="pb-heart"
        @click.stop="toggleLike"
      />
    </div>

    <!-- 中：走带 + 进度线 -->
    <div class="pb-center">
      <div class="pb-transport">
        <IconButtonWithTooltip
          icon="mdi-shuffle"
          :active="shuffleOn"
          :tooltip="t('playbackMode.shuffle')"
          size="x-small"
          @click.stop="toggleShuffle"
        />
        <IconButtonWithTooltip
          icon="mdi-skip-previous"
          :tooltip="t('player.previous')"
          size="small"
          @click.stop="previous"
        />
        <button
          class="play-main"
          :disabled="!currentSong"
          :aria-label="t('player.play')"
          @click.stop="togglePlayPause"
        >
          <v-icon :icon="isPlaying ? 'mdi-pause' : 'mdi-play'" size="18"></v-icon>
        </button>
        <IconButtonWithTooltip
          icon="mdi-skip-next"
          :tooltip="t('player.next')"
          size="small"
          @click.stop="next"
        />
        <div class="repeat-wrap">
          <IconButtonWithTooltip
            :icon="playbackMode === 'repeat_one' ? 'mdi-repeat-once' : 'mdi-repeat'"
            :active="playbackMode === 'repeat_all' || playbackMode === 'repeat_one'"
            :tooltip="repeatLabel"
            size="x-small"
            @click.stop="cycleRepeat"
          />
        </div>
      </div>
      <div class="pb-progress">
        <span class="pb-time num">{{ formatTime(displayProgress) }}</span>
        <AppSlider
          v-model="progressModel"
          variant="line"
          class="pb-slider"
          :aria-label="t('player.play')"
          :disabled="!currentSong"
          @start="isDragging = true"
          @end="isDragging = false"
        />
        <span class="pb-time num">{{ formatTime(duration) }}</span>
      </div>
    </div>

    <!-- 右：功能键 + 音量 -->
    <div class="pb-right">
      <IconButtonWithTooltip
        icon="mdi-subtitles-outline"
        icon-active="mdi-subtitles"
        :active="desktopLyricsVisible"
        :tooltip="t('lyrics.desktopLyrics.toggle')"
        size="small"
        @click.stop="toggleDesktopLyrics"
      />
      <IconButtonWithTooltip
        icon="mdi-playlist-music-outline"
        icon-active="mdi-playlist-music"
        :tooltip="t('player.playQueue')"
        size="small"
        @click.stop="emit('toggleQueue')"
      />
      <IconButtonWithTooltip
        icon="mdi-picture-in-picture-top-right"
        :tooltip="t('miniPlayer.enter')"
        size="small"
        @click.stop="enterMini"
      />
      <v-menu v-model="sleepMenuOpen" :close-on-content-click="false" location="top">
        <template #activator="{ props: menuProps }">
          <IconButtonWithTooltip
            v-bind="menuProps"
            icon="mdi-power-sleep"
            :active="sleepTimerActive"
            :tooltip="sleepTooltip"
            size="small"
            @click.stop
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
      <div class="vol-group">
        <IconButtonWithTooltip
          :icon="isMuted ? 'mdi-volume-off' : 'mdi-volume-high'"
          :active="isMuted"
          :tooltip="() => (isMuted ? t('player.unmute') : t('player.mute'))"
          size="small"
          @click.stop="toggleMute"
        />
        <AppSlider
          v-model="volumeModel"
          variant="mini"
          :aria-label="t('player.mute')"
        />
      </div>
    </div>
  </footer>
</template>

<style scoped>
.player-bar {
  height: 100%;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 0 18px 0 16px;
  background: var(--glass-bg);
  -webkit-backdrop-filter: blur(var(--glass-blur)) saturate(1.5);
  backdrop-filter: blur(var(--glass-blur)) saturate(1.5);
  border-top: var(--hairline-w) solid var(--hairline);
}
:global(.scx-light) .player-bar {
  background: var(--surface-0);
  -webkit-backdrop-filter: none;
  backdrop-filter: none;
}

/* 左段 */
.pb-left {
  display: flex;
  align-items: center;
  gap: 11px;
  width: 290px;
  min-width: 0;
}
.pb-cover {
  width: 44px;
  height: 44px;
  border-radius: 8px;
  overflow: hidden;
  flex-shrink: 0;
  border: var(--hairline-w) solid var(--hairline);
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.3);
  cursor: pointer;
  padding: 0;
  background: none;
}
.pb-cover:disabled {
  cursor: default;
}
.cover-fill {
  width: 100%;
  height: 100%;
}
.pb-meta {
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 1px;
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  font-family: inherit;
  text-align: left;
}
.pb-meta:disabled {
  cursor: default;
}
.pb-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}
.pb-artist {
  font-size: 11.5px;
  color: var(--text-2);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}
.pb-meta:not(:disabled):hover .pb-title {
  color: var(--accent);
}
.pb-heart {
  flex-shrink: 0;
}
.pb-heart :deep(.ib-active),
.pb-heart :deep(.v-btn--variant-plain .v-icon) {
  color: var(--accent);
}

/* 中段 */
.pb-center {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 0 8px;
}
.pb-transport {
  display: flex;
  align-items: center;
  gap: 6px;
}
.play-main {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: var(--accent);
  color: var(--on-accent);
  display: grid;
  place-items: center;
  border: none;
  cursor: pointer;
  flex-shrink: 0;
  transition: transform 0.16s var(--ease), filter 0.16s;
  box-shadow: 0 2px 10px rgb(var(--accent-rgb) / 0.35);
}
.play-main:hover:not(:disabled) {
  transform: scale(1.06);
  filter: brightness(1.06);
}
.play-main:active:not(:disabled) {
  transform: scale(0.95);
}
.play-main:disabled {
  opacity: 0.55;
  cursor: default;
}
/* 激活态按钮：accent 着色 */
.pb-transport :deep(.ib-active .v-icon),
.pb-right :deep(.ib-active .v-icon) {
  color: var(--accent);
}
.pb-progress {
  display: flex;
  align-items: center;
  gap: 9px;
  width: 100%;
  max-width: 560px;
}
.pb-slider {
  flex: 1;
}
.pb-time {
  font-size: 11px;
  color: var(--text-3);
  min-width: 34px;
}
.pb-time:last-child {
  text-align: left;
}

/* 右段 */
.pb-right {
  display: flex;
  align-items: center;
  gap: 2px;
  width: 290px;
  justify-content: flex-end;
}
.vol-group {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: 6px;
  padding-left: 8px;
  border-left: var(--hairline-w) solid var(--hairline);
}

/* 窄窗口：隐藏次要控件（音量用 ArrowUp/Down 快捷键调节） */
@media (max-width: 960px) {
  .pb-left {
    width: 220px;
  }
  .pb-right {
    width: auto;
  }
  .vol-group {
    display: none;
  }
}
@media (max-width: 780px) {
  .pb-left {
    width: auto;
    flex: 1;
  }
  .pb-heart {
    display: none;
  }
  .pb-progress .pb-time {
    display: none;
  }
}
</style>
