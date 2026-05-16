# scx-music 项目文档

## 项目作用

本地音乐播放器桌面应用，支持音频播放、音乐库管理、播放列表。

## AI 快速入口

**重要**: AI 助手（Claude Code、Cursor、Copilot）请先阅读 [.wiki/ai-context.md](.wiki/ai-context.md) 以最快速度理解项目。

## 技术栈

**前端：** Vue 3.5 + TypeScript 5.6 + Vuetify 4.0 + Vite 6.0 + Pinia 3.0
**后端：** Tauri v2 + Rust (Edition 2021) + Rodio 0.19 + SQLite

## 启动方式

```bash
# 完整开发模式
pnpm app:dev

# 仅前端
pnpm dev
```

## 打包方式

```bash
pnpm app:build
```

## 核心目录

```
src/                        # Vue 3 前端
├── stores/                 # Pinia 状态管理 (player, library, settings)
├── composables/            # Vue composables (useToast)
├── components/             # UI 组件
│   └── library/           # 音乐库子组件
├── utils/                  # 工具函数 (virtualScroll, errorHandler)
└── App.vue                 # 根组件

src-tauri/                  # Rust 后端
├── src/
│   ├── lib.rs              # Tauri 主入口
│   ├── audio.rs            # 音频引擎
│   ├── commands/           # IPC 命令
│   └── db/                 # SQLite 数据库
├── Cargo.toml              # Rust 依赖
└── tauri.conf.json         # Tauri 配置
```

## 关键文档

- [architecture.md](architecture.md) - 架构设计
- [frontend.md](frontend.md) - 前端结构
- [backend.md](backend.md) - 后端结构
- [ipc.md](ipc.md) - IPC 通信
- [ai-context.md](ai-context.md) - AI 快速上下文
