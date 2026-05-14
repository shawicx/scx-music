<script setup lang="ts">
import { computed } from 'vue'
import { usePlayer } from '../composables/usePlayer'

defineEmits<{ expand: [] }>()

const {
  currentSong, isPlaying, progress, duration, volume,
  togglePlayPause, seek, setVolume, next, previous,
  formatTime,
} = usePlayer()

const progressModel = computed({
  get: () => duration.value > 0 ? (progress.value / duration.value) * 100 : 0,
  set: (val: number) => { if (duration.value > 0) seek((val / 100) * duration.value) },
})

const volumeModel = computed({
  get: () => Math.round(volume.value * 100),
  set: (val: number) => setVolume(val / 100),
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
      <v-btn icon size="x-small" variant="plain" class="like-btn">
        <v-icon icon="mdi-heart" size="16" color="secondary"></v-icon>
      </v-btn>
    </div>
    <div class="player-center">
      <div class="controls">
        <v-btn icon size="small" variant="plain" density="compact" disabled>
          <v-icon icon="mdi-shuffle" ></v-icon>
        </v-btn>
        <v-btn icon size="small" variant="plain" density="compact" @click.stop="previous">
          <v-icon icon="mdi-skip-previous"></v-icon>
        </v-btn>
        <v-btn icon size="small" color="secondary" elevation="4" class="play-btn" @click.stop="togglePlayPause">
          <v-icon :icon="isPlaying ? 'mdi-pause' : 'mdi-play'"  color="white"></v-icon>
        </v-btn>
        <v-btn icon size="small" variant="plain" density="compact" @click.stop="next">
          <v-icon icon="mdi-skip-next"></v-icon>
        </v-btn>
        <v-btn icon size="small" variant="plain" density="compact" disabled>
          <v-icon icon="mdi-repeat"></v-icon>
        </v-btn>
      </div>
      <div class="progress-row">
        <span class="time">{{ formatTime(progress) }}</span>
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
        <span class="time">{{ formatTime(duration) }}</span>
      </div>
    </div>
    <div class="player-right">
      <v-btn icon size="x-small" variant="plain" density="compact">
        <v-icon icon="mdi-playlist-music" size="14"></v-icon>
      </v-btn>
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
