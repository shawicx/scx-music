<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { usePlayerStore } from '../stores/player'
import { useLibraryStore } from '../stores/library'
import IconButtonWithTooltip from './IconButtonWithTooltip.vue'

defineEmits<{ expand: [] }>()

const playerStore = usePlayerStore()
const libraryStore = useLibraryStore()

const {
  currentSong,
  isPlaying,
  progress,
  duration,
  volume,
  playbackMode,
} = storeToRefs(playerStore)

const {
  togglePlayPause,
  seek,
  setVolume,
  next,
  previous,
  setMode,
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

function cycleMode() {
  const modes: Array<'sequential' | 'repeat_all' | 'repeat_one' | 'shuffle'> =
    ['sequential', 'repeat_all', 'repeat_one', 'shuffle']
  const idx = modes.indexOf(playbackMode.value)
  setMode(modes[(idx + 1) % modes.length])
}

// Local state for dragging
const isDragging = ref(false)
const localProgress = ref(0)

const progressModel = computed({
  get: () => {
    if (isDragging.value) {
      return isNaN(localProgress.value) ? 0 : localProgress.value
    }
    if (duration.value > 0) {
      const result = (progress.value / duration.value) * 100
      return isNaN(result) ? 0 : result
    }
    return 0
  },
  set: (val: number) => {
    localProgress.value = val
    if (duration.value > 0 && !isNaN(val)) {
      seek((val / 100) * duration.value)
    }
  },
})

const volumeModel = computed({
  get: () => Math.round(volume.value * 100),
  set: (val: number) => setVolume(val / 100),
})

// Display current time during drag
const displayProgress = computed(() => {
  if (isDragging.value && duration.value > 0) {
    const result = (localProgress.value / 100) * duration.value
    return isNaN(result) ? progress.value : result
  }
  return progress.value
})

const repeatTooltip = computed(() => {
  switch (playbackMode.value) {
    case 'repeat_all':
      return '列表循环'
    case 'repeat_one':
      return '单曲循环'
    case 'shuffle':
      return '随机播放'
    default:
      return '顺序播放'
  }
})

const modeIcons: Record<string, string> = {
  sequential: 'mdi-arrow-right',
  repeat_all: 'mdi-repeat',
  repeat_one: 'mdi-repeat-once',
  shuffle: 'mdi-shuffle',
}

const repeatIcon = computed(() => modeIcons[playbackMode.value] ?? 'mdi-arrow-right')

const isRepeatActive = computed(() => {
  return playbackMode.value === 'repeat_all' || playbackMode.value === 'repeat_one' || playbackMode.value === 'shuffle'
})
</script>

<template>
  <div class="player-bar">
    <div class="player-left" @click="$emit('expand')">
      <div class="cover-art">
        <v-icon v-if="!currentSong" icon="mdi-music-note" size="20" color="rgba(255,255,255,0.6)"></v-icon>
      </div>
      <div class="song-meta">
        <div class="song-name">{{ currentSong?.title ?? '未在播放' }}</div>
        <div class="song-artist">{{ currentSong?.artist ?? '--' }}</div>
      </div>
      <IconButtonWithTooltip
        :icon="isLiked ? 'mdi-heart' : 'mdi-heart-outline'"
        icon-active="mdi-heart"
        :active="isLiked"
        :tooltip="() => isLiked ? '取消喜欢' : '添加到喜欢'"
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
        tooltip="上一曲"
        @click.stop="previous"
      />

      <IconButtonWithTooltip
        :icon="isPlaying ? 'mdi-pause' : 'mdi-play'"
        icon-active="mdi-pause"
        :active="isPlaying"
        tooltip="播放"
        color="secondary"
        size="small"
        class="play-btn"
        :disabled="!currentSong"
        @click.stop="togglePlayPause"
      />

      <IconButtonWithTooltip
        icon="mdi-skip-next"
        tooltip="下一曲"
        @click.stop="next"
      />

      <IconButtonWithTooltip
        :icon="repeatIcon"
        :active="isRepeatActive"
        :tooltip="repeatTooltip"
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
        icon="mdi-playlist-music"
        tooltip="播放列表"
        size="x-small"
      />
      <!-- <v-icon size="14" class="volume-icon">mdi-volume-high</v-icon> -->
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
  background: linear-gradient(transparent, rgb(var(--v-theme-background)));
  backdrop-filter: blur(20px);
  border-top: 1px solid var(--v-border-color);
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
  border-radius: 8px;
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
}
.like-btn { margin-left: 4px; }

.player-center { display: flex; flex-direction: column; align-items: center; flex: 1; gap: 0; padding: 0 24px; }
.controls { display: flex; align-items: center; gap: 0.5rem; }
.play-btn { box-shadow: 0 4px 12px var(--v-accent-shadow); transition: transform 0.2s; }
.play-btn:hover { transform: scale(1.1); }

.progress-row { display: flex; align-items: center; gap: 8px; width: 100%; max-width: 600px; }
.time { font-size: var(--text-xs); color: var(--v-text-secondary); min-width: 32px; text-align: center; }
.progress-slider { flex: 1; }

.player-right { display: flex; align-items: center; gap: 4px; min-width: 180px; justify-content: flex-end; }
.volume-icon { color: var(--v-text-secondary); opacity: 0.7; }
.volume-slider { width: 80px; }
</style>
