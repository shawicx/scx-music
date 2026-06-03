import type { Renderer } from './types'

export const waveRenderer: Renderer = ({ ctx, width, height, frequencyData, timeData, themeColor }) => {
  ctx.clearRect(0, 0, width, height)

  const { r, g, b } = themeColor
  const layers = [
    { amplitude: 0.3, frequency: 0.5, speed: 0.0008, alpha: 0.15, brightness: -20 },
    { amplitude: 0.5, frequency: 0.8, speed: 0.0012, alpha: 0.25, brightness: 0 },
    { amplitude: 0.7, frequency: 1.2, speed: 0.002, alpha: 0.4, brightness: 20 },
  ]

  const midY = height / 2

  for (const layer of layers) {
    const energy = frequencyData.reduce((a, b) => a + b, 0) / frequencyData.length / 255
    const lr = Math.max(0, Math.min(255, r + layer.brightness))
    const lg = Math.max(0, Math.min(255, g + layer.brightness))
    const lb = Math.max(0, Math.min(255, b + layer.brightness))

    ctx.beginPath()

    const segments = 100
    for (let i = 0; i <= segments; i++) {
      const x = (i / segments) * width
      const freqIndex = Math.floor((i / segments) * frequencyData.length)
      const freqValue = frequencyData[freqIndex] / 255

      const wave1 = Math.sin((i * layer.frequency * 0.1) + timeData * layer.speed) * layer.amplitude
      const wave2 = Math.sin((i * layer.frequency * 0.05) + timeData * layer.speed * 1.3) * layer.amplitude * 0.5

      const y = midY + (wave1 + wave2) * (height * 0.3) * (0.3 + freqValue * 0.7 + energy * 0.3)

      if (i === 0) {
        ctx.moveTo(x, y)
      } else {
        const prevX = ((i - 1) / segments) * width
        ctx.quadraticCurveTo(prevX + (x - prevX) * 0.5, y, x, y)
      }
    }

    ctx.strokeStyle = `rgba(${lr}, ${lg}, ${lb}, ${layer.alpha})`
    ctx.lineWidth = 2
    ctx.stroke()

    ctx.lineTo(width, height)
    ctx.lineTo(0, height)
    ctx.closePath()
    const fillGradient = ctx.createLinearGradient(0, midY, 0, height)
    fillGradient.addColorStop(0, `rgba(${lr}, ${lg}, ${lb}, ${layer.alpha * 0.3})`)
    fillGradient.addColorStop(1, 'transparent')
    ctx.fillStyle = fillGradient
    ctx.fill()
  }
}
