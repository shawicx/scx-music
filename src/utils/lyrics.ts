/**
 * @description LRC 歌词解析（纯函数）。
 */

export interface LrcLine {
  time: number
  text: string
}

const LRC_REGEX = /\[(\d{2}):(\d{2})\.(\d{2,3})](.*)/

export function parseLrc(raw: string): LrcLine[] {
  const lines: LrcLine[] = []
  for (const line of raw.split('\n')) {
    const match = line.trim().match(LRC_REGEX)
    if (match) {
      const min = parseInt(match[1])
      const sec = parseInt(match[2])
      const ms = match[3].length === 2 ? parseInt(match[3]) * 10 : parseInt(match[3])
      lines.push({
        time: min * 60 + sec + ms / 1000,
        text: match[4].trim(),
      })
    }
  }
  lines.sort((a, b) => a.time - b.time)
  return lines
}
