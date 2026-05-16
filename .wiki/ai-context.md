# AI 快速上下文

## 核心架构

Tauri v2 桌面应用：Vue 3 前端 + Rust 后端

**通信：** IPC (invoke + emit)
**数据：** SQLite 数据库
**音频：** Rodio 引擎

## 关键文件

### 前端
```
src/composables/usePlayer.ts      # 播放器状态和控制
src/composables/useLibrary.ts     # 音乐库数据管理
src/composables/useTheme.ts       # 主题切换
src/components/PlayerBar.vue      # 播放控制 UI
src/components/LibraryView.vue    # 音乐库视图
```

### 后端
```
src-tauri/src/lib.rs              # Tauri 主入口
src-tauri/src/audio.rs            # 音频引擎 (核心)
src-tauri/src/commands/songs.rs   # 歌曲数据操作
src-tauri/src/commands/playlists.rs  # 播放列表操作
src-tauri/src/db/mod.rs           # 数据库管理
```

## 调用链

### 播放歌曲
```
UI -> usePlayer.playFromQueue()
-> invoke('player_set_queue')
-> audio.rs::player_set_queue()
-> audio.rs::play_file_at_index()
-> emit('audio:state_change')
-> usePlayer 监听器
-> UI 更新
```

### 导入歌曲
```
UI -> useLibrary.importToPlaylist()
-> invoke('scan_music_folder')
-> lib.rs::scan_music_folder()
-> invoke('upsert_songs')
-> commands/songs.rs::upsert_songs()
-> 数据库插入
-> useLibrary 状态更新
```

## 模块职责

| 模块 | 职责 | 关键点 |
|------|------|--------|
| usePlayer | 播放器状态 | 进度跟踪 (500ms)、队列管理 |
| useLibrary | 音乐库 | 搜索/筛选/排序、播放列表 CRUD |
| audio.rs | 音频引擎 | Rodio 封装、线程安全、设备切换 |
| db/ | 数据库 | SQLite 操作、迁移管理 |

## 注意事项

### 前端
- 所有 IPC 调用通过 Composables，组件不直接调用
- 使用 Composition API，不用 Vuex/Pinia
- TypeScript 严格模式

### 后端
- 所有 Command 返回 `Result<T, String>`
- 使用 `?` 传播错误
- 线程安全：Arc<Mutex<T>>
- 进度线程独立运行，需要正确管理生命周期

### 风险点
- 音频设备切换需要重建引擎
- 大型音乐库扫描性能
- 数据库迁移要谨慎
- 文件路径变化需要更新数据库

### 开发规范
- 前端组件：PascalCase
- 前端函数：camelCase
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
