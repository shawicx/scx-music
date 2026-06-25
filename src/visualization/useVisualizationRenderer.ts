import { onMounted, onUnmounted, type Ref } from 'vue'
import { useTheme } from 'vuetify'
import type { VisualizationStyle } from '../types'
import type { Renderer, RenderMode } from './renderers/types'
import { barRenderer } from './renderers/barRenderer'
import { circularRenderer } from './renderers/circularRenderer'
import { waveRenderer } from './renderers/waveRenderer'
import { mirrorRenderer } from './renderers/mirrorRenderer'
import { radialRenderer } from './renderers/radialRenderer'
import { waveFillRenderer } from './renderers/waveFillRenderer'

const RENDERERS: Record<VisualizationStyle, Renderer> = {
  bar: barRenderer,
  circular: circularRenderer,
  wave: waveRenderer,
  mirror: mirrorRenderer,
  radial: radialRenderer,
  'wave-fill': waveFillRenderer,
}

function parseHexColor(hex: string): { r: number; g: number; b: number } {
  const h = hex.replace('#', '')
  return {
    r: parseInt(h.slice(0, 2), 16),
    g: parseInt(h.slice(2, 4), 16),
    b: parseInt(h.slice(4, 6), 16),
  }
}

export function useVisualizationRenderer(
  canvas: Ref<HTMLCanvasElement | null>,
  style: Ref<VisualizationStyle>,
  frequencyData: Ref<Uint8Array>,
) {
  const vuetifyTheme = useTheme()
  let animFrameId: number | null = null

  function render() {
    const el = canvas.value
    if (!el) return

    const ctx = el.getContext('2d')
    if (!ctx) return

    const dpr = window.devicePixelRatio || 1
    const rect = el.getBoundingClientRect()
    if (el.width !== rect.width * dpr || el.height !== rect.height * dpr) {
      el.width = rect.width * dpr
      el.height = rect.height * dpr
      ctx.scale(dpr, dpr)
    }

    const colors = vuetifyTheme.current.value.colors
    const themeColor = parseHexColor(colors.secondary as string)
    const mode: RenderMode = vuetifyTheme.current.value.dark ? 'glow' : 'flat'

    const renderer = RENDERERS[style.value]
    renderer({
      ctx,
      width: rect.width,
      height: rect.height,
      frequencyData: frequencyData.value,
      timeData: performance.now(),
      themeColor,
      mode,
    })

    animFrameId = requestAnimationFrame(render)
  }

  function start() {
    if (animFrameId !== null) return
    render()
  }

  function stop() {
    if (animFrameId !== null) {
      cancelAnimationFrame(animFrameId)
      animFrameId = null
    }
  }

  function handleVisibility() {
    if (document.hidden) {
      stop()
    } else {
      start()
    }
  }

  onMounted(() => {
    document.addEventListener('visibilitychange', handleVisibility)
  })

  onUnmounted(() => {
    document.removeEventListener('visibilitychange', handleVisibility)
    stop()
  })

  return { start, stop }
}
