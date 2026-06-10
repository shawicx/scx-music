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

const GLOW_LAYERS = 4
const GLOW_BASE_SIZE = 48
const glowCanvases: HTMLCanvasElement[] = []

function buildGlowCache(r: number, g: number, b: number) {
  glowCanvases.length = 0
  for (let layer = 0; layer < GLOW_LAYERS; layer++) {
    const alpha = 0.25 + (layer / GLOW_LAYERS) * 0.55
    const size = GLOW_BASE_SIZE * (1 - layer * 0.15)
    const c = document.createElement('canvas')
    c.width = size
    c.height = size
    const cx = c.getContext('2d')!
    const half = size / 2
    const gradient = cx.createRadialGradient(half, half, 0, half, half, half)
    gradient.addColorStop(0, `rgba(${r}, ${g}, ${b}, ${alpha})`)
    gradient.addColorStop(0.4, `rgba(${r}, ${g}, ${b}, ${alpha * 0.5})`)
    gradient.addColorStop(1, 'transparent')
    cx.fillStyle = gradient
    cx.fillRect(0, 0, size, size)
    glowCanvases.push(c)
  }
}

let cachedColorKey = ''

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
  const colorKey = `${r},${g},${b}`
  if (colorKey !== cachedColorKey) {
    buildGlowCache(r, g, b)
    cachedColorKey = colorKey
  }

  const cx = width / 2
  const cy = height / 2
  const maxDist = Math.min(width, height) * 0.45

  ctx.globalCompositeOperation = 'lighter'

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
    const alpha = 0.3 + value * 0.7

    const layerIdx = Math.min(GLOW_LAYERS - 1, Math.floor(alpha * GLOW_LAYERS))
    const glow = glowCanvases[layerIdx]
    const drawSize = size * 6
    ctx.globalAlpha = alpha
    ctx.drawImage(glow, p.x - drawSize / 2, p.y - drawSize / 2, drawSize, drawSize)
  }

  ctx.globalCompositeOperation = 'source-over'
  ctx.globalAlpha = 1
}
