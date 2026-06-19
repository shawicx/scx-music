# 前端结构

## 页面结构

- **App.vue** - 根组件，侧边栏 + 主区域布局，负责初始化 Store 和事件监听
- **LibraryView.vue** - 音乐库主视图
- **PlayerBar.vue** - 底部播放控制条
- **PlayQueueDrawer.vue** - 播放队列右侧抽屉（GSAP Flip 重排动画、当前歌曲高亮、模式切换）
- **SettingsView.vue** - 设置页面
- **AnalysisView.vue** - 曲库分析（概览卡片 + ECharts 图表 + 排行列表）
- **StatsView.vue** - 听歌统计（双 Tab：`统计`=概览卡片+最爱歌曲/歌手排行+流派分布+播放趋势+年度热力图；`报告`=基于自然周期的听歌总结）
- **NowPlayingOverlay.vue** - 正在播放覆盖层
- **LyricsDisplay.vue** - 歌词显示组件（LRC 解析、同步滚动、点击跳转）

### 独立窗口根组件
- **App.vue** - 主窗口根（侧边栏 + 主区域 + PlayerBar）
- **DesktopLyricsApp.vue** - 桌面歌词窗口根（极简，无 AppShell）
- **MiniPlayerApp.vue** - 迷你播放器窗口根（挂载 `MiniPlayer.vue`）

## Router

无复杂路由，使用 `activeView` 状态切换视图：library / settings / analysis / stats。

## 状态管理 (Pinia Stores)

项目使用 Pinia 进行状态管理，核心业务逻辑集中在 `stores/` 目录：

### stores/player.ts - 播放器状态 (usePlayerStore)

**关键状态：**
- `currentSong` - 当前歌曲
- `isPlaying` - 播放状态
- `progress` / `duration` - 播放进度
- `queue` - 播放队列（由 usePlayQueue 按播放模式生成）
- `sourceSongs` - 原始歌曲列表（队列的生成源）
- `playbackMode` - 播放模式 (sequential/repeat_all/repeat_one/shuffle)
- `volume` - 音量

**IPC 封装：**
- `playFromQueue()` -> `player_set_queue`（通过 `generateQueue` 按模式生成队列后发送）
- `regenerateQueue()` -> `player_set_queue`（切换模式时重新生成队列，保持当前歌曲位置）
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
- `updateSongInQueue(updatedSong)` - 更新队列数组中对应歌曲，同步更新 `currentSong` ref（用于歌曲重命名后保持播放状态一致）

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
- `renameSong(songId, newTitle, newArtist?, newAlbum?)` -> `rename_song` — 更新本地 `songs` 状态，通过 `usePlayerStore.updateSongInQueue` 同步播放队列，显示 Toast
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

### stores/analysis.ts - 曲库分析 (useAnalysisStore)

**薄 Store 包装：** 实际逻辑在 `composables/useLibraryAnalysis.ts`

**关键状态：**
- `stats` - LibraryStats 聚合数据（总量、排行、分布）
- `loading` - 加载状态

**IPC 封装：**
- `loadStats()` -> `get_library_stats`

**工具方法：**
- `formattedTotalSize` / `formattedTotalDuration` - 格式化计算属性
- `formatFileSize()` / `formatDuration()` - 格式化工具

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
- **PlayerBar.vue** - 底部播放控制（toggleQueue 事件触发 PlayQueueDrawer）
- **PlayQueueDrawer.vue** - 播放队列抽屉（GSAP Flip 列表重排动画、点击歌曲播放、模式切换按钮）
- **SettingsView.vue** - 设置页面
- **AnalysisView.vue** - 曲库分析（5 概览卡片 + 4 ECharts 图表 + 专辑排行列表）
- **NowPlayingOverlay.vue** - 正在播放覆盖层
- **LyricsDisplay.vue** - 歌词显示（同步滚动、点击跳转、骨架屏加载态）
- **IconButtonWithTooltip.vue** - 通用图标按钮 + Tooltip 组件

### ReportTab.vue

报告 Tab 组件，展示基于自然周期（周/月/年）的听歌总结。嵌入 StatsView 的 `v-window-item value="report"` 中。

**依赖**: `useListeningReport` composable
**功能**:
- 周期选择器（周/月/年 + offset 翻页 + 进行中标记）
- 4 张概览卡（时长/次数/独立歌曲/艺术家）
- 24 小时听歌时段分布柱状图（ECharts，按 4 时段配色）
- 峰值时段洞察标签（夜猫子/早起鸟/通用模板）

### Library 子组件 (components/library/)
- **BrowseCards.vue** - 浏览卡片视图（专辑/艺术家网格）
- **EmptyStates.vue** - 空状态提示组件
- **LibraryHeader.vue** - 库头部工具栏（搜索、显示模式切换）
- **RenameDialog.vue** - 歌曲重命名对话框（标题必填、艺术家、专辑可选，loading/error 状态，Enter 提交）
- **UpdateDialog.vue** - 自动更新提示弹窗（available/downloading/ready/error 四状态，进度条，重启按钮）
- **SongGrid.vue** - 歌曲网格视图（小数据量）
- **SongTable.vue** - 歌曲表格视图（小数据量）
- **SortMenu.vue** - 排序菜单（标题/艺术家/专辑/时长排序）
- **VirtualSongTable.vue** - 虚拟滚动表格（大数据优化，>100首歌）

### Settings 子组件 (components/settings/)

### `ShortcutSettings` / `KeyCaptureField`

`src/components/settings/ShortcutSettings.vue` 和 `KeyCaptureField.vue`

- `ShortcutSettings` — 设置面板子组件，分组列出 11 个动作，处理冲突检测、rebind、reset
- `KeyCaptureField` — 单行控件，捕获用户按下的组合键并 emit `rebind` 事件
- 捕获流程：点击「重新绑定」→ 显示「请按下组合键…」→ 捕获到完整组合 → 显示预览 + 「保存」按钮

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
- **composables/usePlaybackMode.ts** - 播放模式切换（cycleMode 后调用 regenerateQueue 重新生成队列）
- **composables/usePlayQueue.ts** - 播放队列生成（Fisher-Yates 洗牌、模式映射：sequential/repeat_all=原序、repeat_one=仅当前歌曲、shuffle=洗牌后当前歌曲排首位）
- **composables/useLyrics.ts** - 歌词获取 + LRC 解析 + 同步跟踪
- **composables/useDebounceSearch.ts** - 搜索防抖（300ms）
- **composables/useOptimizedSort.ts** - 排序缓存（中文 locale 支持）
- **composables/useImportExport.ts** - 导入导出功能（歌单导出 M3U/PLS、音乐库备份恢复、设置迁移）
- **composables/useAutoUpdate.ts** - 自动更新逻辑（启动延迟检查、下载进度跟踪、重启安装）
- **composables/useLibraryAnalysis.ts** - 曲库分析（loadStats IPC、格式化工具）
- **composables/useListeningReport.ts** - 报告周期状态管理（见下文）
- **composables/useMiniPlayer.ts** - 迷你播放器窗口生命周期管理（互斥切换、置顶、位置持久化，见下文）
- **composables/useGlobalShortcuts.ts** - 全局快捷键注册与路由（见下文）

### useListeningReport.ts

管理报告周期状态（`PeriodState: { kind, offset }`），推导自然周期的 start/end UTC 时间范围。

**核心导出**:
- `periodState` — 当前周期状态（周/月/年 + offset）
- `periodRange` — 计算出的 start/end（Date 对象 + UTC 字符串）
- `periodLabel` — 周期显示标签
- `isInProgress` — 当前周期是否未结束
- `overview` / `hourlyDistribution` — 加载的统计数据
- `dominantSlot` / `peakHourRange` — 峰值时段计算
- `shiftPeriod` / `setKind` — 周期导航方法

### useMiniPlayer.ts

迷你播放器窗口生命周期管理 composable。镜像 `useDesktopLyrics` 模式，按 `getCurrentWindow().label === 'mini-player'` 区分主窗口/迷你窗口两端行为。

**职责：**
- **窗口切换**：`enter()` 主窗口 hide + 迷你窗口 show；`exit()` 反之；`toggle()` 根据 `active` 状态分派
- **置顶管理**：`toggleAlwaysOnTop()` 切换 `alwaysOnTop` 并通过事件跨窗口同步
- **位置持久化**：`restoreFromSettings()` 仅迷你窗口调用，恢复尺寸/位置（物理/逻辑坐标已处理）
- **状态同步**：通过 `mini-player:active-changed` 和 `mini-player:always-on-top-changed` 事件跨窗口同步

**核心导出：**
- `active` / `alwaysOnTop` — 响应式状态
- `enter()` / `exit()` / `toggle()` — 窗口切换方法
- `toggleAlwaysOnTop()` — 置顶切换
- `restoreFromSettings()` — 位置恢复（仅迷你窗口）
- `isMiniPlayerWindow` — 当前是否处于迷你窗口

**持久化键（settings 表，无数据库迁移）：**
- `mini-player.active` — 是否处于迷你模式
- `mini-player.always-on-top` — 置顶状态
- `mini-player.position-x` / `mini-player.position-y` — 窗口位置（逻辑坐标）

**多实例说明：** `useMiniPlayer()` 可在主窗口被多处调用（App.vue、PlayerBar.vue）。`toggling` 标志已提升到模块级，跨实例共享，防止并发 toggle 竞争。

### `useGlobalShortcuts`

`src/composables/useGlobalShortcuts.ts`

- 模块级单例 — 多次调用 `useGlobalShortcuts()` 共享状态（`unlisten`、`defaultsCache`、`initPromise`）
- `init()` — 应用启动时调用；拉取默认值 + 存储绑定，批量注册，监听 `shortcut-triggered` 事件
- `rebind(actionId, newCombo)` — 事务性重绑，失败回滚到原组合
- `setEnabled(actionId, enabled)` — 启用/禁用某动作
- `isComboRegistered(combo)` — 系统层冲突预检
- `resetAll()` — 重置到默认值
- 内部维护 `ACTION_HANDLERS` 表，把 action_id 路由到 `usePlayerStore`、`useMiniPlayer`、`useDesktopLyrics`、`usePlaybackMode` 等
- mute 通过模块级 `mutedVolume` 变量 + `shortcut.muted-volume` 设置项实现（usePlayer 无 toggleMute）

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
| PlayQueueDrawer | Play queue reorder | GSAP Flip list reorder on mode switch (0.35s) |
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
common / sidebar / library / player / settings / playbackMode / toast / empty / importExport / update / analysis / report

### player 命名空间（播放队列相关）
- `playQueue` - "播放队列" / "Play Queue"
- `songCount` - "{count} 首" / "{count} songs"
- `noQueue` - "暂无播放队列" / "No play queue"

## IPC 封装位置

所有 IPC 调用集中在 Pinia Stores 中，组件通过 Store 操作数据，不直接调用 `invoke`。错误处理通过 `utils/errorHandler.ts` 统一管理。

## 桌面歌词窗口（Desktop Lyrics Window）

独立 Tauri 窗口（label: `desktop-lyrics`），半透明、置顶、双行显示当前播放歌词（当前句高亮 + 下一句预览）。

### 文件结构

- `src/main.ts`：检测 `window.location.hash === '#desktop-lyrics'`，条件挂载 `DesktopLyricsApp.vue`（极简根）或 `App.vue`（主窗口根）
- `src/desktop-lyrics/DesktopLyricsApp.vue`：歌词窗口根组件（无 AppShell/Sidebar/PlayerBar）
- `src/components/desktop-lyrics/DesktopLyricsWindow.vue`：主容器（拖拽、状态恢复、渲染）
- `src/components/desktop-lyrics/LyricLinePair.vue`：双行渲染
- `src/components/desktop-lyrics/LockBadge.vue`：悬浮锁定按钮（仅 unlocked 态可见）
- `src/composables/useDesktopLyrics.ts`：状态 + IPC + 持久化，**在主窗口与歌词窗口两端实例化**，按 `getCurrentWindow().label === 'desktop-lyrics'` 区分行为

### 窗口属性

`tauri.conf.json` 第二个窗口：`transparent: true`、`decorations: false`、`alwaysOnTop: true`、`skipTaskbar: true`、`closable: false`（防 Alt+F4 销毁）、`visible: false`（默认隐藏，PlayerBar 按钮触发显示）。

### 与主窗口的关系

- **事件复用**：`audio:progress` / `audio:track_change` 已广播到所有 webview，歌词窗口自动接收，无需新增 IPC
- **配置同步**：自定义事件 `desktop-lyrics:config-changed`（主窗口 → 歌词窗口）、`desktop-lyrics:lock-changed`（双向）
- **持久化**：所有配置走既有 settings 表，前缀 `desktop-lyrics.*`，无数据库迁移

### 锁定/点击穿透状态机

- **unlocked**：可拖拽、LockBadge 可见、点击 LockBadge 进入 locked
- **locked**：`setIgnoreCursorEvents(true)` 整窗光标穿透、LockBadge 不可点击、只能通过 SettingsView 解锁

## 迷你播放器窗口（Mini Player Window）

独立 Tauri 窗口（label: `mini-player`），与主窗口互斥切换。无边框、置顶、固定尺寸，提供极简播放控制（封面 + 标题/艺术家 + 上一首/播放暂停/下一首 + 置顶 + 展开 + 关闭）。

### 文件结构

- `src/main.ts`：检测 `window.location.hash === '#mini-player'`，条件挂载 `MiniPlayerApp.vue`
- `src/mini-player/MiniPlayerApp.vue`：迷你窗口根组件（无 AppShell/Sidebar/PlayerBar）
- `src/mini-player/MiniPlayer.vue`：主容器（拖拽、置顶按钮、展开/关闭、还原位置）
- `src/composables/useMiniPlayer.ts`：状态 + IPC + 持久化，**在主窗口与迷你窗口两端实例化**，按 `getCurrentWindow().label === 'mini-player'` 区分行为

### 窗口属性

`tauri.conf.json` 第三个业务窗口：`transparent: false`（不透明）、`decorations: false`、`alwaysOnTop: true`、`resizable: false`、固定 360×100、`visible: false`（默认隐藏，由 PlayerBar 按钮 / 快捷键 / 最小化按钮触发显示）。

### 与主窗口的关系（互斥）

- **互斥切换**：进入迷你模式时主窗口完全 `hide()`，退出时主窗口 `show()` + 迷你窗口 `hide()`。与桌面歌词窗口的"共存"模式不同。
- **事件复用**：`audio:progress` / `audio:track_change` / `audio:state_change` 已广播到所有 webview，迷你窗口自动接收，无需新增 IPC
- **状态同步**：自定义事件 `mini-player:active-changed`（双向）、`mini-player:always-on-top-changed`（双向）
- **持久化**：所有配置走既有 settings 表，前缀 `mini-player.*`，无数据库迁移
- **启动恢复**：App.vue onMounted 读取 `mini-player.active`，若为 `true` 则直接进入迷你模式（主窗口不显示）

### 触发入口

- PlayerBar 上的迷你模式按钮（`enter()`）
- Cmd/Ctrl+M 快捷键（App.vue 注册，`toggle()`）
- 系统最小化按钮拦截（可选，进入迷你模式而非最小化）

## 关键业务逻辑

1. **播放控制** - stores/player.ts 封装所有播放相关 IPC + 事件监听 + 队列生成（通过 usePlayQueue）
2. **播放队列** - composables/usePlayQueue.ts 按播放模式从歌曲列表派生只读队列，PlayQueueDrawer 右侧抽屉展示
2. **歌曲导入** - stores/library.ts 的 `importToPlaylist()` 方法
3. **歌曲重命名** - stores/library.ts 的 `renameSong()` 方法（后端更新元数据+文件+数据库，前端同步歌曲列表和播放队列）
4. **播放列表管理** - stores/library.ts 的增删改查 + 清空方法
5. **主题切换** - stores/settings.ts 的主题管理逻辑
6. **歌词同步** - composables/useLyrics.ts 进度驱动歌词行匹配
7. **数据导入导出** - composables/useImportExport.ts 歌单导出、备份恢复、设置迁移
8. **自动更新** - composables/useAutoUpdate.ts 启动时延迟检查更新，用户确认后下载安装，UpdateDialog 显示进度和状态
