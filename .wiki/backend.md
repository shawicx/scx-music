# 后端结构

## 错误处理

**文件位置：** `src-tauri/src/error.rs`

**统一错误类型：** `AppError` 枚举
- `FileOperation` - 文件操作错误
- `Database` - 数据库错误
- `AudioParse` - 音频解析错误
- `AudioPlayback` - 音频播放错误
- `DeviceNotFound` - 设备未找到
- `UnsupportedFormat` - 不支持的音频格式
- `InvalidArgument` - 无效参数
- `OperationFailed` - 操作失败

**特性：**
- 使用 `thiserror` crate 自动生成错误信息
- 自动转换常见错误类型 (std::io::Error, rusqlite::Error)
- 统一的 `AppResult<T>` 类型别名
- 自动转换为 String 以便通过 Tauri IPC 传递

## Commands (IPC 处理)

### 核心模块

**audio.rs** - 音频播放引擎
- `player_set_queue` - 设置播放队列并开始播放
- `player_pause` / `player_resume` / `player_stop` - 播放控制
- `player_seek` - 跳转播放位置
- `player_set_volume` - 设置音量
- `player_next` / `player_previous` - 切换曲目
- `player_set_mode` - 设置播放模式 (sequential/repeat_all/repeat_one/shuffle)
- `player_get_state` - 获取当前播放状态（恢复用）
- `player_get_output_devices` - 枚举音频输出设备
- `player_set_output_device` - 切换音频输出设备
- `player_get_current_device` - 获取当前输出设备名
- `analyzer_start` / `analyzer_stop` - 启动/停止音频频谱分析

**commands/songs.rs** - 歌曲数据操作
- `get_all_songs` - 获取所有歌曲
- `upsert_songs` - 批量插入/更新歌曲（返回实际 DB ID）
- `delete_songs` - 删除歌曲
- `rename_song` - 重命名歌曲（更新元数据标签 + 重命名文件 + 更新数据库）

**commands/playlists.rs** - 播放列表管理
- `get_playlists` - 获取所有播放列表
- `create_playlist` - 创建播放列表
- `rename_playlist` - 重命名播放列表
- `delete_playlist` - 删除播放列表
- `get_playlist_songs` - 获取播放列表中的歌曲
- `add_songs_to_playlist` - 添加歌曲到播放列表
- `remove_song_from_playlist` - 从播放列表移除歌曲
- `clear_playlist` - 清空播放列表（保留列表本身）

**commands/settings.rs** - 设置管理
- `get_all_settings` - 获取所有设置
- `get_setting` - 获取单个设置
- `set_setting` - 设置单个键值对
- `get_system_locale` - 获取系统语言

**commands/bootstrap.rs** - 启动批量加载
- `get_bootstrap_data` - 单次 IPC 返回全部应用数据（songs + playlists + playlist_songs + settings）

**commands/lyrics.rs** - 歌词管理
- `get_lyrics` - 获取歌词（缓存 → 内嵌 → LRCLIB API）
- `refresh_lyrics` - 强制刷新歌词（跳过缓存）

**commands/import_export.rs** - 数据导入导出
- `export_playlist_m3u` - 导出播放列表为 M3U 格式
- `export_playlist_pls` - 导出播放列表为 PLS 格式
- `export_backup` - 导出完整备份（歌曲+歌单+歌词+设置 → JSON）
- `import_backup` - 导入备份（支持 replace/merge 策略）
- `export_settings` - 导出设置为 JSON 文件
- `import_settings` - 从 JSON 文件导入设置

**commands/songs.rs - rename_song 详解**

**文件位置：** `src-tauri/src/commands/songs.rs`

**参数：**
- `song_id` (String) - 歌曲 ID
- `new_title` (String) - 新标题
- `new_artist` (Option<String>) - 新艺术家（None 则保留原值）
- `new_album` (Option<String>) - 新专辑（None 则保留原值）

**返回：** 更新后的 `Song` 对象

**处理流程：**
1. 从数据库查询当前歌曲（获取 file_path、artist、album）
2. 验证文件存在且可写
3. 使用 Lofty 写入元数据标签（TrackTitle、TrackArtist、AlbumTitle）
4. 构建新文件名（基于 new_title，通过 `sanitize_filename` 清理非法字符）
5. 文件名冲突解决：若目标文件已存在且非自身，添加 `(2)`, `(3)` ... 后缀
6. 重命名磁盘文件（`std::fs::rename`）
7. 更新数据库记录（title、artist、album、file_path）
8. 返回更新后的完整 Song 对象

**错误情况：**
- 歌曲未找到（DB 查询失败）
- 文件在磁盘上不存在
- 文件只读（permissions.readonly）
- 元数据写入失败（Lofty save 错误）
- 文件重命名失败（`std::fs::rename` 错误）

**辅助函数：** `sanitize_filename(name)` — 将 `/ \ : * ? " < > |` 替换为 `_`

**lib.rs** - 文件扫描
- `scan_music_folder` - 扫描音乐文件夹并提取元数据

## Services

### audio.rs - 音频引擎

**文件位置：** `src-tauri/src/audio.rs`

**作用：** 封装 Rodio 音频播放，提供播放控制 API

**核心结构：**
- `AudioStateInner` - 音频状态（队列、模式、音量、进度、分析器、输出设备）
- `AudioEngine` - Rodio OutputStream 和 Sink 封装
- `QueuedSong` - 队列中的歌曲数据结构

**被谁调用：** 所有 `player_*` Commands 和 `analyzer_*` Commands

### analyzer.rs - FFT 频谱分析器

**文件位置：** `src-tauri/src/analyzer.rs`

**作用：** 从音频流中提取频谱数据，通过 Tauri 事件推送到前端

**核心结构：**
- `AnalyzerHandle` - 分析器句柄（SampleBuffer + 运行状态）
- `TeeSource<S>` - Source 包装器，在流经时复制 f32 样本到分析器

**工作流程：**
1. `TeeSource` 在音频播放时，将 f32 样本批量 (1024个) 推入 SampleBuffer
2. `AnalyzerHandle::start()` 启动后台线程，每 33ms 从 buffer 读取 256 个样本
3. 应用 Hann 窗函数 → 256 点 FFT → 计算 64 个频率 bin 的幅度
4. 缩放到 0-255 并通过 `audio:spectrum` 事件推送到前端

**关键参数：**
- FFT_SIZE = 256, NUM_BINS = 64, 采样率 ~30fps
- 依赖 `rustfft` crate

### lyrics.rs - 歌词服务

**文件位置：** `src-tauri/src/commands/lyrics.rs`

**作用：** 多源歌词获取与缓存

**获取链：**
1. 查询 SQLite `lyrics` 表缓存
2. 使用 Lofty 从音频文件提取内嵌歌词
3. 调用 LRCLIB API (`https://lrclib.net/api/search`) 搜索歌词
4. 结果写入 SQLite 缓存

**LRCLIB 搜索策略：**
- 参数：track_name, artist_name, duration
- 优先选择有 synced_lyrics 的结果
- 持久化 source 标记 (embedded / lrclib / none)

### bootstrap.rs - 启动数据聚合

**文件位置：** `src-tauri/src/commands/bootstrap.rs`

**作用：** 单次数据库查询返回全部应用初始化数据

**返回 `BootstrapData`：**
- `songs` - 全部歌曲
- `playlists` - 全部播放列表
- `playlist_songs` - HashMap<playlist_id, Vec<song_id>>
- `settings` - HashMap<key, value>

### db/mod.rs - 数据库管理

**文件位置：** `src-tauri/src/db/mod.rs`

**作用：** SQLite 数据库连接和初始化

**被谁调用：** lib.rs 在应用启动时初始化

### db/migrations.rs - 数据库迁移

**Schema 版本：**
- V1: 核心表 (songs, playlists, playlist_songs, settings) + 基础索引
- V2: 性能索引 (title, created_at, playlist position)
- V3: 歌词表 (lyrics)

### db/models.rs - 数据模型

**Rust 结构体：**
- `Song` - id, title, artist, album, duration, duration_secs, quality, file_path, art_gradient
- `Playlist` - id, name, sort_order
- `PlaylistSong` - playlist_id, song_id, sort_order
- `Lyric` - song_id, raw_lrc, source

## Async Task

**audio.rs** 中的进度跟踪线程
- 独立线程每 500ms 推送播放进度
- 检测曲目播放完毕并自动播放下一曲
- 使用 `Arc<AtomicBool>` 控制线程生命周期

## 多线程

**进度跟踪线程** - audio.rs
```rust
pub fn start_progress_thread(state: AudioState, app: AppHandle) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(500));
            // 推送进度事件
            // 检测曲目结束并自动下一曲
        }
    });
}
```

**线程安全：**
- 使用 `Arc<Mutex<T>>` 保护共享状态
- 使用 `AtomicBool` 进行线程间通信
- 及时释放锁避免死锁

## 文件系统

**调用位置：**
- `lib.rs::scan_music_folder()` - 扫描音乐文件夹
- `audio.rs` - Rodio 读取音频文件
- `lyrics.rs` - Lofty 读取音频文件内嵌歌词

**支持格式：** MP3, FLAC, WAV, AAC, OGG, M4A, Opus, WMA

## Shell 调用

无直接 shell 调用，所有操作通过 Rust API 完成。
