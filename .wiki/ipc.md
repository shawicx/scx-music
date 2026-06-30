# IPC 通信

## 通信原语选型

项目使用三种 Tauri v2 IPC 原语,按场景选择:

| 原语 | 适用场景 | 项目内用法 |
|------|---------|-----------|
| **`invoke`** | 请求-响应(一次性) | 绝大多数 CRUD 命令(songs/playlists/settings/lyrics…) |
| **`emit`/`listen`** | 事件广播(多消费者) | `audio:state_change`/`track_change`(多窗口消费)、跨窗口业务事件(`mini-player:*`/`desktop-lyrics:*`) |
| **`Channel<T>`** | 流式点对点(单消费者+高频) | `audio:spectrum`(频谱分析器,30Hz,仅可视化渲染器消费) |

**选型决策矩阵:**
- 消费者数 ≥2 个窗口 → `emit` 广播
- 恰好 1 个消费者 + 高频(>10Hz) → `Channel<T>`
- 一次性请求-响应 → `invoke`

> **2026-06-26 优化:** 频谱分析器从 `emit` 广播改为 `Channel<T>` 点对点。原实现 30Hz 广播给全部 4 个 webview,但仅可视化渲染器消费,其余窗口白白反序列化。Channel 提供生命周期自洽(channel 销毁即停推,无需手动 `analyzer_stop` 配对)。

## invoke 封装

前端使用 `@tauri-apps/api/core` 的 `invoke` 函数，并通过 `utils/errorHandler.ts` 统一错误处理：

```typescript
import { invokeCommand } from '@/utils/errorHandler'

const result = await invokeCommand('command_name', { param: value })
```

## command 映射

| command | frontend caller | rust file | purpose |
|---------|----------------|-----------|---------|
| **播放器** | | | |
| `player_set_queue` | stores/player.ts | audio/commands.rs | 设置播放队列并开始播放 |
| `player_pause` | stores/player.ts | audio/commands.rs | 暂停播放 |
| `player_resume` | stores/player.ts | audio/commands.rs | 恢复播放 |
| `player_stop` | stores/player.ts | audio/commands.rs | 停止播放 |
| `player_seek` | stores/player.ts | audio/commands.rs | 跳转播放位置 |
| `player_set_volume` | stores/player.ts | audio/commands.rs | 设置音量 |
| `player_next` | stores/player.ts | audio/commands.rs | 下一曲 |
| `player_previous` | stores/player.ts | audio/commands.rs | 上一曲 |
| `player_set_mode` | stores/player.ts | audio/commands.rs | 设置播放模式 |
| `player_get_state` | stores/player.ts | audio/commands.rs | 获取当前播放状态 |
| `player_get_output_devices` | - | audio/device.rs | 枚举音频输出设备 |
| `player_set_output_device` | - | audio/device.rs | 切换音频输出设备 |
| `player_get_current_device` | - | audio/device.rs | 获取当前输出设备 |
| **歌曲** | | | |
| `get_all_songs` | - | commands/songs.rs | 获取所有歌曲 |
| `upsert_songs` | stores/library.ts | commands/songs.rs | 批量插入/更新歌曲 |
| `delete_songs` | - | commands/songs.rs | 删除歌曲 |
| **播放列表** | | | |
| `get_playlists` | - | commands/playlists.rs | 获取所有播放列表 |
| `create_playlist` | stores/library.ts | commands/playlists.rs | 创建播放列表 |
| `rename_playlist` | stores/library.ts | commands/playlists.rs | 重命名播放列表 |
| `delete_playlist` | stores/library.ts | commands/playlists.rs | 删除播放列表 |
| `get_playlist_songs` | - | commands/playlists.rs | 获取播放列表歌曲 |
| `add_songs_to_playlist` | stores/library.ts | commands/playlists.rs | 添加歌曲到播放列表 |
| `remove_song_from_playlist` | stores/library.ts | commands/playlists.rs | 从播放列表移除歌曲 |
| `clear_playlist` | stores/library.ts | commands/playlists.rs | 清空播放列表 |
| `replace_playlist_songs` | stores/library.ts | commands/playlists.rs | 原子替换歌单全部歌曲（单事务 DELETE+INSERT，2026-06-26 新增，替代 clear+add 两次 IPC） |
| **设置** | | | |
| `get_all_settings` | - | commands/settings.rs | 获取所有设置 |
| `get_setting` | stores/settings.ts | commands/settings.rs | 获取单个设置 |
| `set_setting` | stores/settings.ts, stores/library.ts | commands/settings.rs | 设置单个键值对（**key 白名单校验**：精确 9 项 + 前缀 4 项；前缀必须带非空子键） |
| `set_window_position` | composables/useMiniPlayer.ts, composables/useDesktopLyrics.ts | commands/settings.rs | 批量写入窗口位置（单事务双 key，2026-06-26 新增，替代拖动后的两次 set_setting） |
| `get_system_locale` | composables/useI18n.ts | commands/settings.rs | 获取系统语言 |
| **文件扫描** | | | |
| `scan_music_folder` | stores/library.ts | lib.rs | 扫描音乐文件夹 |
| **启动加载** | | | |
| `get_bootstrap_data` | stores/library.ts | commands/bootstrap.rs | 单次加载全部应用数据（2026-06-21：playlist_songs 改单条查询 + Rust 侧 HashMap 分组，消除 N+1 prepare） |
| **歌词** | | | |
| `get_lyrics` | composables/useLyrics.ts | commands/lyrics.rs | 获取歌词 (缓存→内嵌→LRCLIB) |
| `refresh_lyrics` | - | commands/lyrics.rs | 强制刷新歌词 |
| **缓存清理** | | | |
| `get_lyrics_cache_stats` | composables/useCache.ts | commands/cache.rs | 歌词缓存统计（总数/大小/孤儿数/by_source） |
| `get_play_history_stats` | composables/useCache.ts | commands/cache.rs | 播放历史统计（总数/最早时间/估算大小） |
| `clear_lyrics_cache` | composables/useCache.ts | commands/cache.rs | 清空全部歌词缓存（含 source='none' 负缓存），不打断当前播放 |
| `clear_orphan_lyrics` | composables/useCache.ts | commands/cache.rs | 清理孤儿歌词（song_id 不在 songs 表，删歌曲残留） |
| `clear_play_history` | composables/useCache.ts | commands/cache.rs | 按时间段清理播放历史（beforeDays：null=全部，正数=保留近 N 天） |
| **导入导出** | | | |
| `export_playlist_m3u` | composables/useImportExport.ts | commands/import_export.rs | 导出播放列表为 M3U 格式（**路径校验**：绝对路径 + 拒绝 `..`） |
| `export_playlist_pls` | composables/useImportExport.ts | commands/import_export.rs | 导出播放列表为 PLS 格式（同上） |
| `export_backup` | composables/useImportExport.ts | commands/import_export.rs | 导出完整备份到 JSON（同上） |
| `import_backup` | composables/useImportExport.ts | commands/import_export.rs | 导入备份（replace/merge）（同上） |
| `export_settings` | composables/useImportExport.ts | commands/import_export.rs | 导出设置到 JSON（同上） |
| `import_settings` | composables/useImportExport.ts | commands/import_export.rs | 从 JSON 导入设置（同上） |
| **曲库分析** | | | |
| `get_library_stats` | stores/analysis.ts | commands/stats.rs | 获取曲库聚合统计数据 |
| **听歌统计/报告** | | | |
| `stats_listening_overview` | composables/useListeningReport.ts | commands/stats.rs | 概览卡（报告 Tab 传 start/end）。`play_count` 从 `play_history` 派生（`COUNT(*)`），与 `total_duration_secs` 等其他指标一致地参与时间过滤。**统计 Tab 已改用 `stats_dashboard` 聚合命令（2026-06-26），此命令现仅报告 Tab 调用** |
| `stats_top_songs` | - | commands/stats.rs | 最爱歌曲 Top N。`play_count` 字段是窗口内 `COUNT(*) FROM play_history`，不是 `songs.play_count` 全库累计。**统计 Tab 已改用 `stats_dashboard`，此命令保留但无调用方** |
| `stats_top_artists` | - | commands/stats.rs | 最爱歌手 Top N。同上,**统计 Tab 已改用 `stats_dashboard`** |
| `stats_genre_distribution` | - | commands/stats.rs | 流派播放时长分布。**统计 Tab 已改用 `stats_dashboard`** |
| `stats_trend` | - | commands/stats.rs | 按天聚合播放时长趋势。**统计 Tab 已改用 `stats_dashboard`** |
| `stats_heatmap` | - | commands/stats.rs | 365 天每日播放时长热力图。**统计 Tab 已改用 `stats_dashboard`** |
| `stats_dashboard` | composables/useListeningStats.ts | commands/stats.rs | **统计 Tab 仪表盘聚合（2026-06-26 新增）**：单次 IPC 返回 overview+topSongs+topArtists+genreDistribution+trend+heatmap，替代前端 Promise.all 发 6 个命令。一次锁、一次 prepare，消除 6 次往返与重复锁竞争 |
| `stats_hourly_distribution` | `{ start, end }` | `HourDuration[]` | 报告 Tab 时段分布图 |
| **频谱分析** | | | |
| `analyzer_start` | visualization/useAudioAnalyzer.ts | audio/analyzer_cmds.rs | 启动频谱分析。**2026-06-26 改用 Channel API**：接收 `on_data: Channel<Vec<u8>>` 参数，FFT 线程通过 `channel.send()` 点对点推送，不再 `emit('audio:spectrum')` 广播。channel 销毁即自动停推，无需手动 analyzer_stop 配对 |
| `analyzer_stop` | visualization/useAudioAnalyzer.ts | audio/analyzer_cmds.rs | 停止频谱分析（主动停止用，channel 自然销毁也会触发后端退出） |
| **快捷键** | | | |
| `shortcuts_list_defaults` | composables/useGlobalShortcuts.ts | commands/shortcuts.rs | 返回内置动作清单+默认绑定 |
| `shortcuts_register` | composables/useGlobalShortcuts.ts | commands/shortcuts.rs | 注册单个快捷键，失败返回错误 |
| `shortcuts_unregister` | composables/useGlobalShortcuts.ts | commands/shortcuts.rs | 注销快捷键 |
| `shortcuts_is_registered` | composables/useGlobalShortcuts.ts | commands/shortcuts.rs | 检查组合键是否已被注册（系统层冲突预检） |
| `shortcuts_register_all` | composables/useGlobalShortcuts.ts | commands/shortcuts.rs | 批量注册（启动场景） |
| `app_toggle_main_window` | composables/useGlobalShortcuts.ts | commands/window.rs | 切换主窗口可见性（仅 main，不影响其他窗口） |

## 前后端调用链

### 应用启动流程

```
App.vue onMounted
-> useLibraryStore.loadFromDb()
-> invokeCommand('get_bootstrap_data')
-> bootstrap.rs::get_bootstrap_data()
-> 单次 DB 查询返回 { songs, playlists, playlist_songs, settings }
-> Store 状态初始化
```

### 播放歌曲流程

```
用户点击播放
-> usePlayerStore.playFromQueue()
-> usePlayQueue.generateQueue(songs, index, mode)
   ├─ sequential / repeat_all → 歌曲原序
   ├─ repeat_one → 仅当前歌曲
   └─ shuffle → Fisher-Yates 洗牌，当前歌曲排首位
-> invokeCommand('player_set_queue', {songs: ordered, index})
-> audio/commands.rs::player_set_queue()
-> audio/engine.rs::play_file_at_index()
-> audio/engine.rs::ensure_engine()
-> audio/mod.rs::start_progress_thread()
-> emit('audio:state_change')
-> usePlayerStore 监听器
-> UI 更新
```

### 播放模式切换流程

```
用户切换播放模式
-> usePlaybackMode.cycleMode()
-> usePlayerStore.setMode(nextMode)
-> invokeCommand('player_set_mode', {mode})
-> usePlayerStore.regenerateQueue()
-> usePlayQueue.generateQueue(sourceSongs, currentSongIndex, newMode)
-> invokeCommand('player_set_queue', {songs: ordered, index})
-> PlayQueueDrawer GSAP Flip 动画重排队列列表
```

### 导入歌曲流程

```
用户选择文件夹
-> useLibraryStore.importToPlaylist()
-> invokeCommand('scan_music_folder', {dirPath})
-> lib.rs::scan_music_folder()
-> 返回歌曲数组
-> invokeCommand('upsert_songs', {songs})
-> commands/songs.rs::upsert_songs()
   ├─ RETURNING id（2026-06-26）：单条 upsert 同时取回 id，消除原 N 次 SELECT 回查
   └─ 返回实际 DB ID
-> invokeCommand('replace_playlist_songs', {playlistId, songIds})
   └─ 单事务 DELETE + 批量 INSERT（2026-06-26，替代原 clear_playlist + add_songs_to_playlist 两次串行 IPC）
-> useLibraryStore 状态更新
```

### 歌词获取流程

```
歌曲切换
-> useLyrics.loadLyrics(song)
-> invoke('get_lyrics', {songId, filePath, title, artist, duration})
-> lyrics.rs::get_lyrics()
  ├─ SQLite 缓存命中 → 直接返回
  ├─ Lofty 内嵌歌词 → 缓存 + 返回
  └─ LRCLIB API 搜索 → 缓存 + 返回
-> 前端 LRC 解析
-> LyricsDisplay 同步显示
```

### 主题切换流程

```
用户切换主题
-> useSettingsStore.setColorTheme()
-> invokeCommand('set_setting', {key: 'theme_color', value: name})
-> commands/settings.rs::set_setting()
-> 数据库更新
-> Vuetify 主题系统响应
-> UI 更新
```

### 歌单导出流程

```
用户右键歌单 → 导出歌单
-> useImportExport.exportPlaylist()
-> Tauri Dialog 选择保存路径 (M3U/PLS)
-> invokeCommand('export_playlist_m3u' 或 'export_playlist_pls')
-> import_export.rs 查询歌单歌曲并写入文件
```

### 备份恢复流程

```
用户在设置页 → 备份音乐库
-> useImportExport.exportBackup()
-> Tauri Dialog 选择保存路径
-> invokeCommand('export_backup')
-> import_export.rs 导出 songs+playlists+lyrics+settings 到 JSON

用户在设置页 → 恢复音乐库
-> Tauri Dialog 选择备份文件
-> 用户选择策略 (replace/merge)
-> invokeCommand('import_backup', {filePath, strategy})
-> import_export.rs 事务导入，返回 ImportResult
-> useLibraryStore.loadFromDb() 刷新前端状态
```

## 核心事件

| event | payload | 触发时机 | 监听位置 |
|-------|---------|----------|----------|
| `audio:progress` | `{current, duration}` | 每 500ms（**仅 Playing 状态推送**，2026-06-26 优化：Paused/Stopped 跳过，避免重复推相同值） | stores/player.ts（含跨窗口消费：mini-player/桌面歌词） |
| `audio:state_change` | `{state, currentSong, queueIndex, mode}` | 播放状态变化 | stores/player.ts |
| `audio:track_change` | `Song \| null` | 当前曲目变化 | stores/player.ts |
| `audio:error` | `string` | 音频错误发生 | stores/player.ts |

> **`audio:spectrum` 已于 2026-06-26 从广播事件移除**,改用 `Channel<T>` 点对点推送(见上方"通信原语选型")。原 30Hz 广播给全部 4 个 webview,但仅可视化渲染器消费;现通过 `analyzer_start` 的 `on_data: Channel<Vec<u8>>` 参数直接推给订阅方,channel 销毁即停推。

## 桌面歌词相关事件（自定义）

| 事件 | 方向 | Payload | 说明 |
|------|------|---------|------|
| `desktop-lyrics:config-changed` | 主窗口 → 歌词窗口 | `{ key: keyof DesktopLyricsConfig, value: any }` | SettingsView 修改配置后广播，歌词窗口实时响应 |
| `desktop-lyrics:lock-changed` | 双向 | `boolean` | 锁定状态同步（任一窗口切换时广播） |

### 复用的既有事件

歌词窗口直接订阅以下广播事件，无需新增 IPC：

- `audio:progress`（payload `{ current, duration }`，500ms 间隔）：驱动歌词当前行高亮
- `audio:track_change`（payload 是 `Song` 对象）：歌曲切换时重新加载歌词

## 全局快捷键相关事件

| 事件 | 方向 | Payload | 说明 |
|------|------|---------|------|
| `shortcut-triggered` | Rust → 前端 | `String`（action_id） | 全局快捷键被按下时由 OS 回调触发，前端按 action_id 路由到对应 handler |

## 错误返回（2026-06-21 重构，P2 D4 清理后）

所有命令的 `Err` 分支返回 `AppError` 结构体（序列化为 `{"type": "...", "message": "..."}`），
不再返回字符串。前端 `utils/errorHandler.ts::AppInvokeError` 保留 type 字段供分类处理。

AppError variants：
- `FileOperation` - 文件 IO 错误
- `Database` - SQLite 错误
- `AudioParse` / `AudioPlayback` - 音频相关
- `DeviceNotFound` - 音频设备不存在
- `UnsupportedFormat` - 不支持的音频格式
- `InvalidArgument` - 参数校验失败（如 settings key 白名单、路径校验）
- `OperationFailed` - 其他操作失败（含网络错误、JSON 解析、String 桥接等）

`From` 自动转换已让 `?` 操作符直接工作，无需 `.map_err(|e| e.to_string())?` 样板：
- `io::Error` → `FileOperation`
- `rusqlite::Error` → `Database`
- `serde_json::Error` → `OperationFailed`
- `tauri::Error` → `OperationFailed`（P2 D4 新增）
- `tauri_plugin_global_shortcut::Error` → `OperationFailed`（P2 D4 新增）
- `String` → `OperationFailed`（兜底，用于自定义上下文和无 `From` 的第三方错误）

**例外（保留 `.map_err(|e| e.to_string())?`）**：
- `audio/commands.rs` / `audio/device.rs` / `audio/analyzer_cmds.rs` 的 **AudioState 锁**：单步 IPC 命令保留错误传播，让前端能感知 lock 中毒（与 P0 doc 中 `lock_or_recover` 的使用策略一致）
- `commands/shortcuts.rs` 的 `parse::<Shortcut>()`：`HotKeyParseError` 来自非直接依赖 crate（`global_hotkey`），无法在 `error.rs` 写 `From`，仍用 `.map_err(|e| e.to_string())?` 经 `From<String>` 桥接。同文件的 `register/unregister`（`tauri_plugin_global_shortcut::Error`）已有 `From`，已改裸 `?`
- `commands/songs.rs::rename_song`：`format!("Song not found: {}", e)?` 等自定义上下文（有诊断价值，依赖 `From<String>`）

**新增 `From` 实现（P2 D4 补充）**：`tauri::Error` 和 `tauri_plugin_global_shortcut::Error` 各加了一个 `From<E> for AppError` → `OperationFailed`，让 `window.rs` 全部、`shortcuts.rs` 的 register/unregister 部分改裸 `?`。

**注意**：`audio:error` 事件仍发送字符串（兼容既有监听），仅命令 reject 走结构化 AppError。

## 错误处理

所有 IPC 调用都通过 `utils/errorHandler.ts` 统一处理：

- **invokeCommand** - 唯一的命令调用入口，失败时抛 `AppInvokeError`（保留原始 `{type, message}` payload，可用 `isAppInvokeError(e)` + `e.errorType` 分类处理）

> **2026-06-26 清理：** 移除了 `safeInvoke`/`batchInvoke`/`retry` 三个零调用函数,以及 `invokeCommand` 的 `showMessage` 参数(永远 true 且仅 console.error,无真实 UI 反馈)。UI 错误提示(toast)由各 composable 在 catch 中自行处理,本函数只做错误结构化传递。

## 参数签名变化

所有 `stats_*` 命令的 `range` 参数从 `String` 改为 `Option<String>`（向后兼容：传 `undefined` 时回退到默认值或绝对日期）。新增可选 `start`/`end` 参数支持绝对日期范围过滤，由 `commands/stats.rs` 内部的 `build_time_filter` 统一处理。统计 Tab 仍传 `range: "7d" | "30d" | "all"`，报告 Tab 传 `start`/`end`（UTC 字符串）。
