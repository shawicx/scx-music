<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { usePlayerStore } from '../stores/player'
import { useLibraryStore } from '../stores/library'
import { usePlaybackMode } from '../composables/usePlaybackMode'
import { useI18n } from '../composables/useI18n'
import { useLyrics } from '../composables/useLyrics'
import { getCoverUrl } from '../composables/useCoverArt'
import { AudioVisualizer } from '../visualization'
import LyricsDisplay from './LyricsDisplay.vue'
import PulseDots from './common/PulseDots.vue'
import IconButtonWithTooltip from './IconButtonWithTooltip.vue'
import CoverArt from './player/CoverArt.vue'

const emit = defineEmits<{ close: []; toggleQueue: [] }>()

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

// 沉浸背景：与 CoverArt 共享 useCoverArt 的缓存/inflight，不会重复 IPC
const bgUrl = ref<string | null>(null)
watch(
  () => currentSong.value?.id,
  async (id) => {
    bgUrl.value = null
    if (!id) return
    const url = await getCoverUrl(id)
    if (currentSong.value?.id === id) bgUrl.value = url
  },
  { immediate: true },
)

// 频谱：默认隐藏，会话内记忆（不持久化）
const showVisualizer = ref(false)
</script>

<template>
  <div class="overlay">
    <div class="bg-layer" :class="{ 'has-cover': !!bgUrl }">
      <img v-if="bgUrl" :src="bgUrl" class="bg-img" alt="" />
      <div class="bg-dim" />
      <div class="vignette" />
    </div>
    <AudioVisualizer v-if="showVisualizer" class="overlay-visualizer" />

    <div class="mode-status-bar" v-if="isModeActive">
      <div class="status-item">
        <v-icon :icon="modeIcon" size="14" color="secondary"></v-icon>
        <span>{{ modeLabel }}</span>
      </div>
    </div>
    <v-btn variant="text" size="small" class="close-btn" @click="emit('close')">
      <v-icon icon="mdi-chevron-down" size="18"></v-icon>
      {{ t('player.collapse') }}
    </v-btn>

    <div class="content">
      <div class="main-row">
        <div class="cover-column">
          <CoverArt :song-id="currentSong?.id" class="hero-cover" />
        </div>
        <div class="lyrics-column">
          <div class="song-header">
            <div class="song-title">{{ currentSong?.title ?? t('player.notPlaying') }}</div>
            <div class="song-artist-row">
              <span class="song-artist">{{ currentSong ? `${currentSong.artist} · ${currentSong.album}` : '--' }}</span>
              <PulseDots v-if="isPlaying && currentSong" :size="7" :gap="5" />
            </div>
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
        </div>
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

        <div class="controls-extra">
          <IconButtonWithTooltip
            :icon="showVisualizer ? 'mdi-equalizer' : 'mdi-equalizer-outline'"
            icon-active="mdi-equalizer"
            :active="showVisualizer"
            :tooltip="t('player.visualizer')"
            size="small"
            @click.stop="showVisualizer = !showVisualizer"
          />
          <IconButtonWithTooltip
            icon="mdi-playlist-music-outline"
            :tooltip="t('player.playlist')"
            size="small"
            @click.stop="emit('toggleQueue')"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: absolute; inset: 0;
  background: rgb(var(--v-theme-background)); /* 无封面回退底色 */
  display: flex; flex-direction: column; align-items: center;
  z-index: 20; overflow: hidden;
}

/* ===== 沉浸背景层 ===== */
.bg-layer {
  position: absolute; inset: 0;
  pointer-events: none;
  z-index: 0;
}
.bg-img {
  width: 100%; height: 100%;
  object-fit: cover;
  transform: scale(1.2); /* blur 边缘露白补偿 */
  filter: blur(60px);
}
/* 暗化遮罩：保证前景可读性，深浅两档 */
.bg-dim {
  position: absolute; inset: 0;
  background: rgb(var(--v-theme-background));
  opacity: 0.35;
}
:global(.v-theme--dark) .bg-layer.has-cover .bg-dim { opacity: 0.45; }
/* 主题色 vignette：仅暗色模式，浅色极简无装饰背景 */
.vignette {
  position: absolute; inset: 0;
  background: radial-gradient(ellipse at center top, rgb(var(--v-theme-primary) / 0.12), transparent 60%);
}
:global(.v-theme--light) .vignette { display: none; }

.overlay-visualizer {
  position: absolute; inset: 0;
  z-index: 1;
  opacity: 0.35;
  pointer-events: none;
}

/* 顶部悬浮件须高于 .content（同 z-index 时 DOM 靠后的 .content padding 区会盖住点击） */
.close-btn { position: absolute; top: 16px; left: 20px; z-index: 3; color: var(--v-text-secondary); }
.mode-status-bar { position: absolute; top: 16px; right: 20px; z-index: 3; }
.status-item {
  display: flex; align-items: center; gap: 6px;
  padding: 6px 12px;
  background: rgb(var(--v-theme-surface-variant) / 0.3);
  border-radius: 16px; font-size: var(--text-xs); color: var(--v-text-secondary);
}

/* ===== 主内容 ===== */
.content {
  position: relative; z-index: 2;
  width: 100%; max-width: 960px; flex: 1;
  display: flex; flex-direction: column;
  padding: 56px 32px 32px;
  min-height: 0;
}

.main-row {
  flex: 1;
  display: flex;
  align-items: stretch;
  gap: 48px;
  min-height: 0;
}

.cover-column {
  flex-shrink: 0;
  display: flex;
  align-items: center;
}
.hero-cover {
  width: min(38vw, 380px);
  height: min(38vw, 380px);
  border-radius: var(--radius-xl);
  box-shadow: 0 16px 48px rgb(0 0 0 / 0.35);
}

.lyrics-column {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.song-header { flex-shrink: 0; margin-bottom: 8px; }
.song-title {
  font-size: var(--text-2xl); font-weight: 600;
  color: rgb(var(--v-theme-on-background)); margin-bottom: 6px;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.song-artist-row { display: flex; align-items: center; gap: 10px; }
.song-artist {
  font-size: var(--text-md); color: var(--v-text-secondary);
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.lyrics-column :deep(.lyrics-container) { flex: 1; min-height: 0; }

.progress-section { width: 100%; z-index: 2; flex-shrink: 0; }
.time-row { display: flex; justify-content: space-between; margin-top: 6px; font-size: var(--text-xs); color: var(--v-text-muted); }

.controls {
  display: flex; align-items: center; gap: 12px; z-index: 2; flex-shrink: 0;
  width: 100%;
}
.play-btn-lg { box-shadow: var(--shadow-accent-lg); transition: transform 0.15s, box-shadow 0.15s; }
.play-btn-lg:hover { transform: scale(1.06); }
.muted { opacity: 0.5; }
.controls-extra {
  margin-left: auto;
  display: flex; align-items: center; gap: var(--space-sm);
}

/* ===== 窄屏：<880px 纵向排列 ===== */
@media (max-width: 880px) {
  .main-row { flex-direction: column; align-items: center; gap: 16px; overflow-y: auto; }
  .hero-cover { width: 200px; height: 200px; }
  .lyrics-column { width: 100%; align-items: center; }
  .song-header { text-align: center; }
  .song-title, .song-artist-row { justify-content: center; }
}
</style>
