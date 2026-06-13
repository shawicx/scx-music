# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

**scx-music** — 本地音乐播放器桌面应用（Tauri v2 双进程架构）。

**技术栈：**
- 前端：Vue 3 (Composition API) + TypeScript + Vite 6
- UI：Vuetify 4.0 + GSAP 动画
- 状态管理：Pinia 3.0（stores → composables 分层）
- 国际化：vue-i18n v11（中文/英文）
- 桌面框架：Tauri v2（Rust 后端）
- 数据库：SQLite（Rusqlite，WAL 模式）
- 音频：Rodio 0.19 + Symphonia + rustfft（频谱分析）
- 包管理器：pnpm

## 规则

1. **禁止提交代码** — 不要执行 `git commit`、`git push` 等提交操作，由用户自行提交
2. **删除文件前必须告知用户并取得同意** — 不能未经确认删除任何文件
3. **Superpowers spec/plan 文件位置** — 仅存放在 `docs/superpowers/spec/` 和 `docs/superpowers/plan/` 下。这两个目录已在 `.gitignore` 中，禁止从 `.gitignore` 中删除 `docs/superpowers/` 条目，禁止将 spec/plan 文件放到其他文件夹

## 命令

```bash
pnpm dev          # 仅 Vite 开发服务器（端口 1420）
pnpm build        # 类型检查（vue-tsc）然后 Vite 构建
pnpm app:dev      # Tauri 开发模式（Vite + Rust，热重载）
pnpm app:build    # 生产构建（Tauri 打包桌面应用）

cd src-tauri && cargo build    # 构建 Rust 后端
cd src-tauri && cargo test     # 运行 Rust 测试
```

## 架构

```
用户操作 → Vue 组件 → Pinia Store → IPC invoke → Rust 命令
Rust 命令 → IPC emit → Pinia Store → UI 更新
```

- **`src/`** — Vue 3 前端（Tauri webview 中的 SPA）
- **`src-tauri/`** — Rust 后端（原生窗口、文件系统、音频）
- **IPC**：前端通过 `utils/errorHandler.ts` 的 `invokeCommand()` 调用；后端通过 `app.emit()` 发送事件
- 命令在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中注册

### 前端

- Stores（`stores/`）是薄封装，实际逻辑在 composables（`composables/`）中
- 所有 IPC 调用通过 Pinia Stores，组件不直接调用 `invoke`
- 可视化：`src/visualization/` — 4 种 Canvas 渲染器 + `useAudioAnalyzer` composable
- 国际化：`src/locales/zh-CN.ts`、`src/locales/en.ts`，命名空间：common/sidebar/library/player/settings/playbackMode/toast/empty/importExport/update
- 动画：GSAP + Flip 插件，composables 在 `src/composables/useAnimation*.ts`

### 后端

- `audio/` — 音频引擎模块（engine.rs 核心逻辑、commands.rs 播放命令、device.rs 设备管理、types.rs 类型定义、analyzer_cmds.rs 频谱命令、mod.rs 进度线程）
- `analyzer.rs` — FFT 频谱（256 点 → 64 bins，30fps，通过 `audio:spectrum` 事件推送）
- `commands/` — IPC 处理器：bootstrap、songs、playlists、settings、lyrics、import_export
- `db/` — SQLite 数据库，含迁移（V1-V3）和数据模型（Song、Playlist、PlaylistSong、Lyric）
- 错误处理：统一 `AppError` 枚举在 `error.rs`，所有命令返回 `AppResult<T>`

### 关键数据流

- **启动**：`App.vue` → `useLibraryStore.loadFromDb()` → `get_bootstrap_data`（单次 IPC 加载全部数据）
- **播放**：`usePlayerStore.playFromQueue()` → `usePlayQueue.generateQueue()`（按模式排序）→ `player_set_queue`
- **歌词**：三级回退：SQLite 缓存 → Lofty 内嵌 → LRCLIB API
- **模式切换**：`usePlaybackMode.cycleMode()` → `setMode` + `regenerateQueue` → PlayQueueDrawer GSAP Flip 动画

## Tauri IPC 约定

Rust：`src-tauri/src/` 中的文件使用 `#[tauri::command]`，在 `lib.rs` 的 `invoke_handler` 中注册。
前端：通过 `@tauri-apps/api/core` 的 `invokeCommand("command_name", { args })` 调用。

## 命名规范

| 上下文 | 风格 |
|--------|------|
| Vue 组件 | PascalCase |
| 前端函数 | camelCase |
| Store | `useXxxStore` |
| Composable | `useXxx` |
| Rust 结构体 | PascalCase |
| Rust 函数 | snake_case |
| IPC 命令 | snake_case |
| 提交信息 | Conventional Commits（feat/fix/refactor/docs/perf） |

## CI / 发布

- **GitHub Actions**：`.github/workflows/build.yml` — 推送 `v*` tag 时触发
- **Changelog**：`git-cliff` 从 Conventional Commits 自动生成 → 写入 GitHub Release body
- **配置**：`cliff.toml`（过滤 feat/fix/perf/refactor，中文分组标题）
- **平台**：macOS（arm64 + x86_64）、Windows

## Wiki 驱动开发

本项目使用 `.wiki/` 目录作为知识库。每个功能任务遵循两条规则：

1. **开发功能前**：阅读相关 `.wiki/` 页面，了解当前架构和设计决策。
2. **完成功能后**：更新受影响的 `.wiki/` 页面，保持文档与代码同步。
