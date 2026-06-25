import type { Renderer, RendererContext } from './types'

/**
 * 径向放射：从中心向四周放射的线条，长度随频谱跳动。
 * 比 circular（环形圆环）更有爆发力，像声音的脉冲扩散。
 */
export const radialRenderer: Renderer = ({ ctx, width, height, frequencyData, themeColor, mode }: RendererContext) => {
  ctx.clearRect(0, 0, width, height)

  const { r, g, b } = themeColor
  const cx = width / 2
  const cy = height / 2
  const innerRadius = Math.min(width, height) * 0.08
  const maxRayLength = Math.min(width, height) * 0.4
  const rayCount = Math.min(frequencyData.length, 72)

  if (mode === 'glow') {
    ctx.shadowBlur = 6
    ctx.shadowColor = `rgba(${r}, ${g}, ${b}, 0.6)`
    ctx.globalCompositeOperation = 'lighter'
  }

  ctx.lineCap = 'round'

  for (let i = 0; i < rayCount; i++) {
    const angle = (i / rayCount) * Math.PI * 2
    const value = frequencyData[i] / 255
    const rayLength = innerRadius + value * maxRayLength

    const x1 = cx + Math.cos(angle) * innerRadius
    const y1 = cy + Math.sin(angle) * innerRadius
    const x2 = cx + Math.cos(angle) * rayLength
    const y2 = cy + Math.sin(angle) * rayLength

    ctx.beginPath()
    ctx.moveTo(x1, y1)
    ctx.lineTo(x2, y2)

    if (mode === 'glow') {
      ctx.strokeStyle = `rgba(${r}, ${g}, ${b}, ${0.3 + value * 0.7})`
      ctx.lineWidth = 2.5
    } else {
      // 浅色：描边 + 透明度随能量
      ctx.strokeStyle = `rgba(${r}, ${g}, ${b}, ${0.35 + value * 0.4})`
      ctx.lineWidth = 2
    }
    ctx.stroke()
  }

  // 中心点
  ctx.beginPath()
  ctx.arc(cx, cy, innerRadius * 0.6, 0, Math.PI * 2)
  ctx.fillStyle = mode === 'glow'
    ? `rgba(${r}, ${g}, ${b}, 0.8)`
    : `rgba(${r}, ${g}, ${b}, 0.6)`
  ctx.fill()

  ctx.shadowBlur = 0
  ctx.globalCompositeOperation = 'source-over'
}
