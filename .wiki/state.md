# 状态管理

## 状态来源

**前端：** Vue 3 Composition API + Reactive Refs
**后端：** Rust structs + SQLite
**IPC：** Tauri Events (后端 -> 前端)

## 状态更新方式

### 前端状态 (Vue)

**直接更新：** Composable 函数直接修改 ref 值
```typescript
const isPlaying = ref(false)
function togglePlayPause() {
  isPlaying.value = !isPlaying.value
}
```

**IPC 触发更新：** 监听后端事件
```typescript
listen('audio:state_change', (e) => {
  isPlaying.value = e.payload.state === 'playing'
})
```

**数据库同步：**
- 应用启动时从数据库加载 (`loadFromDb()`)
- 状态变化时保存到数据库 (`set_setting()`)

### 后端状态 (Rust)

**内存状态：** `AudioStateInner` struct
- 使用 `Arc<Mutex<T>>` 保护共享状态
- 通过 Command 修改状态
- 通过 Event 推送状态变更

**持久化状态：** SQLite 数据库
- songs 表
- playlists 表
- playlist_songs 表
- settings 表

## 关键状态

### 播放器状态 (usePlayer.ts)

**来源：** Rust audio.rs + Events
**更新方式：**
- 用户操作 -> IPC Command -> Rust 状态变更 -> Event -> 前端状态
- 进度线程直接推送 progress 事件

**关键状态：**
- `currentSong` - 当前歌曲 (来自 track_change 事件)
- `isPlaying` - 播放状态 (来自 state_change 事件)
- `progress` / `duration` - 播放进度 (来自 progress 事件)
- `queue` - 播放队列 (用户设置)
- `playbackMode` - 播放模式 (用户设置)

### 音乐库状态 (useLibrary.ts)

**来源：** SQLite 数据库
**更新方式：**
- 启动时从数据库加载
- 用户操作后同步到数据库

**关键状态：**
- `songs` - 所有歌曲 (来自数据库)
- `playlists` - 播放列表 (来自数据库)
- `activePlaylistId` - 当前播放列表 (来自 settings)
- `searchQuery` - 搜索关键词 (用户输入)
- `displayMode` - 显示模式 (用户选择)

### 主题状态 (useTheme.ts)

**来源：** Vuetify + SQLite
**更新方式：**
- 启动时从数据库加载
- 用户切换后保存到数据库

**关键状态：**
- `theme` - 'dark' | 'light'

## 状态持久化策略

**前端内存状态：** Vue reactive refs
**后端持久化：** SQLite 数据库
**同步时机：**
- 应用启动：从数据库加载到内存
- 状态变化：立即保存到数据库 (settings)
- 批量操作：事务提交

## 最关键状态

1. **播放队列** - queue + queueIndex
2. **当前歌曲** - currentSong
3. **播放状态** - isPlaying + progress
4. **歌曲库** - songs (核心数据)
5. **播放列表** - playlists + playlistSongs
