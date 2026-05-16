# 前端结构

## 页面结构

- **App.vue** - 根组件，侧边栏 + 主区域布局
- **LibraryView.vue** - 音乐库主视图
- **PlayerBar.vue** - 底部播放控制条
- **SettingsView.vue** - 设置页面
- **NowPlayingOverlay.vue** - 正在播放覆盖层

## Router

无复杂路由，使用 `activeView` 状态切换视图。

## Store (Composables)

### usePlayer.ts - 播放器状态

**最复杂逻辑：** 进度跟踪和播放队列管理

**关键状态：**
- `currentSong` - 当前歌曲
- `isPlaying` - 播放状态
- `progress` / `duration` - 播放进度
- `queue` - 播放队列

**IPC 封装：**
- `playFromQueue()` -> `player_set_queue`
- `togglePlayPause()` -> `player_pause` / `player_resume`
- `seek()` -> `player_seek`
- `setVolume()` -> `player_set_volume`

**风险点：**
- 进度更新频率 (500ms) 需要合理处理
- 播放队列索引越界检查

### useLibrary.ts - 音乐库管理

**最复杂逻辑：** 搜索、筛选、排序计算属性链

**关键状态：**
- `songs` - 所有歌曲
- `playlists` - 播放列表
- `searchQuery` - 搜索关键词
- `displayMode` - 显示模式 (songs/albums/artists)

**计算属性链：**
`currentPlaylistSongs` -> `searchedSongs` -> `displayedSongs`

**IPC 封装：**
- `loadFromDb()` -> `get_all_songs`, `get_playlists`
- `importToPlaylist()` -> `scan_music_folder`, `upsert_songs`

**容易改崩：**
- 计算属性依赖关系修改
- 搜索筛选逻辑改动

### useTheme.ts - 主题切换

**简单逻辑：** Vuetify 主题切换 + 数据库持久化

## Hooks

无额外 hooks，逻辑全部在 Composables 中。

## IPC 封装位置

所有 IPC 调用集中在 Composables 中，组件不直接调用 `invoke`。

## 关键业务逻辑

1. **播放控制** - usePlayer.ts 封装所有播放相关 IPC
2. **歌曲导入** - useLibrary.ts 的 `importToPlaylist()` 方法
3. **播放列表管理** - useLibrary.ts 的增删改查方法
