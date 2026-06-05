<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import type { LrcLine } from '../composables/useLyrics'
import { useI18n } from '../composables/useI18n'

const props = defineProps<{
  lines: LrcLine[]
  currentLineIndex: number
  isLoading: boolean
}>()

const emit = defineEmits<{
  seek: [time: number]
}>()

const { t } = useI18n()
const containerRef = ref<HTMLElement | null>(null)
const userScrolling = ref(false)
let scrollTimer: ReturnType<typeof setTimeout> | null = null

function onScroll() {
  userScrolling.value = true
  if (scrollTimer) clearTimeout(scrollTimer)
  scrollTimer = setTimeout(() => {
    userScrolling.value = false
  }, 5000)
}

watch(() => props.currentLineIndex, async (idx) => {
  if (idx < 0 || userScrolling.value || !containerRef.value) return
  await nextTick()
  const active = containerRef.value.querySelector('.lyric-line.active') as HTMLElement
  active?.scrollIntoView({ behavior: 'smooth', block: 'center' })
})

function onClickLine(line: LrcLine) {
  if (line.time > 0) {
    emit('seek', line.time)
  }
}
</script>

<template>
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
</template>

<style scoped>
.lyrics-display {
  flex: 1;
  width: 100%;
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
  transition: color 0.3s ease, font-size 0.3s ease;
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
</style>
