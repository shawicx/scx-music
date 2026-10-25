<script setup lang="ts">
import { computed } from 'vue'
import type { Song } from '../../types'
import { useI18n } from '../../composables/useI18n'
import VirtualSongTable from './VirtualSongTable.vue'

const props = defineProps<{
  songs: Song[]
  currentSongId?: string | null
  isPlaying: boolean
  likedIds?: Set<string>
  /** 是否强制启用虚拟滚动；不传则按 songs.length > 100 自动切换 */
  virtualEnabled?: boolean
}>()

const emit = defineEmits<{
  'songClick': [index: number]
  'songMenu': [event: MouseEvent, songId: string]
  'toggleLike': [songId: string]
}>()

const { t } = useI18n()

const useVirtual = computed(() => props.virtualEnabled ?? props.songs.length > 100)
</script>

<template>
  <VirtualSongTable
    v-if="useVirtual"
    :songs="songs"
    :current-song-id="currentSongId"
    :is-playing="isPlaying"
    :liked-ids="likedIds"
    :container-height="600"
    @song-click="emit('songClick', $event)"
    @song-menu="(event, songId) => emit('songMenu', event, songId)"
    @toggle-like="emit('toggleLike', $event)"
  />
  <div v-else class="table-scroll" :class="{ 'is-playing': isPlaying }">
    <div class="song-table-head">
      <span class="col-num-th">#</span>
      <span>{{ t('library.title') }}</span>
      <span class="col-album-th">{{ t('library.album') }}</span>
      <span></span>
      <span style="text-align: right">{{ t('library.duration') }}</span>
    </div>
    <div
      v-for="(song, i) in songs"
      :key="song.id"
      :data-song-id="song.id"
      class="song-row"
      :class="{ current: song.id === currentSongId }"
      @contextmenu="emit('songMenu', $event, song.id)"
    >
      <div class="col-num">
        <span v-if="song.id === currentSongId" class="eq" aria-hidden="true"><i></i><i></i><i></i></span>
        <span v-else class="num">{{ i + 1 }}</span>
      </div>
      <div class="col-title">
        <div class="s-title" @click="emit('songClick', i)">{{ song.title }}</div>
        <div class="s-artist">{{ song.artist }}</div>
      </div>
      <div class="col-album" @click="emit('songClick', i)">{{ song.album }}</div>
      <div class="col-heart">
        <button
          class="heart-btn"
          :class="{ on: likedIds?.has(song.id) }"
          :aria-label="t('player.addToFavorite')"
          @click.stop="emit('toggleLike', song.id)"
        >
          <v-icon :icon="likedIds?.has(song.id) ? 'mdi-heart' : 'mdi-heart-outline'" size="15"></v-icon>
        </button>
      </div>
      <div class="col-dur" @click="emit('songClick', i)">{{ song.duration }}</div>
    </div>
  </div>
</template>

<style src="../../styles/song-table.css"></style>

<style scoped>
.table-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 0 28px 28px;
}
</style>
