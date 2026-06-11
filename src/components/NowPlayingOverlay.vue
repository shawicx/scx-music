<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { usePlayerStore } from '../stores/player'
import { useLibraryStore } from '../stores/library'
import { usePlaybackMode } from '../composables/usePlaybackMode'
import { useI18n } from '../composables/useI18n'
import { useLyrics } from '../composables/useLyrics'
import { AudioVisualizer } from '../visualization'
import LyricsDisplay from './LyricsDisplay.vue'

defineEmits<{ close: [] }>()

const playerStore = usePlayerStore()
const { t } = useI18n()
const libraryStore = useLibraryStore()
const { modeIcon, modeLabel, isModeActive, cycleMode } = usePlaybackMode()

const {
  currentSong,
  isPlaying,
  progress,
  duration,
} = storeToRefs(playerStore)

const {
  togglePlayPause,
  seek,
  next,
  previous,
  formatTime,
} = playerStore

const { addSongToPlaylist: addSong, removeSongFromPlaylist: removeSong } = libraryStore
const { playlistSongs } = storeToRefs(libraryStore)

const { lines, currentLineIndex, isLoading, offsetSecs, adjustOffset, resetOffset, getSeekTime } = useLyrics(currentSong)

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

const progressModel = computed({
  get: () => duration.value > 0 ? (progress.value / duration.value) * 100 : 0,
  set: (val: number) => { if (duration.value > 0) seek((val / 100) * duration.value) },
})

function onLyricSeek(time: number) {
  seek(time)
}
</script>

<template>
  <div class="overlay">
    <div class="glow glow-primary" />
    <div class="glow glow-secondary" />
    <AudioVisualizer />
    <div class="mode-status-bar" v-if="isModeActive">
      <div class="status-item">
        <v-icon :icon="modeIcon" size="14" color="secondary"></v-icon>
        <span>{{ modeLabel }}</span>
      </div>
    </div>
    <v-btn variant="text" size="small" class="close-btn" @click="$emit('close')">
      <v-icon icon="mdi-close" size="16"></v-icon>
      {{ t('player.collapse') }}
    </v-btn>
    <div class="top-section">
        <div class="song-title">{{ currentSong?.title ?? t('player.notPlaying') }}</div>
        <div class="song-artist">{{ currentSong ? `${currentSong.artist} · ${currentSong.album}` : '--' }}</div>
    </div>
    <LyricsDisplay
      :lines="lines"
      :current-line-index="currentLineIndex"
      :is-loading="isLoading"
      :offset-secs="offsetSecs"
      :adjust-offset="adjustOffset"
      :reset-offset="resetOffset"
      :get-seek-time="getSeekTime"
      @seek="onLyricSeek"
    />
    <div class="progress-section">
      <v-slider
        v-model="progressModel"
        :max="100"
        :step="0.1"
        hide-details
        density="compact"
        color="secondary"
        track-color="surface-variant"
        class="progress-slider"
      />
      <div class="time-row">
        <span>{{ formatTime(progress) }}</span>
        <span>{{ formatTime(duration) }}</span>
      </div>
    </div>
    <div class="controls">
      <v-btn icon variant="plain" :class="{ muted: !isModeActive }" @click="cycleMode">
        <v-icon :icon="modeIcon"></v-icon>
      </v-btn>
      <v-btn icon variant="plain" @click="previous">
        <v-icon icon="mdi-skip-previous"></v-icon>
      </v-btn>
      <v-btn icon size="x-large" color="secondary" elevation="8" class="play-btn-lg" @click="togglePlayPause">
        <v-icon size="large" :icon="isPlaying ? 'mdi-pause' : 'mdi-play'" color="white"></v-icon>
      </v-btn>
      <v-btn icon variant="plain" @click="next">
        <v-icon icon="mdi-skip-next"></v-icon>
      </v-btn>
      <v-btn icon variant="plain" :disabled="!currentSong" @click="toggleLike">
        <v-icon :icon="isLiked ? 'mdi-heart' : 'mdi-heart-outline'" :color="isLiked ? 'secondary' : undefined"></v-icon>
      </v-btn>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: absolute; inset: 0;
  background: rgb(var(--v-theme-background) / 0.82);
  backdrop-filter: blur(40px);
  -webkit-backdrop-filter: blur(40px);
  display: flex; flex-direction: column; align-items: center;
  z-index: 20; overflow: hidden;
  padding: 48px 24px 36px;
}

.glow { position: absolute; border-radius: 50%; pointer-events: none; }
.glow-primary {
  top: -80px; left: 50%; transform: translateX(-50%);
  width: 500px; height: 400px;
  background: radial-gradient(ellipse at center, var(--v-accent-glow), transparent 70%);
  filter: blur(40px);
}
.glow-secondary {
  bottom: -60px; right: -40px;
  width: 350px; height: 300px;
  background: radial-gradient(ellipse at center, var(--v-accent-glow), transparent 70%);
  filter: blur(40px);
}

.close-btn { position: absolute; top: 16px; left: 20px; z-index: 1; color: var(--v-text-secondary); }

.mode-status-bar {
  position: absolute; top: 16px; right: 20px; z-index: 1;
}
.status-item {
  display: flex; align-items: center; gap: 6px;
  padding: 6px 12px;
  background: rgb(var(--v-theme-surface-variant) / 0.3);
  border-radius: 16px; font-size: var(--text-xs); color: var(--v-text-secondary);
}

.top-section {
  display: flex; align-items: center; gap: 16px;
  width: 100%; z-index: 1; flex-shrink: 0;
  justify-content: center;
}
.album-art {
  width: 120px; height: 120px;
  background: var(--v-gradient-brand); border-radius: 14px;
  display: flex; align-items: center; justify-content: center;
  box-shadow: 0 12px 36px var(--v-accent-shadow);
  flex-shrink: 0;
}
.song-info { flex: 1; min-width: 0; z-index: 1; }
.song-title {
  font-size: var(--text-xl); font-weight: 600;
  color: rgb(var(--v-theme-on-background)); margin-bottom: 4px;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.song-artist {
  font-size: var(--text-md); color: var(--v-text-secondary);
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}

.progress-section { width: 100%; z-index: 1; flex-shrink: 0; }
.time-row { display: flex; justify-content: space-between; margin-top: 6px; font-size: var(--text-xs); color: var(--v-text-muted); }

.controls { display: flex; align-items: center; gap: 12px; z-index: 1; flex-shrink: 0; }
.play-btn-lg { box-shadow: 0 4px 20px var(--v-accent-shadow); transition: transform 0.15s, box-shadow 0.15s; }
.play-btn-lg:hover { transform: scale(1.06); }
.muted { opacity: 0.5; }
</style>
