<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { usePlayerStore } from '../stores/player'
import { useLibraryStore } from '../stores/library'
import { useAudioEffectsStore } from '../stores/audioEffects'
import { usePlaybackMode } from '../composables/usePlaybackMode'
import { useI18n } from '../composables/useI18n'
import { useLyrics } from '../composables/useLyrics'
import { useMiniPlayer } from '../composables/useMiniPlayer'
import LyricsDisplay from './LyricsDisplay.vue'
import AppSlider from './common/AppSlider.vue'
import CoverArt from './player/CoverArt.vue'

const emit = defineEmits<{ close: []; toggleQueue: [] }>()

const playerStore = usePlayerStore()
const { t } = useI18n()
const libraryStore = useLibraryStore()
const audioEffectsStore = useAudioEffectsStore()
const { presets, currentPresetId: eqPresetId, enabled: eqEnabled } = storeToRefs(audioEffectsStore)
const { switchPreset, setEnabled } = audioEffectsStore
const eqPanelOpen = ref(false)

// v-switch 的 update:model-value 类型为 boolean | null（indeterminate 态），null 时忽略
function onEqToggle(v: boolean | null) {
  if (v !== null) setEnabled(v)
}
const { playbackMode, toggleShuffle, cycleRepeat } = usePlaybackMode()
const { enter: enterMini } = useMiniPlayer()

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
  get: () => (duration.value > 0 ? (progress.value / duration.value) * 100 : 0),
  set: (val: number) => {
    if (duration.value > 0) seek((val / 100) * duration.value)
  },
})

function onLyricSeek(time: number) {
  seek(time)
}

// 歌词偏移步进（仅有同步歌词时显示，悬浮于控制面板上方）
const offsetLabel = computed(() => {
  const v = offsetSecs.value
  return v === 0 ? '0.0s' : v > 0 ? `+${v.toFixed(1)}s` : `${v.toFixed(1)}s`
})

// 播放模式双按钮状态
const shuffleOn = computed(() => playbackMode.value === 'shuffle')
const repeatLabel = computed(() =>
  playbackMode.value === 'repeat_one'
    ? t('playbackMode.repeatOne')
    : playbackMode.value === 'repeat_all'
      ? t('playbackMode.repeatAll')
      : t('playbackMode.off'),
)
</script>

<template>
  <section class="np" role="dialog" aria-modal="true" :aria-label="t('player.nowPlaying')">
    <!-- 顶栏 -->
    <div class="np-top">
      <button class="np-glass-btn close-btn" :aria-label="t('player.collapse')" @click="emit('close')">
        <v-icon icon="mdi-chevron-down" size="18"></v-icon>
      </button>
      <button class="np-glass-btn" :aria-label="t('player.playQueue')" @click.stop="emit('toggleQueue')">
        <v-icon icon="mdi-playlist-music-outline" size="16"></v-icon>
      </button>
    </div>

    <!-- 主体：左封面 / 右信息 + 歌词 -->
    <div class="np-body">
      <div class="cover-column">
        <CoverArt :song-id="currentSong?.id" class="np-cover" />
      </div>
      <div class="lyrics-column">
        <div class="np-eyebrow">{{ currentSong ? `${t('library.album')} · ${currentSong.album}` : '' }}</div>
        <h1 class="np-title">{{ currentSong?.title ?? t('player.notPlaying') }}</h1>
        <div class="np-artist">{{ currentSong?.artist ?? '' }}</div>
        <LyricsDisplay
          :lines="lines"
          :current-line-index="currentLineIndex"
          :is-loading="isLoading"
          :get-seek-time="getSeekTime"
          @seek="onLyricSeek"
        />
      </div>
    </div>

    <!-- 底部玻璃控制面板 -->
    <div class="np-panel">
      <div class="np-progress progress-section">
        <span class="np-time num">{{ formatTime(progress) }}</span>
        <AppSlider
          v-model="progressModel"
          variant="line"
          class="np-slider"
          :fill-style="{ background: 'var(--np-panel-fg)' }"
          :aria-label="t('player.play')"
          :disabled="!currentSong"
        />
        <span class="np-time num">{{ formatTime(duration) }}</span>
      </div>
      <div class="np-mid controls">
        <button
          class="np-icobtn"
          :class="{ on: isLiked }"
          :aria-label="isLiked ? t('player.unlike') : t('player.addToFavorite')"
          :disabled="!currentSong"
          @click="toggleLike"
        >
          <v-icon :icon="isLiked ? 'mdi-heart' : 'mdi-heart-outline'" size="18"></v-icon>
        </button>
        <div class="np-transport">
          <button
            class="np-icobtn"
            :class="{ on: shuffleOn }"
            :aria-label="t('playbackMode.shuffle')"
            @click="toggleShuffle"
          >
            <v-icon icon="mdi-shuffle" size="17"></v-icon>
          </button>
          <button class="np-icobtn" :aria-label="t('player.previous')" @click="previous">
            <v-icon icon="mdi-skip-previous" size="22"></v-icon>
          </button>
          <button class="np-play-main" :aria-label="t('player.play')" :disabled="!currentSong" @click="togglePlayPause">
            <v-icon :icon="isPlaying ? 'mdi-pause' : 'mdi-play'" size="20"></v-icon>
          </button>
          <button class="np-icobtn" :aria-label="t('player.next')" @click="next">
            <v-icon icon="mdi-skip-next" size="22"></v-icon>
          </button>
          <button class="np-icobtn" :class="{ on: playbackMode !== 'sequential' && playbackMode !== 'shuffle' }" :aria-label="repeatLabel" @click="cycleRepeat">
            <v-icon :icon="playbackMode === 'repeat_one' ? 'mdi-repeat-once' : 'mdi-repeat'" size="17"></v-icon>
          </button>
        </div>
        <div class="np-panel-right">
          <div class="eq-entry">
            <div v-if="eqPanelOpen" class="eq-backdrop" @click="eqPanelOpen = false" />
            <Transition name="eq-pop">
              <div v-if="eqPanelOpen" class="eq-panel">
                <div class="eq-panel-header">
                  <span class="eq-panel-title">{{ t('settings.eq.title') }}</span>
                  <v-switch
                    :model-value="eqEnabled"
                    density="compact"
                    color="primary"
                    hide-details
                    :aria-label="t('settings.eq.enabled')"
                    @update:model-value="onEqToggle"
                  />
                </div>
                <button
                  v-for="preset in presets"
                  :key="preset.id"
                  :class="['eq-preset', { active: eqPresetId === preset.id }]"
                  @click="switchPreset(preset.id)"
                >
                  <span class="eq-preset-name">{{ t(`settings.eq.presets.${preset.id}`) }}</span>
                  <span class="eq-preset-curve" aria-hidden="true">
                    <i
                      v-for="(g, i) in preset.gainsDb"
                      :key="i"
                      :style="{ height: `${Math.max(2, 8 + g)}px` }"
                    />
                  </span>
                </button>
              </div>
            </Transition>
            <button class="np-icobtn" :aria-label="t('settings.eq.title')" @click.stop="eqPanelOpen = !eqPanelOpen">
              <v-icon icon="mdi-tune-variant" size="17"></v-icon>
            </button>
          </div>
          <button class="np-icobtn" :aria-label="t('miniPlayer.enter')" @click="enterMini">
            <v-icon icon="mdi-picture-in-picture-top-right" size="17"></v-icon>
          </button>
        </div>
      </div>

      <!-- 歌词偏移步进条：面板内居中一行 -->
      <div v-if="lines.length > 0" class="offset-bar">
        <button aria-label="-0.1s" @click="adjustOffset(-0.1)">
          <v-icon icon="mdi-minus" size="18" />
        </button>
        <span class="offset-value num">{{ offsetLabel }}</span>
        <button aria-label="+0.1s" @click="adjustOffset(0.1)">
          <v-icon icon="mdi-plus" size="18" />
        </button>
        <button v-if="offsetSecs !== 0" :aria-label="t('lyrics.resetOffset')" @click="resetOffset">
          <v-icon icon="mdi-restore" size="18" />
        </button>
      </div>
    </div>
  </section>
</template>

<style scoped>
/* ===== 沉浸层 ===== */
.np {
  position: absolute;
  inset: 0;
  z-index: 60;
  display: flex;
  flex-direction: column;
  /* 玻璃底：窗口透明 + 原生 UnderWindowBackground 毛玻璃（lib.rs），
   * 此处只铺半透明着色，桌面从背后透出 */
  background: var(--glass-bg);
  overflow: hidden;
  /* 沉浸层文字色（浅色主题为深色文字，遮罩相应变浅）。
   * fg-3 给到 0.55：非活跃行叠加 spotlight 距离衰减后仍需可读 */
  --np-fg: #ffffff;
  --np-fg-2: rgba(255, 255, 255, 0.72);
  --np-fg-3: rgba(255, 255, 255, 0.55);
  --np-panel-bg: rgba(22, 22, 30, 0.44);
  --np-panel-border: rgba(255, 255, 255, 0.12);
  --np-panel-fg: #ffffff;
  --np-play-bg: #ffffff;
  --np-play-fg: #16161d;
}
:global(.scx-light) .np {
  background: rgba(240, 240, 238, 0.55); /* 浅色玻璃着色 */
  --np-fg: #1d1d1f;
  --np-fg-2: rgba(29, 29, 31, 0.68);
  --np-fg-3: rgba(29, 29, 31, 0.55);
  --np-panel-bg: rgba(255, 255, 255, 0.6);
  --np-panel-border: rgba(0, 0, 0, 0.08);
  --np-panel-fg: #1d1d1f;
  --np-play-bg: #1d1d1f;
  --np-play-fg: #ffffff;
}

/* ===== 顶栏 ===== */
.np-top {
  position: relative;
  z-index: 2;
  height: 56px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 18px 0 20px;
}
.np-glass-btn {
  margin-left: auto;
  width: 34px;
  height: 34px;
  border-radius: 9px;
  display: grid;
  place-items: center;
  color: var(--np-fg-2);
  background: rgb(255 255 255 / 0.07);
  border: 1px solid rgb(255 255 255 / 0.1);
  cursor: pointer;
}
.np-glass-btn + .np-glass-btn {
  margin-left: 0;
}
.np-glass-btn:hover {
  background: rgb(255 255 255 / 0.15);
  color: var(--np-fg);
}
:global(.scx-light) .np-glass-btn {
  background: rgb(255 255 255 / 0.55);
  border-color: rgb(0 0 0 / 0.08);
}
:global(.scx-light) .np-glass-btn:hover {
  background: rgb(255 255 255 / 0.8);
}

/* ===== 主体 =====
 * flex 行布局：封面列定宽、歌词列占满剩余高度（stretch + min-height:0），
 * 歌词容器 flex 0 1 auto——内容少时整块垂直居中，多时收缩为内部滚动区 */
.np-body {
  position: relative;
  z-index: 2;
  flex: 1;
  min-height: 0;
  display: flex;
  /* 不设 align-items:center——stretch 才能让歌词列继承有界高度（center 下列高=内容高，长歌词溢出） */
  gap: clamp(28px, 5vw, 72px);
  padding: clamp(6px, 1.6vh, 16px) clamp(28px, 6vw, 84px);
  max-width: 1160px;
  width: 100%;
  margin: 0 auto;
}
.cover-column {
  flex: 0 1 380px;
  min-width: 240px;
  align-self: center;
  display: flex;
  align-items: center;
}
.cover-column .np-cover {
  width: 100%;
  aspect-ratio: 1;
  /* height:auto 必须压过 CoverArt 内部的 height:100%（列被 stretch 拉满时会变长方形），
   * 故用两级选择器提高优先级；圆角按需求去除 */
  height: auto;
  border-radius: 0;
  box-shadow: 0 30px 80px rgba(0, 0, 0, 0.5), 0 4px 18px rgba(0, 0, 0, 0.35);
}
.np-eyebrow {
  font-size: 11px;
  letter-spacing: 0.14em;
  font-weight: 620;
  color: var(--np-fg-2);
  text-transform: uppercase;
}
.np-title {
  font-family: var(--font-display);
  font-size: clamp(30px, 4.2vw, 52px);
  font-weight: 540;
  line-height: 1.12;
  letter-spacing: 0.005em;
  margin-top: 10px;
  color: var(--np-fg);
  text-shadow: 0 2px 26px rgba(0, 0, 0, 0.25);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
:global(.scx-light) .np-title {
  text-shadow: none;
}
.np-artist {
  font-size: 15px;
  color: var(--np-fg-2);
  margin-top: 8px;
}
/* 高度约束链：np-body(flex:1 有界, stretch) → lyrics-column(占满列高) →
 * lyrics-container(flex 0 1 auto：短内容居中、长内容收缩) → lyrics-display(overflow 滚动)。
 * 歌词绝不越过底部控制面板。 */
.lyrics-column {
  flex: 1 1 0;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
}
.lyrics-column :deep(.lyrics-container) {
  flex: 0 1 auto;
  min-height: 0;
  margin-top: clamp(16px, 3vh, 32px);
}

/* ===== 底部玻璃控制面板 ===== */
.np-panel {
  position: relative;
  z-index: 2;
  flex-shrink: 0;
  margin: 0 auto clamp(16px, 3vh, 32px);
  width: min(620px, calc(100% - 48px));
  border-radius: 18px;
  padding: 14px 20px 16px;
  background: var(--np-panel-bg);
  /* 背景已是 Canvas 预模糊的平滑图，backdrop-filter 无视觉增益，故不再使用 */
  border: 1px solid var(--np-panel-border);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.12), 0 18px 50px rgba(0, 0, 0, 0.4);
}
:global(.scx-light) .np-panel {
  box-shadow: 0 18px 50px rgba(20, 20, 24, 0.16);
}
.np-progress {
  display: flex;
  align-items: center;
  gap: 10px;
}
.np-slider {
  flex: 1;
}
.np-time {
  font-size: 11px;
  color: var(--np-fg-2);
  min-width: 36px;
}
.np-mid {
  display: grid;
  grid-template-columns: 44px 1fr auto;
  align-items: center;
  margin-top: 6px;
}
.np-transport {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
}
.np-icobtn {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  display: grid;
  place-items: center;
  color: var(--np-fg-2);
  background: none;
  border: none;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}
.np-icobtn:hover {
  background: rgb(255 255 255 / 0.12);
  color: var(--np-fg);
}
:global(.scx-light) .np-icobtn:hover {
  background: rgb(0 0 0 / 0.06);
}
.np-icobtn.on {
  color: var(--np-accent, var(--np-fg));
}
.np-icobtn:disabled {
  opacity: 0.4;
  cursor: default;
}
.np-play-main {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background: var(--np-play-bg);
  color: var(--np-play-fg);
  display: grid;
  place-items: center;
  border: none;
  cursor: pointer;
  box-shadow: 0 6px 22px rgba(0, 0, 0, 0.35);
  transition: transform 0.16s var(--ease), filter 0.16s;
}
.np-play-main:hover:not(:disabled) {
  transform: scale(1.05);
}
.np-play-main:active:not(:disabled) {
  transform: scale(0.95);
}
.np-play-main:disabled {
  opacity: 0.5;
  cursor: default;
}
.np-panel-right {
  display: flex;
  align-items: center;
  gap: 2px;
  justify-self: end;
}

/* ===== 歌词偏移步进条：面板内居中一行（不再浮在歌词上方） ===== */
.offset-bar {
  margin: 8px auto 0;
  width: fit-content;
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 2px 8px;
  border-radius: 99px;
  background: rgb(255 255 255 / 0.06);
  border: 1px solid var(--np-panel-border);
}
.offset-bar button {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  color: var(--np-fg-2);
  background: none;
  border: none;
  cursor: pointer;
}
.offset-bar button:hover {
  background: rgb(255 255 255 / 0.12);
  color: var(--np-fg);
}
.offset-value {
  font-size: 12px;
  color: var(--np-fg-2);
  min-width: 48px;
  text-align: center;
}

/* ===== 音效快捷面板 ===== */
.eq-entry {
  position: relative;
  display: flex;
  align-items: center;
}
.eq-backdrop {
  position: fixed;
  inset: 0;
  z-index: 4;
}
.eq-panel {
  position: absolute;
  bottom: calc(100% + 12px);
  right: 0;
  z-index: 5;
  width: 248px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  background: var(--glass-bg);
  -webkit-backdrop-filter: blur(24px) saturate(1.5);
  backdrop-filter: blur(24px) saturate(1.5);
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  box-shadow: var(--shadow-pop);
}
.eq-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}
.eq-panel-title {
  font-size: 12px;
  font-weight: 640;
  color: var(--text-1);
}
.eq-preset {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 7px 10px;
  border-radius: 8px;
  border: 1px solid transparent;
  background: transparent;
  cursor: pointer;
  transition: border-color 0.15s, background 0.15s;
  color: var(--text-1);
  font-size: 12px;
  font-family: inherit;
  text-align: left;
}
.eq-preset:hover {
  background: var(--surface-2);
}
.eq-preset.active {
  border-color: var(--accent);
  background: rgb(var(--accent-rgb) / 0.08);
  font-weight: 600;
}
.eq-preset-name {
  white-space: nowrap;
}
/* 迷你 EQ 曲线：0dB=8px，±12dB→-4..20px（下限 2px） */
.eq-preset-curve {
  display: flex;
  align-items: center;
  gap: 2px;
  flex: 0 0 auto;
}
.eq-preset-curve i {
  width: 3px;
  border-radius: 1.5px;
  background: var(--text-3);
}
.eq-preset.active .eq-preset-curve i {
  background: var(--accent);
}

.eq-pop-enter-active,
.eq-pop-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}
.eq-pop-enter-from,
.eq-pop-leave-to {
  opacity: 0;
  transform: translateY(6px);
}

/* ===== 窄屏纵向排列 ===== */
@media (max-width: 880px) {
  .np-body {
    flex-direction: column;
    align-items: center;
    gap: 22px;
    padding: 0 24px;
    overflow-y: auto;
  }
  .cover-column {
    flex: 0 0 auto;
  }
  .np-cover {
    max-width: 230px;
  }
  .lyrics-column {
    flex: 0 1 auto;
    width: 100%;
    text-align: left;
  }
}
</style>
