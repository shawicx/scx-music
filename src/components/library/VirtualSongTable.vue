<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { Song } from '../../types'
import { useVirtualScroll } from '../../utils/virtualScroll'
import { useI18n } from '../../composables/useI18n'

const props = defineProps<{
  songs: Song[]
  currentSongId?: string | null
  isPlaying: boolean
  likedIds?: Set<string>
  containerHeight?: number
}>()

const emit = defineEmits<{
  'songClick': [index: number]
  'songMenu': [event: MouseEvent, songId: string]
  'toggleLike': [songId: string]
}>()

const { t } = useI18n()

const ITEM_HEIGHT = 46
const OVERSCAN = 5 // 预渲染行数
const HEADER_HEIGHT = 36

const currentContainerHeight = ref(props.containerHeight || 600)

const {
  containerRef,
  visibleItems,
  containerStyle,
  contentStyle,
  offsetY,
  handleScroll,
  scrollToIndex,
} = useVirtualScroll(computed(() => props.songs), {
  itemHeight: ITEM_HEIGHT,
  containerHeight: currentContainerHeight.value,
  overscan: OVERSCAN,
})

// 监听容器高度变化
watch(() => props.containerHeight, (newHeight) => {
  if (newHeight) {
    currentContainerHeight.value = newHeight
  }
})

// 暴露滚动方法
defineExpose({
  scrollToIndex,
})
</script>

<template>
  <div
    ref="containerRef"
    :style="containerStyle"
    :class="{ 'is-playing': isPlaying }"
    class="virtual-table-container"
    @scroll="handleScroll"
  >
    <div
      :style="{ ...contentStyle, height: `${songs.length * ITEM_HEIGHT + HEADER_HEIGHT}px` }"
      class="virtual-table-content"
    >
      <!-- Header -->
      <div class="song-table-head">
        <span class="col-num-th">#</span>
        <span>{{ t('library.title') }}</span>
        <span class="col-album-th">{{ t('library.album') }}</span>
        <span></span>
        <span style="text-align: right">{{ t('library.duration') }}</span>
      </div>

      <!-- Virtual rows -->
      <div :style="{ transform: `translateY(${offsetY}px)` }" class="virtual-rows">
        <div
          v-for="{ item: song, index } in visibleItems"
          :key="song.id"
          :data-song-id="song.id"
          class="song-row"
          :class="{ current: song.id === currentSongId }"
          :style="{ height: `${ITEM_HEIGHT}px` }"
          @contextmenu="emit('songMenu', $event, song.id)"
        >
          <div class="col-num">
            <span v-if="song.id === currentSongId" class="eq" aria-hidden="true"><i></i><i></i><i></i></span>
            <span v-else class="num">{{ index + 1 }}</span>
          </div>
          <div class="col-title">
            <div class="s-title" @click="emit('songClick', index)">{{ song.title }}</div>
            <div class="s-artist">{{ song.artist }}</div>
          </div>
          <div class="col-album" @click="emit('songClick', index)">{{ song.album }}</div>
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
          <div class="col-dur" @click="emit('songClick', index)">{{ song.duration }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style src="../../styles/song-table.css"></style>

<style scoped>
.virtual-table-container {
  flex: 1;
  overflow-y: auto;
  position: relative;
  padding: 0 28px 28px;
}

.virtual-table-content {
  position: relative;
}

.virtual-rows {
  position: absolute;
  left: 0;
  right: 0;
  top: 36px; /* header height */
}
</style>
