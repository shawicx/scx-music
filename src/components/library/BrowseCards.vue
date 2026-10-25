<script setup lang="ts">
import CoverArt from '../player/CoverArt.vue'
import { useI18n } from '../../composables/useI18n'

export interface CardData {
  name: string
  count: number
  sampleSongId: string | null
}

defineProps<{
  cards: CardData[]
  type: 'albums' | 'artists'
}>()

const emit = defineEmits<{
  'cardClick': [name: string]
}>()

const { t } = useI18n()
</script>

<template>
  <!-- 专辑：gcard 网格 -->
  <div v-if="type === 'albums'" class="card-scroll">
    <div
      v-for="card in cards"
      :key="card.name"
      class="gcard"
      role="button"
      tabindex="0"
      :aria-label="card.name"
      @click="emit('cardClick', card.name)"
      @keydown.enter="emit('cardClick', card.name)"
    >
      <div class="gc-cover">
        <CoverArt :song-id="card.sampleSongId" />
        <span class="gc-play" aria-hidden="true">
          <v-icon icon="mdi-play" size="16"></v-icon>
        </span>
      </div>
      <div class="gc-name">{{ card.name }}</div>
      <div class="gc-sub">{{ t('common.songs', { count: card.count }) }}</div>
    </div>
  </div>

  <!-- 艺术家：圆头像行 -->
  <div v-else class="artist-scroll">
    <div
      v-for="card in cards"
      :key="card.name"
      class="artist-row"
      role="button"
      tabindex="0"
      :aria-label="card.name"
      @click="emit('cardClick', card.name)"
      @keydown.enter="emit('cardClick', card.name)"
    >
      <span class="ar-avatar"><CoverArt :song-id="card.sampleSongId" /></span>
      <span class="ar-meta">
        <span class="ar-name">{{ card.name }}</span>
        <span class="ar-sub">{{ t('common.songs', { count: card.count }) }}</span>
      </span>
    </div>
  </div>
</template>

<style scoped>
/* 专辑网格 */
.card-scroll {
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

/* 艺术家行 */
.artist-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 6px 28px 28px;
  display: flex;
  flex-direction: column;
  max-width: 760px;
}
.artist-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px 6px;
  border-radius: 10px;
  cursor: pointer;
  border-bottom: var(--hairline-w) solid var(--hairline);
  transition: background 0.14s;
}
.artist-row:hover {
  background: var(--surface-2);
}
.ar-avatar {
  width: 46px;
  height: 46px;
  border-radius: 50%;
  overflow: hidden;
  flex-shrink: 0;
  box-shadow: var(--cover-shadow);
  background: var(--surface-2);
}
.ar-meta {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}
.ar-name {
  font-size: 13px;
  font-weight: 580;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.ar-sub {
  font-size: 11.5px;
  color: var(--text-3);
}
</style>
