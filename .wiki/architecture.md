# 架构设计

## 前端结构

- **框架：** Vue 3.5 Composition API
- **状态管理：** Pinia 3.0 (stores/player.ts, stores/library.ts, stores/settings.ts)
- **UI 库：** Vuetify 4.0
- **工具库：** VueUse 14.3
- **构建：** Vite 6.0
- **国际化：** vue-i18n v11

## Rust 结构

- **音频引擎：** Rodio 0.19 + Symphonia
- **数据库：** SQLite (Rusqlite 0.31, WAL 模式)
- **IPC：** Tauri Commands + Events
- **线程：** 独立进度跟踪线程
- **频谱分析：** rustfft (256 点 FFT → 64 bins)
- **元数据：** Lofty (音频标签解析 + 内嵌歌词提取)
- **歌词源：** LRCLIB API (网络歌词搜索)

## IPC 通信方式

**前端 -> 后端：** `invoke('command', {args})`
**后端 -> 前端：** `emit('event', payload)`

## Plugin 使用

- `tauri-plugin-opener` - URL 打开
- `tauri-plugin-dialog` - 文件对话框

## 数据流

```
用户操作 -> Vue Component -> Pinia Store -> IPC Command -> Rust Handler
Rust Handler -> IPC Event -> Pinia Store -> UI Update
```

## 启动数据流

```
App.vue onMounted
  -> useLibraryStore.loadFromDb()
  -> invokeCommand('get_bootstrap_data')   # 单次 IPC 获取全部数据
  -> 返回 { songs, playlists, playlistSongs, settings }
  -> Store 状态初始化
```

## 架构图

```mermaid
graph TB
    UI[Vue Components] --> STORES[Pinia Stores]
    STORES --> IPC[invoke/listen]
    IPC --> CMDS[Rust Commands]
    CMDS --> AUDIO[Audio Engine]
    CMDS --> DB[(SQLite)]
    CMDS --> LYRICS[Lyrics System]
    AUDIO --> EVENTS[Events]
    EVENTS --> STORES
    UTILS[Utils / Composables] --> STORES
    UTILS --> UI
    LYRICS --> LRCLIB[LRCLIB API]
    LYRICS --> DB
```

## 模块职责

| 模块 | 作用 | 关键文件 |
|------|------|----------|
| 播放器 | 音频播放控制 | stores/player.ts, audio.rs |
| 音乐库 | 歌曲和播放列表管理 | stores/library.ts, commands/ |
| 设置主题 | 深色/浅色模式 + 主题颜色 | stores/settings.ts |
| 歌词 | LRC 解析 + 同步显示 | composables/useLyrics.ts, commands/lyrics.rs, LyricsDisplay.vue |
| 音频可视化 | FFT 频谱渲染 | visualization/, analyzer.rs |
| 启动加载 | 单次批量数据获取 | commands/bootstrap.rs |
| 数据库 | 数据持久化 | db/mod.rs, db/migrations.rs |
| 虚拟滚动 | 大数据性能优化 | utils/virtualScroll.ts |
| 错误处理 | 统一错误处理 | utils/errorHandler.ts |
| 类型系统 | TypeScript 类型定义 | types/index.ts |

## TypeScript 类型系统

项目使用完整的 TypeScript 类型定义：

**核心数据类型：**
- `Song` - 歌曲信息 (id, title, artist, album, duration, durationSecs, quality, filePath, artGradient)
- `Playlist` - 播放列表

**播放器类型：**
- `PlaybackMode` - 播放模式 (sequential/repeat_all/repeat_one/shuffle)
- `PlaybackState` - 播放状态

**主题类型：**
- `ThemeColor` - 主题颜色
- `ThemeMode` - 主题模式 (light/dark/system)

**视图类型：**
- `ViewMode` - 视图模式
- `DisplayMode` - 显示模式 (songs/albums/artists)
- `SortBy` - 排序字段
- `SortOrder` - 排序顺序

**歌词类型：**
- `LrcLine` - 歌词行 (time + text)

**API 类型：**
- `ApiResponse<T>` - API 响应包装
- `Result<T>` - Result 类型

**音频设备类型：**
- `AudioDeviceInfo` - 音频设备信息
- `AudioDevicesResponse` - 音频设备列表响应

## 性能优化

- **Bootstrap 启动：** `get_bootstrap_data` 单次 IPC 加载全部应用数据
- **虚拟滚动：** VirtualSongTable.vue 处理大量歌曲数据
- **防抖搜索：** useDebounceSearch 300ms 防抖，减少计算频率
- **排序缓存：** useOptimizedSort 缓存排序结果
- **懒加载图片：** useLazyImages IntersectionObserver 按需加载
- **计算属性缓存：** Pinia computed properties 避免重复计算
- **事件监听优化：** 合理管理事件监听器生命周期
- **类型安全：** 完整的 TypeScript 类型系统确保编译时错误检查
- **WAL 模式：** SQLite WAL 模式提升并发读写性能
