export const ALBUM_GRADIENTS = [
  'linear-gradient(135deg, #14b8a6, #0d9488)',
  'linear-gradient(135deg, #f093fb, #f5576c)',
  'linear-gradient(135deg, #43e97b, #38f9d7)',
  'linear-gradient(135deg, #fa709a, #fee140)',
  'linear-gradient(135deg, #4facfe, #00f2fe)',
  'linear-gradient(135deg, #a18cd1, #fbc2eb)',
  'linear-gradient(135deg, #667eea, #764ba2)',
  'linear-gradient(135deg, #89f7fe, #66a6ff)',
] as const

export function getGradientForIndex(index: number): string {
  return ALBUM_GRADIENTS[index % ALBUM_GRADIENTS.length]
}

export function getGradientForString(str: string): string {
  return ALBUM_GRADIENTS[str.length % ALBUM_GRADIENTS.length]
}