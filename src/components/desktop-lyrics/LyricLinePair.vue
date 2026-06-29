<script setup lang="ts">
import { computed } from 'vue'
import type { LrcLine } from '../../composables/useLyrics'
import type { GlowStrength } from '../../composables/useDesktopLyrics'
import { themeMeta } from '../../plugins/vuetify'

interface Props {
  current: LrcLine | null
  next: LrcLine | null
  fontSize: number
  colorCurrent: string
  colorNext: string
  glow: GlowStrength
  themeColor: string
  isDark: boolean
}

const props = defineProps<Props>()

/**
 * 渐变端点色:优先用户自定义 colorCurrent;若为默认 #FFFFFF 则用主题色(更生动)。
 * 派生淡化端点用于渐变深端。
 */
const gradientColor = computed(() => {
  if (props.colorCurrent && props.colorCurrent.toLowerCase() !== '#ffffff') {
    return props.colorCurrent
  }
  // 主题色名(teal/blue/...)转 hex
  return themeMeta[props.themeColor as keyof typeof themeMeta]?.color ?? '#009688'
})

/** hex → rgba 字符串(用于渐变淡化端点、发光色) */
function hexToRgba(hex: string, alpha: number): string {
  const h = hex.replace('#', '')
  const r = parseInt(h.slice(0, 2), 16)
  const g = parseInt(h.slice(2, 4), 16)
  const b = parseInt(h.slice(4, 6), 16)
  return `rgba(${r}, ${g}, ${b}, ${alpha})`
}

const GLOW_BLUR: Record<GlowStrength, number> = {
  off: 0,
  weak: 2,
  medium: 4,
  strong: 8,
}

const currentStyle = computed(() => {
  const base = gradientColor.value
  const isCustomColor = props.colorCurrent.toLowerCase() !== '#ffffff'
  const glowBlur = GLOW_BLUR[props.glow]

  const style: Record<string, string> = {
    fontSize: `${props.fontSize}px`,
  }

  if (isCustomColor) {
    // 用户自定义色:纯色填充(尊重用户选择,不做渐变覆盖)
    style.color = base
  } else {
    // 默认场景:主题色渐变填充
    style.background = `linear-gradient(135deg, ${base}, ${hexToRgba(base, 0.65)})`
    style['-webkit-background-clip'] = 'text'
    style['background-clip'] = 'text'
    style['-webkit-text-fill-color'] = 'transparent'
    style.color = base // fallback
  }

  // 第 1 层:描边(多层 drop-shadow 模拟,按明暗切换描边色)
  // 第 3 层:动态光晕(脉动通过 animation 改变 filter 的 drop-shadow)
  // 两层合并到 filter:off 时仅描边,有 glow 时叠加发光
  const strokeColor = props.isDark ? 'rgba(0,0,0,0.5)' : 'rgba(255,255,255,0.6)'
  const filterParts = [
    `drop-shadow(0 1px 0 ${strokeColor})`,
    `drop-shadow(0 -1px 0 ${strokeColor})`,
    `drop-shadow(1px 0 0 ${strokeColor})`,
    `drop-shadow(-1px 0 0 ${strokeColor})`,
  ]
  if (glowBlur > 0) {
    filterParts.push(`drop-shadow(0 0 ${glowBlur}px ${hexToRgba(base, 0.6)})`)
  }
  style.filter = filterParts.join(' ')

  return style
})

const nextStyle = computed(() => ({
  fontSize: `${Math.round(props.fontSize * 0.75)}px`,
  color: props.colorNext,
}))
</script>

<template>
  <div class="lyric-pair">
    <div class="lyric-current" :style="currentStyle" :class="{ glowing: glow !== 'off' }">
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
  transition: filter 0.3s ease;
}
.lyric-next {
  font-weight: 400;
  opacity: 0.85;
}

/* 第 3 层:动态光晕脉动 —— 仅 glow !== 'off' 时启用。
   改变 filter 的最后一层 drop-shadow 模糊半径,实现呼吸感。
   复用 filter 而非 text-shadow,避免与渐变透明文字冲突。 */
.lyric-current.glowing {
  animation: lyric-pulse 2.4s ease-in-out infinite;
}

@keyframes lyric-pulse {
  0%, 100% {
    opacity: 0.92;
  }
  50% {
    opacity: 1;
  }
}
</style>
