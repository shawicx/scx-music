// 基础类型定义
export interface Song {
  id: string
  title: string
  artist: string
  album: string
  duration: string
  durationSecs: number
  quality: string
  filePath: string
  artGradient: string
}

export interface Playlist {
  name: string
  id: string
}

// 播放器相关类型
export type PlaybackMode = 'sequential' | 'repeat_all' | 'repeat_one' | 'shuffle'
export type PlaybackState = 'playing' | 'paused' | 'stopped'
export type VisualizationStyle = 'bar' | 'circular' | 'wave' | 'particle'

// 主题相关类型 — ThemeColor 和 ThemeMode 统一从 plugins/vuetify.ts 导出

// 视图相关类型
export type ViewMode = 'list' | 'grid'
export type DisplayMode = 'songs' | 'albums' | 'artists'
export type SortBy = 'title' | 'artist' | 'album' | 'duration' | 'default'
export type SortOrder = 'asc' | 'desc'

// 设置相关类型
export interface AppSettings {
  theme_color: string
  theme_mode: string
  output_device: string | null
  currentSongId: string | null
  viewMode: ViewMode
  activePlaylistId: string | null
  displayMode: DisplayMode
}

// API 响应类型
export interface ApiResponse<T> {
  data?: T
  error?: string
}

export type Result<T> =
  | { success: true; data: T }
  | { success: false; error: string }

// 音频设备相关类型
export interface AudioDeviceInfo {
  name: string
  isDefault: boolean
}

export interface AudioDevicesResponse {
  devices: AudioDeviceInfo[]
  defaultDeviceName: string | null
}