<script setup lang="ts">
import { computed } from 'vue'
import type { LrcLine } from '../../composables/useLyrics'
import type { GlowStrength } from '../../composables/useDesktopLyrics'

interface Props {
  current: LrcLine | null
  next: LrcLine | null
  fontSize: number
  colorCurrent: string
  colorNext: string
  glow: GlowStrength
}

const props = defineProps<Props>()

const GLOW_BLUR: Record<GlowStrength, number> = {
  off: 0,
  weak: 2,
  medium: 4,
  strong: 8,
}

const currentStyle = computed(() => ({
  fontSize: `${props.fontSize}px`,
  color: props.colorCurrent,
  textShadow: GLOW_BLUR[props.glow] > 0
    ? `0 0 ${GLOW_BLUR[props.glow]}px ${props.colorCurrent}`
    : 'none',
}))

const nextStyle = computed(() => ({
  fontSize: `${Math.round(props.fontSize * 0.75)}px`,
  color: props.colorNext,
}))
</script>

<template>
  <div class="lyric-pair">
    <div class="lyric-current" :style="currentStyle">
      {{ current?.text ?? '' }}
    </div>
    <div v-if="next" class="lyric-next" :style="nextStyle">
      {{ next.text }}
    </div>
  </div>
</template>

<style scoped>
.lyric-pair {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  width: 100%;
  text-align: center;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  font-weight: 600;
  line-height: 1.3;
  user-select: none;
  transition: opacity 0.2s ease;
}
.lyric-current {
  transition: text-shadow 0.3s ease, color 0.3s ease;
}
.lyric-next {
  font-weight: 400;
  opacity: 0.85;
}
</style>
