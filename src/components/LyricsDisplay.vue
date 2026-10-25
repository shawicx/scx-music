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
}

.lyrics-display {
  /* basis auto + min-height:0：短内容撑开容器（列内居中）、长内容收缩为内部滚动区；
   * 缺 min-height:0 时 flex 子项默认下限是内容全高，会把歌词推出面板外 */
  flex: 1 1 auto;
  min-height: 0;
  /* 定位锚点：歌词行 offsetTop 以此容器为基准（scrollToCurrent 的滚动数学依赖它，
   * 若锚点落到更外层的 .np-body，每次滚动都会多滚一个头部高度、把歌词推到面板外） */
  position: relative;
  overflow-y: auto;
  overflow-x: hidden;
  -webkit-mask-image: linear-gradient(transparent 0%, black 10%, black 90%, transparent 100%);
  mask-image: linear-gradient(transparent 0%, black 10%, black 90%, transparent 100%);
  scrollbar-width: none;
}
.lyrics-display::-webkit-scrollbar { display: none; }

/* 行颜色：正在播放沉浸层通过 --np-fg* 覆盖；其他场景回退主题文本色 */
.lyric-line {
  padding: 8px 0;
  font-size: 18px;
  line-height: 1.6;
  color: var(--np-fg-3, rgba(var(--v-theme-on-background), 0.3));
  cursor: pointer;
  text-align: left;
  transition: color 0.4s, opacity 0.4s;
}
.lyric-line:hover {
  color: var(--np-fg-2, rgba(var(--v-theme-on-background), 0.55));
}
.lyric-line.active {
  color: var(--np-fg, rgb(var(--v-theme-on-background)));
  font-family: var(--font-display);
  font-size: 26px;
  font-weight: 560;
}
.lyric-spacer {
  height: 38%;
}
.lyrics-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--np-fg-3, var(--v-text-muted));
  font-size: var(--text-md);
}
.lyrics-skeleton {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 38% 20px 0;
}
.skeleton-line {
  height: 14px;
  border-radius: 7px;
  background: var(--np-fg-3, rgba(var(--v-theme-on-background), 0.08));
  animation: skeleton-pulse 1.5s ease-in-out infinite;
}
.skeleton-line:nth-child(2) { width: 75%; }
.skeleton-line:nth-child(4) { width: 85%; }
.skeleton-line:nth-child(6) { width: 60%; }
@keyframes skeleton-pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 1; }
}
</style>
