import type { Renderer, RendererContext } from './types'

const BAR_COUNT = 64
const BAR_GAP = 3

/**
 * 镜像对称频谱：柱条以中线为轴上下镜像，像声波倒影。
 * 复用 bar 的柱条计算逻辑，绘制两份（中线之上 + 之下镜像）。
 */
export const mirrorRenderer: Renderer = ({ ctx, width, height, frequencyData, themeColor, mode }: RendererContext) => {
  ctx.clearRect(0, 0, width, height)

  const { r, g, b } = themeColor
  const totalGap = (BAR_COUNT - 1) * BAR_GAP
  const barWidth = Math.max(1, (width - totalGap) / BAR_COUNT)
  const maxHeight = (height * 0.85) / 2 // 镜像后每侧占一半高度
  const midY = height / 2

  if (mode === 'glow') {
    ctx.shadowBlur = 8
    ctx.shadowColor = `rgba(${r}, ${g}, ${b}, 0.6)`
  }

  for (let i = 0; i < BAR_COUNT; i++) {
    const dataIndex = Math.floor((i / BAR_COUNT) * frequencyData.length)
    const value = frequencyData[dataIndex] / 255
    const barHeight = value * maxHeight
    const x = i * (barWidth + BAR_GAP)

    if (mode === 'glow') {
      ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${0.3 + value * 0.6})`
    } else {
      ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${i % 2 === 0 ? 0.7 : 0.4})`
    }

    const radius = Math.min(barWidth / 2, 3)

    // 上半部分（从中线向上）
    const topY = midY - barHeight
    drawRoundedTopBar(ctx, x, topY, barWidth, barHeight, radius)
    // 下半部分（镜像，从中线向下）
    drawRoundedTopBar(ctx, x, midY, barWidth, barHeight, radius)
  }

  ctx.shadowBlur = 0
}

/** 绘制带圆角顶部的柱条（从 baseY 向上/下延伸 height）。 */
function drawRoundedTopBar(
  ctx: CanvasRenderingContext2D,
  x: number,
  baseY: number,
  barWidth: number,
  height: number,
  radius: number,
) {
  ctx.beginPath()
  ctx.moveTo(x + radius, baseY)
  ctx.lineTo(x + barWidth - radius, baseY)
  ctx.quadraticCurveTo(x + barWidth, baseY, x + barWidth, baseY + (height > 0 ? radius : -radius))
  ctx.lineTo(x + barWidth, baseY + height)
  ctx.lineTo(x, baseY + height)
  ctx.lineTo(x, baseY + (height > 0 ? radius : -radius))
  ctx.quadraticCurveTo(x, baseY, x + radius, baseY)
  ctx.fill()
}
