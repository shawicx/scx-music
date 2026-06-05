# 数据存储

## SQLite

**位置：** 系统应用数据目录
- macOS: `~/Library/Application Support/com.scx-music.app/scx-music.db`
- Windows: `%APPDATA%/com.scx-music.app/scx-music.db`
- Linux: `~/.config/com.scx-music.app/scx-music.db`

**模式：** WAL (Write-Ahead Logging)
**生命周期：** 应用删除时保留
**风险：** 数据库文件损坏导致应用无法启动

## 数据表结构

### songs (V1)
**作用：** 存储歌曲元数据
**关键字段：** id (TEXT PK), title, artist, album, duration (TEXT), duration_secs (REAL), quality, file_path (TEXT UNIQUE), art_gradient, created_at
**索引：** id (PK), file_path (UNIQUE), artist, album, title, created_at

### playlists (V1)
**作用：** 存储播放列表
**关键字段：** id (TEXT PK), name, sort_order, created_at
**索引：** id (PK)
**初始数据：** 'fav' (我喜欢的)

### playlist_songs (V1)
**作用：** 播放列表和歌曲的多对多关系
**关键字段：** playlist_id, song_id, sort_order
**约束：** PRIMARY KEY (playlist_id, song_id), FOREIGN KEYS (ON DELETE CASCADE)
**索引：** (playlist_id), (playlist_id, sort_order)

### settings (V1)
**作用：** 存储应用设置键值对
**关键字段：** key (TEXT PK), value
**常用设置：**
- `theme_color` - 主题颜色
- `theme_mode` - 主题模式 (light/dark/system)
- `output_device` - 音频输出设备
- `currentSongId` - 当前歌曲 ID
- `viewMode` - 视图模式
- `displayMode` - 显示模式
- `activePlaylistId` - 当前播放列表
- `language` - 界面语言
- `visualization_style` - 可视化风格

### lyrics (V3)
**作用：** 歌词缓存
**关键字段：** song_id (TEXT PK), raw_lrc (TEXT), source (TEXT)
**source 值：** embedded / lrclib / none

## Cache

**前端：** Pinia Store 状态缓存，虚拟滚动优化大量数据渲染
**后端：** 歌词缓存 (lyrics 表)，无其他缓存层，直接查询 SQLite

## App Data

**位置：** 与数据库相同目录
**类型：**
- 数据库文件 (.db)
- WAL 文件 (.db-wal, .db-shm)

## 存储风险

### 数据库锁定
多进程同时访问可能导致锁定
**缓解：** Tauri 单进程架构 + WAL 模式

### 数据库迁移
应用升级时 schema 变更可能导致数据丢失
**缓解：** 使用 migrations.rs 版本化管理 (V1-V3)

### 大文件处理
音频文件不存储在数据库中，只存储路径
**风险：** 文件删除后数据库中有 orphaned records
**缓解：** 未来可添加文件存在性检查

### 数据库大小
大量歌曲元数据可能导致数据库增大
**影响：** 启动时加载变慢
**缓解：** 添加索引、使用 bootstrap 单次加载

## 数据库初始化

**位置：** `src-tauri/src/db/mod.rs::init()`
**时机：** 应用启动时
**流程：**
1. 创建应用数据目录
2. 打开/创建数据库文件
3. 执行 PRAGMA (WAL 模式, 外键启用)
4. 执行 migrations (V1→V2→V3)
5. 将数据库连接注入 Tauri State
