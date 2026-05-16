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
| `player_set_queue` | stores/player.ts | audio.rs | 设置播放队列并开始播放 |
| `player_pause` | stores/player.ts | audio.rs | 暂停播放 |
| `player_resume` | stores/player.ts | audio.rs | 恢复播放 |
| `player_seek` | stores/player.ts | audio.rs | 跳转播放位置 |
| `player_set_volume` | stores/player.ts | audio.rs | 设置音量 |
| `player_next` | stores/player.ts | audio.rs | 下一曲 |
| `player_previous` | stores/player.ts | audio.rs | 上一曲 |
| `player_set_mode` | stores/player.ts | audio.rs | 设置播放模式 |
| `get_all_songs` | stores/library.ts | commands/songs.rs | 获取所有歌曲 |
| `upsert_songs` | stores/library.ts | commands/songs.rs | 批量插入/更新歌曲 |
| `get_playlists` | stores/library.ts | commands/playlists.rs | 获取所有播放列表 |
| `create_playlist` | stores/library.ts | commands/playlists.rs | 创建播放列表 |
| `rename_playlist` | stores/library.ts | commands/playlists.rs | 重命名播放列表 |
| `delete_playlist` | stores/library.ts | commands/playlists.rs | 删除播放列表 |
| `get_playlist_songs` | stores/library.ts | commands/playlists.rs | 获取播放列表歌曲 |
| `add_songs_to_playlist` | stores/library.ts | commands/playlists.rs | 添加歌曲到播放列表 |
| `remove_song_from_playlist` | stores/library.ts | commands/playlists.rs | 从播放列表移除歌曲 |
| `get_all_settings` | stores/library.ts | commands/settings.rs | 获取所有设置 |
| `get_setting` | stores/settings.ts | commands/settings.rs | 获取单个设置 |
| `set_setting` | stores/settings.ts, stores/library.ts | commands/settings.rs | 设置单个键值对 |
| `scan_music_folder` | stores/library.ts | lib.rs | 扫描音乐文件夹 |

## 前后端调用链

### 播放歌曲流程

```
用户点击播放
-> usePlayerStore.playFromQueue()
-> invokeCommand('player_set_queue', {songs, index})
-> audio.rs::player_set_queue()
-> audio.rs::play_file_at_index()
-> audio.rs::ensure_engine()
-> audio.rs::start_progress_thread()
-> emit('audio:state_change')
-> usePlayerStore 监听器
-> UI 更新
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
-> 数据库插入
-> invokeCommand('add_songs_to_playlist', {playlistId, songIds})
-> commands/playlists.rs::add_songs_to_playlist()
-> 数据库插入
-> useLibraryStore 状态更新
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

## 核心事件

| event | payload | 触发时机 | 监听位置 |
|-------|---------|----------|----------|
| `audio:progress` | `{current, duration}` | 每 500ms | stores/player.ts |
| `audio:state_change` | `{state, currentSong, queueIndex, mode}` | 播放状态变化 | stores/player.ts |
| `audio:track_change` | `Song | null` | 当前曲目变化 | stores/player.ts |
| `audio:error` | `string` | 音频错误发生 | stores/player.ts |

## 错误处理

所有 IPC 调用都通过 `utils/errorHandler.ts` 统一处理：

- **invokeCommand** - 带错误消息的调用
- **safeInvoke** - 返回 Result 类型
- **batchInvoke** - 批量调用
- **retry** - 重试机制
