<script setup lang="ts">
import type { Song } from '../../types'

defineProps<{
  songs: Song[]
  currentSongId?: string | null
  isPlaying: boolean
}>()

const emit = defineEmits<{
  'songClick': [index: number]
  'songMenu': [event: MouseEvent, songId: string]
}>()
</script>

<template>
  <div class="grid-scroll">
    <div
      v-for="(song, i) in songs"
      :key="song.id"
      class="grid-card"
      :class="{ playing: song.id === currentSongId }"
      @click="emit('songClick', i)"
      @contextmenu="emit('songMenu', $event, song.id)"
    >
      <div class="grid-art" :style="{ background: song.artGradient }">
        <v-icon v-if="song.id === currentSongId" :icon="isPlaying ? 'mdi-play' : 'mdi-pause'" size="24" color="white" class="grid-play-indicator" />
      </div>
      <div class="grid-title">{{ song.title }}</div>
      <div class="grid-artist">{{ song.artist }}</div>
    </div>
  </div>
</template>

<style scoped>
.grid-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 16px;
  align-content: start;
}

.grid-card {
  cursor: pointer;
  transition: transform 0.15s ease;
}

.grid-card:hover {
  transform: translateY(-2px);
}

.grid-card.playing .grid-art {
  box-shadow: 0 8px 24px var(--v-accent-shadow);
}

.grid-art {
  width: 100%;
  aspect-ratio: 1;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 8px;
  position: relative;
}

.grid-play-indicator {
  text-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

.grid-title {
  font-size: var(--text-md);
  font-weight: 500;
  color: rgb(var(--v-theme-on-background));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.grid-artist {
  font-size: var(--text-xs);
  color: var(--v-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>