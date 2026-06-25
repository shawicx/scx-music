import type { Renderer, RendererContext } from './types'

export const circularRenderer: Renderer = ({ ctx, width, height, frequencyData, themeColor, mode }: RendererContext) => {
  ctx.clearRect(0, 0, width, height)

  const { r, g, b } = themeColor
  const cx = width / 2
  const cy = height / 2
  const innerRadius = Math.min(width, height) * 0.12
  const maxBarLength = Math.min(width, height) * 0.3
  const barCount = frequencyData.length
  const rotation = -Math.PI / 2

  // 背景辉光：仅暗色模式
  if (mode === 'glow') {
    const avgLow = frequencyData.slice(0, 8).reduce((a, b) => a + b, 0) / 8 / 255
    const glowGradient = ctx.createRadialGradient(cx, cy, innerRadius, cx, cy, innerRadius + maxBarLength * 0.5)
    glowGradient.addColorStop(0, `rgba(${r}, ${g}, ${b}, ${avgLow * 0.3})`)
    glowGradient.addColorStop(1, 'transparent')
    ctx.fillStyle = glowGradient
    ctx.fillRect(0, 0, width, height)
  }

  for (let i = 0; i < barCount; i++) {
    const angle = rotation + (i / barCount) * Math.PI * 2
    const value = frequencyData[i] / 255
    const barLength = value * maxBarLength

    const x1 = cx + Math.cos(angle) * (innerRadius + 4)
    const y1 = cy + Math.sin(angle) * (innerRadius + 4)
    const x2 = cx + Math.cos(angle) * (innerRadius + 4 + barLength)
    const y2 = cy + Math.sin(angle) * (innerRadius + 4 + barLength)

    ctx.beginPath()
    ctx.moveTo(x1, y1)
    ctx.lineTo(x2, y2)
    if (mode === 'glow') {
      ctx.strokeStyle = `rgba(${r}, ${g}, ${b}, ${0.3 + value * 0.7})`
    } else {
      // 浅色：描边 + 透明度递减
      ctx.strokeStyle = `rgba(${r}, ${g}, ${b}, ${0.4 + value * 0.4})`
    }
    ctx.lineWidth = Math.max(1.5, (Math.PI * 2 * innerRadius) / barCount * 0.6)
    ctx.lineCap = 'round'
    ctx.stroke()
  }

  // 中心圆
  ctx.save()
  if (mode === 'glow') {
    ctx.shadowColor = `rgba(${r}, ${g}, ${b}, 0.4)`
    ctx.shadowBlur = 20
  }
  ctx.beginPath()
  ctx.arc(cx, cy, innerRadius, 0, Math.PI * 2)
  ctx.fillStyle = mode === 'glow' ? 'rgba(20, 20, 30, 0.6)' : 'rgba(255, 255, 255, 0.9)'
  ctx.fill()
  ctx.strokeStyle = `rgba(${r}, ${g}, ${b}, 0.3)`
  ctx.lineWidth = 1.5
  ctx.stroke()
  ctx.restore()
}
