export type RenderMode = 'glow' | 'flat'

export interface RendererContext {
  ctx: CanvasRenderingContext2D
  width: number
  height: number
  frequencyData: Uint8Array
  timeData: number
  themeColor: { r: number; g: number; b: number }
  /** 渲染模式：glow=暗色发光，flat=浅色扁平。由 isDark 决定。 */
  mode: RenderMode
}

export type Renderer = (ctx: RendererContext) => void
