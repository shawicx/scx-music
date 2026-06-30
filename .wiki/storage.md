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

### songs (INIT_SCHEMA + V6 缓存列)
**作用：** 存储歌曲元数据
**关键字段：** id (TEXT PK), title, artist, album, duration (TEXT), duration_secs (REAL), quality, file_path (TEXT UNIQUE), art_gradient, genre, file_size, created_at
**缓存列（V6 新增）：** play_count (INTEGER, 默认 0), total_play_duration (REAL, 默认 0.0)
**索引：** id (PK), file_path (UNIQUE), artist, album, title, created_at, genre

### playlists (INIT_SCHEMA)
**作用：** 存储播放列表
**关键字段：** id (TEXT PK), name, sort_order, created_at
**索引：** id (PK)
**初始数据：** 'fav' (我喜欢的)

### playlist_songs (INIT_SCHEMA)
**作用：** 存储播放列表和歌曲的多对多关系
**关键字段：** playlist_id, song_id, sort_order
**约束：** PRIMARY KEY (playlist_id, song_id), FOREIGN KEYS (ON DELETE CASCADE)
**索引：** (playlist_id), (playlist_id, sort_order)

### settings (INIT_SCHEMA)
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
- `desktop-lyrics.*` - 桌面歌词配置（前缀）
- `mini-player.*` - 迷你播放器配置（前缀）
- `shortcut.*` - 全局快捷键绑定（前缀）

### lyrics (INIT_SCHEMA)
**作用：** 歌词缓存
**关键字段：** song_id (TEXT PK), raw_lrc (TEXT), source (TEXT), offset_secs (REAL)
**source 值：** embedded / lrclib / none

> **2026-06-26 修复：** `Lyric` model（`db/models.rs`）补全 `offset_secs` 字段（原遗漏，与表列不对应，导致备份/恢复丢失歌词偏移）。`import_export.rs` 的导出查询和导入 INSERT 同步补上该列。model 用 `#[serde(default)]` 确保旧备份文件（无此字段）反序列化兼容。

### play_history (V6_PLAY_HISTORY)
**作用：** 播放历史明细，驱动听歌统计（周/月/年报告、时段分布）
**关键字段：** id (INTEGER PK AUTOINCREMENT), song_id (TEXT, FK), played_at (TEXT, 默认 now), duration_secs (REAL)
**约束：** FOREIGN KEY song_id → songs(id) ON DELETE CASCADE
**索引：** song_id, played_at
**写入触发：** 进度线程每 ~30s 自动 flush（仅时长，不增 play_count）；切歌/退出时 finalize（时长 + 满足 50% 阈值才 play_count +1）

## Cache

**前端：** Pinia Store 状态缓存，虚拟滚动优化大量数据渲染
**后端：** 歌词缓存 (lyrics 表)，无其他缓存层，直接查询 SQLite

**缓存清理（2026-06-30 新增）：** `commands/cache.rs` 提供 5 个命令：
- `get_lyrics_cache_stats` / `get_play_history_stats` — 统计规模（条数/大小/孤儿数/最早时间）
- `clear_lyrics_cache` — 清空全部歌词缓存（含 source='none' 负缓存）
- `clear_orphan_lyrics` — 清理 song_id 不在 songs 表的孤儿歌词（删歌曲残留，因 lyrics 表无外键）
- `clear_play_history(before_days)` — 按时间段清理播放历史（None=全部，Some(n)=保留近 n 天）

入口：设置 → 数据管理 → "缓存与数据清理"卡片（`DataManagementSettings.vue`）。设计：核心 SQL 提取为 `_inner(&Connection)` 纯函数，便于内存 SQLite 单元测试。

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
**缓解：** 使用 migrations.rs 版本化管理 (INIT_SCHEMA + V6_PLAY_HISTORY)

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
