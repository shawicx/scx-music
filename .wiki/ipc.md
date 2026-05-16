# IPC 通信

## invoke 封装

前端使用 `@tauri-apps/api/core` 的 `invoke` 函数：

```typescript
import { invoke } from '@tauri-apps/api/core'

const result = await invoke('command_name', { param: value })
```

## command 映射

| command | frontend caller | rust file | purpose |
|---------|----------------|-----------|---------|
| `player_set_queue` | usePlayer.ts | audio.rs | 设置播放队列并开始播放 |
| `player_pause` | usePlayer.ts | audio.rs | 暂停播放 |
| `player_resume` | usePlayer.ts | audio.rs | 恢复播放 |
| `player_seek` | usePlayer.ts | audio.rs | 跳转播放位置 |
| `player_set_volume` | usePlayer.ts | audio.rs | 设置音量 |
| `player_next` | usePlayer.ts | audio.rs | 下一曲 |
| `player_previous` | usePlayer.ts | audio.rs | 上一曲 |
| `player_set_mode` | usePlayer.ts | audio.rs | 设置播放模式 |
| `get_all_songs` | useLibrary.ts | commands/songs.rs | 获取所有歌曲 |
| `upsert_songs` | useLibrary.ts | commands/songs.rs | 批量插入/更新歌曲 |
| `get_playlists` | useLibrary.ts | commands/playlists.rs | 获取所有播放列表 |
| `create_playlist` | useLibrary.ts | commands/playlists.rs | 创建播放列表 |
| `get_playlist_songs` | useLibrary.ts | commands/playlists.rs | 获取播放列表歌曲 |
| `add_songs_to_playlist` | useLibrary.ts | commands/playlists.rs | 添加歌曲到播放列表 |
| `remove_song_from_playlist` | useLibrary.ts | commands/playlists.rs | 从播放列表移除歌曲 |
| `get_all_settings` | useLibrary.ts | commands/settings.rs | 获取所有设置 |
| `set_setting` | useTheme.ts, useLibrary.ts | commands/settings.rs | 设置单个键值对 |
| `scan_music_folder` | useLibrary.ts | lib.rs | 扫描音乐文件夹 |

## 前后端调用链

### 播放歌曲流程

```
用户点击播放
-> usePlayer.playFromQueue()
-> invoke('player_set_queue', {songs, index})
-> audio.rs::player_set_queue()
-> audio.rs::play_file_at_index()
-> audio.rs::ensure_engine()
-> audio.rs::start_progress_thread()
-> emit('audio:state_change')
-> usePlayer 监听器
-> UI 更新
```

### 导入歌曲流程

```
用户选择文件夹
-> useLibrary.importToPlaylist()
-> invoke('scan_music_folder', {dirPath})
-> lib.rs::scan_music_folder()
-> 返回歌曲数组
-> invoke('upsert_songs', {songs})
-> commands/songs.rs::upsert_songs()
-> 数据库插入
-> invoke('add_songs_to_playlist', {playlistId, songIds})
-> commands/playlists.rs::add_songs_to_playlist()
-> 数据库插入
-> useLibrary 状态更新
```

## 核心事件

| event | payload | 触发时机 |
|-------|---------|----------|
| `audio:progress` | `{current, duration}` | 每 500ms |
| `audio:state_change` | `{state, currentSong, queueIndex, mode}` | 播放状态变化 |
| `audio:track_change` | `Song | null` | 当前曲目变化 |
| `audio:error` | `string` | 音频错误发生 |
