import { describe, it, expect } from 'vitest'
import { waveFillRenderer } from './waveFillRenderer'
import type { RendererContext } from './types'

/** 创建一个自动 mock 所有方法的 Canvas 上下文（渲染器调用任何方法都不报错）。 */
function makeMockCtx(): CanvasRenderingContext2D {
  const gradientStub = { addColorStop: () => {} }
  return new Proxy({}, {
    get: (_t, prop) => {
      if (prop === 'createLinearGradient') return () => gradientStub
      return () => {}
    },
  }) as unknown as CanvasRenderingContext2D
}

function makeCtx(overrides: Partial<RendererContext> = {}): RendererContext {
  return {
    ctx: makeMockCtx(),
    width: 300,
    height: 120,
    frequencyData: new Uint8Array(64).fill(128),
    timeData: 1000,
    themeColor: { r: 0, g: 150, b: 136 },
    mode: 'glow',
    ...overrides,
  }
}

describe('waveFillRenderer', () => {
  it('调用 clearRect 清空画布', () => {
    let clearedArgs: unknown[] | null = null
    const gradientStub = { addColorStop: () => {} }
    const ctx = makeCtx({
      ctx: new Proxy({}, {
        get: (_t, prop) => {
          if (prop === 'createLinearGradient') return () => gradientStub
          return (...args: unknown[]) => {
            if (prop === 'clearRect') clearedArgs = args
          }
        },
      }) as unknown as CanvasRenderingContext2D,
    })
    waveFillRenderer(ctx)
    expect(clearedArgs).toEqual([0, 0, 300, 120])
  })

  it('glow 模式不抛错', () => {
    expect(() => waveFillRenderer(makeCtx({ mode: 'glow' }))).not.toThrow()
  })

  it('flat 模式不抛错', () => {
    expect(() => waveFillRenderer(makeCtx({ mode: 'flat' }))).not.toThrow()
  })
})
