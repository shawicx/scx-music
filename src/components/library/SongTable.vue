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
  <div class="table-scroll">
    <div class="table-header">
      <div class="col col-num">#</div>
      <div class="col col-title">标题</div>
      <div class="col col-album">专辑</div>
      <div class="col col-artist">艺术家</div>
      <div class="col col-duration">时长</div>
      <div class="col col-actions"></div>
    </div>
    <div class="table-body">
      <div
        v-for="(song, i) in songs"
        :key="song.id"
        class="table-row"
        :class="{ playing: song.id === currentSongId }"
        @click="emit('songClick', i)"
        @contextmenu="emit('songMenu', $event, song.id)"
      >
        <div class="col col-num">
          <v-icon v-if="song.id === currentSongId && isPlaying" size="12" icon="mdi-play" color="secondary"></v-icon>
          <v-icon v-else-if="song.id === currentSongId" size="12" icon="mdi-pause" color="secondary"></v-icon>
          <span v-else class="row-num">{{ i + 1 }}</span>
        </div>
        <div class="col col-title">
          <div class="song-art" :style="{ background: song.artGradient }" />
          <div class="song-info">
            <div class="song-title" :class="{ active: song.id === currentSongId }">{{ song.title }}</div>
            <div class="song-quality">{{ song.quality }}</div>
          </div>
        </div>
        <div class="col col-album">{{ song.album }}</div>
        <div class="col col-artist">{{ song.artist }}</div>
        <div class="col col-duration">{{ song.duration }}</div>
        <div class="col col-actions" @click.stop>
          <v-btn icon size="x-small" variant="plain" density="compact" @click="emit('songMenu', $event, song.id)">
            <v-icon icon="mdi-dots-horizontal" size="14"></v-icon>
          </v-btn>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.table-scroll {
  flex: 1;
  overflow-y: auto;
}

.table-header {
  display: grid;
  grid-template-columns: 32px 2.5fr 1.5fr 1.2fr 50px 32px;
  align-items: center;
  padding: 8px 12px;
  position: sticky;
  top: 0;
  background: rgb(var(--v-theme-background));
  z-index: 2;
  border-bottom: 1px solid var(--v-border-color);
  font-size: var(--text-xs);
  color: var(--v-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.table-body {
  padding: 4px 0;
}

.table-row {
  display: grid;
  grid-template-columns: 32px 2.5fr 1.5fr 1.2fr 50px 32px;
  align-items: center;
  padding: 6px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s ease;
}

.table-row:hover {
  background: rgb(var(--v-theme-surface));
}

.table-row.playing {
  background: rgb(var(--v-theme-surface));
}

.col-num {
  text-align: center;
  font-size: var(--text-sm);
  color: var(--v-text-muted);
}

.row-num {
  color: var(--v-text-muted);
}

.col-title {
  display: flex;
  align-items: center;
  gap: 10px;
}

.song-art {
  width: 36px;
  height: 36px;
  border-radius: 6px;
  flex-shrink: 0;
}

.song-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.song-title {
  font-size: var(--text-md);
  color: rgb(var(--v-theme-on-background));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.song-title.active {
  color: rgb(var(--v-theme-secondary));
}

.song-quality {
  font-size: var(--text-xs);
  color: var(--v-text-muted);
}

.col-album {
  font-size: var(--text-sm);
  color: var(--v-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.col-artist {
  font-size: var(--text-sm);
  color: var(--v-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.col-duration {
  font-size: var(--text-sm);
  color: var(--v-text-muted);
  text-align: right;
}

.col-actions {
  text-align: center;
}
</style>