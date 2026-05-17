<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { usePlayerStore } from '../stores/player'
import { useLibraryStore } from '../stores/library'

defineEmits<{ close: [] }>()

const playerStore = usePlayerStore()
const libraryStore = useLibraryStore()

const {
  currentSong,
  isPlaying,
  progress,
  duration,
  playbackMode,
  queue,
  queueIndex,
} = storeToRefs(playerStore)

const {
  togglePlayPause,
  seek,
  next,
  previous,
  setMode,
  formatTime,
} = playerStore

const { addSongToPlaylist: addSong, removeSongFromPlaylist: removeSong } = libraryStore
const { playlistSongs } = storeToRefs(libraryStore)

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

function cycleMode() {
  const modes: Array<'sequential' | 'repeat_all' | 'repeat_one' | 'shuffle'> =
    ['sequential', 'repeat_all', 'repeat_one', 'shuffle']
  const idx = modes.indexOf(playbackMode.value)
  setMode(modes[(idx + 1) % modes.length])
}

const modeIcons: Record<string, string> = {
  sequential: 'mdi-arrow-right',
  repeat_all: 'mdi-repeat',
  repeat_one: 'mdi-repeat-once',
  shuffle: 'mdi-shuffle',
}

const upNext = computed(() => {
  const start = queueIndex.value + 1
  return queue.value.slice(start, start + 3)
})
</script>

<template>
  <div class="overlay">
    <div class="glow glow-primary" />
    <div class="glow glow-secondary" />
    <v-btn variant="text" size="small" class="close-btn" @click="$emit('close')">
      <v-icon icon="mdi-close" size="16"></v-icon>
      收起
    </v-btn>
    <div class="album-art">
      <v-icon icon="mdi-music-note" size="56" color="rgba(255,255,255,0.6)"></v-icon>
    </div>
    <div class="song-info">
      <div class="song-title">{{ currentSong?.title ?? '未在播放' }}</div>
      <div class="song-artist">{{ currentSong ? `${currentSong.artist} · ${currentSong.album}` : '--' }}</div>
    </div>
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
      <v-btn icon variant="plain" :class="{ muted: playbackMode === 'sequential' }" @click="cycleMode">
        <v-icon :icon="modeIcons[playbackMode] || 'mdi-arrow-right'"></v-icon>
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
    <div v-if="upNext.length" class="up-next">
      <div class="up-next-label">接下来</div>
      <div v-for="song in upNext" :key="song.id" class="up-next-item">
        <div class="up-next-art" :style="{ background: song.artGradient }" />
        <div>
          <div class="up-next-title">{{ song.title }}</div>
          <div class="up-next-artist">{{ song.artist }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: absolute; inset: 0;
  background: rgb(var(--v-theme-background) / 0.82);
  backdrop-filter: blur(40px);
  -webkit-backdrop-filter: blur(40px);
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  z-index: 20; gap: 16px; overflow: hidden;
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

.close-btn { position: absolute; top: 16px; left: 20px; z-index: 1; color: var(--v-text-secondary); animation: fade-up 0.4s 0.05s cubic-bezier(0.16, 1, 0.3, 1) both; }

.album-art {
  width: 200px; height: 200px;
  background: var(--v-gradient-brand); border-radius: 16px;
  display: flex; align-items: center; justify-content: center;
  box-shadow: 0 16px 48px var(--v-accent-shadow);
  position: relative; z-index: 1;
  animation: art-expand 0.5s cubic-bezier(0.16, 1, 0.3, 1) both;
}

.song-info { text-align: center; z-index: 1; animation: fade-up 0.4s 0.1s cubic-bezier(0.16, 1, 0.3, 1) both; }
.song-title { font-size: var(--text-2xl); font-weight: 600; color: rgb(var(--v-theme-on-background)); margin-bottom: 4px; }
.song-artist { font-size: var(--text-md); color: var(--v-text-secondary); }

.progress-section { width: 320px; z-index: 1; animation: fade-up 0.4s 0.15s cubic-bezier(0.16, 1, 0.3, 1) both; }
.time-row { display: flex; justify-content: space-between; margin-top: 6px; font-size: var(--text-xs); color: var(--v-text-muted); }

.controls { display: flex; align-items: center; gap: 12px; z-index: 1; animation: fade-up 0.4s 0.2s cubic-bezier(0.16, 1, 0.3, 1) both; }
.play-btn-lg { box-shadow: 0 4px 20px var(--v-accent-shadow); transition: transform 0.15s, box-shadow 0.15s; }
.play-btn-lg:hover { transform: scale(1.06); }
.muted { opacity: 0.5; }

@keyframes art-expand {
  from { transform: scale(0.24) translateY(120px); opacity: 0; }
}
@keyframes fade-up {
  from { opacity: 0; transform: translateY(16px); }
}

.up-next { animation: fade-up 0.4s 0.25s cubic-bezier(0.16, 1, 0.3, 1) both;
  position: absolute; bottom: 16px; right: 20px; width: 180px;
  background: rgb(var(--v-theme-surface)); opacity: 0.92;
  backdrop-filter: blur(10px); border: 1px solid var(--v-border-color);
  border-radius: 10px; padding: 12px; z-index: 1;
}
.up-next-label { font-size: var(--text-xs); color: var(--v-text-muted); margin-bottom: 10px; text-transform: uppercase; letter-spacing: 0.5px; }
.up-next-item { display: flex; align-items: center; gap: 10px; padding: 6px 0; }
.up-next-item + .up-next-item { border-top: 1px solid var(--v-border-color); padding-top: 8px; }
.up-next-art { width: 32px; height: 32px; border-radius: 6px; flex-shrink: 0; }
.up-next-title { font-size: var(--text-sm); color: rgb(var(--v-theme-on-background)); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.up-next-artist { font-size: var(--text-xs); color: var(--v-text-muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
</style>
