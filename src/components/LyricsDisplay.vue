<script setup lang="ts">
import { ref, computed } from 'vue'
import type { LrcLine } from '../composables/useLyrics'
import { useI18n } from '../composables/useI18n'
import { useLyricsAnimation } from '../composables/useLyricsAnimation'

const props = defineProps<{
  lines: LrcLine[]
  currentLineIndex: number
  isLoading: boolean
  offsetSecs: number
  adjustOffset: (delta: number) => void
  resetOffset: () => void
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

const offsetLabel = computed(() => {
  const v = props.offsetSecs
  return v === 0 ? '0.0s' : v > 0 ? `+${v.toFixed(1)}s` : `${v.toFixed(1)}s`
})
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
    <div v-if="lines.length > 0" class="offset-bar">
      <v-btn icon variant="text" size="x-small" density="compact" @click="adjustOffset(-0.1)">
        <v-icon icon="mdi-minus" size="16" />
      </v-btn>
      <span class="offset-value">{{ offsetLabel }}</span>
      <v-btn icon variant="text" size="x-small" density="compact" @click="adjustOffset(0.1)">
        <v-icon icon="mdi-plus" size="16" />
      </v-btn>
      <v-btn
        v-if="offsetSecs !== 0"
        icon
        variant="text"
        size="x-small"
        density="compact"
        @click="resetOffset"
      >
        <v-icon icon="mdi-restore" size="16" />
      </v-btn>
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
}
.lyric-line:hover {
  color: rgba(var(--v-theme-on-background), 0.5);
}
.lyric-line.active {
  color: rgb(var(--v-theme-on-background));
  font-size: 19px;
  font-weight: 600;
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

.offset-bar {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 4px 8px;
  flex-shrink: 0;
  border-top: 1px solid rgba(var(--v-theme-on-background), 0.06);
}
.offset-value {
  font-size: 12px;
  color: var(--v-text-secondary);
  min-width: 44px;
  text-align: center;
  font-variant-numeric: tabular-nums;
}
</style>
