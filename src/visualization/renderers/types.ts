export interface RendererContext {
  ctx: CanvasRenderingContext2D
  width: number
  height: number
  frequencyData: Uint8Array
  timeData: number
  themeColor: { r: number; g: number; b: number }
}

export type Renderer = (ctx: RendererContext) => void
