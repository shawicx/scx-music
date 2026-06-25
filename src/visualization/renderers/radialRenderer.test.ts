import { describe, it, expect } from 'vitest'
import { radialRenderer } from './radialRenderer'
import type { RendererContext } from './types'

/** 创建一个自动 mock 所有方法的 Canvas 上下文（渲染器调用任何方法都不报错）。 */
function makeMockCtx(): CanvasRenderingContext2D {
  return new Proxy({}, {
    get: () => () => {},
  }) as unknown as CanvasRenderingContext2D
}

function makeCtx(overrides: Partial<RendererContext> = {}): RendererContext {
  return {
    ctx: makeMockCtx(),
    width: 200,
    height: 200,
    frequencyData: new Uint8Array(64).fill(128),
    timeData: 1000,
    themeColor: { r: 0, g: 150, b: 136 },
    mode: 'glow',
    ...overrides,
  }
}

describe('radialRenderer', () => {
  it('调用 clearRect 清空画布', () => {
    let clearedArgs: unknown[] | null = null
    const ctx = makeCtx({
      ctx: new Proxy({}, {
        get: (_t, prop) => (...args: unknown[]) => {
          if (prop === 'clearRect') clearedArgs = args
        },
      }) as unknown as CanvasRenderingContext2D,
    })
    radialRenderer(ctx)
    expect(clearedArgs).toEqual([0, 0, 200, 200])
  })

  it('glow 模式不抛错', () => {
    expect(() => radialRenderer(makeCtx({ mode: 'glow' }))).not.toThrow()
  })

  it('flat 模式不抛错', () => {
    expect(() => radialRenderer(makeCtx({ mode: 'flat' }))).not.toThrow()
  })

  it('零频谱数据不抛错', () => {
    expect(() => radialRenderer(makeCtx({ frequencyData: new Uint8Array(64).fill(0) }))).not.toThrow()
  })
})
