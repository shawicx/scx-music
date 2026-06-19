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

**audio/** - 音频播放引擎模块
- `player_set_queue` - 设置播放队列并开始播放 (`commands.rs`)
- `player_pause` / `player_resume` / `player_stop` - 播放控制 (`commands.rs`)
- `player_seek` - 跳转播放位置 (`commands.rs`)
- `player_set_volume` - 设置音量 (`commands.rs`)
- `player_next` / `player_previous` - 切换曲目 (`commands.rs`)
- `player_set_mode` - 设置播放模式 (sequential/repeat_all/repeat_one/shuffle) — Shuffle 队列由前端洗牌，后端按队列顺序播放 (`commands.rs`)
- `player_get_state` - 获取当前播放状态（恢复用）(`commands.rs`)
- `player_get_output_devices` - 枚举音频输出设备 (`device.rs`)
- `player_set_output_device` - 切换音频输出设备 (`device.rs`)
- `player_get_current_device` - 获取当前输出设备名 (`device.rs`)
- `analyzer_start` / `analyzer_stop` - 启动/停止音频频谱分析 (`analyzer_cmds.rs`)

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

**commands/stats.rs** - 统计分析
- `get_library_stats` - 曲库聚合统计（歌曲/艺术家/专辑总量、存储大小、艺术家排行、专辑排行、流派/音质/时长分布）
- `stats_listening_overview` - 听歌概览（播放次数、时长、流派数、歌手数、独立歌曲数），支持 `range?`（7d/30d/all）或 `start`/`end` 绝对日期范围
- `stats_top_songs` - 最爱歌曲 Top N（按播放次数排序）
- `stats_top_artists` - 最爱歌手 Top N（按播放次数排序）
- `stats_genre_distribution` - 流派播放时长分布
- `stats_trend` - 按天聚合的播放时长趋势
- `stats_heatmap` - 最近 365 天每日播放时长（GitHub 风格热力图数据）
- `stats_hourly_distribution(start, end)` — 指定时间范围内按小时（0-23，本地时区）聚合的听歌时长分布。用于周/月/年报告的时段分布图表。返回 `Vec<HourDuration>`。

**参数签名变化（所有 `stats_*` 命令）：** `range: String` → `range: Option<String>`，并新增可选 `start`/`end` 参数（绝对日期，格式 `YYYY-MM-DD HH:MM:SS` UTC）。向后兼容：统计 Tab 传 `range`，报告 Tab 传 `start`/`end`。

**commands/shortcuts.rs** - 全局快捷键管理
- `shortcuts_list_defaults` - 返回内置动作清单（11 项，3 默认开 + 8 默认关）+ 默认绑定
- `shortcuts_register` - 注册单个快捷键（`action_id`, `combo`），失败返回错误
- `shortcuts_unregister` - 注销快捷键（`action_id`）
- `shortcuts_is_registered` - 检查组合键是否已被注册（`combo`）— 系统层冲突预检
- `shortcuts_register_all` - 批量注册（`bindings: Vec<(String, String)>`，启动场景）

**commands/window.rs** - 窗口可见性管理
- `app_toggle_main_window` - 切换主窗口可见性（hide ↔ show + set_focus）；仅操作 main 窗口，不影响 mini-player / desktop-lyrics

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
- `scan_music_folder` - 扫描音乐文件夹并提取元数据（含 genre、file_size）

## 自动更新

**依赖：** `tauri-plugin-updater`, `tauri-plugin-process`

**插件注册（lib.rs）：**
- `tauri_plugin_updater::Builder::new().build()` — 更新检查与下载
- `tauri_plugin_process::init()` — 提供重启功能（`relaunch()`）

**配置（tauri.conf.json）：**
- `bundle.createUpdaterArtifacts: true` — 构建时生成更新签名产物
- `plugins.updater.endpoints` — 更新 JSON 端点（GitHub Releases）
- `plugins.updater.pubkey` — 签名验证公钥

**权限（capabilities/default.json）：**
- `updater:default` — 允许前端调用更新 API

**签名密钥：**
- 私钥：`~/.tauri/scx-music.key`（CI 通过 `TAURI_SIGNING_PRIVATE_KEY` 环境变量注入）
- 公钥：嵌入 `tauri.conf.json`

## Services

### audio/ - 音频引擎模块

**文件位置：** `src-tauri/src/audio/`

**模块结构：**
- `mod.rs` — 模块入口：Re-exports + `AudioState` 类型别名 + `start_progress_thread()`
- `types.rs` — Serde 类型：`PlaybackMode`、`PlaybackState`、`QueuedSong`、`PlayerStatePayload`、`AudioDeviceInfo`、`AudioDevicesResponse`
- `engine.rs` — `AudioEngine`（Rodio OutputStream/Sink 封装）+ `AudioStateInner`（播放状态与逻辑）
- `commands.rs` — 10 个播放控制命令（player_set_queue、player_pause 等）
- `device.rs` — 设备辅助函数（`try_output_stream_for_device` 等）+ 3 个设备命令
- `analyzer_cmds.rs` — `analyzer_start` / `analyzer_stop` 命令
- `tracker.rs` — 播放会话追踪（`PlaySession` struct + `flush_session` 写入数据库）

**核心结构：**
- `AudioStateInner` - 音频状态（队列、模式、音量、进度、分析器、输出设备、播放会话）
- `AudioEngine` - Rodio OutputStream 和 Sink 封装
- `QueuedSong` - 队列中的歌曲数据结构

**被谁调用：** 所有 `player_*` Commands 和 `analyzer_*` Commands

### 音频输出设备处理

**涉及函数：** `find_device_by_name`、`try_output_stream_for_device`、`try_build_hardcoded_stream`

**问题背景：**
macOS CoreAudio 通过 CPAL 暴露设备时存在两个已知问题：
1. **复合设备变体混淆** — 带麦克风的音箱（如 EDIFIER Halo Soundbar）在 `host.devices()` 中会出现两次（输入变体 + 输出变体），`find` 匹配到第一个（输入变体）时，创建输出流必然返回 `StreamTypeNotSupported`
2. **设备配置查询失败** — 部分内置设备（如 Mac mini扬声器）的 `default_output_config()` 和 `supported_output_configs()` 均返回 `BackendSpecificError { description: "Invalid property value" }`，无法获取有效的音频配置

**修复方案 — 三层回退链：**

```
1. find_device_by_name: 优先使用 default_output_device() 句柄匹配名称
   └─ 确保拿到输出变体而非输入变体

2. try_output_stream_for_device: 三级流创建回退
   ├─ 第一级: OutputStream::try_from_device（标准路径，依赖 default_output_config）
   ├─ 第二级: 遍历 supported_output_configs 逐个尝试
   └─ 第三级: try_build_hardcoded_stream（硬编码标准配置）

3. try_build_hardcoded_stream: 绕过 CPAL 配置查询，直接尝试常见配置
   └─ 48kHz/44.1kHz/96kHz × 1ch/2ch × F32/I16 共 7 种组合
```

**调用点：**
- `player_set_output_device` — 验证设备可用性时使用 `try_output_stream_for_device`
- `ensure_engine` — 创建音频引擎时使用 `try_output_stream_for_device`，失败后回退到 `OutputStream::try_default()`

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

### commands/shortcuts.rs - 全局快捷键

**文件位置：** `src-tauri/src/commands/shortcuts.rs`

**作用：** 通过 `tauri-plugin-global-shortcut` 注册 OS 级全局快捷键，触发时 emit `shortcut-triggered` 事件给前端。

**核心结构：**
- `ShortcutDefault` — 内置动作定义（`action_id`、`description`、`default_combo`、`default_enabled`）
- `ShortcutRegistry` — App state，保存「combo → action_id」映射；线程安全（`Mutex<HashMap<String, String>>`）

**关键函数：**
- `defaults() -> Vec<ShortcutDefault>` — 内置动作清单（11 项，3 默认开 + 8 默认关）
- `shortcut_to_string(&Shortcut) -> String` — 把 Tauri 的 Shortcut 结构序列化为字符串（用于反查；macOS 与 Windows 分别识别 SUPER/CONTROL 作为 CommandOrCtrl）
- `find_combo_by_action` — 私有助手，反向查找当前 action_id 绑定的 combo

**命令（5 个 `#[tauri::command]`）：**
- `shortcuts_list_defaults` / `shortcuts_register` / `shortcuts_unregister` / `shortcuts_is_registered` / `shortcuts_register_all`

### commands/window.rs - 窗口可见性

**文件位置：** `src-tauri/src/commands/window.rs`

**作用：** 提供主窗口可见性切换命令，供「显示/隐藏主窗口」全局快捷键调用。

**命令：**
- `app_toggle_main_window` — 切换主窗口可见性（hide ↔ show + set_focus）；仅操作 `main` 窗口，不影响 `mini-player` / `desktop-lyrics`

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

### db/migrations.rs - 数据库初始化

**Schema 结构：**
- `INIT_SCHEMA` — 核心表（songs、playlists、playlist_songs、settings、lyrics）
- `V6_PLAY_HISTORY` — 播放历史表（play_history）+ songs 缓存字段（play_count、total_play_duration）

**play_history 表：** id (自增主键), song_id (FK), played_at (时间戳), duration_secs (播放时长)
**songs 缓存字段：** play_count (播放次数), total_play_duration (累计播放时长)

### db/models.rs - 数据模型

**Rust 结构体：**
- `Song` - id, title, artist, album, duration, duration_secs, quality, file_path, art_gradient, genre, file_size
- `Playlist` - id, name, sort_order
- `PlaylistSong` - playlist_id, song_id, sort_order
- `Lyric` - song_id, raw_lrc, source
- `ListeningOverview` - 总播放次数、总播放时长、流派数、歌手数、**独立歌曲数（unique_song_count）**
- `HourDuration` - 小时（0-23）+ 累计播放时长（用于时段分布图）

### build_time_filter

`stats.rs` 中的私有函数，支持两种时间过滤模式：

- `range: Option<"7d" | "30d" | "all">` — 滚动窗口（统计 Tab 使用）
- `start/end: Option<String>` — 绝对日期范围（报告 Tab 使用，格式 `YYYY-MM-DD HH:MM:SS` UTC）

当 `start` + `end` 同时提供时优先使用绝对日期，否则回退到滚动窗口。所有 `stats_*` 命令的 `range` 参数为 `Option<String>` 以兼容此函数签名。

### 时区约定

`play_history.played_at` 使用 SQLite `datetime('now')` 默认值，存储为 **UTC 时间**。

- **过滤**：前端将本地时间转为 UTC 字符串后传入 `start`/`end`，SQL 直接比较原始列（索引友好）
- **小时聚合**：`strftime('%H', played_at, 'localtime')` 将 UTC 转为本地时间后提取小时

## Async Task

**audio/mod.rs** 中的进度跟踪线程
- 独立线程每 500ms 推送播放进度
- 检测曲目播放完毕并自动播放下一曲
- 每 30 秒自动 flush 播放会话到数据库（防崩溃丢数据）
- 使用 `Arc<AtomicBool>` 控制线程生命周期

**播放会话追踪（tracker.rs）：**
- `PlaySession` 追踪当前歌曲的播放时长（支持暂停/恢复累积）
- 播放/暂停/恢复/切歌时更新会话状态
- 切歌或停止时将累计时长 >= 5s 的会话写入 play_history 表
- 事务内同时更新 songs.play_count 和 songs.total_play_duration 缓存字段

## 多线程

**进度跟踪线程** - audio/mod.rs
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
- `audio/engine.rs` - Rodio 读取音频文件
- `lyrics.rs` - Lofty 读取音频文件内嵌歌词

**支持格式：** MP3, FLAC, WAV, AAC, OGG, M4A, Opus, WMA

## Shell 调用

无直接 shell 调用，所有操作通过 Rust API 完成。
