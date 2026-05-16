# 前端结构

## 页面结构

- **App.vue** - 根组件，侧边栏 + 主区域布局
- **LibraryView.vue** - 音乐库主视图
- **PlayerBar.vue** - 底部播放控制条
- **SettingsView.vue** - 设置页面
- **NowPlayingOverlay.vue** - 正在播放覆盖层

## Router

无复杂路由，使用 `activeView` 状态切换视图。

## 状态管理 (Pinia Stores)

项目使用 Pinia 进行状态管理，所有业务逻辑集中在 `stores/` 目录：

### stores/player.ts - 播放器状态 (usePlayerStore)

**最复杂逻辑：** 进度跟踪和播放队列管理

**关键状态：**
- `currentSong` - 当前歌曲
- `isPlaying` - 播放状态
- `progress` / `duration` - 播放进度
- `queue` - 播放队列
- `playbackMode` - 播放模式 (sequential/repeat_all/repeat_one/shuffle)

**IPC 封装：**
- `playFromQueue()` -> `player_set_queue`
- `togglePlayPause()` -> `player_pause` / `player_resume`
- `seek()` -> `player_seek`
- `setVolume()` -> `player_set_volume`
- `setMode()` -> `player_set_mode`

**事件监听：**
- `audio:progress` - 播放进度更新
- `audio:state_change` - 播放状态变化
- `audio:track_change` - 曲目切换
- `audio:error` - 音频错误

**风险点：**
- 进度更新频率 (500ms) 需要合理处理
- 播放队列索引越界检查
- 事件监听器生命周期管理

### stores/library.ts - 音乐库管理 (useLibraryStore)

**最复杂逻辑：** 搜索、筛选、排序计算属性链

**关键状态：**
- `songs` - 所有歌曲
- `playlists` - 播放列表
- `playlistSongs` - 播放列表歌曲映射
- `searchQuery` - 搜索关键词
- `displayMode` - 显示模式 (songs/albums/artists)
- `sortBy` / `sortOrder` - 排序方式

**计算属性链：**
`currentPlaylistSongs` -> `searchedSongs` -> `displayedSongs`

**IPC 封装：**
- `loadFromDb()` -> `get_all_songs`, `get_playlists`, `get_all_settings`
- `importToPlaylist()` -> `scan_music_folder`, `upsert_songs`
- 播放列表管理 -> `create_playlist`, `rename_playlist`, `delete_playlist`

**容易改崩：**
- 计算属性依赖关系修改
- 搜索筛选逻辑改动
- 排序函数的 localeCompare 参数

### stores/settings.ts - 设置和主题管理 (useSettingsStore)

**功能：** Vuetify 主题切换 + 数据库持久化

**关键状态：**
- `colorName` - 主题颜色
- `mode` - 主题模式
- `isDark` - 当前是否深色模式 (计算属性)

**IPC 封装：**
- `loadThemeFromDb()` -> `get_setting`
- `setColorTheme()` / `setMode()` -> `set_setting`

**主题系统：**
- 使用 Vuetify 3.x 主题系统
- 支持 6 种主题颜色：青色、靛蓝、蓝色、深紫、红色、琥珀
- 支持 3 种主题模式：浅色、深色、跟随系统
- 系统主题检测通过 VueUse 的 `usePreferredDark` 实现
- 数据库持久化用户偏好
- 响应式主题切换，自动更新 Vuetify 全局主题

**主题配置位置：**
- `plugins/vuetify.ts` - Vuetify 主题定义和颜色配置
- 12 种主题变体（6 颜色 × 2 模式）
- 支持动态 CSS 变量和渐变效果

## 组件结构

### 主要组件
- **AppSidebar.vue** - 侧边栏导航
- **LibraryView.vue** - 音乐库主容器
- **PlayerBar.vue** - 底部播放控制
- **SettingsView.vue** - 设置页面
- **NowPlayingOverlay.vue** - 正在播放覆盖层

### Library 子组件 (components/library/)
- **BrowseCards.vue** - 浏览卡片视图（专辑/艺术家网格）
- **EmptyStates.vue** - 空状态提示组件
- **LibraryHeader.vue** - 库头部工具栏（搜索、显示模式切换）
- **SongGrid.vue** - 歌曲网格视图（小数据量）
- **SongTable.vue** - 歌曲表格视图（小数据量）
- **SortMenu.vue** - 排序菜单（标题/艺术家/专辑/时长排序）
- **VirtualSongTable.vue** - 虚拟滚动表格（大数据优化，>100首歌）

### 组件使用策略
- **小数据量** (< 100首歌): 使用 SongTable.vue 或 SongGrid.vue
- **大数据量** (> 100首歌): 自动切换到 VirtualSongTable.vue 提升性能
- **虚拟滚动优势**: 仅渲染可见区域的 DOM 节点，大幅减少内存占用

## 工具函数

### utils/virtualScroll.ts - 虚拟滚动
**用途：** 大数据列表性能优化
- 仅渲染可见区域的 DOM 节点
- 支持动态高度计算
- 滚动位置记忆

### utils/errorHandler.ts - 错误处理
**用途：** 统一的 Tauri IPC 调用错误处理
- `invokeCommand<T>()` - 带错误处理的命令调用
- `safeInvoke<T>()` - 返回 Result 类型的安全调用
- `batchInvoke()` - 批量命令执行
- `retry()` - 重试机制

## Composables

保留的 composables 用于 UI 交互：
- **composables/useToast.ts** - Toast 通知封装（全局消息提示）

### useToast 功能
- `showToast()` - 显示通用通知
- `showSuccess()` - 显示成功消息
- `showError()` - 显示错误消息  
- `showWarning()` - 显示警告消息
- `showInfo()` - 显示信息消息
- `hideToast()` - 隐藏通知

## IPC 封装位置

所有 IPC 调用集中在 Pinia Stores 中，组件通过 Store 操作数据，不直接调用 `invoke`。错误处理通过 `utils/errorHandler.ts` 统一管理。

## 关键业务逻辑

1. **播放控制** - stores/player.ts 封装所有播放相关 IPC
2. **歌曲导入** - stores/library.ts 的 `importToPlaylist()` 方法
3. **播放列表管理** - stores/library.ts 的增删改查方法
4. **主题切换** - stores/settings.ts 的主题管理逻辑
