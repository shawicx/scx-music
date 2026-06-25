import type { Renderer, RendererContext } from './types'

const BAR_COUNT = 64
const BAR_GAP = 3
const CAP_HEIGHT = 3
const CAP_FALL_SPEED = 1.5

let caps: number[] = []

export const barRenderer: Renderer = ({ ctx, width, height, frequencyData, themeColor, mode }: RendererContext) => {
  ctx.clearRect(0, 0, width, height)

  if (caps.length !== BAR_COUNT) {
    caps = new Array(BAR_COUNT).fill(0)
  }

  const { r, g, b } = themeColor
  const totalGap = (BAR_COUNT - 1) * BAR_GAP
  const barWidth = Math.max(1, (width - totalGap) / BAR_COUNT)
  const maxHeight = height * 0.85

  for (let i = 0; i < BAR_COUNT; i++) {
    const dataIndex = Math.floor((i / BAR_COUNT) * frequencyData.length)
    const value = frequencyData[dataIndex] / 255
    const barHeight = value * maxHeight

    const capY = height - barHeight
    if (capY < (caps[i] || height)) {
      caps[i] = capY
    } else {
      caps[i] = Math.min(height, (caps[i] || height) + CAP_FALL_SPEED)
    }

    const x = i * (barWidth + BAR_GAP)

    if (mode === 'glow') {
      // 暗色：高对比 + 发光
      ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${0.3 + value * 0.6})`
      ctx.shadowBlur = 8
      ctx.shadowColor = `rgba(${r}, ${g}, ${b}, 0.6)`
    } else {
      // 浅色：柔和半透明，奇偶交替，无发光
      ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${i % 2 === 0 ? 0.7 : 0.4})`
      ctx.shadowBlur = 0
    }

    ctx.beginPath()
    const radius = Math.min(barWidth / 2, 3)
    const y = height - barHeight
    ctx.moveTo(x + radius, y)
    ctx.lineTo(x + barWidth - radius, y)
    ctx.quadraticCurveTo(x + barWidth, y, x + barWidth, y + radius)
    ctx.lineTo(x + barWidth, height)
    ctx.lineTo(x, height)
    ctx.lineTo(x, y + radius)
    ctx.quadraticCurveTo(x, y, x + radius, y)
    ctx.fill()
    ctx.shadowBlur = 0

    if ((caps[i] ?? height) < height - 2 && mode === 'glow') {
      const lr = Math.min(255, r + 30)
      const lg = Math.min(255, g + 30)
      const lb = Math.min(255, b + 30)
      ctx.fillStyle = `rgba(${lr}, ${lg}, ${lb}, 0.9)`
      ctx.fillRect(x, caps[i], barWidth, CAP_HEIGHT)
    }
  }
}
