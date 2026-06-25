import type { Renderer, RendererContext } from './types'

/**
 * 波形填充山峦：复用 wave 的路径计算，线条下方加渐变填充，像连绵山峦。
 */
export const waveFillRenderer: Renderer = ({ ctx, width, height, frequencyData, timeData, themeColor, mode }: RendererContext) => {
  ctx.clearRect(0, 0, width, height)

  const { r, g, b } = themeColor
  const baseY = height * 0.7

  const energy = frequencyData.reduce((a, c) => a + c, 0) / frequencyData.length / 255

  // 收集波形点
  const points: { x: number; y: number }[] = []
  const segments = 100
  for (let i = 0; i <= segments; i++) {
    const x = (i / segments) * width
    const freqIndex = Math.floor((i / segments) * frequencyData.length)
    const freqValue = frequencyData[freqIndex] / 255
    const wave = Math.sin(i * 0.15 + timeData * 0.001) * height * 0.15
    const y = baseY - freqValue * height * 0.45 + wave * (0.3 + freqValue * 0.7 + energy * 0.3)
    points.push({ x, y })
  }

  // 绘制填充
  ctx.beginPath()
  ctx.moveTo(0, height)
  ctx.lineTo(points[0].x, points[0].y)
  for (let i = 1; i < points.length; i++) {
    const prev = points[i - 1]
    const curr = points[i]
    ctx.quadraticCurveTo(prev.x + (curr.x - prev.x) * 0.5, curr.y, curr.x, curr.y)
  }
  ctx.lineTo(width, height)
  ctx.closePath()

  const fillGradient = ctx.createLinearGradient(0, baseY - height * 0.45, 0, height)
  fillGradient.addColorStop(0, `rgba(${r}, ${g}, ${b}, ${mode === 'glow' ? 0.5 : 0.35})`)
  fillGradient.addColorStop(1, `rgba(${r}, ${g}, ${b}, 0)`)
  ctx.fillStyle = fillGradient
  ctx.fill()

  // 绘制顶部线条
  ctx.beginPath()
  ctx.moveTo(points[0].x, points[0].y)
  for (let i = 1; i < points.length; i++) {
    const prev = points[i - 1]
    const curr = points[i]
    ctx.quadraticCurveTo(prev.x + (curr.x - prev.x) * 0.5, curr.y, curr.x, curr.y)
  }
  if (mode === 'glow') {
    ctx.strokeStyle = `rgba(${r}, ${g}, ${b}, 0.9)`
    ctx.shadowBlur = 6
    ctx.shadowColor = `rgba(${r}, ${g}, ${b}, 0.6)`
  } else {
    ctx.strokeStyle = `rgba(${r}, ${g}, ${b}, 0.8)`
  }
  ctx.lineWidth = 2
  ctx.stroke()
  ctx.shadowBlur = 0
}
