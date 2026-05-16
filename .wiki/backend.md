# 后端结构

## Commands (IPC 处理)

### 核心模块

**audio.rs** - 音频播放引擎
- `player_set_queue` - 设置播放队列并开始播放
- `player_pause` / `player_resume` - 播放控制
- `player_seek` - 跳转播放位置
- `player_set_volume` - 设置音量
- `player_next` / `player_previous` - 切换曲目
- `player_set_mode` - 设置播放模式 (sequential/repeat_all/repeat_one/shuffle)
- `player_get_output_devices` - 枚举音频输出设备
- `player_set_output_device` - 切换音频输出设备

**commands/songs.rs** - 歌曲数据操作
- `get_all_songs` - 获取所有歌曲
- `upsert_songs` - 批量插入/更新歌曲
- `delete_songs` - 删除歌曲

**commands/playlists.rs** - 播放列表管理
- `get_playlists` - 获取所有播放列表
- `create_playlist` - 创建播放列表
- `rename_playlist` - 重命名播放列表
- `delete_playlist` - 删除播放列表
- `get_playlist_songs` - 获取播放列表中的歌曲
- `add_songs_to_playlist` - 添加歌曲到播放列表
- `remove_song_from_playlist` - 从播放列表移除歌曲

**commands/settings.rs** - 设置管理
- `get_all_settings` - 获取所有设置
- `get_setting` - 获取单个设置
- `set_setting` - 设置单个键值对

**lib.rs** - 文件扫描
- `scan_music_folder` - 扫描音乐文件夹并提取元数据

## Services

### audio.rs - 音频引擎

**文件位置：** `src-tauri/src/audio.rs`

**作用：** 封装 Rodio 音频播放，提供播放控制 API

**核心结构：**
- `AudioStateInner` - 音频状态（队列、模式、音量、进度）
- `AudioEngine` - Rodio OutputStream 和 Sink 封装
- `QueuedSong` - 队列中的歌曲数据结构

**被谁调用：** 所有 `player_*` Commands

**风险点：**
- 线程安全：使用 `Arc<Mutex<AudioStateInner>>` 保护共享状态
- 进度线程：独立线程每 500ms 推送进度，需要正确管理生命周期
- 设备切换：需要重建整个音频引擎，可能导致播放中断

### db/mod.rs - 数据库管理

**文件位置：** `src-tauri/src/db/mod.rs`

**作用：** SQLite 数据库连接和初始化

**被谁调用：** lib.rs 在应用启动时初始化

**风险点：**
- 数据库文件位置：使用系统应用数据目录
- 迁移系统：应用升级时需要正确执行迁移

## Utils

**lib.rs** - 文件扫描和元数据提取
- 使用 `lofty` 库提取音频元数据
- 递归遍历目录
- 支持格式：MP3, FLAC, WAV, AAC, OGG, M4A, Opus, WMA

## Async Task

**audio.rs** 中的进度跟踪线程
- 独立线程每 500ms 推送播放进度
- 检测曲目播放完毕并自动播放下一曲
- 使用 `Arc<AtomicBool>` 控制线程生命周期

## IO 操作

**文件系统：**
- `scan_music_folder` - 递归读取音乐文件夹
- Rodio - 音频文件读取和解码

**数据库：**
- 所有 commands/ 模块中的数据库操作
- 使用参数化查询防止 SQL 注入

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

**风险点：**
- 大型音乐库扫描可能较慢
- 文件权限问题
- 符号链接可能导致重复扫描

## Shell 调用

无直接 shell 调用，所有操作通过 Rust API 完成。
