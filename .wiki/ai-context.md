# AI 快速上下文

## 核心架构

Tauri v2 桌面应用：Vue 3 前端 + Rust 后端

**通信：** IPC (invoke + emit)
**数据：** SQLite 数据库 (WAL 模式)
**音频：** Rodio 引擎
**状态管理：** Pinia 3.0
**国际化：** vue-i18n v11

## 关键文件

### 前端
```
src/stores/player.ts             # 播放器状态和控制 (usePlayerStore)
src/stores/library.ts            # 音乐库数据管理 (useLibraryStore)
src/stores/settings.ts           # 设置和主题管理 (useSettingsStore)
src/composables/useLyrics.ts     # 歌词获取 + LRC 解析 + 同步
src/composables/useToast.ts      # Toast 通知
src/composables/useI18n.ts       # 国际化
src/composables/usePlaybackMode.ts # 播放模式
src/composables/useDebounceSearch.ts # 搜索防抖
src/composables/useOptimizedSort.ts  # 排序缓存
src/composables/useLazyImages.ts # 图片懒加载
src/components/PlayerBar.vue     # 播放控制 UI
src/components/LibraryView.vue   # 音乐库视图
src/components/LyricsDisplay.vue # 歌词显示
src/components/NowPlayingOverlay.vue # 正在播放覆盖层
src/visualization/               # 音频可视化 (4 种渲染器)
src/utils/virtualScroll.ts       # 虚拟滚动工具
src/utils/errorHandler.ts        # 统一错误处理
```

### 后端
```
src-tauri/src/lib.rs              # Tauri 主入口 (命令注册)
src-tauri/src/audio.rs            # 音频引擎 (核心)
src-tauri/src/analyzer.rs         # FFT 频谱分析器
src-tauri/src/commands/bootstrap.rs  # 启动批量加载
src-tauri/src/commands/lyrics.rs  # 歌词获取 (缓存→内嵌→LRCLIB)
src-tauri/src/commands/songs.rs   # 歌曲数据操作
src-tauri/src/commands/playlists.rs  # 播放列表操作
src-tauri/src/commands/settings.rs   # 设置管理
src-tauri/src/db/mod.rs           # 数据库管理
src-tauri/src/db/migrations.rs    # 数据库迁移 (V1-V3)
src-tauri/src/db/models.rs        # 数据模型 (Song, Playlist)
```

## 调用链

### 应用启动
```
App.vue onMounted
-> useLibraryStore.loadFromDb()
-> invokeCommand('get_bootstrap_data')
-> 单次 IPC 获取 { songs, playlists, playlistSongs, settings }
-> Store 初始化
```

### 播放歌曲
```
UI -> usePlayerStore.playFromQueue()
-> invokeCommand('player_set_queue')
-> audio.rs::player_set_queue()
-> audio.rs::play_file_at_index()
-> emit('audio:state_change')
-> usePlayerStore 监听器
-> UI 更新
```

### 导入歌曲
```
UI -> useLibraryStore.importToPlaylist()
-> invokeCommand('scan_music_folder')
-> lib.rs::scan_music_folder()
-> invokeCommand('upsert_songs') -> 返回实际 DB ID
-> invokeCommand('clear_playlist') + invokeCommand('add_songs_to_playlist')
-> useLibraryStore 状态更新
```

### 获取歌词
```
歌曲切换 -> useLyrics.loadLyrics(song)
-> invoke('get_lyrics', {songId, filePath, title, artist, duration})
-> lyrics.rs: SQLite 缓存 → Lofty 内嵌 → LRCLIB API
-> 前端 LRC 解析 -> LyricsDisplay 同步显示
```

## 模块职责

| 模块 | 职责 | 关键点 |
|------|------|--------|
| usePlayerStore | 播放器状态 | 进度跟踪 (500ms)、队列管理、事件监听、状态恢复 |
| useLibraryStore | 音乐库 | Bootstrap 加载、搜索/筛选/排序、播放列表 CRUD |
| useSettingsStore | 设置主题 | Vuetify 主题、系统检测、数据库持久化 |
| useLyrics | 歌词 | LRC 解析、多源获取、实时同步 |
| audio.rs | 音频引擎 | Rodio 封装、线程安全、设备切换 |
| analyzer.rs | 频谱分析 | FFT 256点→64bins、30fps 推送 |
| bootstrap.rs | 启动加载 | 单次 IPC 全量数据 |
| lyrics.rs | 歌词后端 | 缓存→内嵌→LRCLIB 三级获取 |
| db/ | 数据库 | SQLite WAL、迁移管理 (V1-V3) |

## 注意事项

### 前端
- 所有 IPC 调用通过 Pinia Stores，组件不直接调用
- 使用 Composition API + Pinia 状态管理
- TypeScript 严格模式
- 虚拟滚动处理大数据列表
- 防抖搜索 + 排序缓存优化性能

### 后端
- 所有 Command 返回 `Result<T, String>`
- 使用 `?` 传播错误
- 线程安全：Arc<Mutex<T>>
- 进度线程独立运行，需要正确管理生命周期
- SQLite WAL 模式 + 外键约束

### 风险点
- 音频设备切换需要重建引擎
- 大型音乐库扫描性能
- 数据库迁移要谨慎
- 文件路径变化需要更新数据库
- LRCLIB API 不可用时的降级处理
- Pinia Store 事件监听器生命周期管理

### 开发规范
- 前端组件：PascalCase
- 前端函数：camelCase
- Store 命名：useXxxStore
- Composable 命名：useXxx
- 后端结构体：PascalCase
- 后端函数：snake_case
- Command 命名：snake_case

## 快速开始

```bash
# 开发
pnpm app:dev

# 构建
pnpm app:build

# Rust 后端测试
cd src-tauri && cargo test
```
