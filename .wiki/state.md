# 状态管理

## 状态来源

**前端：** Vue 3 Composition API + Pinia 3.0
**后端：** Rust structs + SQLite
**IPC：** Tauri Events (后端 -> 前端)

## 状态更新方式

### 前端状态 (Vue + Pinia)

**直接更新：** Store actions 直接修改 state
```typescript
// stores/player.ts
const isPlaying = ref(false)
function togglePlayPause() {
  isPlaying.value = !isPlaying.value
}
```

**IPC 触发更新：** 监听后端事件
```typescript
// stores/player.ts
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

### 播放器状态 (stores/player.ts - usePlayerStore)

**来源：** Rust audio.rs + Events
**更新方式：**
- 用户操作 -> IPC Command -> Rust 状态变更 -> Event -> Store 状态
- 进度线程直接推送 progress 事件

**关键状态：**
- `currentSong` - 当前歌曲 (来自 track_change 事件)
- `isPlaying` - 播放状态 (来自 state_change 事件)
- `progress` / `duration` - 播放进度 (来自 progress 事件)
- `queue` - 播放队列 (用户设置)
- `queueIndex` - 当前队列索引
- `playbackMode` - 播放模式 (用户设置)
- `volume` - 音量 (用户设置)

### 音乐库状态 (stores/library.ts - useLibraryStore)

**来源：** SQLite 数据库
**更新方式：**
- 启动时从数据库加载
- 用户操作后同步到数据库

**关键状态：**
- `songs` - 所有歌曲 (来自数据库)
- `playlists` - 播放列表 (来自数据库)
- `playlistSongs` - 播放列表歌曲映射 (来自数据库)
- `activePlaylistId` - 当前播放列表 (来自 settings)
- `searchQuery` - 搜索关键词 (用户输入)
- `displayMode` - 显示模式 (用户选择)
- `sortBy` / `sortOrder` - 排序方式 (用户选择)
- `drilldown` - 专辑/艺术家筛选 (用户选择)

### 设置状态 (stores/settings.ts - useSettingsStore)

**来源：** Vuetify + SQLite
**更新方式：**
- 启动时从数据库加载
- 用户切换后保存到数据库

**关键状态：**
- `colorName` - 主题颜色 (teal/blue/green/etc.)
- `mode` - 主题模式 (light/dark/system)
- `isDark` - 当前是否深色模式 (计算属性)

## 状态持久化策略

**前端状态：** Pinia Store (Vue reactive refs)
**后端持久化：** SQLite 数据库
**同步时机：**
- 应用启动：从数据库加载到 Store
- 状态变化：立即保存到数据库 (settings)
- 批量操作：事务提交

## 最关键状态

1. **播放队列** - queue + queueIndex
2. **当前歌曲** - currentSong
3. **播放状态** - isPlaying + progress
4. **歌曲库** - songs (核心数据)
5. **播放列表** - playlists + playlistSongs
6. **主题设置** - colorName + mode

## 性能优化

- **虚拟滚动：** VirtualSongTable.vue 仅渲染可见 DOM 节点
- **计算属性缓存：** Pinia computed properties 避免重复计算
- **事件监听管理：** Store 生命周期管理监听器
