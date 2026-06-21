# 风险点

## 高频 IPC

**风险：** 进度更新事件每 500ms 推送一次，频谱数据每 33ms 推送
**影响：** CPU 占用和前端渲染压力
**缓解：** 进度频率合理；频谱数据仅在播放且可视化开启时推送

## 外部依赖

### LRCLIB API
**风险：** LRCLIB 服务不可用或响应慢
**影响：** 歌词获取失败
**缓解：** 三级降级策略 (SQLite 缓存 → 内嵌歌词 → LRCLIB API)，缓存命中不依赖网络

## 阻塞 IO

### 文件扫描
**风险：** `scan_music_folder` 递归扫描大型文件夹可能阻塞
**影响：** UI 冻结
**缓解：** 目前在主线程，未来可改为异步。**2026-06-21：** 加递归深度上限 16、文件总数上限 50000、跳过符号链接（`symlink_metadata`），防止循环符号链接栈溢出和超大目录卡死

### 数据库操作
**风险：** 大批量 upsert 可能阻塞
**影响：** UI 冻结
**缓解：** 使用事务批量操作，bootstrap 单次加载

## Shell 风险

**当前状态：** 无直接 shell 调用
**未来风险：** 如果添加 shell plugin 需要严格参数验证

## Capability 风险（最小权限设计，2026-06-20 收紧）

**当前权限分布（按窗口）：**
- **主窗口 `default.json`**：`core:default` + `opener:default` + `dialog:default` + `updater:default` + `process:default` + 7 个具体 window 权限
- **`shortcuts.json`**：3 个具体 global-shortcut 权限（注册/注销/查询）
- **`desktop-lyrics.json`**：15 项（webview/app/event + 11 个 window allow-*）
- **`mini-player.json`**：16 项（同上 + set-always-on-top / is-visible / set-focus）
- **`desktop-lyrics-lock.json`**：4 项（webview/app/event listen/emit，36×36 锁按钮窗口最小集）

**风险：** 3 个子窗口已移除 `core:default` 聚合权限，改用具体 allow-* 列表。命令级权限（自定义 permission 文件）尚未引入，所有 `#[tauri::command]` 默认对拥有 `core:webview:default` 的窗口可达 —— 这是已知 P1 收紧方向。

**风险：** Tauri v2 权限模型按**发起调用的窗口**检查 capability。跨窗口调用（如 `mini-player` 窗口调 `main.setFocus()`）需要发起方持有相应权限，不是目标窗口。所有子窗口的 capability 已补齐 `allow-set-focus`/`allow-is-visible` 等跨窗口操作所需项。

## P0 加固（2026-06-20）

本次加固修复了全量代码审查发现的 8 个 P0 级问题。详细 spec/plan 见 `docs/superpowers/specs|plans/2026-06-20-p0-hardening-*.md`。

### SQL 注入
**风险点：** `commands/stats.rs::build_time_filter` 曾用 `format!()` 把前端传入的 `start`/`end` 日期字符串拼进 SQL
**缓解：** 重构返回 `(String, Vec<SqlValue>)` 元组，绝对日期模式改用 `?` 占位符 + `SqlValue::Text` 参数化绑定。5 个 `stats_*` 命令全部改用 `params_from_iter`

### 统计数据口径一致性
**风险点：** `stats_listening_overview.play_count` 曾累计 `songs.play_count`（全库），不参与时间过滤；`stats_top_songs`/`stats_top_artists` 同问题
**缓解：** 三处 `play_count` 字段统一改为 `COUNT(*) FROM play_history`，与同函数其他指标（duration/genre/artist）保持口径一致。前端报告 Tab 选「周/月/年」时显示的播放次数现在会随周期变化

### 配置注入（set_setting 白名单）
**风险点：** `set_setting(key, value)` 曾接受任意 key
**缓解：** `ALLOWED_EXACT_KEYS`（9 项）+ `ALLOWED_KEY_PREFIXES`（4 项 `mini-player.` / `desktop-lyrics.` / `shortcut.` / `lyric.offset.`）白名单，前缀匹配要求非空子键（`shortcut.` 本身会被拒绝）。`get_setting`/`get_all_settings` 不加白名单（读取无副作用）

### 路径遍历
**风险点：** `export_playlist_m3u`/`export_playlist_pls`/`export_backup`/`export_settings`/`import_backup`/`import_settings` 共 6 个命令曾直接 `fs::write/read` 前端传入的任意路径
**缓解：** 新增 `validate_user_path` 函数，要求绝对路径 + 拒绝 `..` 段。6 个命令开头统一调用。合法用户流程（dialog 返回的路径都是绝对且无 `..`）不受影响

### Mutex 中毒连锁 panic
**风险点：** 音频模块 14 处 `lock().unwrap()`，任一线程 panic 让 Mutex 中毒后所有 lock 都会连锁 panic
**缓解：** 新增 `audio::lock_or_recover` 辅助函数，中毒时 `unwrap_or_else(|e| e.into_inner())` 取出内部数据继续服务。**使用策略（见函数 doc）：** 进度线程和批量操作（player_set_queue/next/previous）用本函数；单步 IPC 命令（player_pause/resume/seek 等）保留 `.lock().map_err(...)?`，让前端能感知错误。另：`commands/playlists.rs` 的 `UNIX_EPOCH.unwrap()` 同步改为 `unwrap_or_default()`

### Sequential/Shuffle 行为
**风险点：** `engine.rs::next_index` 中 `Sequential` 和 `RepeatAll` 行为完全相同（都循环），`Shuffle` 不是真随机（只是顺序往后走）
**缓解：**
- **Sequential**：到末尾返回 `None`（不再循环）；`player_next` 命令检测到 None 不切歌；歌曲自然播完时进度线程检测到 None 不触发下一首，播放自然停止 —— 这是 Sequential 的正确语义
- **Shuffle**：用 `rand::seq::SliceRandom::choose` 从「非当前 index」中随机选，避免连续两次同一首；单曲队列（len==1）特判返回 `Some(0)`

### 监听器累积泄漏
**风险点：** `useMiniPlayer`/`useDesktopLyrics`/`useLyrics` 三处 composable 在主窗口被多次调用时每次都注册新的 Tauri 事件监听器，组件挂载/卸载累积导致监听器数量单调增长
**缓解：**
- **useMiniPlayer/useDesktopLyrics**：模块级状态 + 幂等 init guard（参考 `usePlayer.ts::listenersSetup` 模式）。删除 `onUnmounted` 中对模块级 unlistens 的清理（监听器随 webview 销毁自动回收）
- **useLyrics**：因为接受 `currentSong` ref 参数不能完全单例化，改用 `_listenPromise` 追踪 listen 的 promise，`onUnmounted` 改为 async 先 await promise 再 unlisten —— 修复"组件在 listen 完成前卸载导致监听器泄漏"的竞态

### useDesktopLyrics lyrics 实例缓存
**风险点：** `useDesktopLyrics` 在 lyrics 窗口内调用 `useLyrics(currentSong)`，每次调用 useDesktopLyrics 都会新建 useLyrics 实例（叠加多个 audio:progress 监听）
**缓解：** `lyricsInstance` 模块级缓存，仅在 lyrics 窗口首次调用时创建 useLyrics 实例。`currentSong` 是模块级 ref，lyrics 实例的 `watch(currentSong)` 只注册一次

## P1 后端健壮性（2026-06-21）

本次修复了后端架构债务 + 5 项命令级健壮性问题。详细 spec/plan 见 `docs/superpowers/specs|plans/2026-06-21-p1-backend-robustness-*.md`。

### AppError 全量迁移（架构债务）
**风险点：** `error.rs` 的 `AppError`/`AppResult` 曾是死代码（未编译进 crate），50+ 命令返回 `Result<T, String>` 用 `.map_err(|e| e.to_string())?` 样板
**缓解：** 启用 `mod error` + `serde::Serialize`，全量迁移 commands/*.rs 和 audio/*.rs 到 `AppResult`。`From<String>/From<io::Error>/From<rusqlite::Error>/From<serde_json::Error>` 自动转换让 `?` 直接工作。前端 `errorHandler.ts` 适配 `{type, message}` 结构（`AppInvokeError` 类保留 payload 供分类处理）

### rename_song 原子性
**风险点：** `rename_song` 6 步操作无事务，文件重命名成功后 DB 更新失败导致文件找不到；Lofty 写失败静默跳过
**缓解：** DB UPDATE 包裹在 `conn.transaction()` 内；若 DB 失败且文件已重命名，best-effort 回滚文件名（`rename(new→old)`，失败打日志含两路径）；Lofty 读/写失败显式报错（不再静默跳过）

### fetch_lrclib 错误区分
**风险点：** `fetch_lrclib` 曾把所有错误 `.ok()?` 吞掉返回 `None`，前端把 `source='none'` 缓存 —— 一次网络抖动后这首歌歌词永久被判无
**缓解：** 区分「无歌词」（HTTP 200 空结果 → `Ok(None)`，缓存 `source='none'`）和「网络错误」（请求失败/非 2xx/JSON 解析失败 → `Err(AppError)`，不缓存，向上传播让用户可重试）。模块级 `OnceLock<reqwest::Client>` 10s 超时

### scan_music_folder 递归防护
**风险点：** `scan_music_folder` 递归无深度上限，跟随符号链接，无文件数上限 —— 循环符号链接可致栈溢出，超大目录可卡死
**缓解：** `MAX_RECURSION_DEPTH=16` + `MAX_FILES_TOTAL=50_000` + `symlink_metadata` 跳过符号链接。超限静默截断/跳过（用户可能不知道有循环符号链接）

### bootstrap N+1 prepare
**风险点：** `get_bootstrap_data` 循环内对每个 playlist 重复 prepare 同一 SQL（N+1 prepare）
**缓解：** 改单条 `SELECT playlist_id, song_id FROM playlist_songs ORDER BY playlist_id, sort_order` + Rust 侧 `HashMap::entry().or_default()` 分组。行为无变化（每个 playlist 内歌曲顺序仍按 sort_order 保留）

## P2 D4 AppError 残留清理（2026-06-21）

P1 启用 AppError 时保留了 60 处 `.map_err(|e| e.to_string())?` 残留（依赖 `From<String>` 自动转 `OperationFailed`，前端拿不到细分 variant）。本次清理替换了 **45 处**：

- 所有 `db.0.lock().map_err(|e| e.to_string())?` → `crate::audio::lock_or_recover(&db.0)`（P0 中毒自愈模式，bootstrap/settings/playlists/stats/songs/lyrics/import_export/device 共 41 处）
- `commands/window.rs`（2 处）：`win.hide()`/`win.show()` → 裸 `?`，依赖新增的 `From<tauri::Error>`
- `commands/shortcuts.rs` 的 `register/unregister`（2 处）：→ 裸 `?`，依赖新增的 `From<tauri_plugin_global_shortcut::Error>`
- rusqlite/fs/serde_json 调用在 P1 迁移时已改为直接 `?`（走 `From<rusqlite::Error>`→Database、`From<io::Error>`→FileOperation、`From<serde_json::Error>`→OperationFailed），本计划无需再改

**新增 2 个 `From` 实现**（`error.rs`）：`From<tauri::Error>` 和 `From<tauri_plugin_global_shortcut::Error>`，均转 `OperationFailed`。

**保留 11 处**（设计性决策）：
- `audio/commands.rs`（7）+ `audio/device.rs`（2）+ `audio/analyzer_cmds.rs`（2）共 11 处 **AudioState 锁**：单步 IPC 命令保留 `.map_err(|e| e.to_string())?`，让前端能感知 lock 中毒错误（与 P0 doc 中 `lock_or_recover` 的使用策略一致 —— 进度线程/批量操作用自愈，单步命令保留错误传播）

**另保留 9 处**（技术限制 + 自定义上下文）：
- `commands/shortcuts.rs`（4）：`parse::<Shortcut>()` 的 `HotKeyParseError` 来自非直接依赖 crate（`global_hotkey`），无法在 `error.rs` 写 `From`，仍经 `From<String>` 桥接
- `commands/songs.rs::rename_song`（5）：`format!("Song not found: {}", e)?` 等自定义上下文 —— 有上下文价值，依赖 `From<String>` 转 OperationFailed

**前端影响**：`AppInvokeError.errorType` 现在能拿到 `FileOperation` / `Database` 等细分 variant，便于按错误类别分类显示 toast（实际改造留待 P3）。

## 大文件风险

### 音频文件
**风险：** 大型音频文件 (如 FLAC 专辑) 可能占用大量内存
**影响：** 内存占用增加
**缓解：** Rodio 流式解码，不一次性加载整个文件

### 数据库
**风险：** 大量歌曲元数据导致数据库增大
**影响：** 启动加载时间
**缓解：** 已添加索引，bootstrap 单次加载

## 内存风险

### 进度线程
**风险：** 线程未正确停止导致内存泄漏
**缓解：** 使用 `Arc<AtomicBool>` 控制线程生命周期

### 音频引擎
**风险：** OutputStream 未正确释放
**缓解：** 使用 `_stream` 字段保持生命周期，stop 时正确清理

### 音频输出设备兼容性
**风险：** CPAL/CoreAudio 在部分 macOS 设备上 `default_output_config()` 和 `supported_output_configs()` 均失败
**影响：** 用户无法切换到特定输出设备
**缓解：** 三层回退链（标准路径 → supported_configs → 硬编码配置），复合设备优先使用 `default_output_device()` 句柄避免匹配到输入变体

### 前端状态
**风险：** 大量歌曲数据占用内存
**影响：** 浏览器内存压力
**缓解：** 已实现虚拟滚动 (VirtualSongTable.vue)，仅渲染可见节点

## 并发风险

### 音频状态
**风险：** 多个 Command 同时修改 AudioState
**缓解：** 使用 `Arc<Mutex<T>>` 保护

### 数据库
**风险：** 并发写入导致锁定
**缓解：** Tauri 单进程 + WAL 模式

## 文件系统风险

### 路径处理
**风险：** 符号链接、相对路径可能导致文件重复或丢失
**缓解：** 使用绝对路径，upsert_songs 基于 file_path UNIQUE 约束去重

### 权限问题
**风险：** 文件权限不足导致读取失败
**缓解：** 错误处理和用户提示

### 文件删除
**风险：** 音频文件被删除后数据库中有 orphaned records
**缓解：** 未来可添加文件存在性检查

## 线程安全

### 进度线程
**风险：** 访问共享状态时数据竞争
**缓解：** 正确使用 Mutex，及时释放锁

### 音频引擎
**风险：** 跨线程传递不安全的类型
**缓解：** 使用 `unsafe impl Send for AudioEngine` (已验证安全性)

### FFT 分析线程
**风险：** SampleBuffer 并发读写
**缓解：** Mutex 保护 buffer，批量读写减少竞争

## 错误处理

### IPC 调用
**风险：** 网络错误、序列化错误
**缓解：** 使用 `Result<T, String>` 返回，前端 try-catch

### 文件 IO
**风险：** 文件不存在、权限不足
**缓解：** 使用 `?` 传播错误，前端显示错误提示

## 桌面歌词窗口跨平台风险

- **macOS 透明窗口在某些桌面壁纸下出现黑边**：已通过 `shadow: false` + 圆角 8px 缓解，但仍可能在动态壁纸/特定主题下出现
- **Windows 透明窗口 + alwaysOnTop 在部分全屏应用（游戏、视频播放器全屏）下可能被遮挡**：MVP 接受此限制，用户可退出全屏或调整 always-on-top 行为（未来扩展）
- **锁定后无法从窗口本身解锁**：`setIgnoreCursorEvents(true)` 是整窗穿透，LockBadge 也不可点击；依赖 SettingsView "锁定"复选框作为唯一解锁入口（已加 tooltip 提示"锁定后窗口点击穿透，可在此解锁"）
- **窗口位置在多显示器场景的持久化**：拖到副屏后位置仍持久化，但若副屏被移除/分辨率变化，Tauri 自动夹紧到主屏可见区
- **跨 webview 的事件订阅开销**：`audio:progress` 每 500ms 广播到所有 webview，desktop-lyrics 窗口隐藏时仍接收；useLyrics 内部仅更新 `currentLineIndex` ref，开销可忽略
