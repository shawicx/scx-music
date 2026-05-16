# 数据存储

## SQLite

**位置：** 系统应用数据目录
- macOS: `~/Library/Application Support/com.scx-music.app/scx-music.db`
- Windows: `%APPDATA%/com.scx-music.app/scx-music.db`
- Linux: `~/.config/com.scx-music.app/scx-music.db`

**生命周期：** 应用删除时保留
**风险：** 数据库文件损坏导致应用无法启动

## 数据表结构

### songs
**作用：** 存储歌曲元数据
**关键字段：** id, title, artist, album, file_path (UNIQUE)
**索引：** id (PRIMARY KEY), file_path (UNIQUE)

### playlists
**作用：** 存储播放列表
**关键字段：** id, name, sort_order
**索引：** id (PRIMARY KEY)

### playlist_songs
**作用：** 播放列表和歌曲的多对多关系
**关键字段：** playlist_id, song_id
**约束：** PRIMARY KEY (playlist_id, song_id), FOREIGN KEYS

### settings
**作用：** 存储应用设置键值对
**关键字段：** key (PRIMARY KEY), value
**常用设置：** theme_color, theme_mode, output_device, currentSongId, viewMode, displayMode

## Cache

**前端：** Pinia Store 状态缓存，虚拟滚动优化大量数据渲染
**后端：** 无缓存层，直接查询 SQLite

## App Data

**位置：** 与数据库相同目录
**类型：**
- 数据库文件 (.db)
- 配置文件 (可能在未来添加)

## 存储风险

### 数据库锁定
多进程同时访问可能导致锁定
**缓解：** Tauri 单进程架构避免此问题

### 数据库迁移
应用升级时 schema 变更可能导致数据丢失
**缓解：** 使用 migrations.rs 版本化管理

### 大文件处理
音频文件不存储在数据库中，只存储路径
**风险：** 文件删除后数据库中有 orphaned records
**缓解：** 未来可添加文件存在性检查

### 数据库大小
大量歌曲元数据可能导致数据库增大
**影响：** 启动时加载变慢
**缓解：** 添加索引、分页加载

## 数据库初始化

**位置：** `src-tauri/src/db/mod.rs::init()`
**时机：** 应用启动时
**流程：**
1. 创建应用数据目录
2. 打开/创建数据库文件
3. 执行 migrations
4. 将数据库连接注入 Tauri State
