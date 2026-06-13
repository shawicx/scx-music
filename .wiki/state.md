# 状态管理

## 状态来源

**前端：** Vue 3 Composition API + Pinia 3.0
**后端：** Rust structs + SQLite
**IPC：** Tauri Events (后端 -> 前端)

## 状态更新方式

### 前端状态 (Vue + Pinia)

**直接更新：** Store actions 直接修改 state
```typescript
// stores/player.ts
const isPlaying = ref(false)
function togglePlayPause() {
  isPlaying.value = !isPlaying.value
}
```

**IPC 触发更新：** 监听后端事件
```typescript
// stores/player.ts
listen('audio:state_change', (e) => {
  isPlaying.value = e.payload.state === 'playing'
})
```

**数据库同步：**
- 应用启动时通过 `get_bootstrap_data` 一次性加载
- 设置变化时保存到数据库 (`set_setting()`)

### 后端状态 (Rust)

**内存状态：** `AudioStateInner` struct
- 使用 `Arc<Mutex<T>>` 保护共享状态
- 通过 Command 修改状态
- 通过 Event 推送状态变更

**持久化状态：** SQLite 数据库
- songs 表
- playlists 表
- playlist_songs 表
- settings 表
- lyrics 表 (V3 新增)

## 关键状态

### 播放器状态 (stores/player.ts - usePlayerStore)

**来源：** Rust audio.rs + Events
**更新方式：**
- 用户操作 -> IPC Command -> Rust 状态变更 -> Event -> Store 状态
- 进度线程直接推送 progress 事件
- `getState()` 从后端恢复播放状态

**关键状态：**
- `currentSong` - 当前歌曲 (来自 track_change 事件)
- `isPlaying` - 播放状态 (来自 state_change 事件)
- `progress` / `duration` - 播放进度 (来自 progress 事件)
- `queue` - 播放队列 (用户设置)
- `queueIndex` - 当前队列索引
- `playbackMode` - 播放模式 (用户设置)
- `volume` - 音量 (用户设置)

### 音乐库状态 (stores/library.ts - useLibraryStore)

**来源：** SQLite 数据库 (通过 get_bootstrap_data 单次加载)
**更新方式：**
- 启动时从 `get_bootstrap_data` 一次性加载全部数据
- 用户操作后同步到数据库

**关键状态：**
- `songs` - 所有歌曲 (来自 bootstrap)
- `playlists` - 播放列表 (来自 bootstrap)
- `playlistSongs` - 播放列表歌曲映射 (来自 bootstrap)
- `activePlaylistId` - 当前播放列表 (来自 settings)
- `searchQuery` - 搜索关键词 (用户输入, 防抖)
- `displayMode` - 显示模式 (用户选择)
- `drilldown` - 专辑/艺术家筛选 (用户选择)
- `sortBy` / `sortOrder` - 排序方式 (useOptimizedSort)
- `ready` - 数据加载完成标志

**计算属性链：**
`currentPlaylistSongs` → `searchedSongs` → `drilldownFilter` → `displayedSongs`

### 设置状态 (stores/settings.ts - useSettingsStore)

**来源：** Vuetify + SQLite
**更新方式：**
- 启动时从数据库加载
- 用户切换后保存到数据库

**关键状态：**
- `colorName` - 主题颜色 (teal/blue/green/etc.)
- `mode` - 主题模式 (light/dark/system)
- `isDark` - 当前是否深色模式 (计算属性)

### 歌词状态 (composables/useLyrics.ts)

**来源：** SQLite 缓存 + Lofty + LRCLIB API
**更新方式：**
- 歌曲切换时触发加载
- 进度事件驱动同步

**关键状态：**
- `lines` - 解析后的歌词行数组
- `currentLineIndex` - 当前行索引
- `isLoading` - 加载状态

### 曲库分析状态 (stores/analysis.ts - useAnalysisStore)

**来源：** Rust commands/stats.rs (SQL 聚合查询)
**更新方式：**
- 进入分析页时调用 `loadStats()` 触发 IPC

**关键状态：**
- `stats` - LibraryStats（总量、排行、分布数据）
- `loading` - 加载状态
- `formattedTotalSize` / `formattedTotalDuration` - 格式化计算属性

## 状态持久化策略

**前端状态：** Pinia Store (Vue reactive refs)
**后端持久化：** SQLite 数据库 (WAL 模式)
**同步时机：**
- 应用启动：`get_bootstrap_data` 一次 IPC 加载全部数据
- 设置变化：立即保存到数据库
- 批量操作：事务提交

## 桌面歌词设置项

所有键以 `desktop-lyrics.*` 前缀存入既有 `settings(key TEXT PK, value TEXT)` 表，**无数据库迁移**。首次启动时表内无这些 key，使用默认值；首次状态变更才写入。

| Key | 默认值 | 类型 | 说明 |
|---|---|---|---|
| `desktop-lyrics.bg-opacity` | `0.3` | float 0-1 | 背景透明度 |
| `desktop-lyrics.font-size` | `32` | int px | 当前句字号（下一句自动按 0.75x 缩放） |
| `desktop-lyrics.color-current` | `#FFFFFF` | hex | 当前句文字颜色 |
| `desktop-lyrics.color-next` | `rgba(255,255,255,0.5)` | css color | 下一句文字颜色 |
| `desktop-lyrics.glow` | `medium` | enum: off/weak/medium/strong | 当前句发光强度 |
| `desktop-lyrics.locked` | `false` | bool | 锁定状态（点击穿透） |
| `desktop-lyrics.position-x` | 运行时计算 | int | 窗口 X 坐标（默认水平居中） |
| `desktop-lyrics.position-y` | 运行时计算 | int | 窗口 Y 坐标（默认底部上方 100px） |

**注意**：可见性 (`visible`) 不持久化 —— 主窗口 PlayerBar 按钮态由本地 ref 维护，应用重启后 lyrics 窗口默认隐藏（来自 `tauri.conf.json`），符合"不做启动自动打开"的设计。

## 最关键状态

1. **播放队列** - queue + queueIndex
2. **当前歌曲** - currentSong
3. **播放状态** - isPlaying + progress
4. **歌曲库** - songs (核心数据)
5. **播放列表** - playlists + playlistSongs
6. **主题设置** - colorName + mode
7. **歌词** - lines + currentLineIndex

## 性能优化

- **Bootstrap 启动：** 单次 IPC 获取全部数据，替代多次单独调用
- **防抖搜索：** useDebounceSearch 300ms 防抖
- **排序缓存：** useOptimizedSort 避免重复排序
- **虚拟滚动：** VirtualSongTable.vue 仅渲染可见 DOM 节点
- **计算属性缓存：** Pinia computed properties 避免重复计算
- **事件监听管理：** Store 生命周期管理监听器
