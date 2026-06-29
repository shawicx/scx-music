import type { Song } from '../types'
import { invokeCommand } from '../utils/errorHandler'

/**
 * 重建播放来源列表。与 useLibrary.currentPlaylistSongs 同逻辑：
 * - 无 playlistId（null）→ 全部曲库
 * - 有 playlistId 但映射缺失/空 → 空数组（歌单已删/已空）
 * 与 useLibrary 的实现保持一致，避免恢复时队列来源错位。
 */
export function rebuildSourceList(
  songs: Song[],
  playlistSongsMap: Record<string, string[]>,
  playlistId: string | null,
): Song[] {
  if (!playlistId) return songs
  const ids = playlistSongsMap[playlistId]
  if (!ids || ids.length === 0) return []
  const idSet = new Set(ids)
  return songs.filter((s) => idSet.has(s.id))
}

/**
 * 解析持久化的播放位置字符串，非法/缺失/负数归零，超过时长则钳制。
 * duration 不传时不做上限钳制。
 */
export function parsePosition(raw: string, duration?: number): number {
  const n = Number(raw)
  if (!Number.isFinite(n) || n < 0) return 0
  if (duration !== undefined && n > duration) return duration
  return n
}

/**
 * debounce 判定：当前时间距上次保存是否已超过阈值。
 */
export function shouldSavePosition(
  now: number,
  lastSaveTime: number,
  intervalMs: number,
): boolean {
  return now - lastSaveTime >= intervalMs
}

const SAVE_INTERVAL_MS = 5000
let lastSaveTime = 0

/**
 * 播放进度 debounce 写库（5s 一次）。
 * 仅在播放时由 audio:progress 监听调用；Paused/Stopped 不推送事件，故无副作用。
 */
export async function savePlaybackPosition(currentSecs: number): Promise<void> {
  const now = Date.now()
  if (!shouldSavePosition(now, lastSaveTime, SAVE_INTERVAL_MS)) return
  lastSaveTime = now
  await invokeCommand('set_setting', {
    key: 'last_position',
    value: String(currentSecs),
  }).catch(() => {})
}

/** 读取恢复播放总开关（key 不存在时默认 false）。 */
export async function isRestoreEnabled(): Promise<boolean> {
  const settings = await invokeCommand<Record<string, string>>('get_all_settings')
  return settings['restore_last_playback'] === 'true'
}

/** 写入恢复播放总开关。 */
export async function setRestoreEnabled(enabled: boolean): Promise<void> {
  await invokeCommand('set_setting', {
    key: 'restore_last_playback',
    value: String(enabled),
  })
}

/** 读取系统真实的开机自启状态。 */
export async function getAutostart(): Promise<boolean> {
  return invokeCommand<boolean>('app_get_autostart')
}

/** 开启/关闭开机自启（写入操作系统启动项）。 */
export async function setAutostart(enabled: boolean): Promise<void> {
  await invokeCommand('app_set_autostart', { enabled })
}

/**
 * 聚合导出：供组件统一解构使用。
 * 内部函数均为模块级单例（位置 debounce 状态、IPC 封装），无需响应式。
 */
export function useStartupOptions() {
  return {
    savePlaybackPosition,
    isRestoreEnabled,
    setRestoreEnabled,
    getAutostart,
    setAutostart,
    rebuildSourceList,
    parsePosition,
    shouldSavePosition,
  }
}
