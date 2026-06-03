import type { Renderer } from './types'

interface Particle {
  x: number
  y: number
  baseSize: number
  speed: number
  angle: number
  binIndex: number
}

let particles: Particle[] = []
let initialized = false

function initParticles(width: number, height: number) {
  const count = 250
  particles = []
  for (let i = 0; i < count; i++) {
    const angle = Math.random() * Math.PI * 2
    const distance = Math.random() * Math.min(width, height) * 0.4
    particles.push({
      x: width / 2 + Math.cos(angle) * distance,
      y: height / 2 + Math.sin(angle) * distance,
      baseSize: 1 + Math.random() * 3,
      speed: 0.1 + Math.random() * 0.3,
      angle: Math.random() * Math.PI * 2,
      binIndex: Math.floor(Math.random() * 64),
    })
  }
  initialized = true
}

export const particleRenderer: Renderer = ({ ctx, width, height, frequencyData, themeColor }) => {
  ctx.clearRect(0, 0, width, height)

  if (!initialized || particles.length === 0) {
    initParticles(width, height)
  }

  const { r, g, b } = themeColor
  const cx = width / 2
  const cy = height / 2
  const maxDist = Math.min(width, height) * 0.45

  for (const p of particles) {
    const value = frequencyData[p.binIndex] / 255

    p.x += Math.cos(p.angle) * p.speed * (0.5 + value)
    p.y += Math.sin(p.angle) * p.speed * (0.5 + value)

    p.angle += (Math.random() - 0.5) * 0.1

    const dx = p.x - cx
    const dy = p.y - cy
    const dist = Math.sqrt(dx * dx + dy * dy)
    if (dist > maxDist) {
      p.x = cx + (dx / dist) * (maxDist * 0.1)
      p.y = cy + (dy / dist) * (maxDist * 0.1)
    }

    const size = p.baseSize * (1 + value * 3)
    const alpha = 0.2 + value * 0.7

    const gradient = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, size * 3)
    gradient.addColorStop(0, `rgba(${r}, ${g}, ${b}, ${alpha})`)
    gradient.addColorStop(0.4, `rgba(${r}, ${g}, ${b}, ${alpha * 0.5})`)
    gradient.addColorStop(1, 'transparent')

    ctx.fillStyle = gradient
    ctx.fillRect(p.x - size * 3, p.y - size * 3, size * 6, size * 6)

    const coreR = Math.min(255, r + 100)
    const coreG = Math.min(255, g + 100)
    const coreB = Math.min(255, b + 100)
    ctx.fillStyle = `rgba(${coreR}, ${coreG}, ${coreB}, ${alpha})`
    ctx.beginPath()
    ctx.arc(p.x, p.y, size * 0.5, 0, Math.PI * 2)
    ctx.fill()
  }
}
