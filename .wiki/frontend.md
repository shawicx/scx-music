# 前端结构

> **2026-10-22 UI 改版**：按「原生 macOS 材质 + 编辑式排版」原型整体重构。设计语言：中性表面分层（`--surface-0..3`）+ 1px 发丝线（`--hairline`）、衬线展示标题（`--font-display`，系统衬线栈零字体加载）、玻璃仅三处（暗色侧栏 / 正在播放控制面板 / 浮层）、封面动态取色驱动 `--accent` CSS 变量组。设计令牌在 `src/styles/tokens.css`（明暗两套），共享表格样式 `src/styles/song-table.css`（含 `eq-indicator.css` EQ 指示器）。

## 页面结构

- **App.vue** - 根组件，CSS Grid 骨架 `236px 1fr / 44px 1fr 64px`（areas: tb/sb/main/pb）：AppTitlebar + AppSidebar + 主区 + **PlayerBar（所有视图常驻底行）**；NowPlayingOverlay 挂 app 层（任意视图可展开）；onMounted 启动 `startCoverAccentWatcher()` 封面动态取色。标题栏挂载条件 `isMac && !hasNativeMacBar`：onMounted 先调 `get_titlebar_inset_height` 探测系统是否真正吃进 `titleBarStyle: Overlay`（个别系统版本如 macOS 26 beta 会静默忽略 Overlay，原生标题栏照常占位），原生栏在场时给 .app-shell 加 `no-titlebar` 收起自绘标题行并清空窗口标题（`setTitle('')` + `document.title = ''`，后者必须做——wry 会把 webview 标题同步回窗口标题，只 setTitle 会被 index.html 的 `<title>` 复活）；Windows 按 UA 判断同样收起
- **AppTitlebar.vue** - macOS overlay 标题栏（44px 拖拽区，红绿灯左侧让位；tauri.conf main 窗口 `titleBarStyle: "Overlay"`，仅在系统真正生效时挂载，见 App.vue 探测）。拖拽用 `@mousedown` 显式调 `getCurrentWindow().startDragging()`（`data-tauri-drag-region` 只在 mousedown 命中属性元素本身时生效，被子元素挡住的区域全失效）；不绘制标题文字——应用名只由侧栏 brand 呈现。原生标题文字由 lib.rs setup 里 `hide_nswindow_title()`（NSWindow `setTitleVisibility:1` + `setTitlebarAppearsTransparent`）无条件隐藏：仅 `setTitle('')` 不够，macOS 对空标题会回退绘制应用名，且 JS 清空可能被 webview 标题同步复活。原生栏在场时 .app-shell 额外挂 `native-mac-bar` 类，AppSidebar 品牌行 `padding-top: 34px` 下移到红绿灯区之下（侧栏从 y=0 开始，品牌否则与红绿灯重叠）
- **LibraryView.vue** - 音乐库主视图
- **PlayerBar.vue** - 底部播放控制条（64px 三段式：左 290px 封面+元信息+心形 / 中 走带+2px 进度线（AppSlider）/ 右 290px 桌面歌词/队列/迷你/睡眠定时+音量组；36px 圆形 accent 播放键；播放模式拆为 shuffle + repeat 双按钮，repeat 循环 关→列表循环→单曲循环）
- **PlayQueueDrawer.vue** - 播放队列右侧抽屉（340px 玻璃化；GSAP Flip 重排动画、当前歌曲 EQ 指示、模式切换）
- **SettingsView.vue** - 设置页面（衬线页头 + 自绘 tabbar + 发丝线分组卡片）
- **AnalysisView.vue** - 曲库分析（概览衬线数字格 + ECharts 图表（accent 单色阶梯）+ 专辑排行横条）
- **StatsView.vue** - 听歌统计（双 Tab：`统计`=概览+最爱歌曲/歌手+流派环形+播放趋势+热力图（accent 单色阶梯）；`报告`=基于自然周期的听歌总结；tab/范围切换用 SegControl）
- **NowPlayingOverlay.vue** - 正在播放沉浸层（全屏：封面 blur(64px) 背景 + accent 径向 tint；左大封面/右 eyebrow+衬线大标题+歌词；底部玻璃控制面板（白色 44px 播放键）；EQ 快捷面板与歌词偏移步进条保留；GSAP usePlayerExpand 进出场）。歌词列必须 `height: 100%` 有界（np-body → lyrics-column → lyrics-container → lyrics-display 滚动链），否则列高被内容撑开、歌词平铺沉到控制面板下方
- **LyricsDisplay.vue** - 歌词显示组件（LRC 解析、同步滚动、点击跳转；14.5px 行 + 20px 衬线当前行，颜色走 `--np-fg*` 变量适配沉浸层明暗）
- **player/CoverArt.vue** - 封面组件（songId → useCoverArt blob URL；无封面回退 accent 16% 着色底；尺寸由父级 CSS 控制）
- **common/AppSlider.vue** - 自定义滑杆（3px track、hover 增粗、thumb 悬停浮现、pointer capture、键盘可达；`line`/`mini` 变体；进度/音量/迷你顶部进度线共用，替代 v-slider）
- **common/SegControl.vue** - 分段控件（曲库浏览方式/显示方式/统计 tab/时间范围等共用）

### 独立窗口根组件
- **App.vue** - 主窗口根（侧边栏 + 主区域 + PlayerBar）
- **DesktopLyricsApp.vue** - 桌面歌词窗口根（极简，无 AppShell）
- **MiniPlayerApp.vue** - 迷你播放器窗口根（挂载 `MiniPlayer.vue`）

## 多窗口架构（2026-06-15 起）

应用包含 4 个 Tauri 窗口，各自独立 JS 上下文，composable 模块级状态在窗口内单例。详见各窗口小节（桌面歌词窗口 / 迷你播放器窗口）和 `.wiki/architecture.md` 窗口系统表。

| 窗口 Label | 根组件 | 用途 |
|------------|--------|------|
| `main` | App.vue | 主窗口：侧边栏 + 主区域 + PlayerBar |
| `mini-player` | MiniPlayerApp.vue | 迷你播放器（与主窗口互斥切换） |
| `desktop-lyrics` | DesktopLyricsApp.vue | 桌面歌词悬浮窗（透明、置顶、双行） |
| `desktop-lyrics-lock` | （辅助窗口） | 锁定态 36×36 锁按钮窗口（光标穿透占位） |

**跨窗口通信**：`listen('event', ...)` + `app.emit('event', payload)` 全局广播。`audio:*` 事件已在 Rust 侧广播到所有 webview；业务事件（`desktop-lyrics:config-changed` / `mini-player:active-changed` 等）走同样路径。

**入口分发**：`src/main.ts` 检测 `window.location.hash`（`#mini-player` / `#desktop-lyrics`）条件挂载对应根组件，否则挂载 `App.vue`。

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
- `formatTime()` - 时间格式化（2026-06-21 D1 重构后，实现在 `utils/format.ts::formatTimecode`，此处为别名 re-export）
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
- `importToPlaylist()` -> `scan_music_folder` + `upsert_songs` + `replace_playlist_songs`（**2026-06-26 优化**：原 clear_playlist + add_songs_to_playlist 两次串行 IPC 合并为单事务原子替换）
- `renameSong(songId, newTitle, newArtist?, newAlbum?)` -> `rename_song` — 更新本地 `songs` 状态，通过 `usePlayerStore.updateSongInQueue` 同步播放队列，显示 Toast
- 播放列表 CRUD -> `create_playlist`, `rename_playlist`, `delete_playlist`
- 播放列表歌曲 -> `add_songs_to_playlist`, `remove_song_from_playlist`, `clear_playlist`

### stores/settings.ts - 设置和主题管理 (useSettingsStore)

**关键状态：**
- `colorName` - 主题颜色（六色回退）
- `mode` - 主题模式 (light/dark/system)
- `isDark` - 当前是否深色模式 (计算属性)
- `dynamicColor` - 封面动态取色开关（设置键 `dynamic_color`，默认开启）
- `accentHex` - 当前生效强调色（hex，ECharts 等非 CSS 场景读取）
- `coverAccent` - 封面提取的原始 HSL（null = 回退）

**IPC 封装：**
- `loadThemeFromDb()` -> `get_setting`（含 theme_color/theme_mode/dynamic_color）
- `setColorTheme()` / `setMode()` / `setDynamicColor()` -> `set_setting`

**动态取色机制（2026-10-22 新增）：**
- Rust 端 `get_cover_accent(songId)` 提取封面主色 HSL（36 色相桶 × 3 明度档饱和度加权直方图，进程内缓存）
- `startCoverAccentWatcher()`（App.vue onMounted）watch `currentSong.id` → 取色 → `setCoverAccent`
- `applyAccent` 按明暗钳制（s 0.34–0.72；l 暗色 0.55–0.66 / 浅色 0.34–0.44）后写 CSS 变量 `--accent/--accent-rgb/--on-accent/--np-accent/--amb-1/--amb-2`，并热更新 Vuetify current theme 的 `primary` 与 `accent-bg`（v-switch/v-slider 自动跟随）
- 低饱和（s < 0.13）/无封面回退六色主题（同样钳制）
- 跨窗口广播 `app:accent-changed`（预计算值 payload），独立 webview 用 `applyAccentBroadcast()` 直接写 CSS 变量（MiniPlayer 已接入）

### stores/analysis.ts - 曲库分析 (useAnalysisStore)

**薄 Store 包装：** 实际逻辑在 `composables/useLibraryAnalysis.ts`

**关键状态：**
- `stats` - LibraryStats 聚合数据（总量、排行、分布）
- `loading` - 加载状态

**IPC 封装：**
- `loadStats()` -> `get_library_stats`

**工具方法：**
- `formattedTotalSize` / `formattedTotalDuration` - 格式化计算属性
- `formatFileSize()` / `formatDuration()` - 格式化工具（2026-06-21 D1 重构后，实现在 `utils/format.ts::formatFileSize` / `formatHoursMinutes`，此处为别名 re-export）

### stores/stats.ts - 听歌统计 (useStatsStore)

**薄 Store 包装：** 实际逻辑在 `composables/useListeningStats.ts`

**IPC 封装：**
- `loadData(range?)` -> `stats_dashboard`（统计 Tab 单次聚合加载 overview + 排行 + 分布 + 趋势 + 热力图，`topLimit: 10`）

**主题系统：**
- 使用 Vuetify 4.0 主题系统（6 色 × 明暗双模式静态枚举 + 运行时 primary 热更新）
- 支持 6 种主题颜色（回退）：青色 #009688 / 靛蓝 #3F51B5 / 蓝色 #2196F3 / 深紫 #673AB7 / 红色 #E53935 / 琥珀 #F2A900
- 支持 3 种主题模式：浅色（极简实底）、深色（玻璃材质）、跟随系统
- **封面动态取色**：当前播放歌曲封面主色驱动 `--accent` 变量组与 Vuetify primary（详见 stores/settings.ts 小节）；曲库界面保持中性，accent 只作用于播放体验
- 中性表面：暗色 `#131318/#1b1b23/#23232e/#2a2a37`、浅色 `#ececea/#f5f5f4/rgba(0,0,0,.045)/#fff`（tokens.css `--surface-0..3`）
- **令牌挂载点是 `<html>` 上的 `scx-dark`/`scx-light` 标记类**（useTheme 的 watch 按 isDark 切换），不是 Vuetify 主题类——Vuetify 类名是 `v-theme--{主题键名}`（如 `v-theme--teal-dark`），并非 `v-theme--dark`；组件浅色覆盖一律写 `:global(.scx-light) .foo`；迷你播放器窗口经 `app:accent-changed` 负载的 `dark` 字段同步标记类
- 分段控件/Tab 激活态：暗色 `rgba(255,255,255,0.1)` 浮起 + 字重 600 + 投影，浅色纯白浮起（SegControl.vue / SettingsView.vue tabbar / settings-card.css mode-toggle 三处一致）
- 系统主题检测通过 VueUse 的 `usePreferredDark` 实现
- 数据库持久化用户偏好；旧主题设置自动迁移

**主题配置位置：**
- `plugins/vuetify.ts` - Vuetify 主题定义（中性表面 + 六色回退；Material 渐变/发光 variables 已移除）
- `styles/tokens.css` - 设计令牌（表面/发丝线/文字/玻璃/字体/间距/圆角 + JS 动态写入组）
- 12 种主题变体（6 颜色 × 2 模式）+ 运行时 primary 覆盖

## 组件结构

### 主要组件
- **AppSidebar.vue** - 侧边栏导航（默认 236px 可拖拽 208–320；「资料库」组（全部歌曲 `'all'` 虚拟歌单 + 我的收藏 `'fav'`）+「歌单」组（可滚动、右键菜单、新建/重命名对话框）+ 底部入口（统计/分析/设置）；暗色玻璃、浅色实底；宽度经 resize 事件写回 App.vue 的 `--sb-w` 网格列）
- **LibraryView.vue** - 音乐库主容器（'all' 虚拟歌单标题映射；likedIds 传递 + toggleLike 处理）
- **PlayerBar.vue** - 底部播放控制（toggleQueue 事件触发 PlayQueueDrawer）
- **PlayQueueDrawer.vue** - 播放队列抽屉（GSAP Flip 列表重排动画、点击歌曲播放、模式切换按钮）
- **SettingsView.vue** - 设置页面（容器，衬线页头 + 自绘 tabbar 切换：外观/音频/桌面歌词/快捷键/数据管理/启动）
- **AnalysisView.vue** - 曲库分析（5 概览衬线数字格 + 4 ECharts 图表（accent 单色阶梯配色）+ 专辑排行横条）
- **NowPlayingOverlay.vue** - 正在播放沉浸层
- **LyricsDisplay.vue** - 歌词显示（同步滚动、点击跳转、骨架屏加载态）
- **IconButtonWithTooltip.vue** - 通用图标按钮 + Tooltip 组件（active 时加 `ib-active` 类供样式定制）

### ReportTab.vue

报告 Tab 组件，展示基于自然周期（周/月/年）的听歌总结。嵌入 StatsView 的 `v-window-item value="report"` 中。

**依赖**: `useListeningReport` composable
**功能**:
- 周期选择器（周/月/年 SegControl + offset 翻页 + 进行中标记）
- 4 张概览数字格（时长/次数/独立歌曲/艺术家）
- 24 小时听歌时段分布柱状图（ECharts，accent 单色）
- 峰值时段洞察标签（夜猫子/早起鸟/通用模板）

### Library 子组件 (components/library/)
- **BrowseCards.vue** - 浏览视图（专辑 = gcard 封面网格（真实封面 + hover 玻璃播放钮）；艺术家 = 圆头像行列表；card 数据含 sampleSongId 供 CoverArt 取封面）
- **EmptyStates.vue** - 空状态（衬线标题 + 安静文案）
- **LibraryHeader.vue** - 库头部（编辑式：衬线大标题 + 元信息行（N 首 · 约 X 分钟）+ 自定义搜索框 + SegControl 浏览/显示方式切换 + 排序按钮；钻取时左侧返回）
- **RenameDialog.vue** - 歌曲重命名对话框（标题必填、艺术家、专辑可选，loading/error 状态，Enter 提交）
- **UpdateDialog.vue** - 自动更新提示弹窗（available/downloading/ready/error 四状态，进度条，重启按钮）
- **SongGrid.vue** - 歌曲网格（gcard：正方形真实封面 + hover 播放钮 + 名称/副标题；当前项 accent）
- **SongTable.vue** - 歌曲表格门面（发丝线 46px 行：# 列（当前行 EQ 动效）/标题两行/专辑/心形（hover 浮现）/时长；共享样式 `styles/song-table.css`；内部按 `songs.length > 100` 自动委托 VirtualSongTable；行内右键菜单 + toggleLike 事件）
- **SortMenu.vue** - 排序菜单（标题/艺术家/专辑/时长排序）
- **VirtualSongTable.vue** - 虚拟滚动表格（行高 46px、sticky 表头，大数据 >100 首；与 SongTable 共享 song-table.css）

### Settings 子组件 (components/settings/)

SettingsView.vue 是容器（tab 切换），6 个 tab 各委托一个子组件（2026-06-21 D2 拆分；2026-06-29 新增启动 tab）：
- `AppearanceSettings.vue` — 外观（语言/主题模式/主题颜色六色板 + **专辑封面动态配色开关**）
- `AudioDeviceSettings.vue` — 音频输出设备（依赖 `useAudioDevice` composable）
- `AudioEffectSettings.vue` — 音效 EQ 预设（启用开关 + 预设单选网格 + 迷你 EQ 曲线；依赖 `useAudioEffectsStore`，2026-09-21 新增）
- `DesktopLyricsSettings.vue` — 桌面歌词配置（bgOpacity/fontSize/colorCurrent/colorNext/glowStrength/锁定）
- `ShortcutSettings.vue` — 快捷键（见下）
- `DataManagementSettings.vue` — 数据管理（备份/恢复/设置导入导出）
- `StartupSettings.vue` — 启动选项（开机自启开关 + 恢复上次播放开关；依赖 `useStartupOptions` composable，2026-06-29 新增）
- `KeyCaptureField.vue` — ShortcutSettings 的子组件
共享 CSS 在 `src/styles/settings-card.css`（`.settings-card`/`.card-*`/`.mode-toggle`/`.mode-button`/`.action-row`），各子组件用 `<style src>` 引用。

### `ShortcutSettings` / `KeyCaptureField`

`src/components/settings/ShortcutSettings.vue` 和 `KeyCaptureField.vue`

- `ShortcutSettings` — 设置面板子组件，分组列出 11 个动作，处理冲突检测、rebind、reset
- `KeyCaptureField` — 单行控件，捕获用户按下的组合键并 emit `rebind` 事件
- 捕获流程：点击「重新绑定」→ 显示「请按下组合键…」→ 捕获到完整组合 → 显示预览 + 「保存」按钮

### 组件使用策略
- **小数据量** (< 100首歌): 使用 SongTable.vue（内部非虚拟模式）或 SongGrid.vue
- **大数据量** (> 100首歌): SongTable.vue 内部自动切换到 VirtualSongTable.vue 提升性能（2026-06-21 D1 合并后，LibraryView 只引 SongTable）

## 工具函数

### utils/virtualScroll.ts - 虚拟滚动
- 仅渲染可见区域的 DOM 节点
- 支持动态高度计算
- 滚动位置记忆

### utils/errorHandler.ts - 错误处理
- `invokeCommand<T>()` - 唯一的命令调用入口，失败时抛 `AppInvokeError`（保留 `{type, message}` payload）

> **2026-06-26 清理：** 移除了 `safeInvoke`/`batchInvoke`/`retry` 三个零调用函数,以及 `invokeCommand` 的 `showMessage` 参数。UI 错误提示(toast)由各 composable 在 catch 中自行处理。

## Composables

活跃的 composables（被 Store 和组件引用）：

- **composables/usePlayer.ts** - 播放器核心逻辑（Store 的实际实现）
- **composables/useLibrary.ts** - 音乐库核心逻辑（Store 的实际实现）
- **composables/useTheme.ts** - 主题核心逻辑（Store 的实际实现，含 useColorTheme + useThemeMode）
- **composables/useToast.ts** - Toast 通知封装（全局消息提示）
- **composables/useI18n.ts** - 国际化封装（语言初始化、切换、持久化）
- **composables/usePlaybackMode.ts** - 播放模式切换（`cycleMode` 四态循环；另有双按钮 UI 用的 `toggleShuffle`（shuffle 独立开关）/ `cycleRepeat`（关→列表循环→单曲循环）/ `setPlaybackMode` 原子设置；变更后 regenerateQueue 重新生成队列）
- **composables/useAudioEffects.ts** - EQ 音效管理（init 启动恢复 eq_preset/eq_enabled、switchPreset/setEnabled 实时生效并持久化；薄 Store 包装 `stores/audioEffects.ts`；App.vue onMounted 并行 init，2026-09-21 新增。全屏播放页有快捷入口：NowPlayingOverlay 控制区 mdi-tune-variant 按钮弹出预设面板）
- **composables/usePlayQueue.ts** - 播放队列生成（Fisher-Yates 洗牌、模式映射：sequential/repeat_all=原序、repeat_one=仅当前歌曲、shuffle=洗牌后当前歌曲排首位）
- **composables/useLyrics.ts** - 歌词获取 + LRC 解析 + 同步跟踪（listen promise 加 await，防卸载竞态泄漏）
- **composables/useDebounceSearch.ts** - 搜索防抖（300ms）
- **composables/useOptimizedSort.ts** - 排序缓存（中文 locale 支持）
- **composables/useImportExport.ts** - 导入导出功能（歌单导出 M3U/PLS、音乐库备份恢复、设置迁移）
- **composables/useAudioDevice.ts** - 音频输出设备管理（封装 player_get_output_devices/current/set 三个 IPC，2026-06-21 D2 抽出，原 SettingsView 内联）
- **composables/useAutoUpdate.ts** - 自动更新逻辑（启动延迟检查、下载进度跟踪、重启安装）
- **composables/useLibraryAnalysis.ts** - 曲库分析（loadStats IPC、格式化工具）
- **composables/useListeningReport.ts** - 报告周期状态管理（见下文）
- **composables/useMiniPlayer.ts** - 迷你播放器窗口生命周期管理（互斥切换、置顶、位置持久化，见下文）。**模块级单例**：状态提升到模块级 + 幂等 init guard，多次调用只注册一次监听器
- **composables/useDesktopLyrics.ts** - 桌面歌词窗口生命周期管理（可见性、锁定、配置、歌词窗口与锁按钮窗口的跨窗口同步）。**模块级单例**：同 useMiniPlayer 模式 + `lyricsInstance` 缓存确保 useLyrics 只创建一次
- **composables/useGlobalShortcuts.ts** - 全局快捷键注册与路由（见下文）
- **composables/useStartupOptions.ts** - 启动选项封装（开机自启 IPC 读写 + 恢复播放持久化 + 纯逻辑：来源重建/位置解析/debounce 判定，2026-06-29 新增）。纯函数 TDD 覆盖（13 测试）。
- **composables/useSleepTimer.ts** - 睡眠定时器（2026-07-01 新增）。单例模式（模块级状态），倒计时 X 分钟后调 `player_stop` 完全停止播放，最后 30 秒音量线性渐弱到 0；取消/到点均恢复原音量。不持久化（一次性操作）。复用 `usePlayer.setVolume` / `usePlayer.stop`。
- **composables/useListeningStats.ts** - 统计 Tab 仪表盘聚合加载（`stats_dashboard` 单次 IPC 替代 6 个命令，2026-06-26；stores/stats.ts 薄封装）
- **composables/useCache.ts** - 缓存清理（歌词缓存/播放历史/封面缓存统计与清理 IPC 封装，2026-06-30；封面清理会同步 `evictCoverUrlCache()` 作废前端 blob URL；入口在设置 → 数据管理）
- **composables/useDraggableProgress.ts** - 进度条拖拽逻辑（PlayerBar 与 MiniPlayer 共用）：`progressModel` 值变实时 seek；`isDragging` 控制拖拽中显示本地进度对应时间，否则显示真实 progress
- **composables/useCoverArt.ts** - 封面加载（2026-08-27 新增）：`getCoverUrl(songId)` IPC raw bytes → 魔数嗅探 MIME → Blob URL；模块级 Map + LRU（50 张，淘汰 revoke）+ 同 songId inflight 去重；失败/无封面静默返回 null；`evictCoverUrlCache()` 供清理后作废。单测覆盖（7 测试）

> **Composable 单例模式（2026-06-20 加固）：** `useMiniPlayer` / `useDesktopLyrics` / `usePlayer` 都采用模块级状态 + 幂等 init guard（`stateSyncDone` / `listenersSetup`），避免主窗口多个组件（App.vue / PlayerBar / SettingsView / useGlobalShortcuts 间接）重复注册 Tauri 事件监听器导致的累积泄漏。模块级监听器不通过 `onUnmounted` 清理，依赖 webview 销毁时 Tauri 运行时自动回收。`useLyrics` 因接受 `currentSong` ref 参数不能完全单例化，改用 `_listenPromise` 追踪 listen 的 promise，`onUnmounted` 改为 async 先 await promise 再 unlisten。详见 `.wiki/risks.md` 的「监听器累积泄漏」章节。

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
| `usePlayerExpand` | Player expand/collapse | Staggered timeline: bg-layer fade → cover scale 0.94→1 → lyrics column fade-up → progress/controls（2026-08-27 适配新沉浸式布局 DOM） |
| PlayQueueDrawer | Play queue reorder | GSAP Flip list reorder on mode switch (0.35s) |
| `useLyricsAnimation` | Lyrics display | Spotlight opacity gradient + GSAP ScrollTo |

### Pattern

Every composable uses `useAnimation()` for scoped GSAP context. Cleanup is automatic on component unmount. Vue `<Transition :css="false">` with JS hooks (`@enter`/`@leave`) bridges GSAP into Vue's transition lifecycle.

### Key attributes

- Song items use `data-song-id` attribute for FLIP animations
- Lyrics lines use `.lyric-line` and `.active` classes for spotlight targeting

## 音频可视化

> **2026-09-19：前端频谱展示已整体移除**（用户决策：旧展示难看且不好用，待重新设计后再实现）。`src/visualization/` 目录已删除，全屏播放页不再有频谱入口/切换器/画布。**Rust 后端完整保留**（analyzer.rs 对数分箱 FFT + analyzer_start/stop Channel 推送 + visualization_* 设置键），未来重建前端时直接对接：`Channel<number[]>`（64 个 f32 0..1，60fps）。

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

### 歌词偏移
- `useLyrics.adjustOffset(±0.1)` / `resetOffset()`：步进 0.1s，范围 ±10s（`OFFSET_MIN`/`OFFSET_MAX`）
- 持久化走 `set_lyric_offset` 命令，直接 `UPDATE lyrics SET offset_secs`（存 lyrics 表，非 settings）
- 写入失败静默降级——内存中的偏移仍生效
- 偏移步进栏 UI 位于 NowPlayingOverlay 底部操作行正中（绝对定位居中，2026-09-22 从 LyricsDisplay 底部迁入；仅在有同步歌词时显示，偏移 ≠ 0 时追加重置按钮；LyricsDisplay 不再接收 offset 相关 props）

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
common / sidebar / library / player / lyrics / miniPlayer / settings / playbackMode / toast / empty / importExport / update / analysis / stats / report

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

> **macOS 透明双保险（2026-08）：** `transparent: true` 在部分 macOS 版本打包后会失效（WKWebView 原生层不透明，tauri#13415）。已加 `app.macOSPrivateApi: true` + `lib.rs::make_nswindow_transparent`（objc 直设 NSWindow 透明），详见 [architecture.md 窗口系统](architecture.md#窗口系统)。

### 与主窗口的关系

- **事件复用**：`audio:progress` / `audio:track_change` 已广播到所有 webview，歌词窗口自动接收，无需新增 IPC
- **配置同步**：自定义事件 `desktop-lyrics:config-changed`（主窗口 → 歌词窗口）、`desktop-lyrics:lock-changed`（双向）
- **持久化**：所有配置走既有 settings 表，前缀 `desktop-lyrics.*`，无数据库迁移

### 锁定/点击穿透状态机

- **unlocked**：可拖拽、LockBadge 可见、点击 LockBadge 进入 locked
- **locked**：`setIgnoreCursorEvents(true)` 整窗光标穿透、LockBadge 不可点击、只能通过 SettingsView 解锁

## 迷你播放器窗口（Mini Player Window）

独立 Tauri 窗口（label: `mini-player`），与主窗口互斥切换。无边框、置顶、固定尺寸（360×120），极简卡片式：顶部全宽 2px 进度线（AppSlider line 变体，兼拖拽区）+ 主行（52px 真实封面 CoverArt + 标题/`艺术家 · 时间/总长` + 上一首/32px accent 播放键/下一首）+ hover 右上角窗口控件（置顶/展开/关闭）。

**动态强调色同步**：MiniPlayer 监听 `app:accent-changed` 事件，用 `applyAccentBroadcast()` 直接写 CSS 变量（独立 webview 有自己的 Vuetify 实例，不走主窗口热更新链路）。

### 文件结构

- `src/main.ts`：检测 `window.location.hash === '#mini-player'`，条件挂载 `MiniPlayerApp.vue`
- `src/mini-player/MiniPlayerApp.vue`：迷你窗口根组件（无 AppShell/Sidebar/PlayerBar）
- `src/mini-player/MiniPlayer.vue`：主容器（拖拽、置顶按钮、展开/关闭、还原位置）
- `src/composables/useMiniPlayer.ts`：状态 + IPC + 持久化，**在主窗口与迷你窗口两端实例化**，按 `getCurrentWindow().label === 'mini-player'` 区分行为

### 窗口属性

`tauri.conf.json` 第三个业务窗口：`transparent: false`（不透明）、`decorations: false`、`alwaysOnTop: true`、`resizable: false`、固定 360×120、`visible: false`（默认隐藏，由 PlayerBar 按钮 / 快捷键触发显示）。

### 与主窗口的关系（互斥）

- **互斥切换**：进入迷你模式时主窗口完全 `hide()`，退出时主窗口 `show()` + 迷你窗口 `hide()`。与桌面歌词窗口的"共存"模式不同。
- **事件复用**：`audio:progress` / `audio:track_change` / `audio:state_change` 已广播到所有 webview，迷你窗口自动接收，无需新增 IPC
- **状态同步**：自定义事件 `mini-player:active-changed`（双向）、`mini-player:always-on-top-changed`（双向）
- **持久化**：所有配置走既有 settings 表，前缀 `mini-player.*`，无数据库迁移
- **启动恢复**：App.vue onMounted 读取 `mini-player.active`，若为 `true` 则直接进入迷你模式（主窗口不显示）

### 触发入口

- PlayerBar 上的迷你模式按钮（`enter()`）
- Cmd/Ctrl+Shift+M 快捷键（App.vue `onKeyStroke('m')` 注册，Shift 修饰避开 macOS Cmd+M 系统最小化加速器）

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

## 已知平台问题：macOS beta WKWebView 合成器卡死

症状：整个 webview 画面冻结在陈旧/扁平色块（沉浸层只剩模糊底、所有文字不绘制），但 DOM 完全健康（元素几何/opacity 正常、JS 可执行）。页面 reload 不恢复，仅重启应用恢复；非确定性复现、与具体 CSS/GSAP/布局无关（同代码多实例表现不一）。排查时曾用「Rust 侧轮询 eval + settings 表回传」建立无 DevTools 的 webview 检查通道确认 DOM 健康，最终定位为系统 beta 的 GPU 合成问题，非应用代码缺陷。遇到时重启应用即可。

缓解措施（2026-10-23）：全窗口 CSS `blur(64px)` 与沉浸层面板/按钮的 `backdrop-filter` 已全部移除——这是此类合成故障的最大嫌疑触发面，且 blur(64px) 本身 GPU 开销大。沉浸背景为**玻璃质感**（用户要求）：主窗口 `transparent: true` + lib.rs 给主窗口 `make_nswindow_transparent` + `apply_main_window_vibrancy`（NSVisualEffectView UnderWindowBackground，桌面从背后被原生模糊——CSS backdrop-filter 模糊不到桌面，必须原生材质）；`.np` 只铺半透明着色（暗 `--glass-bg` / 浅 rgba(240,240,238,.55)）。np 打开期间 body 挂 `np-glass` 类使 html→v-application→app-shell 全链透明，主界面表面（侧栏/主区/播放条）`v-show` 隐藏，after-leave 后才恢复（避免退场动画期间闪回）。封面 `.np-cover` 正方形无圆角（`height:auto` 需两级选择器压过 CoverArt 内部 `height:100%`）；歌词偏移步进条收进控制面板内部（居中一行），不再浮于歌词上方。

歌词可读性与布局（2026-10-23）：非活跃行 18px / 活跃行 26px 衬线；沉浸层 `--np-fg-3` 提到 55%、spotlight 衰减放缓（1/0.85/0.7/0.55，色阶由行基色提供）；`get_lyrics` 加 8s 兜底超时（LRCLIB 挂住时先显示暂无歌词，晚到结果自动补填）。歌词列高度约束用 **flex 链**：np-body 为 flex 行（此前 grid 的 `height:100%` 在 auto 行不解析→列被内容撑高、歌词溢出到面板外）。⚠️ np-body 不能设 `align-items:center`——cross 轴 center 会让歌词列高=内容高、stretch 失效（长歌词再次溢出）；垂直居中由封面列 `align-self:center` + 歌词列自身 `justify-content:center` 承担。歌词容器 `flex:0 1 auto` + 歌词滚动容器 `flex:1 1 auto; min-height:0`（缺 min-height:0 时 flex 子项下限=内容全高，同样溢出；basis 用 auto 而非 0，否则短歌词时容器塌缩为 0）——短内容整块垂直居中、长内容内部滚动；`.lyrics-display` 必须 `position:relative`（scrollToCurrent 的 offsetTop 以最近定位祖先为基准，锚点落在外层 .np-body 会每次多滚一个头部高度、把歌词推到面板外）。
