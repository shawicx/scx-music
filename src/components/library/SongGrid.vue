<script setup lang="ts">
import type { Song } from '../../types'
import CoverArt from '../player/CoverArt.vue'

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
      :data-song-id="song.id"
      class="gcard"
      :class="{ current: song.id === currentSongId }"
      role="button"
      tabindex="0"
      :aria-label="song.title"
      @click="emit('songClick', i)"
      @keydown.enter="emit('songClick', i)"
      @contextmenu="emit('songMenu', $event, song.id)"
    >
      <div class="gc-cover">
        <CoverArt :song-id="song.id" />
        <span class="gc-play" aria-hidden="true">
          <v-icon :icon="song.id === currentSongId && isPlaying ? 'mdi-pause' : 'mdi-play'" size="16"></v-icon>
        </span>
      </div>
      <div class="gc-name">{{ song.title }}</div>
      <div class="gc-sub">{{ song.artist }}</div>
    </div>
  </div>
</template>

<style scoped>
.grid-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 12px 28px 28px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(158px, 1fr));
  gap: 20px;
  align-content: start;
}

.gcard {
  cursor: pointer;
}
.gc-cover {
  position: relative;
  aspect-ratio: 1;
  border-radius: var(--radius-lg);
  overflow: hidden;
  background: var(--surface-2);
  box-shadow: var(--cover-shadow);
}
.gc-play {
  position: absolute;
  right: 10px;
  bottom: 10px;
  width: 38px;
  height: 38px;
  border-radius: 50%;
  background: rgba(20, 20, 26, 0.72);
  color: #fff;
  -webkit-backdrop-filter: blur(14px);
  backdrop-filter: blur(14px);
  border: 1px solid rgba(255, 255, 255, 0.16);
  display: grid;
  place-items: center;
  opacity: 0;
  transform: translateY(6px);
  transition: opacity 0.18s var(--ease), transform 0.18s var(--ease);
}
.gcard:hover .gc-play,
.gcard:focus-visible .gc-play {
  opacity: 1;
  transform: none;
}
.gcard.current .gc-play {
  opacity: 1;
  transform: none;
  background: var(--accent);
  color: var(--on-accent);
  border-color: transparent;
}
.gc-name {
  margin-top: 10px;
  font-size: 13px;
  font-weight: 580;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.gc-sub {
  font-size: 11.5px;
  color: var(--text-3);
  margin-top: 1px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.gcard.current .gc-name {
  color: var(--accent);
}
</style>
