/**
 * 时间/文件大小格式化工具（纯函数）。
 */

/**
 * @description `1h 23m` / `5m` 格式（统计/报告用，无前导 0）。
 * 原 useListeningStats/useListeningReport/useLibraryAnalysis 中的 `formatDuration`。
 */
export function formatHoursMinutes(secs: number): string {
  const hours = Math.floor(secs / 3600)
  const minutes = Math.floor((secs % 3600) / 60)
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
}

/**
 * @description `3:45` / `1:02:30` 格式（播放器进度用，m:ss 或 h:mm:ss，前导 0）。
 * 原 usePlayer 中的 `formatTime`。
 */
export function formatTimecode(secs: number): string {
  if (isNaN(secs) || !isFinite(secs)) return '0:00'
  const s = Math.max(0, Math.floor(secs))
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`
}

/**
 * @description `1.2 KB` / `4.5 MB` / `2.3 GB` 格式（文件大小）。
 * 原 useLibraryAnalysis 中的 `formatFileSize`。
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`
}
