# 前端结构

## 页面结构

- **App.vue** - 根组件，侧边栏 + 主区域布局，负责初始化 Store 和事件监听
- **LibraryView.vue** - 音乐库主视图
- **PlayerBar.vue** - 底部播放控制条
- **SettingsView.vue** - 设置页面
- **NowPlayingOverlay.vue** - 正在播放覆盖层
- **LyricsDisplay.vue** - 歌词显示组件（LRC 解析、同步滚动、点击跳转）

## Router

无复杂路由，使用 `activeView` 状态切换视图。

## 状态管理 (Pinia Stores)

项目使用 Pinia 进行状态管理，核心业务逻辑集中在 `stores/` 目录：

### stores/player.ts - 播放器状态 (usePlayerStore)

**关键状态：**
- `currentSong` - 当前歌曲
- `isPlaying` - 播放状态
- `progress` / `duration` - 播放进度
- `queue` - 播放队列
- `playbackMode` - 播放模式 (sequential/repeat_all/repeat_one/shuffle)
- `volume` - 音量

**IPC 封装：**
- `playFromQueue()` -> `player_set_queue`
- `togglePlayPause()` -> `player_pause` / `player_resume`
- `seek()` / `seekRelative()` -> `player_seek`
- `setVolume()` / `adjustVolume()` -> `player_set_volume`
- `next()` / `previous()` -> `player_next` / `player_previous`
- `setMode()` -> `player_set_mode`
- `stop()` -> `player_stop`
- `getState()` -> `player_get_state` (恢复播放状态)

**事件监听：**
- `audio:progress` - 播放进度更新 (500ms)
- `audio:state_change` - 播放状态变化
- `audio:track_change` - 曲目切换
- `audio:error` - 音频错误

**工具方法：**
- `formatTime()` - 时间格式化
- `progressFormatted` / `durationFormatted` - 格式化计算属性

### stores/library.ts - 音乐库管理 (useLibraryStore)

**关键状态：**
- `songs` - 所有歌曲
- `playlists` - 播放列表
- `playlistSongs` - 播放列表歌曲映射
- `searchQuery` - 搜索关键词 (防抖)
- `displayMode` - 显示模式 (songs/albums/artists)
- `drilldown` - 专辑/艺术家下钻筛选
- `sortBy` / `sortOrder` - 排序方式
- `viewMode` - 视图模式
- `ready` - 数据加载完成标志

**计算属性链：**
`currentPlaylistSongs` -> `searchedSongs` -> `drilldownFilter` -> `displayedSongs`

**额外计算属性：**
- `filteredAlbums` / `filteredArtists` - 按专辑/艺术家聚合
- `currentSong` - 当前播放歌曲
- `activePlaylist` - 当前激活播放列表

**IPC 封装：**
- `loadFromDb()` -> `get_bootstrap_data` (单次加载全部数据)
- `importToPlaylist()` -> `scan_music_folder` + `upsert_songs` + `clear_playlist` + `add_songs_to_playlist`
- 播放列表 CRUD -> `create_playlist`, `rename_playlist`, `delete_playlist`
- 播放列表歌曲 -> `add_songs_to_playlist`, `remove_song_from_playlist`, `clear_playlist`

### stores/settings.ts - 设置和主题管理 (useSettingsStore)

**关键状态：**
- `colorName` - 主题颜色
- `mode` - 主题模式 (light/dark/system)
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
- 旧主题设置自动迁移

**主题配置位置：**
- `plugins/vuetify.ts` - Vuetify 主题定义和颜色配置
- 12 种主题变体（6 颜色 × 2 模式）

## 组件结构

### 主要组件
- **AppSidebar.vue** - 侧边栏导航
- **LibraryView.vue** - 音乐库主容器
- **PlayerBar.vue** - 底部播放控制
- **SettingsView.vue** - 设置页面
- **NowPlayingOverlay.vue** - 正在播放覆盖层
- **LyricsDisplay.vue** - 歌词显示（同步滚动、点击跳转、骨架屏加载态）
- **IconButtonWithTooltip.vue** - 通用图标按钮 + Tooltip 组件

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

## 工具函数

### utils/virtualScroll.ts - 虚拟滚动
- 仅渲染可见区域的 DOM 节点
- 支持动态高度计算
- 滚动位置记忆

### utils/errorHandler.ts - 错误处理
- `invokeCommand<T>()` - 带错误处理的命令调用
- `safeInvoke<T>()` - 返回 Result 类型的安全调用
- `batchInvoke()` - 批量命令执行
- `retry()` - 重试机制

## Composables

活跃的 composables（被 Store 和组件引用）：

- **composables/usePlayer.ts** - 播放器核心逻辑（Store 的实际实现）
- **composables/useLibrary.ts** - 音乐库核心逻辑（Store 的实际实现）
- **composables/useTheme.ts** - 主题核心逻辑（Store 的实际实现，含 useColorTheme + useThemeMode）
- **composables/useToast.ts** - Toast 通知封装（全局消息提示）
- **composables/useI18n.ts** - 国际化封装（语言初始化、切换、持久化）
- **composables/usePlaybackMode.ts** - 播放模式切换（含 i18n 标签）
- **composables/useLyrics.ts** - 歌词获取 + LRC 解析 + 同步跟踪
- **composables/useDebounceSearch.ts** - 搜索防抖（300ms）
- **composables/useOptimizedSort.ts** - 排序缓存（中文 locale 支持）
- **composables/useImportExport.ts** - 导入导出功能（歌单导出 M3U/PLS、音乐库备份恢复、设置迁移）

## Animation System

GSAP-powered animation system across five interaction scenarios:

### Architecture

- **Library:** `gsap` (with Flip, ScrollTo plugins)
- **Shared composable:** `useAnimation.ts` — provides easing presets, `createTimeline()` factory, and auto-cleanup via `gsap.context()` + `onUnmounted()`
- All animations use `transform` and `opacity` only (GPU-composited)

### Composables

| Composable | Scenario | Description |
|---|---|---|
| `usePageTransition` | Library ↔ Settings | Fade + translateY page transition (out-in mode) |
| `usePlaylistTransition` | Playlist switching | Slide-left-out + slide-right-in content transition |
| `useViewModeFlip` | List ↔ Grid toggle | GSAP Flip layout animation with stagger |
| `usePlayerExpand` | Player expand/collapse | Staggered timeline for overlay elements |
| `useLyricsAnimation` | Lyrics display | Spotlight opacity gradient + GSAP ScrollTo |

### Pattern

Every composable uses `useAnimation()` for scoped GSAP context. Cleanup is automatic on component unmount. Vue `<Transition :css="false">` with JS hooks (`@enter`/`@leave`) bridges GSAP into Vue's transition lifecycle.

### Key attributes

- Song items use `data-song-id` attribute for FLIP animations
- Lyrics lines use `.lyric-line` and `.active` classes for spotlight targeting

## 音频可视化

### 文件结构 (src/visualization/)

| 文件 | 作用 |
|------|------|
| `useAudioAnalyzer.ts` | Composable: 监听 `audio:spectrum` 事件，提供响应式频率数据 |
| `useVisualizationRenderer.ts` | Composable: 管理 Canvas、requestAnimationFrame 渲染循环、DPR 适配 |
| `AudioVisualizer.vue` | 主组件: Canvas + 风格选择器 + 样式持久化 |
| `index.ts` | 桶导出 |
| `renderers/types.ts` | Renderer/RendererContext 类型定义 |
| `renderers/barRenderer.ts` | 频谱柱状图渲染器（含顶部帽效果） |
| `renderers/circularRenderer.ts` | 环形放射渲染器 |
| `renderers/waveRenderer.ts` | 流动波形渲染器（3层叠加） |
| `renderers/particleRenderer.ts` | 粒子系统渲染器（250个粒子） |

### 数据流

```
Rust TeeSource (音频流复制样本)
  ↓ push_samples
AnalyzerHandle (FFT 线程)
  ↓ emit('audio:spectrum', 64 bins)
useAudioAnalyzer (前端监听)
  ↓ ref<Uint8Array>
useVisualizationRenderer (Canvas rAF)
  ↓ Renderer(context)
Canvas 2D 渲染
```

### VisualizationStyle 类型
`'bar' | 'circular' | 'wave' | 'particle'` — 通过设置面板切换，持久化到 settings 表

## 歌词系统

### 数据流
```
stores/player.ts (progress 事件)
  ↓ 实时进度
composables/useLyrics.ts
  ↓ 进度匹配 LRC 时间戳
LyricsDisplay.vue
  ↓ 同步滚动 + 高亮当前行
```

### 歌词获取优先级
1. **SQLite 缓存** → 命中则直接返回
2. **内嵌歌词** → Lofty 从音频文件提取
3. **LRCLIB API** → 按歌名/艺术家/时长搜索
4. **缓存写入** → 获取后写入 SQLite

### LRC 解析
- 正则 `\[(\d{2}):(\d{2})\.(\d{2,3})](.*)` 解析时间戳
- `LrcLine { time: number, text: string }` 结构

## 国际化 (i18n)

### 架构
- **前端**: vue-i18n (v11)，支持中文 (zh-CN) 和英文 (en)
- **后端**: sys-locale (v0.3) 检测系统语言，settings 表持久化语言偏好

### 数据流
```
App.vue onMounted
  ↓ useI18n.initLocale()
  ↓ invoke('get_setting', { key: 'language' })
  ├─ 值为 'system' 或 null → invoke('get_system_locale') → 检测系统语言
  ├─ 值为 'zh-CN' 或 'en' → 直接使用
  └─ 无效值 → 回退到系统语言
  ↓ locale.value = resolvedLocale
```

### 文件结构
- `src/i18n.ts`: vue-i18n 配置（legacy: false, fallback: zh-CN）
- `src/locales/zh-CN.ts`: 中文翻译
- `src/locales/en.ts`: 英文翻译
- `src/composables/useI18n.ts`: i18n 组合式函数

### 命名空间
common / sidebar / library / player / settings / playbackMode / toast / empty / importExport

## IPC 封装位置

所有 IPC 调用集中在 Pinia Stores 中，组件通过 Store 操作数据，不直接调用 `invoke`。错误处理通过 `utils/errorHandler.ts` 统一管理。

## 关键业务逻辑

1. **播放控制** - stores/player.ts 封装所有播放相关 IPC + 事件监听
2. **歌曲导入** - stores/library.ts 的 `importToPlaylist()` 方法
3. **播放列表管理** - stores/library.ts 的增删改查 + 清空方法
4. **主题切换** - stores/settings.ts 的主题管理逻辑
5. **歌词同步** - composables/useLyrics.ts 进度驱动歌词行匹配
6. **数据导入导出** - composables/useImportExport.ts 歌单导出、备份恢复、设置迁移
