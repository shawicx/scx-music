# IPC 通信

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
| `player_set_queue` | stores/player.ts | audio.rs | 设置播放队列并开始播放 |
| `player_pause` | stores/player.ts | audio.rs | 暂停播放 |
| `player_resume` | stores/player.ts | audio.rs | 恢复播放 |
| `player_stop` | stores/player.ts | audio.rs | 停止播放 |
| `player_seek` | stores/player.ts | audio.rs | 跳转播放位置 |
| `player_set_volume` | stores/player.ts | audio.rs | 设置音量 |
| `player_next` | stores/player.ts | audio.rs | 下一曲 |
| `player_previous` | stores/player.ts | audio.rs | 上一曲 |
| `player_set_mode` | stores/player.ts | audio.rs | 设置播放模式 |
| `player_get_state` | stores/player.ts | audio.rs | 获取当前播放状态 |
| `player_get_output_devices` | - | audio.rs | 枚举音频输出设备 |
| `player_set_output_device` | - | audio.rs | 切换音频输出设备 |
| `player_get_current_device` | - | audio.rs | 获取当前输出设备 |
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
| **设置** | | | |
| `get_all_settings` | - | commands/settings.rs | 获取所有设置 |
| `get_setting` | stores/settings.ts | commands/settings.rs | 获取单个设置 |
| `set_setting` | stores/settings.ts, stores/library.ts | commands/settings.rs | 设置单个键值对 |
| `get_system_locale` | composables/useI18n.ts | commands/settings.rs | 获取系统语言 |
| **文件扫描** | | | |
| `scan_music_folder` | stores/library.ts | lib.rs | 扫描音乐文件夹 |
| **启动加载** | | | |
| `get_bootstrap_data` | stores/library.ts | commands/bootstrap.rs | 单次加载全部应用数据 |
| **歌词** | | | |
| `get_lyrics` | composables/useLyrics.ts | commands/lyrics.rs | 获取歌词 (缓存→内嵌→LRCLIB) |
| `refresh_lyrics` | - | commands/lyrics.rs | 强制刷新歌词 |
| **导入导出** | | | |
| `export_playlist_m3u` | composables/useImportExport.ts | commands/import_export.rs | 导出播放列表为 M3U 格式 |
| `export_playlist_pls` | composables/useImportExport.ts | commands/import_export.rs | 导出播放列表为 PLS 格式 |
| `export_backup` | composables/useImportExport.ts | commands/import_export.rs | 导出完整备份到 JSON |
| `import_backup` | composables/useImportExport.ts | commands/import_export.rs | 导入备份（replace/merge） |
| `export_settings` | composables/useImportExport.ts | commands/import_export.rs | 导出设置到 JSON |
| `import_settings` | composables/useImportExport.ts | commands/import_export.rs | 从 JSON 导入设置 |
| **频谱分析** | | | |
| `analyzer_start` | visualization/useAudioAnalyzer.ts | audio.rs | 启动频谱分析 |
| `analyzer_stop` | visualization/useAudioAnalyzer.ts | audio.rs | 停止频谱分析 |

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
-> audio.rs::player_set_queue()
-> audio.rs::play_file_at_index()
-> audio.rs::ensure_engine()
-> audio.rs::start_progress_thread()
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
-> commands/songs.rs::upsert_songs() -> 返回实际 DB ID
-> invokeCommand('clear_playlist', {playlistId})
-> invokeCommand('add_songs_to_playlist', {playlistId, songIds})
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
| `audio:progress` | `{current, duration}` | 每 500ms | stores/player.ts |
| `audio:state_change` | `{state, currentSong, queueIndex, mode}` | 播放状态变化 | stores/player.ts |
| `audio:track_change` | `Song \| null` | 当前曲目变化 | stores/player.ts |
| `audio:error` | `string` | 音频错误发生 | stores/player.ts |
| `audio:spectrum` | `number[64]` | 每 33ms (播放时) | visualization/useAudioAnalyzer.ts |

## 错误处理

所有 IPC 调用都通过 `utils/errorHandler.ts` 统一处理：

- **invokeCommand** - 带错误消息的调用
- **safeInvoke** - 返回 Result 类型
- **batchInvoke** - 批量调用
- **retry** - 重试机制
