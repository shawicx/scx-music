<script setup lang="ts">
import { ref, computed } from 'vue'
import type { LrcLine } from '../composables/useLyrics'
import { useI18n } from '../composables/useI18n'
import { useLyricsAnimation } from '../composables/useLyricsAnimation'

const props = defineProps<{
  lines: LrcLine[]
  currentLineIndex: number
  isLoading: boolean
  getSeekTime: (lineTime: number) => number
}>()

const emit = defineEmits<{
  seek: [time: number]
}>()

const { t } = useI18n()
const containerRef = ref<HTMLElement | null>(null)
const userScrolling = ref(false)
let scrollTimer: ReturnType<typeof setTimeout> | null = null

useLyricsAnimation(
  containerRef,
  computed(() => props.currentLineIndex),
  userScrolling,
)

function onScroll() {
  userScrolling.value = true
  if (scrollTimer) clearTimeout(scrollTimer)
  scrollTimer = setTimeout(() => {
    userScrolling.value = false
  }, 3000)
}

function onClickLine(line: LrcLine) {
  const seekTime = props.getSeekTime(line.time)
  if (seekTime > 0) {
    emit('seek', seekTime)
  }
}
</script>

<template>
  <div class="lyrics-container">
    <div class="lyrics-display" ref="containerRef" @wheel="onScroll" @touchstart="onScroll">
      <div v-if="isLoading" class="lyrics-skeleton">
        <div class="skeleton-line" v-for="i in 6" :key="i" />
      </div>
      <div v-else-if="lines.length === 0" class="lyrics-empty">
        {{ t('lyrics.noLyrics') }}
      </div>
      <template v-else>
        <div class="lyric-spacer" />
        <div
          v-for="(line, i) in lines"
          :key="i"
          class="lyric-line"
          :class="{ active: i === currentLineIndex }"
          @click="onClickLine(line)"
        >
          {{ line.text }}
        </div>
        <div class="lyric-spacer" />
      </template>
    </div>
  </div>
</template>

<style scoped>
.lyrics-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  width: 100%;
  min-height: 0;
  position: relative;
  z-index: 1;
  /* 歌词区背景遮罩：压暗中间区域的音频可视化，使可视化退居背景，歌词成为视觉主体 */
  background: radial-gradient(ellipse 95% 80% at center, rgb(var(--v-theme-background) / 0.82) 30%, transparent 90%);
}

.lyrics-display {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  -webkit-mask-image: linear-gradient(transparent 0%, black 12%, black 88%, transparent 100%);
  mask-image: linear-gradient(transparent 0%, black 12%, black 88%, transparent 100%);
  scrollbar-width: none;
}
.lyrics-display::-webkit-scrollbar { display: none; }

.lyric-line {
  padding: 7px 8px;
  font-size: 15px;
  line-height: 1.7;
  color: rgba(var(--v-theme-on-background), 0.25);
  cursor: pointer;
  text-align: center;
  /* 文字描边阴影：保证歌词在任何可视化背景上都清晰可读 */
  text-shadow: 0 1px 4px rgb(var(--v-theme-background) / 0.8), 0 0 12px rgb(var(--v-theme-background) / 0.6);
}
.lyric-line:hover {
  color: rgba(var(--v-theme-on-background), 0.5);
}
.lyric-line.active {
  color: rgb(var(--v-theme-on-background));
  font-size: 19px;
  font-weight: 600;
  text-shadow: 0 1px 6px rgb(var(--v-theme-background) / 0.9), 0 0 16px rgb(var(--v-theme-background) / 0.7);
}
.lyric-spacer {
  height: 40%;
}
.lyrics-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--v-text-muted);
  font-size: var(--text-md);
}
.lyrics-skeleton {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 40% 20px 0;
}
.skeleton-line {
  height: 14px;
  border-radius: 7px;
  background: rgba(var(--v-theme-on-background), 0.06);
  animation: skeleton-pulse 1.5s ease-in-out infinite;
}
.skeleton-line:nth-child(2) { width: 75%; margin: 0 auto; }
.skeleton-line:nth-child(4) { width: 85%; margin: 0 auto; }
.skeleton-line:nth-child(6) { width: 60%; margin: 0 auto; }
@keyframes skeleton-pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 1; }
}
</style>
