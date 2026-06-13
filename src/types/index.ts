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
  genre: string
  fileSize: number
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

// 曲库分析
export interface LibraryStats {
  totalSongs: number
  totalArtists: number
  totalAlbums: number
  totalDurationSecs: number
  totalFileSize: number
  artistRanking: ArtistCount[]
  albumRanking: AlbumCount[]
  genreDistribution: GenreCount[]
  qualityDistribution: QualityCount[]
  durationDistribution: DurationBucket[]
}

export interface ArtistCount {
  artist: string
  songCount: number
  totalDurationSecs: number
}

export interface AlbumCount {
  album: string
  artist: string
  songCount: number
}

export interface GenreCount {
  genre: string
  songCount: number
}

export interface QualityCount {
  quality: string
  songCount: number
}

export interface DurationBucket {
  label: string
  songCount: number
}

// 听歌统计
export interface ListeningOverview {
  playCount: number
  totalDurationSecs: number
  genreCount: number
  artistCount: number
}

export interface TopSong {
  songId: string
  title: string
  artist: string
  playCount: number
  totalDurationSecs: number
}

export interface TopArtist {
  artist: string
  playCount: number
  totalDurationSecs: number
  songCount: number
}

export interface GenreDuration {
  genre: string
  durationSecs: number
}

export interface DayDuration {
  date: string
  durationSecs: number
}

export type ListeningRange = '7d' | '30d' | 'all'